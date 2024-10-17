# graph-vis

web-facing software for graph visualization. 

Templated from [eframe_template](https://github.com/emilk/eframe_template/), visualization from [egui_graphs](https://github.com/blitzarx1/egui_graphs).

## Testing locally

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`
