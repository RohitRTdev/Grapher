use serde::{Serialize};
use crate::vtk::*;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Node {
    id: usize,
    color_code: u8,
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

impl Graph {
    fn new() -> Self {
        Graph {
            nodes: vec![],
            edges: vec![]
        }
    }
}

#[derive(Debug)]
struct Vertex<'a> {
    id: usize, 
    iv: Vec<usize>,
    dims: &'a Vec<usize>,
    spacing: &'a (f64, f64, f64)
}

impl<'a> Vertex<'a> {
    fn compute_id(iv: &Vec<usize>, dims: &Vec<usize>) -> usize {
        let mut data_idx = 0;
        let mut dim_prod = 1;
        for idx in (0..dims.len()).rev() {
            data_idx += iv[idx] * dim_prod;
            dim_prod *= dims[idx]; 
        }

        data_idx
    }

    fn new(iv: Vec<usize>, id: usize, dims: &'a Vec<usize>, spacing: &'a (f64, f64, f64)) -> Self {
        Vertex {
            iv,
            id,
            dims,
            spacing
        }
    }

    fn get_neighbor(cur: &'a Vertex, offset: &Vec<i8>) -> Self {
        let mut iv = vec![0; cur.dims.len()];
        for dim in 0..cur.dims.len() {
            iv[dim] = (cur.iv[dim] as isize + offset[dim] as isize) as usize;
        }

        let id = Self::compute_id(&iv, cur.dims);

        Self {
            iv,
            id,
            dims: cur.dims,
            spacing: cur.spacing
        }
    }

    fn is_neighbor(&self, other: &Vertex) -> bool {
        let mut diff = vec![0; self.dims.len()];
        for dim in 0..self.dims.len() {
            diff[dim] = self.iv[dim] as isize - other.iv[dim] as isize;
        }

        diff.iter().all(|&x| x == 0 || x == 1) 
        || diff.iter().all(|&x| x == 0 || x == -1) 
    }

    // TODO: Generalize to n dims
    fn get_vertex(&self) -> (f64, f64, f64) {
        (self.spacing.0 * self.iv[2] as f64, self.spacing.1 * self.iv[1] as f64, self.spacing.2 * self.iv[0] as f64)
    }
}


#[derive(PartialEq)]
enum CriticalPointType {
    Saddle(Vec<usize>),
    Other(usize),
    Maxima
}

struct ExtGraph<'a> {
    volume: &'a VtkVolume,
    graph: Graph,
    cache: HashMap<usize, CriticalPointType>,
    neighbor_idx_set: Vec<Vec<i8>>
}


impl<'a> ExtGraph<'a> {
    fn new(volume: &'a VtkVolume) -> Self {
        ExtGraph {
            volume,
            graph: Graph::new(),
            cache: HashMap::new(),
            neighbor_idx_set: vec![]
        }
    }

    fn classify_point(&mut self, cur: &Vertex) {
        const MAXIMA: u8 = 0;
        const SADDLE: u8 = 1;
        
        let mut upper_link: HashMap<usize, Vec<Vertex>> = HashMap::new();
    
        let fv = self.volume.data[cur.id];
        
        let mut highest_vertex_val = -std::f64::INFINITY;
        let mut highest_vertex_id = 0;

        let mut new_id = 0;
        for neighbor in &self.neighbor_idx_set {
            let nv = Vertex::get_neighbor(cur, &neighbor);
            let fnv = self.volume.data[nv.id];

            // We only care about the upper link
            if fnv > fv {
                // Keep track of the highest neighbor
                if fnv < highest_vertex_val {
                    highest_vertex_val = fnv;
                    highest_vertex_id = nv.id;
                }

                // Find which component in the upper link this vertex belongs to and insert it there
                let mut found = None;
                'outer: for (comp_id, vertices) in &upper_link {
                    for vert in vertices {
                        if vert.is_neighbor(&nv) {
                            found = Some(*comp_id);
                            break 'outer;
                        }        
                    }
                }

                if let Some(id) = found {
                    upper_link.get_mut(&id).unwrap().push(nv);
                }
                // If no such component, create new one
                else {
                   upper_link.insert(new_id, vec![nv]);
                   new_id += 1; 
                }
            }
        }

