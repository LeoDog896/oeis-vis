use std::io::ErrorKind;

use egui_graphs::{Graph, GraphView};
use lz4_flex::frame::FrameDecoder;
use petgraph::prelude::StableGraph;
use anyhow::Result;
use tokio_util::io::StreamReader;
use futures::stream::TryStreamExt;

type SequenceIdentifier = u32;

const BIN_DATA: &str = "https://github.com/LeoDog896/oeis-vis/releases/download/latest/output.bin";

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GraphVisApp {
    // we do not want to serialize this!
    #[serde(skip)]
    graph: Graph<SequenceIdentifier, ()>
}

async fn create_graph() -> Result<StableGraph<SequenceIdentifier, ()>> {
    let stream = reqwest::get(BIN_DATA).await?.bytes_stream().map_err(|err| std::io::Error::new(ErrorKind::Other, "Uh oh!"));
    let reader = StreamReader::new(stream);
    let decoder = FrameDecoder::new(reader);

    Ok()
}

impl Default for GraphVisApp {
    fn default() -> Self {
        Self {
            graph: Graph::from(&StableGraph::with_capacity(400_000, 100_000))
        }
    }
}

impl GraphVisApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for GraphVisApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("OEIS Visualization");

            ui.add(&mut GraphView::new(&mut self.graph));

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/LeoDog896/oeis-vis",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(", ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(", and");
        ui.hyperlink_to(
            "egui graphs",
            "https://github.com/blitzarx1/egui_graphs"
        );
        ui.label(".");
    });
}
