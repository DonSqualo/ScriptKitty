//! ScriptCAD Server - Manifold CSG
//! - File watcher
//! - Lua parser
//! - Manifold mesh generation
//! - WebSocket binary mesh streaming

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, thread, time::Duration};
use tokio::sync::{broadcast, mpsc, RwLock};
use tower_http::cors::CorsLayer;
use tracing::{error, info};

mod export;
mod field;
mod geometry;

struct AppState {
    mesh_tx: broadcast::Sender<Vec<u8>>,
    current_mesh: RwLock<Option<Vec<u8>>>,
    current_field: RwLock<Option<Vec<u8>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let file_path = std::env::args().nth(1).unwrap_or("../examples/tube.lua".into());
    let file_path = PathBuf::from(file_path);

    info!("Watching: {:?}", file_path);

    let (mesh_tx, _) = broadcast::channel::<Vec<u8>>(16);
    let (lua_tx, lua_rx) = mpsc::unbounded_channel::<(String, PathBuf)>();
    let (result_tx, mut result_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // Lua processing thread (Manifold needs to run on same thread)
    thread::spawn(move || {
        process_lua_files(lua_rx, result_tx);
    });

    let state = Arc::new(AppState {
        mesh_tx: mesh_tx.clone(),
        current_mesh: RwLock::new(None),
        current_field: RwLock::new(None),
    });

    // Handle mesh/field results
    let state_clone = state.clone();
    tokio::spawn(async move {
        while let Some(data) = result_rx.recv().await {
            let is_field = data.len() >= 5 && &data[0..5] == b"FIELD";

            if is_field {
                *state_clone.current_field.write().await = Some(data.clone());
            } else {
                *state_clone.current_mesh.write().await = Some(data.clone());
            }
            let _ = state_clone.mesh_tx.send(data);
        }
    });

    // Load initial file
    if file_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let _ = lua_tx.send((content, file_path.clone()));
        }
    }

    // File watcher
    let lua_tx_clone = lua_tx.clone();
    let watch_path = file_path.clone();
    tokio::spawn(async move {
        watch_file(watch_path, lua_tx_clone).await;
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("Server: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn serialize_view_config(flat_shading: bool) -> Vec<u8> {
    let mut data = Vec::with_capacity(16);
    data.extend_from_slice(b"VIEW\0\0\0\0");
    data.push(if flat_shading { 1 } else { 0 });
    data
}

fn process_lua_files(mut rx: mpsc::UnboundedReceiver<(String, PathBuf)>, tx: mpsc::UnboundedSender<Vec<u8>>) {
    let lua = mlua::Lua::new();

    // Set up package path to include stdlib directory
    let package_path = lua.globals()
        .get::<_, mlua::Table>("package")
        .and_then(|p| p.get::<_, String>("path"))
        .unwrap_or_default();

    let stdlib_path = "../?.lua;../?/init.lua";
    let new_path = format!("{};{}", stdlib_path, package_path);

    if let Ok(package) = lua.globals().get::<_, mlua::Table>("package") {
        let _ = package.set("path", new_path);
    }

    while let Some((content, file_path)) = rx.blocking_recv() {
        let base_dir = file_path.parent().unwrap_or(std::path::Path::new("."));

        // Process mesh and exports
        match process_single_file(&lua, &content, base_dir) {
            Ok(result) => {
                // Send view config first
                let view_binary = serialize_view_config(result.flat_shading);
                let _ = tx.send(view_binary);

                let binary = result.mesh.to_binary();
                info!(
                    "Generated mesh: {} vertices, {} triangles, {} bytes, flat_shading={}",
                    result.mesh.positions.len() / 3,
                    result.mesh.indices.len() / 3,
                    binary.len(),
                    result.flat_shading
                );
                let _ = tx.send(binary);
            }
            Err(e) => error!("Lua error: {}", e),
        }

        // Try to compute magnetic field if this looks like a Helmholtz coil
        if let Some(field_data) = try_compute_helmholtz_field(&lua, &content) {
            let field_binary = field_data.to_binary();
            info!(
                "Generated field: {}x{} slice, {} arrows, {} line points, {} bytes",
                field_data.slice_width,
                field_data.slice_height,
                field_data.arrows_positions.len() / 3,
                field_data.line_z.len(),
                field_binary.len()
            );
            let _ = tx.send(field_binary);
        }
    }
}

fn try_compute_helmholtz_field(lua: &mlua::Lua, content: &str) -> Option<field::FieldData> {
    // Check if this file defines Helmholtz coil configuration
    if !content.contains("helmholtz") && !content.contains("coil_mean_radius") {
        return None;
    }

    // Execute the Lua to get config values
    let result: mlua::Value = lua.load(content).eval().ok()?;
    let _table = result.as_table()?;

    // Try to extract Helmholtz parameters from globals or return value
    // The config table should be accessible after running the script
    let globals = lua.globals();

    // Try to get config table
    let config: mlua::Table = globals.get("config").ok()?;

    let coil_mean_radius: f64 = config.get("coil_mean_radius").ok()?;
    let gap: f64 = config.get("gap").ok()?;
    let wire_diameter: f64 = config.get("wire_diameter").unwrap_or(0.8);
    let windings: f64 = config.get("windings").unwrap_or(100.0);
    let layers: f64 = config.get("layers").unwrap_or(10.0);
    let packing_factor: f64 = config.get("packing_factor").unwrap_or(0.82);
    let current: f64 = config.get("current").unwrap_or(1.0);

    // Calculate derived values
    let turns_per_layer = (windings / layers).ceil();
    let wire_pitch = wire_diameter / packing_factor;
    let coil_width = turns_per_layer * wire_pitch;
    let coil_height = layers * wire_pitch;
    let coil_inner_r = coil_mean_radius - coil_height / 2.0;
    let coil_outer_r = coil_mean_radius + coil_height / 2.0;
    let ampere_turns = current * windings;

    info!(
        "Computing Helmholtz field: R={:.1}mm, gap={:.1}mm, {:.0} A·turns",
        coil_mean_radius, gap, ampere_turns
    );

    Some(field::compute_helmholtz_field(
        coil_mean_radius,
        coil_inner_r,
        coil_outer_r,
        coil_width,
        gap,
        ampere_turns,
        layers as usize,
    ))
}

struct ProcessResult {
    mesh: geometry::MeshData,
    flat_shading: bool,
    #[allow(dead_code)]
    circular_segments: u32,
}

fn process_single_file(lua: &mlua::Lua, content: &str, base_dir: &std::path::Path) -> Result<ProcessResult> {
    // Clear scene state before each execution to prevent accumulation
    let _ = lua.load(r#"
        local loaded = package.loaded["stdlib"] or package.loaded["stdlib.init"]
        if loaded and loaded.clear then loaded.clear() end
    "#).exec();

    let result: mlua::Value = lua.load(content).eval()?;

    // Extract view config
    let (flat_shading, circular_segments) = if let Some(table) = result.as_table() {
        if let Ok(view) = table.get::<_, mlua::Table>("view") {
            (
                view.get::<_, bool>("flat_shading").unwrap_or(false),
                view.get::<_, u32>("circular_segments").unwrap_or(32),
            )
        } else {
            (false, 32)
        }
    } else {
        (false, 32)
    };

    info!("Using Manifold backend for CSG, circular_segments={}", circular_segments);
    let mesh = geometry::generate_mesh_from_lua_manifold(lua, &result, circular_segments)?;

    if let Some(table) = result.as_table() {
        export::process_exports_from_table(lua, table, base_dir);
    }

    Ok(ProcessResult { mesh, flat_shading, circular_segments })
}

async fn watch_file(path: PathBuf, tx: mpsc::UnboundedSender<(String, PathBuf)>) {
    let (notify_tx, mut notify_rx) = mpsc::channel::<PathBuf>(10);

    let mut debouncer = new_debouncer(Duration::from_millis(200), move |res: DebounceEventResult| {
        if let Ok(events) = res {
            for event in events {
                let _ = notify_tx.blocking_send(event.path);
            }
        }
    })
    .unwrap();

    let watch_dir = path.parent().unwrap_or(&path);
    debouncer.watcher().watch(watch_dir, RecursiveMode::NonRecursive).unwrap();

    info!("Watching directory: {:?}", watch_dir);

    while let Some(changed) = notify_rx.recv().await {
        if changed == path || changed.file_name() == path.file_name() {
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                info!("File changed, regenerating mesh...");
                let _ = tx.send((content, path.clone()));
            }
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.mesh_tx.subscribe();

    // Send current mesh if available
    if let Some(mesh) = state.current_mesh.read().await.clone() {
        let _ = sender.send(Message::Binary(mesh.into())).await;
    }

    // Send current field if available
    if let Some(field) = state.current_field.read().await.clone() {
        let _ = sender.send(Message::Binary(field.into())).await;
    }

    loop {
        tokio::select! {
            Ok(mesh) = rx.recv() => {
                if sender.send(Message::Binary(mesh.into())).await.is_err() {
                    break;
                }
            }
            Some(msg) = receiver.next() => {
                match msg {
                    Ok(Message::Close(_)) | Err(_) => break,
                    _ => {}
                }
            }
            else => break,
        }
    }
}
