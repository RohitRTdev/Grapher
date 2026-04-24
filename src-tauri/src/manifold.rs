use std::fs::File;
use std::io::{BufReader, Read};
use std::collections::HashSet;

#[derive(Debug)]
pub struct AdjList {
    pub neighbors: Vec<HashSet<usize>>
}

#[derive(Debug)]
pub struct Manifold {
    pub embedding_dim: usize,
    pub vertices: Vec<Vec<f64>>,
    pub values: Vec<f64>,
    pub graph: AdjList 
}

pub fn read_manifold(path: &str) -> Result<Manifold, String> {
    let file = File::open(path).map_err(|f| f.to_string())?;
    let mut reader = BufReader::new(file);
    let read_as_u32 = |reader: &mut BufReader<File>, buf: &mut [u8; 4]| -> Result<usize, String> {
        reader.read_exact(buf).map_err(|f| f.to_string())?;
        Ok(u32::from_le_bytes(*buf) as usize)
    };
    
    let read_as_f64 = |reader: &mut BufReader<File>, buf: &mut [u8; 8]| -> Result<f64, String> {
        reader.read_exact(buf).map_err(|f| f.to_string())?;
        Ok(f64::from_le_bytes(*buf))
    };

    // Read header
    let mut buf_u32 = [0u8; 4];

    let embedding_dim = read_as_u32(&mut reader, &mut buf_u32)?;
    let num_vertices = read_as_u32(&mut reader, &mut buf_u32)?;
    let num_edges = read_as_u32(&mut reader, &mut buf_u32)?;

    // Read vertices
    let mut vertices = Vec::with_capacity(num_vertices);
    let mut values = Vec::with_capacity(num_vertices);

    let mut buf_f64 = [0u8; 8];

    for _ in 0..num_vertices {
        let mut v = Vec::with_capacity(embedding_dim);

        // Read the vertex contents
        for _ in 0..embedding_dim {
            let val = read_as_f64(&mut reader, &mut buf_f64)?;
            v.push(val);
        }

        // Now read the function value
        let fn_val = read_as_f64(&mut reader, &mut buf_f64)?;

        values.push(fn_val);
        vertices.push(v);
    }

    let mut graph = AdjList{neighbors: vec![HashSet::new(); num_vertices]};
    
    // Read edges
    for _ in 0..num_edges {
        let u = read_as_u32(&mut reader, &mut buf_u32)?;
        let v = read_as_u32(&mut reader, &mut buf_u32)?;

        if u >= num_vertices || v >= num_vertices {
            return Err("Edge information invalid!".to_owned());
        }

        graph.neighbors[u].insert(v);
        
        // Since this is an undirected edge
        graph.neighbors[v].insert(u);
    }

    println!("Total edges: {}", num_edges);

    Ok(Manifold {
        embedding_dim,
        vertices,
        values,
        graph
    })
}