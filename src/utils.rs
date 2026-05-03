use std::collections::HashSet;

use eframe::egui;
use itertools::Itertools;

pub fn show_part_dropdown_gui(
    ui: &mut egui::Ui,
    part_var: &mut String,
    mut parts: HashSet<String>,
) {
    let sections = vec![
        (
            "Saxes",
            vec![
                "Alto Sax 1",
                "Alto Sax 2",
                "Tenor Sax 1",
                "Tenor Sax 2",
                "Baritone Sax",
            ],
        ),
        (
            "Trumpets",
            vec![
                "Trumpet 1",
                "Trumpet 2",
                "Trumpet 3",
                "Trumpet 4",
                "Trumpet 5",
            ],
        ),
        (
            "Trombones",
            vec![
                "Trombone 1",
                "Trombone 2",
                "Trombone 3",
                "Trombone 4",
                "Trombone 5",
            ],
        ),
        ("Rhythm", vec!["Guitar", "Piano", "Bass", "Drums"]),
    ];

    let mut is_first_heading = true;
    let mut add_heading = |ui: &mut egui::Ui, heading: &str| {
        if !is_first_heading {
            ui.add_space(10.0);
        }
        is_first_heading = false;

        ui.label(heading);
    };

    // Add parts for which we have heandings
    for (heading, parts_under_heading) in sections {
        // Find which parts we actually have, removing them from the list
        let parts_to_list = parts_under_heading
            .into_iter()
            .filter(|p| parts.remove(*p))
            .collect_vec();
        if !parts_to_list.is_empty() {
            add_heading(ui, heading);
            for p in parts_to_list {
                ui.selectable_value(part_var, p.to_owned(), format!("  {}", p));
            }
        }
    }

    // Any other parts go into "Other"
    if !parts.is_empty() {
        add_heading(ui, "Other");
        for p in parts.iter().sorted() {
            ui.selectable_value(part_var, p.to_owned(), format!("  {}", p));
        }
    }
}
