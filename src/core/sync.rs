//! Offline write queue persisted in IndexedDB.
//!
//! When a store write fails due to being offline, the operation is enqueued
//! here.  On reconnect the queue is flushed: each operation is replayed
//! against Supabase and removed on success.  The queue itself is stored in
//! the `sync_queue` IndexedDB object store.

use crate::core::db;
use crate::core::error::AppResult;
use crate::core::supabase;
use crate::stores::connectivity::use_connectivity;
use serde::{Deserialize, Serialize};

/// The kind of write operation to replay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OpKind {
  /// `POST` upsert (with an `on_conflict` column list).
  Upsert { on_conflict: String },
  /// `DELETE` with filter columns already in the payload.
  Delete,
}

/// A single pending write operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOp {
  /// Unique id for this operation (UUID).
  pub id: String,
  /// Supabase table name (e.g. `"reading_books"`).
  pub table: String,
  /// The operation kind and its parameters.
  pub kind: OpKind,
  /// The JSON body to send (a single row for upsert, or filter params for
  /// delete).
  pub payload: serde_json::Value,
  /// ISO-8601 timestamp when the op was created.
  pub created_at: String,
}

/// Enqueue a write operation for later sync.
pub async fn enqueue(op: PendingOp) -> AppResult<()> {
  db::put(db::stores::SYNC_QUEUE, &op).await?;
  use_connectivity().inc_pending();
  Ok(())
}

/// Remove a single operation from the queue after successful sync.
pub async fn remove(id: &str) -> AppResult<()> {
  db::delete(db::stores::SYNC_QUEUE, id).await?;
  use_connectivity().dec_pending();
  Ok(())
}

/// Return all pending operations, ordered by creation time.
pub async fn all() -> AppResult<Vec<PendingOp>> {
  db::get_all::<PendingOp>(db::stores::SYNC_QUEUE).await
}

/// Flush the entire queue: replay each operation against Supabase.
///
/// Returns `(succeeded, failed)` counts.  Succeeded operations are removed
/// from the queue; failed ones are left for the next attempt.
pub async fn flush_all() -> AppResult<(u32, u32)> {
  let ops = all().await?;
  if ops.is_empty() {
    use_connectivity().clear_pending();
    return Ok((0, 0));
  }

  let mut succeeded: u32 = 0;
  let mut failed: u32 = 0;

  for op in ops {
    match replay(&op).await {
      Ok(()) => {
        remove(&op.id).await?;
        succeeded += 1;
      }
      Err(_) => {
        failed += 1;
      }
    }
  }

  // If all ops succeeded, clear the counter to zero (handles rounding).
  if failed == 0 {
    use_connectivity().clear_pending();
  }

  Ok((succeeded, failed))
}

/// Replay a single operation against Supabase.
async fn replay(op: &PendingOp) -> AppResult<()> {
  let c = supabase::supabase()?;
  let pg = c.postgrest();

  match &op.kind {
    OpKind::Upsert { on_conflict } => {
      // The payload is a single JSON object (the row).
      let _: serde_json::Value = pg
        .from(&op.table)
        .upsert_one(&op.payload, on_conflict)
        .await?;
      Ok(())
    }
    OpKind::Delete => {
      // The payload contains filter columns — we reconstruct the delete
      // by applying each key as an `.eq()` filter.  The payload shape is
      // `{ "col": "value", ... }`.
      let mut builder = pg.from(&op.table);
      if let Some(obj) = op.payload.as_object() {
        for (k, v) in obj {
          let val = match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
          };
          builder = builder.eq(k, val);
        }
      }
      builder.delete().await
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pending_op_upsert_roundtrip() {
    let op = PendingOp {
      id: "test-id".into(),
      table: "reading_notes".into(),
      kind: OpKind::Upsert {
        on_conflict: "user_id,chapter_id".into(),
      },
      payload: serde_json::json!({"user_id": "u1", "chapter_id": "c1", "content": "hello"}),
      created_at: "2025-01-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&op).unwrap();
    let decoded: PendingOp = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, "test-id");
    assert_eq!(decoded.table, "reading_notes");
    assert_eq!(
      decoded.kind,
      OpKind::Upsert {
        on_conflict: "user_id,chapter_id".into()
      }
    );
    assert_eq!(decoded.payload["content"], "hello");
  }

  #[test]
  fn pending_op_delete_roundtrip() {
    let op = PendingOp {
      id: "del-1".into(),
      table: "reading_progress".into(),
      kind: OpKind::Delete,
      payload: serde_json::json!({"id": "row-42"}),
      created_at: "2025-06-15T12:00:00Z".into(),
    };
    let json = serde_json::to_string(&op).unwrap();
    let decoded: PendingOp = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.kind, OpKind::Delete);
    assert_eq!(decoded.payload["id"], "row-42");
  }

  #[test]
  fn op_kind_equality() {
    let a = OpKind::Upsert {
      on_conflict: "id".into(),
    };
    let b = OpKind::Upsert {
      on_conflict: "id".into(),
    };
    let c = OpKind::Upsert {
      on_conflict: "user_id,chapter_id".into(),
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(a, OpKind::Delete);
  }
}
