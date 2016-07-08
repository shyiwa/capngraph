extern crate capnp;
extern crate petgraph;
extern crate bit_set;

pub mod graph_capnp {
    include!(concat!(env!("OUT_DIR"), "/graph_capnp.rs"));
}
use graph_capnp::weighted_directed_graph as graph;
use capnp::serialize_packed;
use petgraph::visit::{Graphlike, NeighborIter, NeighborsDirected, Externals,
                      Visitable, Revisitable, VisitMap};
use petgraph::EdgeDirection;
use bit_set::BitSet;

use std::io::BufReader;
use std::fs::File;

/// A petgraph-compatible Graphlike backed by a Cap'n Proto graph
/// file.
struct CapnGraph {
    store: capnp::message::Reader<capnp::serialize::OwnedSegments>,
    node_cache: BitSet<u32>,
    out_edge_index: Vec<Vec<u32>>,
    in_edge_index: Vec<Vec<u32>>
}

impl CapnGraph {
    pub fn tag(&self) -> &str {
        let root: graph::Reader = self.store.get_root().unwrap();
        root.get_tag().unwrap()
    }

    pub fn node_count(&self) -> usize {
        self.node_cache.len()
    }

    pub fn from_packed_file(p: &str) -> capnp::Result<Self> {
        let f = try!(File::open(p));
        let mut reader = BufReader::new(f);
        let msg = try!(serialize_packed::read_message(&mut reader,
                                                      ::capnp::message::ReaderOptions::new()));

        let (in_edges, out_edges, nodes) = {
            let root: graph::Reader = msg.get_root().unwrap();

            let num_nodes = root.get_num_nodes() as usize;
            let mut in_edges = vec![Vec::new(); num_nodes];
            let mut out_edges = vec![Vec::new(); num_nodes];
            let mut nodes = BitSet::with_capacity(num_nodes);

            let edges = root.borrow().get_edges().unwrap();

            for i in 0..edges.len() {
                let edge = edges.get(i);
                let to = edge.get_to() as usize;
                let from = edge.get_from() as usize;
                in_edges[to].push(from as u32);
                out_edges[from].push(to as u32);
                nodes.insert(to);
                nodes.insert(from);
            }

            (in_edges, out_edges, nodes)
        };

        Ok(CapnGraph {
            store: msg,
            node_cache: nodes,
            in_edge_index: in_edges,
            out_edge_index: out_edges
        })
    }
}

impl Graphlike for CapnGraph {
    type NodeId = u32;
}

struct Nodes<Ix>
    where Ix: Clone {
    internal: Vec<Ix>,
    position: usize
}

impl<Ix> Nodes<Ix>
    where Ix: Clone {
    fn new(data: Vec<Ix>) -> Self {
        Nodes {
            position: 0,
            internal: data,
        }
    }
}

impl<Ix> Iterator for Nodes<Ix>
    where Ix: Clone {
    type Item = Ix;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.internal.len() {
            self.position += 1;
            Some(self.internal[self.position - 1].clone())
        } else {
            None
        }
    }
}

impl<'a> NeighborIter<'a> for CapnGraph {
    type Iter = Nodes<Self::NodeId>;

    fn neighbors(&self, n: Self::NodeId) -> Self::Iter {
        self.neighbors_directed(n, EdgeDirection::Outgoing)
    }
}

impl<'a> NeighborsDirected<'a> for CapnGraph {
    type NeighborsDirected = Nodes<Self::NodeId>;

    fn neighbors_directed(&'a self, n: Self::NodeId, d: EdgeDirection) -> Self::NeighborsDirected {
        let root: graph::Reader = self.store.get_root().unwrap();
        match d {
            EdgeDirection::Outgoing =>
                Nodes::new(self.out_edge_index[n as usize].iter().map(|edge_id| {
                    root.get_edges().unwrap().get(*edge_id).get_to()
                }).collect()),
            EdgeDirection::Incoming =>
                Nodes::new(self.in_edge_index[n as usize].iter().map(|edge_id| {
                    root.get_edges().unwrap().get(*edge_id).get_from()
                }).collect()),
        }
    }
}

impl<'a> Externals<'a> for CapnGraph {
    type Externals = Nodes<Self::NodeId>;

    fn externals(&'a self, d: EdgeDirection) -> Self::Externals {
        match d {
            EdgeDirection::Outgoing =>
                Nodes::new(self.out_edge_index.iter().enumerate().filter_map(|(ix, edges)| {
                    if edges.is_empty() {
                        Some(ix as u32)
                    } else {
                        None
                    }
                }).collect()),
            EdgeDirection::Incoming =>
                Nodes::new(self.in_edge_index.iter().enumerate().filter_map(|(ix, edges)| {
                    if edges.is_empty() {
                        Some(ix as u32)
                    } else {
                        None
                    }
                }).collect()),
        }
    }
}

impl Visitable for CapnGraph {
    type Map = NodeSet;

    fn visit_map(&self) -> Self:: Map {
        NodeSet(BitSet::with_capacity(self.node_count()))
    }
}

impl Revisitable for CapnGraph {
    fn reset_map(&self, map: &mut Self::Map) {
        map.0.clear();
        map.0.reserve_len(self.node_count());
    }
}

/// Wrapper around BitSet to satisfy Rust's orphan rules.
struct NodeSet(BitSet);

impl VisitMap<u32> for NodeSet {
    fn visit(&mut self, n: u32) -> bool {
        !self.0.insert(n as usize)
    }

    fn is_visited(&self, n: &u32) -> bool {
        self.0.contains(*n as usize)
    }
}

#[cfg(test)]
mod test {
    use super::CapnGraph;

    #[test]
    fn reads_graph() {
        let g = CapnGraph::from_packed_file("data/bin/ca-GrQc.bin").unwrap();
        assert!(g.tag() == "ca-GrQc");
    }
}
