use serde::{Serialize};

#[derive(Serialize)]
pub struct Node {
    id: usize,
    name: String,
    x: f64,
    y: f64,
    z: f64
}

#[derive(Serialize)]
pub struct Edge {
    source: usize,
    target: usize,
}

#[derive(Serialize)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

fn process_vtk_file(path: String) -> Result<Graph, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    
    // TODO: Replace this with actual VTK parsing + extremum graph logic
    println!("Processing VTK file with size: {}", content.len());
 
    // Dummy graph for now
    let nodes = if path.ends_with("bonsai.vtk") {
        vec![
            Node { id: 0, name: "Min".into(), x: 0.0, y: 0.0, z: 0.0 },
            Node { id: 1, name: "Saddle".into(), x: 1.0, y: 1.0, z: 1.0 },
            Node { id: 2, name: "Max".into(), x: 1.0, y: 0.0, z: 1.0 },
        ]
    }
    else {
        vec![
            Node { id: 0, name: "Min".into(), x: 1.0, y: 0.0, z: 0.0 },
            Node { id: 1, name: "Saddle".into(), x: 1.0, y: 1.0, z: 1.0 },
            Node { id: 2, name: "Max".into(), x: 1.0, y: 0.0, z: 1.0 },
        ]
    };

    let edges = vec![
        Edge { source: 0, target: 1 },
        Edge { source: 1, target: 2 },
    ];

    Ok(Graph { nodes, edges })
}

#[tauri::command]
pub async fn process_vtk_file_async(path: String) -> Result<Graph, String> {
    tokio::task::spawn_blocking(move || {
        process_vtk_file(path)
    })
    .await.expect("Unexpected tokio error!!")
}
