#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod chart_manager;
mod pad_maker;
mod playlist_manager;
mod utils;

use eframe::egui::{self, Color32, Vec2};
use egui_autocomplete::AutoCompleteTextEdit;
use egui_path_picker::{DefaultIconProvider, PathPicker};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{chart_manager::ChartManager, pad_maker::PadMaker, playlist_manager::PlaylistManager};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Big Band Chart-o-matic",
        options,
        Box::new(|cc| {
            // Theming
            catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::LATTE);

            // Read app
            let mut app = Box::<BigBandApp>::default();
            if let Some(storage) = cc.storage {
                if let Some(config_str) = storage.get_string(STORAGE_KEY) {
                    app = serde_json::from_str(&config_str).unwrap();
                    app.chart_manager.refresh_cache();
                }
            }

            let setlist = app
                .playlist_manager
                .get_playlist_by_name("WSM Rugby Club - 15 May 26")
                .unwrap()
                .read_setlist(&mut app.chart_manager)
                .unwrap();
            dbg!(setlist);

            Ok(app)
        }),
    )
}

const STORAGE_KEY: &'static str = "big-band-app";

#[derive(Serialize, Deserialize, Debug, Default)]
struct BigBandApp {
    // Data under the 'Config' label
    chart_manager: ChartManager,
    playlist_manager: PlaylistManager,

    // Other GUI elements
    pad_maker_gui_playlist: String,
    pad_maker_gui_part: String,

    // Active windows
    is_pad_maker_visible: bool,
    pad_maker: PadMaker,
}

impl eframe::App for BigBandApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(STORAGE_KEY, serde_json::to_string(self).unwrap());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(1.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Configuration");
            ui.add_space(5.0);
            self.draw_config_section(ui);

            // // Imports
            // ui.add_space(20.0);
            // ui.heading("Import New Chart");
            // ui.add_space(5.0);
            // if ui.button("TODO: From Combined PDF").clicked() {
            //     println!("TODO: Implement chart import");
            // }
            // if ui.button("TODO: From Folder of PDFs").clicked() {
            //     println!("TODO: Implement chart import");
            // }

            ui.add_space(20.0);
            ui.heading("Make Pad");
            ui.add_space(5.0);
            self.draw_pad_making_section(ui);
        });

        if self.is_pad_maker_visible {
            egui::Window::new("Pad-o-matic")
                .open(&mut self.is_pad_maker_visible)
                .scroll([true, true])
                .default_size(Vec2::new(400.0, 300.0))
                .show(ctx, |ui| self.pad_maker.show(ui, &mut self.chart_manager));
        }
    }
}

impl BigBandApp {
    fn draw_config_section(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("config-table").show(ui, |ui| {
            // TODO: Use rfd for this
            ui.label("Charts folder:");
            let mut charts_path = self.chart_manager.get_path().to_owned();
            ui.add(PathPicker::<_, DefaultIconProvider>::new(
                &mut charts_path,
                &"/mnt/d/Music/Swing band/Current Music Library",
            ));
            self.chart_manager.update_path(charts_path);
            ui.end_row();

            ui.label("Playlists folder:");
            let mut playlists_path = self.playlist_manager.get_path().to_owned();
            ui.add(PathPicker::<_, DefaultIconProvider>::new(
                &mut playlists_path,
                &"/mnt/d/Music/Swing band/Playlists",
            ));
            self.playlist_manager.update_path(playlists_path);
            ui.end_row();
        });
    }

    fn draw_pad_making_section(&mut self, ui: &mut egui::Ui) {
        if ui.button("Open Pad-o-matic").clicked() {
            self.is_pad_maker_visible = true;
        }
        ui.horizontal(|ui| {
            ui.label("Make pad for playlist:");
            let playlists = self
                .playlist_manager
                .get_playlists()
                .iter()
                .filter_map(|p| p.get_name())
                .collect_vec();
            ui.add(
                AutoCompleteTextEdit::new(&mut self.pad_maker_gui_playlist, playlists)
                    .width(300.0)
                    .popup_on_focus(true)
                    .max_suggestions(10),
            );
            egui::ComboBox::new("pad-maker-part", "")
                .selected_text(&self.pad_maker_gui_part)
                .width(100.0)
                .show_ui(ui, |ui| {
                    let parts = ChartManager::default_part_list();
                    crate::utils::show_part_dropdown_gui(ui, &mut self.pad_maker_gui_part, parts);
                });
            match self
                .playlist_manager
                .get_playlist_by_name(&self.pad_maker_gui_playlist)
            {
                Some(playlist) => {
                    if ui.button("Go!").clicked() {
                        let setlist = playlist.read_setlist(&mut self.chart_manager);
                        match setlist {
                            Ok(setlist) => {
                                self.pad_maker
                                    .set_setlist(&setlist, &self.pad_maker_gui_part);
                                self.is_pad_maker_visible = true;
                            }
                            Err(e) => {
                                // TODO: Better error handling
                                println!("Error reading docx: {}", e);
                            }
                        }
                    }
                }
                None => {
                    ui.colored_label(Color32::RED, "Playlist doesn't exist");
                }
            }
        });
    }
}