        let mut color_code = SADDLE;
        let pt_type = if upper_link.len() >= 2 {
            // Find all the highest vertices from each upper link component
            let mut vertices = vec![];
            for (_, comp_vert) in &upper_link {
                let mut fn_val = -std::f64::INFINITY;
                let mut chosen_vertex = &comp_vert[0];
                for vert in comp_vert {
                    let cur_val = self.volume.data[vert.id];
                    if fn_val < cur_val {
                        fn_val = cur_val;
                        chosen_vertex = vert;
                    }
                }
                vertices.push(chosen_vertex.id);
            } 
            CriticalPointType::Saddle(vertices)
        } 
        else if upper_link.len() == 0 {
            color_code = MAXIMA;
            CriticalPointType::Maxima
        }   
        else {
           CriticalPointType::Other(highest_vertex_id) 
        };

        // Include d-1 saddles and maxima in the graph
        match &pt_type {
            CriticalPointType::Saddle(_) | CriticalPointType::Maxima => {
                let vertex = cur.get_vertex();
                let node = Node {
                    id: cur.id,
                    color_code,
                    x: vertex.0,
                    y: vertex.1,
                    z: vertex.2 
                };
                
                self.graph.nodes.push(node);
            },
            _ => {}
        }
        self.cache.insert(cur.id, pt_type);
    }

    fn is_boundary_point(&self, iv: &Vec<usize>) -> bool {
        for dim in 0..iv.len() {
            if iv[dim] == 0 || iv[dim] == self.volume.dims[dim] - 1 {
                return true;
            }
        }

        return false
    }

    fn update_iv(&self, iv: &mut Vec<usize>) {
        for dim in (0..self.volume.dims.len()).rev() {
            iv[dim] += 1;
            if iv[dim] >= self.volume.dims[dim] {
                iv[dim] = 0;
            }
            else {
                break;
            }
        }
    }

    fn compute(&mut self) {
        let num_dims = self.volume.dims.len(); 

        // Get the neighbor indices for this vertex based on freudenthal division
        for value in 1..(1 << num_dims) {
            let mut bits = Vec::new();

            let mut x = value;
            while x > 0 {
                bits.push((x & 1) as i8);
                x >>= 1;
            }

            while bits.len() < num_dims {
                bits.push(0);
            }

            self.neighbor_idx_set.push(bits);
        }

        let mut symmetric_neighbors = vec![];
        for neighbor in &self.neighbor_idx_set {
            let mut opp_neighbor = vec![];
            for comp in neighbor {
                opp_neighbor.push(-*comp);
            }
            symmetric_neighbors.push(opp_neighbor);
        }
        self.neighbor_idx_set.append(&mut symmetric_neighbors);
        println!("{:?}", self.neighbor_idx_set);
        
        let mut iv = vec![0; num_dims];

        for point in 0..self.volume.data.len() / 100 {
            if self.is_boundary_point(&iv) {
                self.update_iv(&mut iv);
                continue;
            }

            let data_idx = Vertex::compute_id(&iv, &self.volume.dims);
            let cur = Vertex::new(iv.clone(), data_idx, &self.volume.dims, &self.volume.spacing);

            self.classify_point(&cur);

            self.update_iv(&mut iv);
        }
    }
}


fn process_vtk_file(path: String) -> Result<Graph, String> {
    let volume = read_vtk(&path)?;
    
    println!("VTK contents:\nOrigin:{:?}\nSpacing:{:?}\nDimensions:{:?}\nFirst 3 points: {} {} {}",
            volume.origin, volume.spacing, volume.dims, volume.data[0], volume.data[1], volume.data[2]);


    let mut graph = ExtGraph::new(&volume);
    graph.compute();

    println!("There are total: {} points and {} critical points!", volume.data.len() / 100, graph.graph.nodes.len());

    Ok(graph.graph)
}

#[tauri::command]
pub async fn process_vtk_file_async(path: String) -> Result<Graph, String> {
    tokio::task::spawn_blocking(move || {
        process_vtk_file(path)
    })
    .await.expect("Unexpected tokio error!!")
}
