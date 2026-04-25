use crate::graph::Graph;
use std::collections::HashSet;

pub fn get_accuracy(graph1: &Graph, graph2: &Graph) -> f64 {
    // We will measure the jaccard index
    let set1: HashSet<_> = graph1.edges.iter()
        .map(|e| (e.source, e.target))
        .collect();

    let set2: HashSet<_> = graph2.edges.iter()
        .map(|e| (e.source, e.target))
        .collect();

    let intersection = set1.intersection(&set2).count();
    let union = set1.union(&set2).count();

    if union == 0 {
        return 100.0;
    }

    (intersection as f64 / union as f64) * 100.0
}