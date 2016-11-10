extern crate capnp;
extern crate petgraph;

use petgraph::Graph;
use petgraph::visit::EdgeRef;

pub mod graph_capnp {
    include!(concat!(env!("OUT_DIR"), "/graph_capnp.rs"));
}
use graph_capnp::weighted_directed_graph as graph;
use capnp::serialize_packed;

use std::io::{BufReader, BufWriter, Error};
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

pub fn write_graph(p: &str, tag: &str, g: &Graph<(), f32>) -> Result<(), Error> {
    let f = try!(File::create(p));
    let mut writer = BufWriter::new(f);

    // building the capnp message
    let mut message = ::capnp::message::Builder::new_default();

    /* build graph */ {
        let mut graph = message.init_root::<graph::Builder>();
        graph.set_tag(tag);
        graph.set_num_nodes(g.node_count() as u32);

        /* collect edges */ {
            let mut edges_msg = graph.borrow().init_edges(g.edge_count() as u32);
            for (i, edgeref) in g.edge_references().enumerate() {
                let mut edge = edges_msg.borrow().get(i as u32);
                edge.set_from(edgeref.source().index() as u32);
                edge.set_to(edgeref.target().index() as u32);
                edge.set_weight(*edgeref.weight());
            }
        }
    }
    serialize_packed::write_message(&mut writer, &message)
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
