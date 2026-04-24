use serde::{Serialize};
use crate::{manifold::*, vertex::*, vtk::*};
use std::{cell::RefCell, collections::{HashMap, HashSet, VecDeque}, rc::Rc};
use rand::{rngs::ThreadRng, Rng};

const SIMPLICITY_RANDOMNESS: f64 = 0.6;

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




#[derive(Debug, PartialEq)]
enum CriticalPointType {
    Saddle(Vec<usize>),
    Other(Option<usize>),
    Maxima
}

struct VtkExtGraphType {
    volume: Rc<VtkVolume>,
    neighbor_idx_set: Rc<Vec<Vec<i8>>>
}

struct ManifoldExtGraphType<'a> {
    volume: Rc<&'a Manifold>,
    kdtree: Option<Rc<NN>>
}

enum ExtGraphImpl<'a> {
    Vtk(VtkExtGraphType),
    Manifold(ManifoldExtGraphType<'a>)
}

struct ExtGraph<'a> {
    ext_type: ExtGraphImpl<'a>,
    graph: RefCell<Graph>,
    cache: RefCell<HashMap<usize, CriticalPointType>>,
    rng: RefCell<ThreadRng>
}

impl<'a> ExtGraph<'a> {
    fn new_with_vtk(volume: VtkVolume) -> Self {
        let rng = rand::thread_rng();
        let neighbor_idx_set = Rc::new(Self::generate_neighbor_idx_set(&volume));
        ExtGraph {
            ext_type: ExtGraphImpl::Vtk (
                VtkExtGraphType {
                    volume: Rc::new(volume),
                    neighbor_idx_set
                }
            ),
            graph: RefCell::new(Graph::new()),
            cache: RefCell::new(HashMap::new()),
            rng: RefCell::new(rng)
        }
    }

    fn new_with_manifold(volume: &'a Manifold, is_ref_mode: bool) -> Self {
        let rng = rand::thread_rng();
        let nn = if !is_ref_mode {
            let mut nn = NN::new();
            nn.build_neighborhood(volume);
            Some(Rc::new(nn))
        }
        else {
            None
        };

        ExtGraph {
            ext_type: ExtGraphImpl::Manifold(
                ManifoldExtGraphType { 
                    volume: Rc::new(volume),
                    kdtree: nn
                }
            ),
            graph: RefCell::new(Graph::new()),
            cache: RefCell::new(HashMap::new()),
            rng: RefCell::new(rng)
        }
    }

