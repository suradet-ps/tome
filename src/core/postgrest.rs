//! A small PostgREST query builder.
//!
//! It is intentionally minimal — just enough to mirror the queries the
//! application issues against Supabase. It uses `gloo-net` so the same code
//! compiles to `wasm32-unknown-unknown`.

use crate::core::error::{AppError, AppResult};
use gloo_net::http::{Method, Request, RequestBuilder, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A handle for issuing PostgREST queries.
#[derive(Debug, Clone)]
pub struct PostgrestClient {
    base_url: String,
    token: Option<String>,
    api_key: Option<String>,
}

impl PostgrestClient {
    /// Create a new client targeting `base_url` (e.g. `https://xyz.supabase.co`).
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token: None,
            api_key: None,
        }
    }

    /// Set the bearer token used for authenticated requests.
    #[must_use]
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set the `apikey` header (Supabase anon key).
    #[must_use]
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Returns a [`QueryBuilder`] for the `table` URL path.
    #[must_use]
    pub fn from(&self, table: &str) -> QueryBuilder<'_> {
        QueryBuilder {
            client: self,
            path: format!("/rest/v1/{table}"),
            filters: Vec::new(),
            select: None,
            order: Vec::new(),
            range: None,
        }
    }

    /// Issues a `POST /rest/v1/rpc/{name}` call.
    pub async fn rpc<T, B>(&self, name: &str, body: &B) -> AppResult<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = format!("{}/rest/v1/rpc/{name}", self.base_url);
        let mut builder = RequestBuilder::new(&url)
            .method(Method::POST)
            .header("Content-Type", "application/json");
        builder = self.apply_auth(builder);
        let request: Request = builder.body(serde_json::to_string(body)?)?;
        let response = request.send().await?;
        parse_response(response).await
    }

    fn apply_auth(&self, mut builder: RequestBuilder) -> RequestBuilder {
        if let Some(key) = &self.api_key {
            builder = builder.header("apikey", key);
        }
        if let Some(token) = &self.token {
            let value = format!("Bearer {token}");
            builder = builder.header("Authorization", &value);
        }
        builder
    }
}

/// Fluent query builder.
pub struct QueryBuilder<'a> {
    client: &'a PostgrestClient,
    path: String,
    filters: Vec<Filter>,
    select: Option<String>,
    order: Vec<String>,
    range: Option<(u32, u32)>,
}

