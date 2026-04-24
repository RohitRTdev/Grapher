use linfa_nn::NearestNeighbourIndex;
use std::rc::Rc;

use crate::{manifold::*, vtk::VtkVolume};

struct VtkVertexInfo {
    iv: Vec<usize>,
    vtk_volume: Rc<VtkVolume>,
    neighbor_offset: Rc<Vec<Vec<i8>>>
} 

struct ManifoldVertexInfo<'a, 'b> {
    manifold: Rc<&'a Manifold>,
    kdtree: Option<&'a Box<dyn NearestNeighbourIndex<f64> + 'b>>
} 

enum VertexType<'a, 'b> {
    Vtk(VtkVertexInfo),
    Manifold(ManifoldVertexInfo<'a, 'b>)
}

pub struct Vertex<'a, 'b> {
    pub id: usize, 
    pub fn_val: f64,
    vtype: VertexType<'a, 'b>
}

impl<'a, 'b> Vertex<'a, 'b> {
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
        kdtree: Option<&'a Box<dyn NearestNeighbourIndex<f64> + 'b>>
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
                // In this case, we just get the neighbor information directly from the adjacency list
                for &neighbor in vert_info.manifold.graph.neighbors[self.id].iter() {
                    let nv = Self::create_manifold_vertex(
                        neighbor,
                        vert_info.manifold.clone(),
                        vert_info.kdtree
                    );
                    neighbors.push(nv);
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

                vert_info.manifold.graph.neighbors[other.id].contains(&self.id)
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

