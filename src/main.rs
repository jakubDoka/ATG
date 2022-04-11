#![feature(let_else)]
#![feature(result_into_ok_or_err)]
#![feature(int_log)]

use std::{fmt::Write};

use loader::{Edge, Graph};

mod loader;

fn main() {
    let mut args = std::env::args().skip(1);

    let (
        Some(algorithm),
        Some(graph_file),
    ) = (
        args.next(),
        args.next(),
    ) else {
        eprintln!("usage: <algorithm> <graph_file> <...>");
        return;
    };

    let graph = match Graph::new(&graph_file) {
        Ok(graph) => graph,
        Err(err) => {
            println!("Unable to load graph: {}", err);
            return;
        }
    };

    match algorithm.as_str() {
        "label-set" => run_label_set(args, &graph),
        "kruskal" => run_kruskal(&graph),
        "monotone-ordering" => run_monotone_ordering(&graph),
        _ => eprintln!("unknown algorithm: {}", algorithm),
    }
}

fn run_label_set(mut args: impl Iterator<Item = String>, graph: &Graph) {
    let (
        Some(Ok(start)),
        Some(Ok(end)),
    ) = (
        args.next().map(|start| start.parse::<usize>()),
        args.next().map(|end| end.parse::<usize>())
    ) else {
        eprintln!("Expected command format: label-set <file> <start: uint> <end: uint>");
        return;
    };

    let now = std::time::Instant::now();
    let Some((path, cost)) = label_set(start, end, &graph) else {
        eprintln!("No path Found between nodes {} and {}.", start, end);
        return;
    };
    println!("Label-Set ran for: {:?}", now.elapsed());

    let mut output = String::with_capacity(
        path.iter()
            .fold(0, |acc, &i| acc + i.log10() as usize + " -> ".len()),
    );

    for &node in path[1..].iter().rev() {
        write!(output, "{} -> ", node).unwrap();
    }
    writeln!(output, "{}", path[0]).unwrap();

    println!("Path with cost {} is:\n{}", cost, output);
}

fn label_set(start: usize, end: usize, graph: &loader::Graph) -> Option<(Vec<usize>, usize)> {
    let mut t = vec![1 << 32isize; graph.hints.len()];
    let mut x = vec![None; graph.hints.len()];
    
    t[start] = 0;

    let mut e = vec![start];

    while let Some(node) = e.pop() {
        for &Edge { to, weight, .. } in graph.children(node) {
            if t[to] > t[node] + weight {
                t[to] = t[node] + weight;
                x[to] = Some(node);
                if to == end {
                    let mut path = vec![];
                    let mut current = Some(end);
                    while let Some(cur) = current {
                        path.push(cur);
                        current = x[cur];
                    }

                    let explored = x.iter().filter(|elem| elem.is_some()).count();
                    println!("Visited {} nodes, that is {}%", explored, explored as f32 / x.len() as f32 * 100.0);

                    return Some((path, t[end]));
                }

                let Err(pos) = e.binary_search_by(|&i| t[to].cmp(&t[i])) else {
                    continue;
                };

                e.insert(pos, to);
            }
        }
    }

    let explored = x.iter().filter(|elem| elem.is_some()).count();
    println!("Visited {} nodes, that is {}%", explored, explored as f32 / x.len() as f32 * 100.0);


    return None;
}

fn run_monotone_ordering(graph: &Graph) {
    let Some(ordering) = monotone_ordering(graph) else {
        eprintln!("No monotone ordering, graph contains cycles.");
        return;
    };

    let mut output = String::with_capacity(
        ordering.iter()
            .fold(0, |acc, &i| acc + i.log10() as usize + " ".len()),
    );

    for &node in ordering.iter() {
        write!(output, "{} ", node).unwrap();
    }

    println!("Monotone ordering is:\n{}", output);
}

fn monotone_ordering(graph: &Graph) -> Option<Vec<usize>> {
    let mut costs = vec![0; graph.hints.len()];
    for edge in &graph.edges {
        costs[edge.to] += 1;
    }

    let mut used = (1..graph.hints.len()).collect::<Vec<_>>();
    let mut ordering = vec![0; graph.hints.len()];
    let mut counter = 0;
    while !used.is_empty() {
        let prev = used.len();
        
        used.retain(|&node| {
            if costs[node] == 0 {
                for &Edge { to, .. } in graph.children(node) {
                    costs[to] -= 1;
                }
                ordering[node] = counter;
                counter += 1;
                true
            } else {
                false
            }
        });
        
        if prev == used.len() {
            return None;
        }
    }

    Some(ordering)
}

fn run_kruskal(graph: &Graph) {
    let Some(edges) = kruskal(graph) else {
        eprintln!("Graph is not continuous.");
        return;
    };

    println!("Found tree: {:?}", edges);
}

fn kruskal(graph: &loader::Graph) -> Option<Vec<Edge>> {
    let mut groups = vec![None; graph.edges.len()];
    let mut sorted_hints: Vec<usize> = (0..graph.edges.len()).collect();
    sorted_hints.sort_by_key(|&i| graph.edges[i].weight);
    let mut result = Vec::with_capacity(graph.hints.len() - 2);
    let mut group_counter = 0;

    for i in sorted_hints {
        let Edge { from, to, .. } = graph.edges[i];
        match (groups[from], groups[to]) {
            (None, None) => {
                groups[from] = Some(group_counter);
                groups[to] = Some(group_counter);
                group_counter += 1;
            },
            (None, Some(b)) => {
                groups[from] = Some(b);
            },
            (Some(a), None) => {
                groups[to] = Some(a);
            },
            (Some(a), Some(b)) => {
                if a != b {
                    let min = std::cmp::min(a, b);
                    let max = std::cmp::max(a, b);
                    for i in 0..groups.len() {
                        if groups[i] == Some(max) {
                            groups[i] = Some(min);
                        }
                    }
                } else {
                    continue;
                }
            },
        }
        result.push(graph.edges[i]);
    }

    if result.len() != graph.hints.len() - 2 {
        return None;
    }

    return Some(result);
}
