use std::{fs::read_to_string, path::PathBuf};

use crate::{
    database::QueryType,
    queues::query::{QueryTask, QueryTaskEnqueueResult, QueryTaskStatus},
    state::{AsyncState, ServiceAccess},
    utils::{
        self,
        crypto::md5_hash,
        error::{CommandResult, Error},
        fs::paginate_file,
    },
};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlparser::{ast::Statement, dialect::dialect_from_str, parser::Parser};
use std::str;
use tauri::{command, AppHandle, State};
use tracing::info;

fn get_query_type(s: Statement) -> QueryType {
    match s {
        Statement::Query(_) => QueryType::Select,
        //Statement::Insert { .. } => QueryType::Insert,
        //Statement::Update { .. } => QueryType::Update,
        //Statement::Delete { .. } => QueryType::Delete,
        _ => QueryType::Other,
    }
}

#[command]
pub async fn enqueue_query(
    app_handle: AppHandle,
    async_state: State<'_, AsyncState>,
    conn_id: String,
    tab_idx: usize,
    sql: &str,
    auto_limit: bool,
    table: Option<String>,
) -> CommandResult<QueryTaskEnqueueResult> {
    info!(sql, conn_id, tab_idx, "enqueue_query");
    let conn = app_handle.acquire_connection(conn_id.clone());
    // ignore sqlparser when dialect is sqlite and statements contain pragma
    let statements = Parser::parse_sql(
        dialect_from_str(conn.config.dialect.to_string())
            .expect("Failed to get dialect")
            .as_ref(),
        sql,
    )?;
    if statements.is_empty() {
        return Err(Error::from(anyhow!("No statements found")));
    }
    let statements: Vec<(String, QueryType, String)> = statements
        .into_iter()
        .map(|s| {
            let id = conn.config.id.to_string() + &tab_idx.to_string() + &s.to_string();
            (s.to_string(), get_query_type(s), md5_hash(&id))
        })
        .collect();
    let async_proc_input_tx = async_state.tasks.lock().await;
    let enqueued_ids: Vec<String> = vec![];
    for (idx, stmt) in statements.iter().enumerate() {
        let (mut statement, t, id) = stmt.clone();
        info!("Got statement {:?}", statement);
        if enqueued_ids.contains(&id) {
            continue;
        }
        if auto_limit && !statement.to_lowercase().contains("limit") && t == QueryType::Select {
            statement = format!("{} LIMIT 1000", statement);
        }
        let task = QueryTask::new(conn.clone(), statement, t, id, tab_idx, idx, table.clone());
        let res = async_proc_input_tx.send(task).await;
        if let Err(e) = res {
            return Err(Error::from(e));
        }
    }
    Ok(QueryTaskEnqueueResult {
        conn_id,
        tab_idx,
        status: QueryTaskStatus::Progress,
        result_sets: statements.iter().map(|s| (s.2.clone())).collect(),
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResultParams {
    pub path: String,
    pub page: usize,
    pub page_size: usize,
}

#[command]
pub async fn get_schemas(app_handle: AppHandle, conn_id: String) -> CommandResult<Vec<Value>> {
    let connection = app_handle.acquire_connection(conn_id);
    Ok(connection.get_schemas().await?)
}

#[command]
pub async fn get_views(app_handle: AppHandle, conn_id: String) -> CommandResult<Vec<Value>> {
    let connection = app_handle.acquire_connection(conn_id);
    Ok(connection.get_views().await?)
}

#[command]
pub async fn execute_tx(
    app_handle: AppHandle,
    conn_id: String,
    queries: Vec<&str>,
) -> CommandResult<()> {
    let connection = app_handle.acquire_connection(conn_id);
    connection.execute_tx(queries).await?;
    Ok(())
}

#[command]
pub async fn execute_query(
    app_handle: AppHandle,
    conn_id: String,
    query: String,
) -> CommandResult<Value> {
    let conn = app_handle.acquire_connection(conn_id);
    let statements = Parser::parse_sql(
        dialect_from_str(conn.config.dialect.to_string())
            .expect("Failed to get dialect")
            .as_ref(),
        &query,
    )?;
    if statements.is_empty() {
        return Err(Error::from(anyhow!("No statements found")));
    }
    let statements: Vec<(String, QueryType, String)> = statements
        .into_iter()
        .map(|s| {
            let id = conn.config.id.to_string() + &s.to_string();
            (s.to_string(), get_query_type(s), md5_hash(&id))
        })
        .collect();
    let stmt = &statements[0];
    let result = conn.execute_query(&stmt.0, stmt.1).await?;
    Ok(json!(result))
}

#[command]
pub async fn get_query_metadata(_app_handle: AppHandle, path: String) -> CommandResult<Value> {
    let data = read_to_string(path + ".metadata");
    match data {
        Ok(data) => Ok(Value::from(data)),
        Err(..) => Err(Error::QueryExpired),
    }
}

#[command]
pub async fn query_results(
    _app_handle: AppHandle,
    params: QueryResultParams,
) -> CommandResult<Value> {
    info!(?params, "query_results");
    let data = paginate_file(&params.path, params.page, params.page_size);
    match data {
        Ok(data) => Ok(Value::from(data)),
        Err(..) => Err(Error::QueryExpired),
    }
}

#[command]
pub async fn get_table_structure(
    app_handle: AppHandle,
    conn_id: String,
    table: String,
) -> CommandResult<Value> {
    let connection = app_handle.acquire_connection(conn_id);
    let result = connection.get_table_structure(table).await?;
    Ok(result)
}

#[command]
pub async fn get_columns(app_handle: AppHandle, conn_id: String) -> CommandResult<Vec<Value>> {
    let connection = app_handle.acquire_connection(conn_id);
    Ok(connection.get_columns(None).await?)
}

#[command]
pub async fn get_primary_key(
    app_handle: AppHandle,
    conn_id: String,
    table: String,
) -> CommandResult<Vec<Value>> {
    let connection = app_handle.acquire_connection(conn_id);
    Ok(connection.get_primary_key(&table).await?)
}

#[command]
pub async fn get_foreign_keys(
    app_handle: AppHandle,
    conn_id: String,
    table: String,
) -> CommandResult<Vec<Value>> {
    let connection = app_handle.acquire_connection(conn_id);
    Ok(connection.get_foreign_keys(&table).await?)
}

#[command]
pub async fn get_triggers(app_handle: AppHandle, conn_id: String) -> CommandResult<Vec<Value>> {
    let connection = app_handle.acquire_connection(conn_id);
    Ok(connection.get_triggers().await?)
}

#[command]
pub async fn get_functions(app_handle: AppHandle, conn_id: String) -> CommandResult<Vec<Value>> {
    let connection = app_handle.acquire_connection(conn_id);
    Ok(connection.get_functions().await?)
}

#[command]
pub async fn get_procedures(app_handle: AppHandle, conn_id: String) -> CommandResult<Vec<Value>> {
    let connection = app_handle.acquire_connection(conn_id);
    Ok(connection.get_procedures().await?)
}

#[command]
pub async fn download_json(source: &str, destination: &str) -> CommandResult<()> {
    let data = read_to_string(source)?;
    let content: String = data
        .lines()
        .map(|line| format!("{},", line))
        .collect::<Vec<String>>()
        .join("\n");
    let content = content[..content.len() - 1].to_string();
    let content = format!("[\n{}\n]", content);

    Ok(utils::fs::write_file(
        &PathBuf::from(destination),
        &content,
    )?)
}

#[command]
pub async fn download_csv(source: &str, destination: &str) -> CommandResult<()> {
    let data = read_to_string(source)?;
    let content: String = data
        .lines()
        .map(|line| format!("{},", line))
        .collect::<Vec<String>>()
        .join("\n");
    let content = content[..content.len() - 1].to_string();
    let content: Vec<Value> = serde_json::from_str(&format!("[{}]", content))?;
    let keys = content[0]
        .as_object()
        .expect("Failed to get object")
        .keys()
        .map(|k| k.to_string())
        .collect::<Vec<String>>();

    let csv = keys.join(",") + "\n";
    let rows = content
        .iter()
        .map(|row| {
            keys.iter()
                .map(|k| {
                    row.get(k)
                        .unwrap_or_else(|| panic!("Failed to get key {} from {}", k, row))
                        .to_string()
                })
                .collect::<Vec<String>>()
                .join(",")
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(utils::fs::write_file(
        &PathBuf::from(destination),
        &format!("{}{}", csv, rows),
    )?)
}

#[command]
pub async fn invalidate_query(path: &str) -> CommandResult<()> {
    utils::fs::remove_dir(path)?;
    Ok(())
}
