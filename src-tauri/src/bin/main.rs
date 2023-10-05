// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use query_noir::{
    database::database,
    handlers::{connections, queries},
    queues::query::{async_process_model, rs2js},
    state::{self, AsyncState},
    utils::init,
};

use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing_subscriber;

use state::AppState;
use tauri::{Manager, State};

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

fn main() {
    tracing_subscriber::fmt::init();

    let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel(1);
    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel(1);

    tauri::Builder::default()
        .manage(AsyncState {
            tasks: Mutex::new(async_proc_input_tx),
            connections: Default::default(),
        })
        .manage(AppState {
            db: Default::default(),
            connections: Default::default(),
        })
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);

            app.emit_all("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .setup(|app| {
            init::init_app()?;
            let handle = app.handle();

            let app_state: State<AppState> = handle.state();
            let db = database::initialize_database().expect("Database initialize should succeed");
            *app_state.db.lock().unwrap() = Some(db);

            tauri::async_runtime::spawn(async move {
                async_process_model(async_proc_input_rx, async_proc_output_tx).await
            });

            tauri::async_runtime::spawn(async move {
                loop {
                    if let Some(output) = async_proc_output_rx.recv().await {
                        rs2js(output, &handle).await
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connections::add_connection,
            connections::delete_connection,
            connections::get_connections,
            connections::init_connection,
            // connections::disconnect, // TODO
            queries::enqueue_query,
            queries::get_columns,
            queries::get_constraints,
            queries::get_functions,
            queries::get_procedures,
            queries::get_triggers,
            queries::get_table_structure,
            queries::query_results,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
