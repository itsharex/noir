use crate::{
    engine::types::result::{ResultSet, TableMetadata},
    utils::error::Error,
};
use anyhow::Result;
use deadpool_postgres::Pool;
use futures::{pin_mut, TryStreamExt};
use serde_json::Value;

use super::utils::row_to_object;

pub async fn raw_query(pool: Pool, query: &str) -> Result<Vec<Value>> {
    let conn = pool.get().await?;
    let params = vec![];
    let rows = conn.query(query, &params).await?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row_to_object(row)?);
    }
    Ok(result)
}

pub async fn execute_query(pool: &Pool, query: &str) -> Result<ResultSet> {
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let conn = pool.get().await?;
    let params: Vec<String> = vec![];
    let it = conn.query_raw(query, &params).await?;
    let mut rows: Vec<Value> = Vec::new();
    pin_mut!(it);
    while let Some(row) = it.try_next().await? {
        rows.push(row_to_object(row)?);
    }
    let affected_rows = it.rows_affected().unwrap_or(0);
    let end_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let set = ResultSet {
        start_time,
        end_time,
        affected_rows,
        warnings: 0,
        info: "".to_string(),
        rows,
        table: TableMetadata {
            table: String::from(""),
            foreign_keys: None,
            primary_key: None,
            columns: None,
        },
    };
    Ok(set)
}

pub async fn execute_tx(pool: &Pool, queries: Vec<&str>) -> Result<(), Error> {
    let mut conn = pool.get().await?;
    let tx = conn.transaction().await?;
    for q in queries {
        let params: Vec<String> = vec![];
        match tx.execute_raw(q, &params).await {
            Ok(..) => {}
            Err(e) => {
                tx.rollback().await?;
                return Err(Error::TxError(e.to_string()));
            }
        }
    }
    tx.commit().await?;
    Ok(())
}
