use std::path::Path;

use cmd_lib::run_cmd;
use anyhow::Result;

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

    

    Ok(())
}
