use crate::graph::Graph;
use std::collections::HashSet;

pub fn vertex_f1_score(graph1: &Graph, graph2: &Graph) -> f64 {
    let set1: HashSet<_> = graph1.nodes.iter().map(|v| v.id).collect();
    let set2: HashSet<_> = graph2.nodes.iter().map(|v| v.id).collect();

    let t_p = set1.intersection(&set2).count() as f64;
    let f_p = (set1.len() as f64) - t_p;
    let f_n = (set2.len() as f64) - t_p;

    let precision = if t_p + f_p == 0.0 { 
        1.0
    } 
    else { 
        t_p / (t_p + f_p) 
    };
    let recall = if t_p + f_n == 0.0 { 
        1.0 
    } 
    else {
        t_p / (t_p + f_n) 
    };

    if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    }
}

pub fn edge_f1_score(graph1: &Graph, graph2: &Graph) -> f64 {
    let set1: HashSet<_> = graph1.edges.iter()
        .map(|e| (e.source, e.target))
        .collect();

    let set2: HashSet<_> = graph2.edges.iter()
        .map(|e| (e.source, e.target))
        .collect();

    let t_p = set1.intersection(&set2).count() as f64;
    let f_p = (set1.len() as f64) - t_p;
    let f_n = (set2.len() as f64) - t_p;

    let precision = if t_p + f_p == 0.0 {
        1.0
    } else {
        t_p / (t_p + f_p)
    };

    let recall = if t_p + f_n == 0.0 {
        1.0
    } else {
        t_p / (t_p + f_n)
    };

    if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    }
}

pub fn get_f1_score(graph1: &Graph, graph2: &Graph) -> f64 {
    let edge_f1 = edge_f1_score(graph1, graph2);
    let vertex_f1 = vertex_f1_score(graph1, graph2);

    (edge_f1 + vertex_f1) * 50.0
}

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