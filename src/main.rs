#![allow(dead_code, unused_imports, non_snake_case)]

use serde_json::Value;
use std::collections::HashMap;
use Roadgraph::file::read_file;
use Roadgraph::Graph::*;

fn main() {
    let mut nodes: HashMap<String, Roadgraph::Graph::Node> = HashMap::new();
    let mut links: HashMap<String, Roadgraph::Graph::RoadLink> = HashMap::new();

    let mut graph = Roadgraph::Graph::CustomGraph::new();
    graph.add_links(
        read_file("testfile.json"),
        &mut nodes,
        &mut links,
    );
    
    // println!("{}", graph.print(&nodes, &links));
}
