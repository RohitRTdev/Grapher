mod vtk;
mod manifold;
mod vertex;
mod similarity;
mod graph;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init()) 
        .invoke_handler(tauri::generate_handler![
            graph::process_file_async,
            graph::retrieve_last_graph,
            graph::set_graph_mode,
            graph::recompute_graph,
            vertex::set_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
