extern crate capnp;
extern crate petgraph;

use petgraph::Graph;
use petgraph::visit::EdgeRef;

pub mod graph_capnp {
    include!(concat!(env!("OUT_DIR"), "/graph_capnp.rs"));
}
use graph_capnp::{graph_header, edge};
use capnp::serialize_packed;

use std::io::{BufReader, BufWriter, Error};
use std::fs::File;

pub fn load_graph(p: &str) -> capnp::Result<Graph<(), f32>> {
    let f = try!(File::open(p));
    let mut reader = BufReader::new(f);
    let opts = ::capnp::message::ReaderOptions::new();
    let msg = try!(serialize_packed::read_message(&mut reader, opts));
    let header_root: graph_header::Reader = msg.get_root().unwrap();


    let edges = (0..header_root.get_num_edges()).map(|_| {
        let msg = serialize_packed::read_message(&mut reader, opts).unwrap();
        let edge_root: edge::Reader = msg.get_root().unwrap();

        (edge_root.get_from(), edge_root.get_to(), edge_root.get_weight())
    });

    Ok(Graph::from_edges(edges))
}

pub fn write_graph(p: &str, tag: &str, g: &Graph<(), f32>) -> Result<(), Error> {
    let f = try!(File::create(p));
    let mut writer = BufWriter::new(f);

    /* write header */ {
        let mut message = ::capnp::message::Builder::new_default();
        {
            let mut header = message.init_root::<graph_header::Builder>();
            header.set_tag(tag);
            header.set_num_nodes(g.node_count() as u32);
            header.set_num_edges(g.edge_count() as u64);
        }
        try!(serialize_packed::write_message (&mut writer, &message));
    }

    for edge in g.edge_references() {
        let mut message = ::capnp::message::Builder::new_default();
        {
            let mut em = message.init_root::<edge::Builder>();
            em.set_from(edge.source().index() as u32);
            em.set_to(edge.target().index() as u32);
            em.set_weight(*edge.weight());
        }
        try!(serialize_packed::write_message (&mut writer, &message));
    }
    Ok(())
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