    fn classify_point(&self, cur: &Vertex) {
        const MAXIMA: u8 = 0;
        const SADDLE: u8 = 1;
        
        let mut upper_link: HashMap<usize, Vec<Vertex>> = HashMap::new();
    
        let fv = cur.fn_val;
        
        let mut highest_vertex_val = fv;
        let mut highest_vertex_id = None;

        let mut new_id = 0;
        for nv in cur.get_neighbors() {
            let fnv = nv.fn_val; 
            
            // We only care about the upper link
            // A basic implementation of simulation of simplicity
            if fnv > fv || (fnv == fv && self.rng.borrow_mut().gen_bool(SIMPLICITY_RANDOMNESS)) {
                // Keep track of the highest neighbor
                if fnv > highest_vertex_val {
                    highest_vertex_val = fnv;
                    highest_vertex_id = Some(nv.id);
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
                    let cur_val = vert.fn_val;
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
                
                self.graph.borrow_mut().nodes.push(node);
            },
            _ => {}
        }
        self.cache.borrow_mut().insert(cur.id, pt_type);
    }

    fn is_boundary_point(&self, iv: &Vec<usize>) -> bool {
        match &self.ext_type {
            ExtGraphImpl::Vtk(vtk_info) => {
                for dim in 0..iv.len() {
                    if iv[dim] == 0 || iv[dim] == vtk_info.volume.dims[vtk_info.volume.dims.len() - dim - 1] - 1 {
                        return true;
                    }
                }

                false
            },
            _ => {
                panic!("is_boundary_point() called on non vtk type!");
            }
        }
    }

    // Updates the loop index and tells whether iteration must continue
    fn update_iv(&self, iv: &mut Vec<usize>) -> bool {
        match &self.ext_type {
            ExtGraphImpl::Vtk(vtk_info) => {
                for dim in (0..vtk_info.volume.dims.len()).rev() {
                    iv[dim] += 1;
                    if iv[dim] >= vtk_info.volume.dims[vtk_info.volume.dims.len() - dim - 1] {
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

                true
            },
            _ => {
                panic!("update_iv() called on non vtk type!");
            }
        }
    }

    // Follow the gradient until we hit a maxima or boundary
    // Here, we will do an iterative approach
    fn path_traverse(&self, gradients: &Vec<usize>) -> HashSet<usize> {
        let mut reachable_maxima = HashSet::new();
        let mut work_queue = VecDeque::new();
        gradients.iter().for_each(|&item| {
            work_queue.push_back(item);
        });

        let mut visited = HashSet::new();

        while !work_queue.is_empty() {
            let cur = work_queue.pop_front().unwrap();
            
            if visited.contains(&cur) {
                continue;
            }

            if let Some(vertex) = self.cache.borrow().get(&cur) {
                match vertex {
                    CriticalPointType::Maxima => {
                        // We have reached a maxima, stop traversal along this path
                        reachable_maxima.insert(cur);
                    },
                    /* Ideally, we must not hit a saddle since we're traversing from d-1 saddles */
                    /* However, I'm still keeping this here for safety */
                    CriticalPointType::Saddle(vertices) => {
                        for vert in vertices {
                            work_queue.push_back(*vert);
                        }
                    },
                    CriticalPointType::Other(highest_vertex) => {
                        if let Some(highest_vertex_inner) = highest_vertex {
                            work_queue.push_back(*highest_vertex_inner);
                        }
                    }
                }
            }
            else {
                // The vertex info not stored in cache indicates that this is a boundary point
                // Do nothing in this case
            }
            visited.insert(cur);
        }


        reachable_maxima
    }

    fn gradient_ascent(&mut self) {
        for (&vertex_id, vertex_type) in self.cache.borrow().iter() {
            match vertex_type {
                CriticalPointType::Saddle(gradients) => {
                    let targets = self.path_traverse(gradients);

                    targets.iter().for_each(|&target| {
                        // Add an edge from this saddle to every maxima reachable from this saddle
                        self.graph.borrow_mut().edges.push(Edge {
                            source: vertex_id,
                            target
                        })
                    })
                },
                _ => {}
            }
        }
    }

    fn generate_neighbor_idx_set(volume: &VtkVolume) -> Vec<Vec<i8>> {
        let mut neighbor_idx_set: Vec<Vec<i8>> = Vec::new();
        let num_dims = volume.dims.len(); 

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

            neighbor_idx_set.push(bits);
        }

        let mut symmetric_neighbors = vec![];
        for neighbor in neighbor_idx_set.iter() {
            let mut opp_neighbor = vec![];
            for comp in neighbor {
                opp_neighbor.push(-*comp);
            }
            symmetric_neighbors.push(opp_neighbor);
        }

        neighbor_idx_set.append(&mut symmetric_neighbors);
        println!("{:?}", neighbor_idx_set);

        neighbor_idx_set
    }

    fn compute(&mut self) {
        match &self.ext_type {
            ExtGraphImpl::Vtk(vtk_info) => {
                let num_dims = vtk_info.volume.dims.len(); 

                let mut iv = vec![0; num_dims];

                // 1st pass: Point classification
                let mut do_work = true;
                while do_work {
                    if self.is_boundary_point(&iv) {
                        do_work = self.update_iv(&mut iv);
                        continue;
                    }

                    let cur = Vertex::create_vtk_vertex(
                        &iv,
                        vtk_info.volume.clone(),
                        vtk_info.neighbor_idx_set.clone()
                    );

                    self.classify_point(&cur);

                    do_work = self.update_iv(&mut iv);
                }
            },
            ExtGraphImpl::Manifold(manifold_info) => {
                // 1st pass: Point classification
                for id in 0..manifold_info.volume.vertices.len() {
                    let cur = Vertex::create_manifold_vertex(
                        id,
                        manifold_info.volume.clone(),
                        manifold_info.kdtree.as_ref().map(|nn| {
                            nn.clone()
                        })
                    );

                    self.classify_point(&cur);
                }
            }
        }

        // 2nd pass: Gradient arc computation
        self.gradient_ascent();
    }
}

fn get_critical_points(graph: &Graph) -> [usize; 2] {
    let mut maxima = 0;
    let mut saddles = 0;
    for pt in &graph.nodes {
        if pt.color_code == 0 {
            maxima += 1;
        }
        else {
            saddles += 1;
        }
    }

    [maxima, saddles]    
}

fn process_vtk_file(path: &str) -> Result<Graph, String> {
    let volume = read_vtk(path)?;
    
    println!("VTK contents:\nSpacing:{:?}\nDimensions:{:?}\nFirst 3 points: {} {} {}",
            volume.spacing, volume.dims, volume.data[0], volume.data[1], volume.data[2]);

    let total_points = volume.data.len();
    let mut graph = ExtGraph::new_with_vtk(volume);
    graph.compute();

    let [total_maxima, total_saddles] = get_critical_points(&graph.graph.borrow());
    println!("There are total: {} points with {} maxima and {} saddle(s)!", total_points, total_maxima, total_saddles);

    Ok(graph.graph.into_inner())
}

fn process_man_file(path: &str) -> Result<Graph, String> {
    let volume =  read_manifold(path)?;
    let total_points = volume.vertices.len();
    let mut graph_ref = ExtGraph::new_with_manifold(&volume, true);
    graph_ref.compute();

    let mut graph_real = ExtGraph::new_with_manifold(&volume, false);
    graph_real.compute();

    let [total_maxima, total_saddles] = get_critical_points(&graph_real.graph.borrow());
    println!("There are total: {} points with {} maxima and {} saddle(s)!", total_points, total_maxima, total_saddles);

    Ok(graph_real.graph.into_inner())
}

#[tauri::command]
pub async fn process_file_async(path: String) -> Result<Graph, String> {
    tokio::task::spawn_blocking(move || {
        if path.ends_with(".vtk") {
            process_vtk_file(&path)
        }
        else {
            process_man_file(&path)
        }
    })
    .await.expect("Unexpected tokio error!!")
}
