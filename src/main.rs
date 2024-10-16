use petgraph::Graph;
use walkdir::WalkDir;
use std::{fs::{self, FileType}, path::Path};

use cmd_lib::run_cmd;
use anyhow::Result;

struct SequenceName(pub String);

fn main() -> Result<()> {
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

    let mut graph = Graph::<SequenceName, ()>::new();

    for entry in WalkDir::new("./output/seq") {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            // great! found a node: we will first add it
            let path = entry.into_path();
            graph.add_node(SequenceName(path.file_stem().unwrap().to_str().unwrap().to_owned()));
            
            // then start connecting it!
            let contents = fs::read(path).unwrap();
        }
    }

    Ok(())
}
