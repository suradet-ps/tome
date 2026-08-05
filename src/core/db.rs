//! Local-first IndexedDB cache layer.
//!
//! Provides typed async CRUD operations over IndexedDB using raw `js_sys` /
//! `wasm_bindgen` interop.  The database is named `"tome"` (version 1) with
//! one object store per entity kind, all keyed by the entity's `id` field.

use crate::core::error::{AppError, AppResult};
use js_sys::{Array, JSON};
use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

const DB_NAME: &str = "tome";
const DB_VERSION: u32 = 1;

/// Object-store names — each maps to one entity table.
pub mod stores {
  pub const BOOKS: &str = "books";
  pub const CHAPTERS: &str = "chapters";
  pub const PROGRESS: &str = "progress";
  pub const NOTES: &str = "notes";
  pub const FLASHCARDS: &str = "flashcards";
  pub const SYNC_QUEUE: &str = "sync_queue";
}

const ALL_STORES: &[&str] = &[
  stores::BOOKS,
  stores::CHAPTERS,
  stores::PROGRESS,
  stores::NOTES,
  stores::FLASHCARDS,
  stores::SYNC_QUEUE,
];

fn window() -> AppResult<web_sys::Window> {
  web_sys::window().ok_or_else(|| AppError::Other("no window".into()))
}

fn idb_factory() -> AppResult<web_sys::IdbFactory> {
  window()?
    .indexed_db()
    .map_err(|e| AppError::Other(format!("indexed_db(): {e:?}")))?
    .ok_or_else(|| AppError::Other("IndexedDB not supported".into()))
}

/// Wrap an `IdbRequest` into a `JsFuture` that resolves with its `result`.
fn idb_request_future(request: &web_sys::IdbRequest) -> AppResult<JsFuture> {
  let promise = js_sys::Promise::new(&mut |res, rej| {
    // onsuccess
    {
      let res = res.clone();
      let onsuccess = Closure::once(move |_: JsValue| {
        // The result is on the request's target, but we already captured `res`.
        let _ = res.call0(&JsValue::UNDEFINED);
      });
      request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
      onsuccess.forget();
    }
    // onerror
    {
      let rej = rej.clone();
      let onerror = Closure::once(move |_: JsValue| {
        let _ = rej.call1(
          &JsValue::UNDEFINED,
          &JsValue::from_str("IDB request failed"),
        );
      });
      request.set_onerror(Some(onerror.as_ref().unchecked_ref()));
      onerror.forget();
    }
  });

  Ok(JsFuture::from(promise))
}

/// Wrap an `IdbOpenDbRequest` (with upgrade handler) into a `JsFuture`.
fn idb_open_future(request: &web_sys::IdbOpenDbRequest) -> AppResult<JsFuture> {
  let promise = js_sys::Promise::new(&mut |res, rej| {
    // onsuccess
    {
      let res = res.clone();
      let onsuccess = Closure::once(move |_: JsValue| {
        let _ = res.call0(&JsValue::UNDEFINED);
      });
      request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
      onsuccess.forget();
    }
    // onerror
    {
      let rej = rej.clone();
      let onerror = Closure::once(move |_: JsValue| {
        let _ = rej.call1(&JsValue::UNDEFINED, &JsValue::from_str("IDB open failed"));
      });
      request.set_onerror(Some(onerror.as_ref().unchecked_ref()));
      onerror.forget();
    }
  });

  Ok(JsFuture::from(promise))
}

/// Open the `tome` database. Creates object stores on upgrade.
pub async fn open_db() -> AppResult<web_sys::IdbDatabase> {
  let factory = idb_factory()?;
  let request = factory
    .open_with_u32(DB_NAME, DB_VERSION)
    .map_err(|e| AppError::Other(format!("open: {e:?}")))?;

  // Set up the upgrade handler. `onupgradeneeded` fires on the request, and
  // `request.result` is the database we need to create stores on.
  let upgrade = Closure::once(move |_: JsValue, _event: web_sys::Event| {
    // The target of the upgradeneeded event is the IDBOpenDBRequest whose
    // `.result` is the IdbDatabase.
    let target: web_sys::IdbOpenDbRequest = match _event.target() {
      Some(t) => t.unchecked_into(),
      None => return,
    };
    let db: web_sys::IdbDatabase = match target.result() {
      Ok(val) => val.unchecked_into(),
      Err(_) => return,
    };
    let names: Vec<String> = (0..db.object_store_names().length())
      .filter_map(|i| db.object_store_names().get(i))
      .collect();
    for store_name in ALL_STORES {
      if names.iter().any(|n| n == store_name) {
        continue;
      }
      let _ = db.create_object_store(store_name);
    }
  });
  request.set_onupgradeneeded(Some(upgrade.as_ref().unchecked_ref()));
  upgrade.forget();

  let future = idb_open_future(&request)?;
  let value = future
    .await
    .map_err(|e| AppError::Other(format!("open await: {e:?}")))?;
  Ok(value.unchecked_into())
}

fn js_to_json(val: &JsValue) -> AppResult<String> {
  JSON::stringify(val)
    .map_err(|e| AppError::Other(format!("JSON.stringify: {e:?}")))
    .map(|s| String::from(s))
}

