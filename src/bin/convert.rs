extern crate capnp;
pub mod graph_capnp {
    include!(concat!(env!("OUT_DIR"), "/graph_capnp.rs"));
}
use graph_capnp::weighted_directed_graph;
use capnp::serialize_packed;
extern crate docopt;
use docopt::Docopt;
extern crate rustc_serialize;
use std::path::Path;
use std::io::{BufReader, BufRead, BufWriter};
use std::fs::File;
use std::str;
use std::str::FromStr;

const USAGE: &'static str = "
Convert the input edge-list to a packed binary file readable by Cap'n
Proto.

The input edge-list should have the number of nodes and edges on the
first line, followed by #edges lines of <from> <to> <weight> triplets.

Usage:
  convert <source> <dest> [--tag=<tag>]
  convert (-h | --help)

Options:
  -h --help          Show this screen.
  --tag=<tag>        The tag (name) of the graph. Defaults to the basename of the source.
";

#[derive(RustcDecodable)]
struct Args {
    arg_source: String,
    arg_dest: String,
    flag_tag: Option<String>
}

type Node = (u32, f32, f32);
type Edge = (u32, u32, f32);

fn read_graph(fname: &String) -> Result<(u32, Vec<Edge>), String> {
    let input = File::open(fname).unwrap();

    let mut reader = BufReader::new(input);
    let mut first_line = String::new();
    reader.read_line(&mut first_line).unwrap();

    let first_row = first_line.split_whitespace().map(|s| usize::from_str(s).unwrap()).collect::<Vec<usize>>();
    if first_row.len() != 2 {
        return Err(format!("Header inappropriate length: {}. Expected 2.", first_row.len()));
    }
    assert!(first_row.len() == 2);
    let (num_nodes, num_edges) = (first_row[0], first_row[1]);

    let edges: Vec<Edge> = reader.lines().map(|line| {
        let un = line.unwrap();
        let row = un.split_whitespace().collect::<Vec<&str>>();

        (row[0].parse::<u32>().unwrap(),
         row[1].parse::<u32>().unwrap(),
         row[2].parse::<f32>().unwrap())
    }).collect();

    if edges.len() != num_edges {
        return Err(format!("Unable to read all edges: {}/{} read.", edges.len(), num_edges));
    }

    Ok((num_nodes as u32, edges))
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let (num_nodes, edges) = read_graph(&args.arg_source).unwrap();

    // building the capnp message
    let mut message = ::capnp::message::Builder::new_default();

    /* build graph */ {
        let mut graph = message.init_root::<weighted_directed_graph::Builder>();
        if let Some(tag) = args.flag_tag {
            graph.set_tag(tag.as_str());
        } else {
            graph.set_tag(Path::new(&args.arg_source).file_name()
                          .and_then(|s| s.to_str()).unwrap())
        }

        graph.set_num_nodes(num_nodes);

        /* collect edges */ {
            let mut edges_msg = graph.borrow().init_edges(edges.len() as u32);
            for (i, &(from, to, weight)) in edges.iter().enumerate() {
                let mut edge = edges_msg.borrow().get(i as u32);
                edge.set_from(from);
                edge.set_to(to);
                edge.set_weight(weight);
            }
        }
    }

    let f = File::create(&args.arg_dest).unwrap();
    let mut writer = BufWriter::new(f);
    serialize_packed::write_message(&mut writer, &message).unwrap();
}
