use bimap::BiMap;
use brotlic::CompressorWriter;
use indicatif::ProgressBar;
use petgraph::{csr::DefaultIx, graph::NodeIndex, Graph};
use petgraph_graphml::GraphMl;
use regex::Regex;
use std::{
    cmp::max, fmt::Display, fs::{self, OpenOptions}, hash::Hash, io::Write, path::Path
};
use walkdir::WalkDir;

use anyhow::Result;
use cmd_lib::run_cmd;

#[derive(Hash, Clone, Eq, PartialEq)]
struct SequenceName(pub String);

impl Display for SequenceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A graph that consumes more space than a regular graph,
/// but allows for O(1) lookups of nodes, without
/// having to instantiate a large matrix node
struct SpaceConsumingGraph<T> {
    pub graph: Graph<T, ()>,
    pub map: BiMap<T, NodeIndex<DefaultIx>>,
}

impl<T: Hash + Eq + Clone> SpaceConsumingGraph<T> {
    fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            graph: Graph::with_capacity(nodes, edges),
            map: BiMap::with_capacity(max(nodes, edges)),
        }
    }

    fn get_node_or_add(&mut self, name: T) -> NodeIndex<DefaultIx> {
        self.map
            .get_by_left(&name)
            .copied()
            .or_else(|| {
                let index = self.graph.add_node(name.clone());
                self.map.insert(name, index);
                Some(index)
            })
            .unwrap()
            .clone()
    }

    fn add_edge(&mut self, first: T, second: T) {
        let first_idx = self.get_node_or_add(first);
        let second_idx = self.get_node_or_add(second);

        // yes, we are allowing non-simple graphs!
        // we have directional edges
        self.graph.add_edge(first_idx, second_idx, ());
    }
}

trait U32Representable {
    fn represent(&self) -> u32;
}

impl U32Representable for SequenceName {
    fn represent(&self) -> u32 {
        let prefix_stripped_str: String = self.0.chars().skip(1).collect();

        str::parse::<u32>(&prefix_stripped_str).expect("Could not parse prefix_stripped_str")
    }
}

fn create_rawbin<T: U32Representable + Eq + Hash>(graph: &SpaceConsumingGraph<T>) -> Result<()> {
    println!("Writing rawbin data to output.bin...");

    let nodes = graph.graph.raw_nodes();
    let edges = graph.graph.raw_edges();

    let mut writer = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(Path::new("./output.bin"))?;

    for node in nodes {
        writer.write(&[1u8])?;
        writer.write(&node.weight.represent().to_le_bytes())?;
        write!(writer, "\n")?;
    }

    for edge in edges {
        writer.write(&[0u8])?;
        writer.write(&graph.map.get_by_right(&edge.source()).unwrap().represent().to_le_bytes())?;
        writer.write(&graph.map.get_by_right(&edge.target()).unwrap().represent().to_le_bytes())?;
        write!(writer, "\n")?;
    }

    Ok(())
}

fn create_graphmlz<T: Display>(graph: &SpaceConsumingGraph<T>) -> Result<()> {
    println!("Creating GraphML data...");

    let graph_ml = GraphMl::new(&graph.graph).export_node_weights_display();

    let output_path = Path::new("output.graphmlz");
    println!("Writing compressed form...");

    graph_ml.to_writer(CompressorWriter::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(output_path)?,
    ))?;

    Ok(())
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

    let mut graph = SpaceConsumingGraph::<SequenceName>::with_capacity(400_000, 100_000);

    let progress_bar = ProgressBar::new_spinner();

    let mut count = 0;
    for entry in WalkDir::new("./output/seq") {
        let entry = entry?;
        if entry.file_type().is_file() {
            // great! found a node: we will first add it
            // if it doesn't already exist
            let path = entry.into_path();
            let sequence_name =
                SequenceName(path.file_stem().unwrap().to_str().unwrap().to_owned());

            // then start connecting it! we are looking for every single instance of A(6+ digits)
            let contents = fs::read_to_string(path).unwrap();

            for connected_sequence in sequence_name_matching_regex.captures_iter(&contents) {
                let (sub_sequence_name, _) = connected_sequence.extract::<0>();

                graph.add_edge(
                    sequence_name.clone(),
                    SequenceName(sub_sequence_name.to_owned()),
                );
            }

            progress_bar.inc(1);
            progress_bar.set_message(format!("Parsing {count} sequences..."));
            count += 1;
        }
    }

    create_rawbin(&graph)?;
    create_graphmlz(&graph)?;

    progress_bar.finish_with_message("Done building graph!");

    Ok(())
}