fn json_to_js(json: &str) -> AppResult<JsValue> {
  JSON::parse(json).map_err(|e| AppError::Other(format!("JSON.parse: {e:?}")))
}

fn idb_tx(
  db: &web_sys::IdbDatabase,
  store_name: &str,
  mode: web_sys::IdbTransactionMode,
) -> AppResult<(web_sys::IdbTransaction, web_sys::IdbObjectStore)> {
  let tx = db
    .transaction_with_str_and_mode(store_name, mode)
    .map_err(|e| AppError::Other(format!("tx: {e:?}")))?;
  let store = tx
    .object_store(store_name)
    .map_err(|e| AppError::Other(format!("store: {e:?}")))?;
  Ok((tx, store))
}

/// Retrieve a single entity by its `id`.
pub async fn get<T: DeserializeOwned>(store_name: &str, id: &str) -> AppResult<Option<T>> {
  let db = open_db().await?;
  let (_tx, store) = idb_tx(&db, store_name, web_sys::IdbTransactionMode::Readonly)?;
  let req = store
    .get(&JsValue::from_str(id))
    .map_err(|e| AppError::Other(format!("get: {e:?}")))?;

  // Wait for the request to complete, then read the result.
  let _ = idb_request_future(&req)?
    .await
    .map_err(|e| AppError::Other(format!("get await: {e:?}")))?;

  let result = req
    .result()
    .map_err(|e| AppError::Other(format!("result: {e:?}")))?;

  if result.is_undefined() || result.is_null() {
    return Ok(None);
  }
  let json = js_to_json(&result)?;
  let val: T = serde_json::from_str(&json).map_err(|e| AppError::Other(format!("de: {e}")))?;
  Ok(Some(val))
}

/// Retrieve all entities from a named object store.
pub async fn get_all<T: DeserializeOwned>(store_name: &str) -> AppResult<Vec<T>> {
  let db = open_db().await?;
  let (_tx, store) = idb_tx(&db, store_name, web_sys::IdbTransactionMode::Readonly)?;
  let req = store
    .get_all()
    .map_err(|e| AppError::Other(format!("get_all: {e:?}")))?;

  let _ = idb_request_future(&req)?
    .await
    .map_err(|e| AppError::Other(format!("get_all await: {e:?}")))?;

  let result = req
    .result()
    .map_err(|e| AppError::Other(format!("result: {e:?}")))?;
  let array: Array = result.into();
  let mut items = Vec::with_capacity(array.length() as usize);
  for i in 0..array.length() {
    let json = js_to_json(&array.get(i))?;
    let val: T =
      serde_json::from_str(&json).map_err(|e| AppError::Other(format!("de item: {e}")))?;
    items.push(val);
  }
  Ok(items)
}

/// Insert or update a single entity (keyed by its `id` field via serde).
pub async fn put<T: Serialize>(store_name: &str, entity: &T) -> AppResult<()> {
  let db = open_db().await?;
  let (_tx, store) = idb_tx(&db, store_name, web_sys::IdbTransactionMode::Readwrite)?;
  let json = serde_json::to_string(entity).map_err(|e| AppError::Other(format!("ser: {e}")))?;
  let js_val = json_to_js(&json)?;
  let req = store
    .put(&js_val)
    .map_err(|e| AppError::Other(format!("put: {e:?}")))?;
  let _ = idb_request_future(&req)?
    .await
    .map_err(|e| AppError::Other(format!("put await: {e:?}")))?;
  Ok(())
}

/// Insert or update many entities in a single transaction.
pub async fn put_many<T: Serialize>(store_name: &str, entities: &[T]) -> AppResult<()> {
  if entities.is_empty() {
    return Ok(());
  }
  let db = open_db().await?;
  let (_tx, store) = idb_tx(&db, store_name, web_sys::IdbTransactionMode::Readwrite)?;
  for entity in entities {
    let json = serde_json::to_string(entity).map_err(|e| AppError::Other(format!("ser: {e}")))?;
    let js_val = json_to_js(&json)?;
    store
      .put(&js_val)
      .map_err(|e| AppError::Other(format!("put: {e:?}")))?;
  }
  // The transaction auto-commits when all requests are done and there are no
  // more events referencing it. Just wait a tick to let it settle.
  Ok(())
}

/// Delete a single entity by its `id`.
pub async fn delete(store_name: &str, id: &str) -> AppResult<()> {
  let db = open_db().await?;
  let (_tx, store) = idb_tx(&db, store_name, web_sys::IdbTransactionMode::Readwrite)?;
  store
    .delete(&JsValue::from_str(id))
    .map_err(|e| AppError::Other(format!("del: {e:?}")))?;
  Ok(())
}

/// Remove all entities from a named object store.
pub async fn clear(store_name: &str) -> AppResult<()> {
  let db = open_db().await?;
  let (_tx, store) = idb_tx(&db, store_name, web_sys::IdbTransactionMode::Readwrite)?;
  store
    .clear()
    .map_err(|e| AppError::Other(format!("clear: {e:?}")))?;
  Ok(())
}
