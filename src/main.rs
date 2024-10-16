use petgraph::{csr::DefaultIx, graph::NodeIndex, Graph};
use regex::Regex;
use walkdir::WalkDir;
use std::{collections::{HashMap, HashSet}, fs::{self, FileType}, hash::Hash, path::Path};

use cmd_lib::run_cmd;
use anyhow::Result;

#[derive(Hash, Clone, Eq, PartialEq)]
struct SequenceName(pub String);

/// A graph that consumes more space than a regular graph,
/// but allows for O(1) lookups of nodes, without
/// having to instantiate a large matrix node
struct SpaceConsumingGraph<T> {
    graph: Graph<T, ()>,
    map: HashMap<T, NodeIndex<DefaultIx>>
}

impl<T: Hash + Eq + Clone> SpaceConsumingGraph<T> {
    fn new() -> Self {
        Self {
            graph: Graph::new(),
            map: HashMap::new()
        }
    }

    fn get_node_or_add(&mut self, name: T) -> NodeIndex<DefaultIx> {
        self.map.get(&name).copied().or_else(|| {
            let index = self.graph.add_node(name.clone());
            self.map.insert(name, index);
            Some(index)
        }).unwrap().clone()
    }

    fn add_edge(&mut self, first: T, second: T) {
        let first_idx = self.get_node_or_add(first);
        let second_idx = self.get_node_or_add(second);

        // yes, we are allowing non-simple graphs!
        // we have directional edges
        self.graph.add_edge(first_idx, second_idx, ());
    }
}

fn main() -> Result<()> {
    let sequence_name_matching_regex = Regex::new(r"A\d{6,}").unwrap();

    if !Path::new("./output").exists() {
        // Note: this requires that git-lfs is installed.
        // From https://stackoverflow.com/a/60729017/7589775
        run_cmd! (
            git clone --no-checkout --depth=1 "https://github.com/oeis/oeisdata.git" ./output;
            cd output;
            git sparse-checkout init --cone;
            git sparse-checkout set seq;
            git checkout;
        )?;
    }

    let mut graph = SpaceConsumingGraph::<SequenceName>::new();

    for entry in WalkDir::new("./output/seq") {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            // great! found a node: we will first add it
            // if it doesn't already exist
            let path = entry.into_path();
            let sequence_name = SequenceName(path.file_stem().unwrap().to_str().unwrap().to_owned());
            
            // then start connecting it! we are looking for every single instance of A(6+ digits)
            let contents = fs::read_to_string(path).unwrap();

            for connected_sequence in sequence_name_matching_regex.captures_iter(&contents) {
                let (sub_sequence_name, _) = connected_sequence.extract::<0>();

                graph.add_edge(sequence_name.clone(), SequenceName(sub_sequence_name.to_owned()));
            }
        }
    }

    Ok(())
}
