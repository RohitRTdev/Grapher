use serde::{Serialize};
use crate::{manifold::*, vertex::*, vtk::*, similarity::*};
use std::{cell::RefCell, collections::{HashMap, HashSet, VecDeque}, rc::Rc, sync::{Mutex, atomic::{AtomicBool, Ordering}}};
use rand::{rngs::ThreadRng, Rng};
use cubecl::wgpu::WgpuRuntime;
use fast_umap::prelude::*;
use std::time::Instant;

const SIMPLICITY_RANDOMNESS: f64 = 0.6;
pub const MAXIMA: u8 = 0;
pub const SADDLE: u8 = 1;

type MyBackend = burn::backend::wgpu::CubeBackend<WgpuRuntime, f32, i32, u32>;
type MyAutodiffBackend = burn::backend::Autodiff<MyBackend>;

static RETRIEVE_REF_GRAPH: AtomicBool = AtomicBool::new(false);
static LAST_GRAPH: Mutex<GraphInfo> = Mutex::new(GraphInfo::new());

struct GraphInfo {
    manifold: Manifold,
    graph_ref: Graph,
    graph_real: Graph
}

impl GraphInfo {
    const fn new() -> Self {
        Self {
            manifold: Manifold {
                embedding_dim: 0,
                vertices: Vec::new(),
                values: Vec::new(),
                graph: AdjList {
                    neighbors: Vec::new()
                }
            },
            graph_ref: Graph::new(),
            graph_real: Graph::new()
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Node {
    pub id: usize,
    pub color_code: u8,
    pub fn_val: f64,
    x: f64,
    y: f64,
    z: f64
}

#[derive(Serialize, Clone)]
pub struct Edge {
    pub source: usize,
    pub target: usize,
}

#[derive(Serialize, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Serialize)]
pub struct FinalResult {
    graph: Graph,
    time: f64,
    memory: usize,
    accuracy: f64,
    f1score: f64,
    is_vtk: bool
}

impl Graph {
    const fn new() -> Self {
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
    rng: RefCell<ThreadRng>,
    vertices: RefCell<Vec<Vec<f64>>>
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
            rng: RefCell::new(rng),
            vertices: RefCell::new(Vec::new())
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
            rng: RefCell::new(rng),
            vertices: RefCell::new(Vec::new())
        }
    }

    fn classify_point(&self, cur: &Vertex, is_ref_mode: bool) {
        let mut upper_link: HashMap<usize, Vec<Vertex>> = HashMap::new();
    
        let fv = cur.fn_val;
        
        let mut highest_vertex_val = fv;
        let mut highest_vertex_id = None;

        let mut new_id = 0;
        for nv in cur.get_neighbors() {
            let fnv = nv.fn_val; 
            
            // We only care about the upper link
            // A basic implementation of simulation of simplicity
            if fnv > fv || (fnv == fv && self.rng.borrow_mut().gen_bool(SIMPLICITY_RANDOMNESS) && !is_ref_mode) {
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
                
                if vertex.len() > 3{
                    self.vertices.borrow_mut().push(vertex); 
                }
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

                    targets.iter().for_each(|target| {
                        // Add an edge from this saddle to every maxima reachable from this saddle
                        self.graph.borrow_mut().edges.push(Edge {
                            source: vertex_id,
                            target: *target
                        });
                    });
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
        let dims = match &self.ext_type {
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

                    self.classify_point(&cur, false);

                    do_work = self.update_iv(&mut iv);
                }

                num_dims
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

                    self.classify_point(&cur, !manifold_info.kdtree.is_some());
                }

                manifold_info.volume.embedding_dim
            }
        };

        // 2nd pass: Gradient arc computation
        self.gradient_ascent();


        // Project vertices to lower dimensions
        if dims > 3 {
            let config = UmapConfig {
                n_components: 3,
                ..UmapConfig::default()
            };
            let umap = fast_umap::Umap::<MyAutodiffBackend>::new(config);
            let fitted = umap.fit(self.vertices.borrow().clone(), None);

            // Get embedding
            let embeddings = fitted.embedding(); 
            
            for (node, node_3d) in self.graph.borrow_mut().nodes.iter_mut().zip(embeddings.iter()) {
                node.x = node_3d[0];
                node.y = node_3d[1];
                node.z = node_3d[2];
            }
        }
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

fn process_vtk_file(path: &str) -> Result<FinalResult, String> {
    let volume = read_vtk(path)?;
    
    println!("VTK contents:\nSpacing:{:?}\nDimensions:{:?}\nFirst 3 points: {} {} {}",
            volume.spacing, volume.dims, volume.data[0], volume.data[1], volume.data[2]);

    let total_points = volume.data.len();
    let mut graph = ExtGraph::new_with_vtk(volume);
    
    let start = Instant::now();
    graph.compute();
    let time = start.elapsed().as_secs_f64() * 1000.0;

    let [total_maxima, total_saddles] = get_critical_points(&graph.graph.borrow());
    println!("There are total: {} points with {} maxima and {} saddle(s)!", total_points, total_maxima, total_saddles);

    let res = FinalResult {
        graph: graph.graph.into_inner(),
        time,
        memory: 0,
        accuracy: 100.0,
        f1score: 100.0,
        is_vtk: true
    };

    *LAST_GRAPH.lock().unwrap() = GraphInfo::new();

    Ok(res)
}

fn process_man_file(path: Option<&str>) -> Result<FinalResult, String> {
    if path.is_some() {
        let man = read_manifold(path.unwrap())?;
        LAST_GRAPH.lock().unwrap().manifold = man;
    }

    let mut guard = LAST_GRAPH.lock().unwrap();
    let volume = &guard.manifold;

    let total_points = volume.vertices.len();
    let mut graph_ref = ExtGraph::new_with_manifold(volume, true);
    graph_ref.compute();

    let mut graph_real = ExtGraph::new_with_manifold(volume, false);

    let start = Instant::now();
    graph_real.compute();

    let time = start.elapsed().as_secs_f64() * 1000.0;
    let mut memory = 0;
    volume.graph.neighbors.iter().for_each(|neighbor| {
        memory += neighbor.len() * size_of::<usize>();
    });

    let graph1 = graph_ref.graph.borrow().clone();
    let graph2= graph_real.graph.borrow().clone();
    let accuracy = get_accuracy(&graph1, &graph2);
    let f1score = get_f1_score(&graph1, &graph2);

    let [total_maxima, total_saddles] = get_critical_points(&graph2);
    println!("There are total: {} points with {} maxima and {} saddle(s)!", total_points, total_maxima, total_saddles);

    let final_graph = if RETRIEVE_REF_GRAPH.load(Ordering::Relaxed) {
        graph_ref.graph.into_inner()
    }
    else {
        graph_real.graph.into_inner()
    };

    guard.graph_ref = graph1;
    guard.graph_real = graph2;

    let res = FinalResult {
        graph: final_graph,
        time,
        memory,
        accuracy,
        f1score,
        is_vtk: false
    };

    Ok(res)
}

#[tauri::command]
pub async fn process_file_async(path: String) -> Result<FinalResult, String> {
    tokio::task::spawn_blocking(move || {
        if path.ends_with(".vtk") {
            process_vtk_file(&path)
        }
        else {
            process_man_file(Some(&path))
        }
    })
    .await.expect("Unexpected tokio error!!")
}

#[tauri::command]
pub fn retrieve_last_graph() -> Graph {
    let container = LAST_GRAPH.lock().unwrap();
    let graph = if RETRIEVE_REF_GRAPH.load(Ordering::Relaxed) {
        container.graph_ref.clone()
    }
    else {
        container.graph_real.clone()
    };

    graph
}

#[tauri::command]
pub async fn recompute_graph() -> Result<FinalResult, String> {
    tokio::task::spawn_blocking(move || {
        process_man_file(None)
    })
    .await.expect("Unexpected tokio error!!")
}

#[tauri::command]
pub fn set_graph_mode(show_true_graph: bool) {
    RETRIEVE_REF_GRAPH.store(show_true_graph, Ordering::Relaxed);
}

