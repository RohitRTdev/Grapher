use ndarray::{ArrayView1, Array2};
use linfa_nn::{CommonNearestNeighbour, NearestNeighbour, distance::L2Dist};
use std::{collections::HashSet, rc::Rc};
use crate::{manifold::*, vtk::VtkVolume};

const KNN: usize = 10;

struct VtkVertexInfo {
    iv: Vec<usize>,
    vtk_volume: Rc<VtkVolume>,
    neighbor_offset: Rc<Vec<Vec<i8>>>
} 

struct ManifoldVertexInfo<'a> {
    manifold: Rc<&'a Manifold>,
    kdtree: Option<Rc<NN>>
} 

enum VertexType<'a> {
    Vtk(VtkVertexInfo),
    Manifold(ManifoldVertexInfo<'a>)
}

pub struct Vertex<'a> {
    pub id: usize, 
    pub fn_val: f64,
    vtype: VertexType<'a>
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

    pub fn create_vtk_vertex(
        iv: &Vec<usize>, 
        vtk_volume: Rc<VtkVolume>,
        neighbor_offset: Rc<Vec<Vec<i8>>>
    ) -> Self {
        let id = Self::compute_id(&iv, &vtk_volume.dims);
        let fn_val = vtk_volume.data[id]; 

        Vertex {
            id,
            fn_val,
            vtype: VertexType::Vtk(VtkVertexInfo { 
                iv: iv.clone(), 
                vtk_volume,
                neighbor_offset 
            })
        }
    }

    pub fn create_manifold_vertex(
        id: usize,
        manifold: Rc<&'a Manifold>,
        kdtree: Option<Rc<NN>>
    ) -> Self {
        let fn_val = manifold.values[id];
        Vertex {
            id,
            fn_val,
            vtype: VertexType::Manifold(ManifoldVertexInfo { 
                manifold,
                kdtree
            }) 
        }
    }

    pub fn get_neighbors(&self) -> Vec<Self> {
        let mut neighbors = vec![];
        match &self.vtype {
            VertexType::Vtk(vert_info) => {
                for neighbor in vert_info.neighbor_offset.iter() {
                    let mut iv = vec![0; vert_info.vtk_volume.dims.len()];
                    for dim in 0..vert_info.vtk_volume.dims.len() {
                        iv[dim] = (vert_info.iv[dim] as isize + neighbor[dim] as isize) as usize;
                    }
                    
                    let nv = Self::create_vtk_vertex(&iv, vert_info.vtk_volume.clone(), vert_info.neighbor_offset.clone());
                    neighbors.push(nv); 
                }
            },
            VertexType::Manifold(vert_info) => {
                if let Some(nn) = &vert_info.kdtree {
                    for &neighbor in nn.neighbor_info.neighbors[self.id].iter() {
                        let nv = Self::create_manifold_vertex(
                            neighbor,
                            vert_info.manifold.clone(),
                            Some(nn.clone())
                        );
                        neighbors.push(nv);
                    }
                }
                else {
                    // In this case, we just get the neighbor information directly from the adjacency list
                    for &neighbor in vert_info.manifold.graph.neighbors[self.id].iter() {
                        let nv = Self::create_manifold_vertex(
                            neighbor,
                            vert_info.manifold.clone(),
                            None
                        );
                        neighbors.push(nv);
                    }
                }
            }
        }

        neighbors
    }

    pub fn is_neighbor(&self, other: &Vertex) -> bool {
        match &self.vtype {
            VertexType::Vtk(vert_info) => {
                let this_iv = &vert_info.iv;
                match &other.vtype {
                    VertexType::Vtk(other_vert_info) => {
                        let other_iv = &other_vert_info.iv;
                        let mut diff = vec![0; vert_info.vtk_volume.dims.len()];
                        for dim in 0..vert_info.vtk_volume.dims.len() {
                            diff[dim] = this_iv[dim] as isize - other_iv[dim] as isize;
                        }
                        
                        diff.iter().all(|&x| x == 0 || x == 1) 
                        || diff.iter().all(|&x| x == 0 || x == -1) 
                    },
                    _ => {
                        panic!("Critical internal error! Vertex types don't match!");
                    }
                }
            },
            VertexType::Manifold(vert_info) => {
                if !matches!(other.vtype, VertexType::Manifold(_)) {
                    panic!("Critical internal error! Vertex types don't match!");
                }

                if let Some(nn) = &vert_info.kdtree {
                    nn.neighbor_info.neighbors[self.id].contains(&other.id)
                }
                else {
                    vert_info.manifold.graph.neighbors[other.id].contains(&self.id)
                }
            }
        }
    }

    pub fn get_vertex(&self) -> Vec<f64> {
        match &self.vtype {
            VertexType::Vtk(vert_info) => {
                let mut vertex = vec![0f64; vert_info.iv.len()];
                for dim in 0..vert_info.iv.len() {
                    vertex[dim] = vert_info.vtk_volume.spacing[dim] * vert_info.iv[vert_info.iv.len() - dim - 1] as f64 
                    // Translate the structure to bring it to center of camera
                    - ((vert_info.vtk_volume.dims[dim] as f64) / 2.0);
                }
    
                vertex
            },
            VertexType::Manifold(vert_info) => {
                vert_info.manifold.vertices[self.id].clone()
            }
        }
    }
}

pub struct NN {
    neighbor_info: AdjList
}

impl NN {
    pub fn new() -> Self {
        Self {
            neighbor_info: AdjList {
                neighbors: Vec::new()
            }
        }
    }

    pub fn build_neighborhood(&mut self, volume: &Manifold) {
        let n = volume.vertices.len();
        let d = volume.embedding_dim;

        self.neighbor_info.neighbors = vec![HashSet::new(); n];

        let flat: Vec<f64> = volume.vertices.iter().flatten().cloned().collect();

        let data = Array2::from_shape_vec((n, d), flat).unwrap(); 
        
        let kdtree = CommonNearestNeighbour::KdTree
            .from_batch(&data, L2Dist)
            .unwrap();

        // Build the entire adjacency list at init
        for id in 0..volume.vertices.len() {
            let point = &volume.vertices[id];
            let query = ArrayView1::from(point.as_slice());
            
            // Use KNN to find the neighbors when we don't have the neighborhood information
            let result = kdtree.k_nearest(query, KNN).unwrap();
            
            result.iter()
            .filter(|(_, idx)| {
                // Exclude providing the same element as its neighbor
                id != *idx
            })
            .for_each(|(_, idx)| {
                self.neighbor_info.neighbors[id].insert(*idx);
            });
        }
    }
}