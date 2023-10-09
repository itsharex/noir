use anyhow::Result;
use mysql::prelude::Queryable;
use mysql::{from_row, Pool, PooledConn, Row};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::convert_value;
#[derive(Debug, Serialize, Deserialize)]
struct ResultSet {
    affected_rows: u64,
    warnings: u16,
    info: String,
    rows: Vec<serde_json::Value>,
}

fn row_to_object(row: Row) -> serde_json::Value {
    let mut object = json!({});
    for column in row.columns_ref() {
        let column_value = &row[column.name_str().as_ref()];
        let value = convert_value(column_value);
        object[column.name_str().as_ref()] = value;
    }

    return object;
}

pub fn raw_query(mut conn: PooledConn, query: String) -> Result<serde_json::Value> {
    let rows: Vec<Row> = conn.query(&query)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row_to_object(row));
    }
    let result = json!({ "result": result });
    return Ok(result);
}

pub fn execute_query(pool: &Pool, query: &str) -> Result<serde_json::Value> {
    let mut conn = pool.get_conn()?;
    let mut results = conn.query_iter(query)?;
    let mut sets: Vec<ResultSet> = vec![];
    while let Some(result_set) = results.iter() {
        let affected_rows = result_set.affected_rows();
        let warnings = result_set.warnings();
        let info = &result_set.info_str().to_string();
        let mut rows = Vec::new();
        for row in result_set {
            rows.push(row_to_object(from_row(row?)));
        }
        let set = ResultSet {
            affected_rows,
            warnings,
            info: info.to_string(),
            rows,
        };
        sets.push(set);
    }
    let result = json!(sets);

    return Ok(result);
}
