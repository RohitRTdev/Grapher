use serde::{Serialize};
use crate::vtk::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Serialize)]
pub struct Node {
    id: usize,
    color_code: u8,
    fn_val: f64,
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
    spacing: &'a Vec<f64>
}

impl<'a> Vertex<'a> {
    fn compute_id(iv: &Vec<usize>, dims: &Vec<usize>) -> usize {
        let mut data_idx = 0;
        let mut dim_prod = 1;
        for idx in (0..dims.len()).rev() {
            data_idx += iv[idx] * dim_prod;
            dim_prod *= dims[dims.len() - idx - 1]; 
        }

        data_idx
    }

    fn new(iv: Vec<usize>, id: usize, dims: &'a Vec<usize>, spacing: &'a Vec<f64>) -> Self {
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

    fn get_vertex(&self) -> Vec<f64> {
        let mut vertex = vec![0f64; self.iv.len()];
        for dim in 0..self.iv.len() {
            vertex[dim] = self.spacing[dim] * self.iv[self.iv.len() - dim - 1] as f64 
            // Translate the structure to bring it to center of camera
            - ((self.dims[dim] as f64) / 2.0);
        }

        vertex
    }
}


#[derive(Debug, PartialEq)]
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
                if fnv >= highest_vertex_val {
                    highest_vertex_val = fnv;
                    highest_vertex_id = nv.id;
                }

                // Find which component in the upper link this vertex belongs to and insert it there
                let mut found = vec![];
                for (comp_id, vertices) in &upper_link {
                    for vert in vertices {
                        if vert.is_neighbor(&nv) {
                            found.push(*comp_id);
                            break;
                        }        
                    }
                }

                // If no such component, create new one
                if found.len() == 0 {
                    upper_link.insert(new_id, vec![nv]);
                    new_id += 1; 
                }
                else {
                    let mut first_comp = upper_link.remove(&found[0]).unwrap();
                    for comp in found.iter().skip(1) {
                        first_comp.extend(upper_link.remove(&comp).unwrap())
                    }
                    first_comp.push(nv);
                    upper_link.insert(found[0], first_comp);
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
                
                // If 2d case, then just augment 3rd dimension as 0
                let z = if vertex.len() == 2 {
                    0f64
                }
                else {
                    vertex[2]
                };

                let node = Node {
                    id: cur.id,
                    color_code,
                    fn_val: fv,
                    x: vertex[0],
                    y: vertex[1],
                    z 
                };
                
                self.graph.nodes.push(node);
            },
            _ => {}
        }
        self.cache.insert(cur.id, pt_type);
    }

    fn is_boundary_point(&self, iv: &Vec<usize>) -> bool {
        for dim in 0..iv.len() {
            if iv[dim] == 0 || iv[dim] == self.volume.dims[self.volume.dims.len() - dim - 1] - 1 {
                return true;
            }
        }

        false
    }

    // Updates the loop index and tells whether iteration must continue
    fn update_iv(&self, iv: &mut Vec<usize>) -> bool {
        for dim in (0..self.volume.dims.len()).rev() {
            iv[dim] += 1;
            if iv[dim] >= self.volume.dims[self.volume.dims.len() - dim - 1] {
                // We have iterated over all points
                if dim == 0 {
                    return false;
                }
                iv[dim] = 0;
            }
            else {
                break;
            }
        }

        return true
    }

    // Follow the gradient until we hit a maxima or boundary
    // Here, we will do an iterative approach
    fn path_traverse(&self, gradients: &Vec<usize>) -> HashSet<usize> {
        let mut reachable_maxima = HashSet::new();
        let mut work_queue = VecDeque::new();
        gradients.iter().for_each(|&item| {
            work_queue.push_back(item);
        });

        while !work_queue.is_empty() {
            let cur = work_queue.pop_front().unwrap();
            if let Some(vertex) = self.cache.get(&cur) {
                match vertex {
                    CriticalPointType::Maxima => {
                        // We have reached a maxima, stop traversal along this path
                        reachable_maxima.insert(cur);
                    },
                    /* Ideally, we must not hit a saddle since we're traversing from d-1 saddles */
                    /* However, I'm still keeping this here for safety */
                    CriticalPointType::Saddle(vertices) => {
                        // A saddle or other point indicates that we need to continue exploring this path
                        // Add new neighbors to work list
                        vertices.iter().for_each(|&item| {
                            work_queue.push_back(item);
                        });

                        println!("Hit a saddle during traversal {:?}", vertex);
                    },
                    CriticalPointType::Other(highest_vertex) => {
                        work_queue.push_back(*highest_vertex);
                    }
                }
            }
            else {
                // The vertex info not stored in cache indicates that this is a boundary point
                // Do nothing in this case
            }
        }


        reachable_maxima
    }

    fn gradient_ascent(&mut self) {
        for (&vertex_id, vertex_type) in &self.cache {
            match vertex_type {
                CriticalPointType::Saddle(gradients) => {
                    println!("Processing saddle id: {} and type={:?}", vertex_id, vertex_type);
                    let targets = self.path_traverse(gradients);

                    targets.iter().for_each(|&target| {
                        // Add an edge from this saddle to every maxima reachable from this saddle
                        self.graph.edges.push(Edge {
                            source: vertex_id,
                            target
                        })
                    })
                },
                _ => {}
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

        // 1st pass: Point classification
        let mut do_work = true;
        while do_work {
            if self.is_boundary_point(&iv) {
                do_work = self.update_iv(&mut iv);
                continue;
            }

            let data_idx = Vertex::compute_id(&iv, &self.volume.dims);
            let cur = Vertex::new(iv.clone(), data_idx, &self.volume.dims, &self.volume.spacing);

            self.classify_point(&cur);

            do_work = self.update_iv(&mut iv);
        }

        // 2nd pass: Gradient arc computation
        self.gradient_ascent();
    }
}

fn get_total_maxima(graph: &Graph) -> usize {
    let mut maxima = 0;
    for pt in &graph.nodes {
        if pt.color_code == 0 {
            maxima += 1;
        }
    }

    maxima    
}


fn process_vtk_file(path: String) -> Result<Graph, String> {
    let volume = read_vtk(&path)?;
    
    println!("VTK contents:\nSpacing:{:?}\nDimensions:{:?}\nFirst 3 points: {} {} {}",
            volume.spacing, volume.dims, volume.data[0], volume.data[1], volume.data[2]);


    let mut graph = ExtGraph::new(&volume);
    graph.compute();

    println!("There are total: {} points and {} maxima!", volume.data.len(), get_total_maxima(&graph.graph));

    Ok(graph.graph)
}

#[tauri::command]
pub async fn process_vtk_file_async(path: String) -> Result<Graph, String> {
    tokio::task::spawn_blocking(move || {
        process_vtk_file(path)
    })
    .await.expect("Unexpected tokio error!!")
}