impl QueryBuilder<'_> {
    /// Set the columns to fetch (`select`).
    #[must_use]
    pub fn select(mut self, columns: impl Into<String>) -> Self {
        self.select = Some(columns.into());
        self
    }

    /// Add an equality filter (`col=eq.value`).
    #[must_use]
    pub fn eq(mut self, column: &str, value: impl ToString) -> Self {
        self.filters
            .push(Filter::Eq(column.to_string(), value.to_string()));
        self
    }

    /// Add a `lte` filter.
    #[must_use]
    pub fn lte(mut self, column: &str, value: impl ToString) -> Self {
        self.filters
            .push(Filter::Lte(column.to_string(), value.to_string()));
        self
    }

    /// Add a `gte` filter.
    #[must_use]
    pub fn gte(mut self, column: &str, value: impl ToString) -> Self {
        self.filters
            .push(Filter::Gte(column.to_string(), value.to_string()));
        self
    }

    /// Add an `in` filter.
    #[must_use]
    pub fn is_in(mut self, column: &str, values: &[impl ToString]) -> Self {
        let formatted = values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        self.filters
            .push(Filter::In(column.to_string(), format!("({formatted})")));
        self
    }

    /// Add an ordering (`column.asc` or `column.desc`).
    #[must_use]
    pub fn order(mut self, column: &str, ascending: bool) -> Self {
        let direction = if ascending { "asc" } else { "desc" };
        self.order.push(format!("{column}.{direction}"));
        self
    }

    /// Set the row range (`Offset-Limit` pair used by Supabase).
    #[must_use]
    pub fn range(mut self, offset: u32, limit: u32) -> Self {
        self.range = Some((offset, limit));
        self
    }

    /// Issue a `GET` and decode the response as a list of `T`.
    pub async fn get<T>(self) -> AppResult<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let request = self.build_request(Method::GET, None::<&()>)?;
        let response = request.send().await?;
        parse_response(response).await
    }

    /// Issue a `GET` and decode at most one row.
    pub async fn get_one<T>(self) -> AppResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let request = self.build_request(Method::GET, None::<&()>)?;
        let _ = request
            .headers()
            .set("Accept", "application/vnd.pgrst.object+json");
        let response = request.send().await?;
        if response.status() == 406 {
            return Ok(None);
        }
        let value: serde_json::Value = parse_response(response).await?;
        if value.is_null() {
            return Ok(None);
        }
        let row: T = serde_json::from_value(value)?;
        Ok(Some(row))
    }

    /// Issue a `POST` with a JSON body.
    pub async fn insert<T, B>(self, body: &B) -> AppResult<Vec<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let builder = self.prepare_builder(Method::POST);
        let request: Request = builder
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .body(serde_json::to_string(body)?)?;
        let response = request.send().await?;
        parse_response(response).await
    }

    /// Issue a `POST` and decode the first row only.
    pub async fn insert_one<T, B>(self, body: &B) -> AppResult<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let rows: Vec<T> = self.insert(body).await?;
        rows.into_iter().next().ok_or(AppError::NoData)
    }

    /// Issue a `PATCH` updating rows that match the filters.
    pub async fn update<T, B>(self, body: &B) -> AppResult<Vec<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let builder = self.prepare_builder(Method::PATCH);
        let request: Request = builder
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .body(serde_json::to_string(body)?)?;
        let response = request.send().await?;
        parse_response(response).await
    }

    /// Issue a `POST` upsert.
    ///
    /// `on_conflict` accepts a comma-separated list of columns that form the
    /// conflict target (e.g. `"user_id,chapter_id"`).
    pub async fn upsert<T, B>(self, body: &B, on_conflict: &str) -> AppResult<Vec<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let builder = self.prepare_builder(Method::POST);
        let prefer =
            format!("resolution=merge-duplicates,return=representation,on_conflict={on_conflict}");
        let request: Request = builder
            .header("Content-Type", "application/json")
            .header("Prefer", &prefer)
            .body(serde_json::to_string(body)?)?;
        let response = request.send().await?;
        parse_response(response).await
    }

    /// Issue a `POST` upsert that decodes a single row.
    pub async fn upsert_one<T, B>(self, body: &B, on_conflict: &str) -> AppResult<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let rows: Vec<T> = self.upsert(body, on_conflict).await?;
        rows.into_iter().next().ok_or(AppError::NoData)
    }

    /// Issue a `DELETE` on rows that match the filters.
    pub async fn delete(self) -> AppResult<()> {
        let request = self.build_request::<()>(Method::DELETE, None)?;
        let response = request.send().await?;
        if !response.ok() {
            return Err(error_from_response(response).await);
        }
        Ok(())
    }

    fn prepare_builder(&self, method: Method) -> RequestBuilder {
        let mut builder = RequestBuilder::new(&self.url()).method(method);
        builder = self.client.apply_auth(builder);
        if let Some((offset, limit)) = self.range {
            let range = format!(
                "{offset}-{}",
                offset.saturating_add(limit).saturating_sub(1)
            );
            builder = builder
                .header("Range-Unit", "items")
                .header("Range", &range);
        }
        builder
    }

    fn build_request<B>(&self, method: Method, body: Option<&B>) -> AppResult<Request>
    where
        B: Serialize,
    {
        let mut builder = self.prepare_builder(method);
        if let Some(body) = body {
            builder = builder.header("Content-Type", "application/json");
            return Ok(builder.body(serde_json::to_string(body)?)?);
        }
        Ok(builder.build()?)
    }

    fn url(&self) -> String {
        let mut url = format!("{}{}", self.client.base_url, self.path);
        let mut query = BTreeMap::new();
        if let Some(select) = &self.select {
            query.insert("select".to_string(), select.clone());
        }
        for filter in &self.filters {
            match filter {
                Filter::Eq(col, v) => query.insert(col.clone(), format!("eq.{v}")),
                Filter::Lte(col, v) => query.insert(col.clone(), format!("lte.{v}")),
                Filter::Gte(col, v) => query.insert(col.clone(), format!("gte.{v}")),
                Filter::In(col, v) => query.insert(col.clone(), format!("in.{v}")),
            };
        }
        if !self.order.is_empty() {
            query.insert("order".to_string(), self.order.join(","));
        }
        if !query.is_empty() {
            url.push('?');
            let mut first = true;
            for (k, v) in &query {
                if first {
                    first = false;
                } else {
                    url.push('&');
                }
                let _ = write!(&mut url, "{k}={v}");
            }
        }
        url
    }
}

#[derive(Debug, Clone)]
enum Filter {
    Eq(String, String),
    Lte(String, String),
    Gte(String, String),
    In(String, String),
}

async fn parse_response<T>(response: Response) -> AppResult<T>
where
    T: DeserializeOwned,
{
    if !response.ok() {
        return Err(error_from_response(response).await);
    }
    let text = response.text().await?;
    if text.is_empty() {
        return Err(AppError::NoData);
    }
    let value: T = serde_json::from_str(&text)
        .map_err(|e| AppError::other(format!("Failed to decode response: {e}; body={text}")))?;
    Ok(value)
}

async fn error_from_response(response: Response) -> AppError {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if matches!(status, 401 | 403) {
        return AppError::Unauthorized;
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) {
        if let Some(message) = value.get("message").and_then(|m| m.as_str()) {
            return AppError::http(status, message.to_string());
        }
        if let Some(message) = value.get("error_description").and_then(|m| m.as_str()) {
            return AppError::http(status, message.to_string());
        }
    }
    AppError::http(
        status,
        if body.is_empty() {
            status.to_string()
        } else {
            body
        },
    )
}
