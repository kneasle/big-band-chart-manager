#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod chart_manager;
mod pad_maker;

use eframe::egui::{self, Vec2};
use egui_path_picker::{DefaultIconProvider, PathPicker};
use serde::{Deserialize, Serialize};

use crate::{chart_manager::ChartManager, pad_maker::PadMaker};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Big Band Chart Manager",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            // Read app
            let mut app = Box::<BigBandApp>::default();
            if let Some(storage) = cc.storage {
                if let Some(config_str) = storage.get_string(STORAGE_KEY) {
                    app = serde_json::from_str(&config_str).unwrap();
                }
            }
            Ok(app)
        }),
    )
}

const STORAGE_KEY: &'static str = "big-band-app";

#[derive(Serialize, Deserialize, Debug)]
struct BigBandApp {
    #[serde(default)]
    chart_manager: ChartManager,

    #[serde(default)]
    playlists_dir: String,

    // Active windows
    #[serde(default)]
    is_pad_maker_visible: bool,
    pad_maker: PadMaker,
}

impl Default for BigBandApp {
    fn default() -> Self {
        Self {
            chart_manager: ChartManager::default(),
            playlists_dir: "/".to_owned(),

            is_pad_maker_visible: false,
            pad_maker: PadMaker::default(),
        }
    }
}

impl eframe::App for BigBandApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(STORAGE_KEY, serde_json::to_string(self).unwrap());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Configuration
            ui.heading("Configuration");
            ui.add_space(5.0);
            egui::Grid::new("config-table").show(ui, |ui| {
                // TODO: Use rfd for this
                ui.label("Charts folder:");
                let mut charts_path = self.chart_manager.get_path().to_owned();
                ui.add(PathPicker::<_, DefaultIconProvider>::new(
                    &mut charts_path,
                    &"/mnt/d/Music/Swing band/Current Music Library/",
                ));
                self.chart_manager.update_path(charts_path);
                ui.end_row();

                ui.label("Playlists folder:");
                ui.add(PathPicker::<_, DefaultIconProvider>::new(
                    &mut self.playlists_dir,
                    &"/",
                ));
                ui.end_row();
            });

            // Actions
            ui.add_space(20.0);
            ui.heading("Import New Chart");
            ui.add_space(5.0);
            if ui.button("TODO: From Combined PDF").clicked() {
                println!("TODO: Implement chart import");
            }
            if ui.button("TODO: From Folder of PDFs").clicked() {
                println!("TODO: Implement chart import");
            }

            // Instrument pad making
            ui.add_space(20.0);
            ui.heading("Instrument Pads");
            ui.add_space(5.0);
            if ui.button("Open Pad Maker").clicked() {
                self.is_pad_maker_visible = true;
            }
        });

        if self.is_pad_maker_visible {
            egui::Window::new("Pad Maker")
                .open(&mut self.is_pad_maker_visible)
                .scroll([true, true])
                .default_size(Vec2::new(400.0, 300.0))
                .show(ctx, |ui| self.pad_maker.show(ui, &mut self.chart_manager));
        }
    }
}
