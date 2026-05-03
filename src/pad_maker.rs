use std::collections::HashSet;

use eframe::egui;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PadMaker {
    entries: Vec<Entry>,
}

/// An entry within a musician's pad
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct Entry {
    piece: String,
    part: String,
}

impl PadMaker {
    pub fn new() -> Self {
        Self {
            entries: vec![
                Entry {
                    piece: "In the Mood".to_owned(),
                    part: "Trombone 3".to_owned(),
                },
                Entry {
                    piece: "A Few Good Men".to_owned(),
                    part: "Trombone 3".to_owned(),
                },
            ],
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.show_entries_gui(ui);
        ui.add_space(30.0);
        self.show_export_gui(ui);
    }

    fn show_entries_gui(&mut self, ui: &mut egui::Ui) {
        let parts = [
            "Trumpet 1",
            "Trumpet 2",
            "Trumpet 3",
            "Trumpet 4",
            "Trombone 1",
            "Trombone 2",
            "Trombone 3",
            "Trombone 4",
        ];

        ui.heading("Select Parts:");

        let mut entries_to_clone = HashSet::<usize>::new();
        let mut entries_to_delete = HashSet::<usize>::new();
        egui_dnd::dnd(ui, "charts-in-pad").show_vec(
            &mut self.entries,
            |ui, entry, handle, item_state| {
                ui.horizontal(|ui| {
                    handle.ui(ui, |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(" ≡ ").monospace())
                                .selectable(false),
                        );
                    });

                    // TODO: Index of piece within the set
                    egui::TextEdit::singleline(&mut entry.piece)
                        .desired_width(300.0)
                        .show(ui);
                    egui::ComboBox::new(entry as *const _, "")
                        .selected_text(&entry.part)
                        .width(150.0)
                        .show_ui(ui, |ui| {
                            for part in parts {
                                ui.selectable_value(&mut entry.part, part.to_owned(), part);
                            }
                        });

                    // Clone and delete buttons
                    if ui.button("🗐").clicked() {
                        entries_to_clone.insert(item_state.index);
                    }
                    if ui.button("🗑").clicked() {
                        entries_to_delete.insert(item_state.index);
                    }
                });
            },
        );

        // New entry button
        ui.add_space(5.0);
        if ui.button("Add New Part").clicked() {
            self.entries.push(Entry {
                piece: "".to_owned(),
                part: self.get_most_common_part().to_owned(),
            });
        }

        // Delete/clone items
        if !entries_to_clone.is_empty() || !entries_to_delete.is_empty() {
            for (idx, entry) in std::mem::take(&mut self.entries).into_iter().enumerate() {
                if entries_to_delete.contains(&idx) {
                    // Don't re-add this as it's been deleted
                } else if entries_to_clone.contains(&idx) {
                    // Add this entry twice, as it's been cloned
                    self.entries.push(entry.clone());
                    self.entries.push(entry);
                } else {
                    // Neither deleted nor cloned, so just push it once
                    self.entries.push(entry);
                }
            }
        }
    }

    fn show_export_gui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Export");
        if ui.button("Export to Single PDF").clicked() {
            println!("TODO: Export to PDF");
        }
        if ui.button("TODO: Export to Multiple PDFs").clicked() {
            println!("TODO: Export to Multiple PDFs");
        }
    }

    /// Get the part name most commonly used in this pad.
    ///
    /// TODO: Set a tiebreak, e.g. last part mentioned
    fn get_most_common_part(&self) -> &str {
        let part_counts = self.entries.iter().map(|entry| &entry.part).counts();
        let most_common_part = part_counts
            .iter()
            .max_by_key(|&(_part, count)| *count)
            .map(|(part, _count)| part.as_str())
            .unwrap_or("Trombone 1");
        most_common_part
    }
}
