use crate::graph::{Graph, SaddleCache, SADDLE};
use tda::persistence_diagram::{PersistenceDiagram, bottleneck_distance};

const INF_PERSISTENCE_VALUE: f64 = 10000.0;

fn build_persistence_diagram(graph: &Graph, mut cache: SaddleCache) -> PersistenceDiagram {
    let mut persistence_pairs = Vec::new();

    let mut nodes = graph.nodes.clone();
    nodes.sort_by(|a, b| a.fn_val.partial_cmp(&b.fn_val).unwrap());

    nodes.iter()
    .filter(|node| {
        node.color_code == SADDLE
    })
    .for_each(|node| {
        // For each saddle, add a persistence pair as (Saddle fn val, Maxima fn val)
        // Repeat this for all saddles
        if let Some(maximas) = cache.get_mut(&node.id) {
            maximas.sort_by(|a, b| a.partial_cmp(b).unwrap());
            // Create a persistence pair for each edge from this saddle to a maxima in ascending order of maxima fn values
            // The code is written to tackle multi-saddles too.
            // Create persistence pair for every edge except from saddle to the highest maxima
            
            // If a saddle is connected only to a single maxima, then create persistence pair
            let num_maximas = if maximas.len() == 1 {
                1
            }
            else {
                maximas.len() - 1
            };
            
            maximas.iter()
            .take(num_maximas)
            .for_each(|maxima| {
                persistence_pairs.push((node.fn_val, *maxima, 0));
            })
        }
        else {
            // We treat this saddle as creator whose high dimension homology cycle never gets destroyed
            persistence_pairs.push((node.fn_val, INF_PERSISTENCE_VALUE, 0));
        }
    });


    PersistenceDiagram {
        points: persistence_pairs
    }
}

pub fn get_accuracy(graph1: &Graph, graph1_cache: SaddleCache, graph2: &Graph, graph2_cache: SaddleCache) -> f64 {
    let diag1 = build_persistence_diagram(graph1, graph1_cache);
    let diag2 = build_persistence_diagram(graph2, graph2_cache);

    let dist = bottleneck_distance(&diag1, &diag2, None).unwrap();

    (1.0 / (1.0 + dist)) * 100.0
}
