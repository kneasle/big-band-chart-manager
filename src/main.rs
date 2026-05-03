#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod pad_maker;

use eframe::egui::{self, Vec2};
use egui_path_picker::{DefaultIconProvider, PathPicker};
use serde::{Deserialize, Serialize};

use crate::pad_maker::PadMaker;

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
    config: Config,

    // Active windows
    pad_maker: Option<PadMaker>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    charts_dir: String,
    playlists_dir: String,
}

impl Default for BigBandApp {
    fn default() -> Self {
        Self {
            config: Config {
                charts_dir: "/mnt/d/Music/Swing band/Current Music Library/".to_owned(),
                playlists_dir: "/".to_owned(),
            },

            pad_maker: None,
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
                ui.add(PathPicker::<_, DefaultIconProvider>::new(
                    &mut self.config.charts_dir,
                    &"/mnt/d/Music/Swing band/Current Music Library/",
                ));
                ui.end_row();

                ui.label("Playlists folder:");
                ui.add(PathPicker::<_, DefaultIconProvider>::new(
                    &mut self.config.playlists_dir,
                    &"~",
                ));
                ui.end_row();
            });

            // Actions
            ui.add_space(20.0);
            ui.heading("Actions");
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Import New Chart:");
                if ui.button("From Combined PDF").clicked() {
                    println!("TODO: Implement chart import");
                }
                if ui.button("From Folder of PDFs").clicked() {
                    println!("TODO: Implement chart import");
                }
            });
            if ui.button("Make Instrument Pad").clicked() {
                if self.pad_maker.is_none() {
                    self.pad_maker = Some(PadMaker::new());
                }
            }
        });

        if let Some(pad_maker) = &mut self.pad_maker {
            let mut is_open = true;
            egui::Window::new("Pad Maker")
                .open(&mut is_open)
                .scroll([true, true])
                .default_size(Vec2::new(400.0, 300.0))
                .show(ctx, |ui| pad_maker.show(ui));

            // Close the pad maker if user hits the 'x' button
            if !is_open {
                self.pad_maker = None;
            }
        }
    }
}
