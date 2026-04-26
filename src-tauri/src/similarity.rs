use crate::graph::Graph;
use std::collections::HashSet;

pub fn vertex_score(graph1: &Graph, graph2: &Graph) -> f64 {
    let set1: HashSet<_> = graph1.nodes.iter().map(|v| v.id).collect();
    let set2: HashSet<_> = graph2.nodes.iter().map(|v| v.id).collect();

    let intersection = set1.intersection(&set2).count();
    let union = set1.union(&set2).count();

    if union == 0 {
        return 1.0;
    }

    intersection as f64 / union as f64
}


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

    let edge_score = if union == 0 {
        1.0
    }
    else {
        intersection as f64 / union as f64
    };

    let vertex_score = vertex_score(graph1, graph2);

    // Get average of both these scores
    // We include the vertex scores, because edge score was just straight up zero in many cases
    (edge_score + vertex_score) * 50.0
}