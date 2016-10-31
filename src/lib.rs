extern crate capnp;
extern crate petgraph;

use petgraph::Graph;

pub mod graph_capnp {
    include!(concat!(env!("OUT_DIR"), "/graph_capnp.rs"));
}
use graph_capnp::weighted_directed_graph as graph;
use capnp::serialize_packed;

use std::io::BufReader;
use std::fs::File;

pub fn load_graph(p: &str) -> capnp::Result<Graph<(), f32>> {
    let f = try!(File::open(p));
    let mut reader = BufReader::new(f);
    let msg = try!(serialize_packed::read_message(&mut reader,
                                                  ::capnp::message::ReaderOptions::new()));

    let root: graph::Reader = msg.get_root().unwrap();
    let edges = root.borrow().get_edges().unwrap().iter()
        .map(| edge | (edge.get_from(), edge.get_to(), edge.get_weight()));

    Ok(Graph::from_edges(edges))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_petgraph() {
        let g = load_graph("data/bin/ca-GrQc.bin").unwrap();
        assert!(g.node_count() == 5242);
    }
}
