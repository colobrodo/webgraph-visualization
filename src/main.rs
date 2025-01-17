use clap::command;

use std::{
    collections::{HashSet, VecDeque},
    path::PathBuf,
};
use webgraph::prelude::*;

fn lerp(color_a: (u8, u8, u8), color_b: (u8, u8, u8), k: f32) -> (u8, u8, u8) {
    let (ra, ga, ba) = color_a;
    let (rb, gb, bb) = color_b;
    let r = (ra as f32 + (rb as i32 - ra as i32) as f32 * k) as u8;
    let g = (ga as f32 + (gb as i32 - ga as i32) as f32 * k) as u8;
    let b = (ba as f32 + (bb as i32 - ba as i32) as f32 * k) as u8;
    (r, g, b)
}

/// Traverse a webgraph in a bfs and visualize it using rerun, the depth of the visit is limited by the parameter max-depth
#[derive(clap::Parser, Debug)]
#[clap(name = "webgraph-visualize", version)]
struct Args {
    #[command(flatten)]
    rerun: rerun::clap::RerunArgs,
    /// The basename of the graph to visualize
    basename: PathBuf,
    /// The starting node of the visit
    #[arg(short, long, default_value = "1")]
    start_node: usize,
    /// The maximum visit depth
    #[arg(short, long, default_value = "6")]
    max_depth: usize,
    /// The maximum visit depth
    #[arg(long, default_value = "false")]
    with_labels: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser as _;
    let args = Args::parse();

    let graph = BvGraph::with_basename(&args.basename)
        .mode::<Mmap>()
        .flags(MemoryFlags::TRANSPARENT_HUGE_PAGES | MemoryFlags::RANDOM_ACCESS)
        .load()?;

    let green = (0, 255, 0);
    let red = (255, 0, 0);

    let mut depth = 0;
    let mut nodes = VecDeque::new();
    let mut seen = HashSet::new();
    let mut node_labels = Vec::new();
    let mut edges = Vec::new();
    let mut colors = vec![green];
    nodes.push_back(args.start_node);
    node_labels.push(args.start_node.to_string());
    while !nodes.is_empty() && depth < args.max_depth {
        depth += 1;
        let k = (args.max_depth - depth) as f32 / args.max_depth as f32;
        let level_color = lerp(green, red, 1.0 - k);

        let level_size = nodes.len();
        for _ in 0..level_size {
            let node = nodes.pop_front().unwrap();
            if seen.contains(&node) {
                continue;
            }
            seen.insert(node);
            let src_label = node.to_string();
            for successor in graph.successors(node) {
                let label = successor.to_string();
                if !seen.contains(&successor) {
                    nodes.push_back(successor);
                    node_labels.push(label.clone());
                    edges.push((src_label.clone(), label));
                    colors.push(level_color);
                }
            }
        }
    }

    let edges_iter = edges
        .iter()
        .map(|(src, dest)| (src.as_str(), dest.as_str()));

    let (rec, _serve_guard) = args.rerun.init("webgraph_visualize")?;

    let mut graph_nodes =
        rerun::GraphNodes::new(node_labels.iter().map(|s| s.to_owned())).with_colors(colors);
    if args.with_labels {
        graph_nodes = graph_nodes.with_labels(node_labels.iter().map(|s| s.to_owned()));
    }
    rec.log(
        "graph",
        &[
            &graph_nodes as &dyn rerun::AsComponents,
            &rerun::GraphEdges::new(edges_iter).with_directed_edges(),
        ],
    )?;

    Ok(())
}
