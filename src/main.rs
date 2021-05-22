#![allow(non_snake_case)]

use Roadgraph::file::read_file;
use Roadgraph::Graph::Graph;

fn main() {
    let mut graph = Graph::new();
    graph.add_links(read_file("testfile.json"));

    graph.breadth_first();
    // println!("{}", graph.print());
}
