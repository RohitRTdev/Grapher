use crate::graph::SaddleCache;
use tda::persistence_diagram::{PersistenceDiagram, bottleneck_distance};

const INF_PERSISTENCE_VALUE: f64 = 10000.0;

// Small note here: We cannot compute the real persistence diagram from just the extremum graph
// That requires the morse smale complex. The computation here gives an approximation of the diagram.
// Our goal is not to compute the persistence diagram but rather a good enough topological 
// description of the underlying graphs so that it can be compared. 
fn build_persistence_diagram(mut cache: SaddleCache) -> PersistenceDiagram {
    let mut persistence_pairs = Vec::new();

    cache.iter_mut()
    .for_each(|(_, (saddle_fn_val, maximas))| {
        // For each saddle, add a persistence pair as (Saddle fn val, Maxima fn val)
        // Repeat this for all saddles
        if maximas.len() != 0 {
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
                persistence_pairs.push((*saddle_fn_val, *maxima, 0));
            })
        }
        else {
            // We treat this saddle as creator whose high dimension homology cycle never gets destroyed
            persistence_pairs.push((*saddle_fn_val, INF_PERSISTENCE_VALUE, 0));
        }
    });


    PersistenceDiagram {
        points: persistence_pairs
    }
}

pub fn get_accuracy(graph1_cache: SaddleCache, graph2_cache: SaddleCache) -> f64 {
    let diag1 = build_persistence_diagram(graph1_cache);
    let diag2 = build_persistence_diagram(graph2_cache);

    let start = std::time::Instant::now();
    let dist = bottleneck_distance(&diag1, &diag2, None).unwrap();
    println!("Distance computation: {}s", start.elapsed().as_secs_f64());
    (1.0 / (1.0 + dist)) * 100.0
}

