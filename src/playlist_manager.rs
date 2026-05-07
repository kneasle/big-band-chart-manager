use std::{borrow::Cow, path::PathBuf};

use anyhow::anyhow;
use docx_rust::{
    DocxFile,
    document::{BodyContent, Table, TableCellContent, TableRowContent},
};
use serde::{Deserialize, Serialize};

use crate::chart_manager::ChartManager;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistManager {
    path: String,

    #[serde(skip)]
    cache: Cache,
}

#[derive(Debug, Default)]
struct Cache {
    playlists: Option<Vec<Playlist>>,
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub path: PathBuf,
}

impl Default for PlaylistManager {
    fn default() -> Self {
        Self {
            path: "/mnt/d/Music/Swing band/Playlists".to_owned(),
            cache: Cache::default(),
        }
    }
}

impl PlaylistManager {
    pub fn get_path(&self) -> &str {
        &self.path
    }

    /// Sets a path for this ChartManager.  If this is already the same path, nothing happens.  If
    /// the path has changed, then all the caches are cleared
    pub fn update_path(&mut self, new_path: String) {
        self.get_playlists();

        if self.path == new_path {
            return; // No change, nothing to do
        }

        self.path = new_path;
        self.cache = Cache::default();
    }

    /// Get a list of the playlists in the playlists folder, reading the filesystem if the cache is
    /// invalid.
    pub fn get_playlists(&mut self) -> &[Playlist] {
        if self.cache.playlists.is_none() {
            // No cache, need to update
            let mut playlists = Vec::<Playlist>::new();
            if let Ok(dir_iter) = std::fs::read_dir(&self.path) {
                for entry in dir_iter.filter_map(Result::ok) {
                    playlists.push(Playlist::new(entry.path()));
                }
            }
            self.cache.playlists = Some(playlists);
        }

        // Cache must be up-to-date, so unwrap is safe
        self.cache.playlists.as_ref().unwrap()
    }

    pub fn get_playlist_by_name(&mut self, name: &str) -> Option<&Playlist> {
        let playlists = self.get_playlists();
        playlists
            .iter()
            .find(|p| p.get_name() == Some(Cow::Borrowed(name)))
    }
}

// READING FROM DOCX ------------------------------------------------------------------------------

impl Playlist {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn get_name(&self) -> Option<Cow<'_, str>> {
        Some(self.path.file_stem()?.to_string_lossy())
    }

    // Read a list of pieces from the DOCX file
    pub fn read_setlist(&self, chart_manager: &mut ChartManager) -> anyhow::Result<Vec<String>> {
        // Read docx file from disk
        let docx_file = DocxFile::from_file(&self.path)?;
        let docx = docx_file.parse()?;

        // Scan to find the relevant table(s)
        let mut candidate_tables = Vec::<&Table>::new();
        for element in &docx.document.body.content {
            if let BodyContent::Table(table) = element {
                // We're usually looking for two column tables
                let num_cols = table.grids.columns.len();
                let num_rows = table.rows.len();
                if num_cols == 2 && num_rows == 2 {
                    candidate_tables.push(table);
                }
            }
        }

        // There should be only one candidate table
        let setlist_table = if candidate_tables.len() == 1 {
            candidate_tables[0]
        } else {
            return Err(anyhow!("No relevant tables found in {:?}.", self.path));
        };

        // Read the setlist table.  The setlist is in the second row of a 2x2 table
        let mut setlist = Vec::<String>::new();
        for cell in &setlist_table.rows[1].cells {
            if let TableRowContent::TableCell(cell) = cell {
                for content in &cell.content {
                    let TableCellContent::Paragraph(paragraph) = content;
                    setlist.extend(self.read_setlist_line(&paragraph.text(), chart_manager));
                }
            }
        }
        Ok(setlist)
    }

    fn read_setlist_line(&self, line: &str, chart_manager: &mut ChartManager) -> Option<String> {
        // Remove lines which obviously aren't chart names
        let line = line.trim();
        if line == "" {
            return None; // Empty lines can't correspond to a song
        }
        if line.ends_with(":") {
            return None; // Lines ending in ':' are probably a section e.g. "Encore:"
        }
        if line.to_lowercase().contains("encore") {
            return None; // Encore is just a label
        }

        // Remove vocal markings (iterator always has at least one element, so unwrap is safe)
        let line = line.split(" (").next().unwrap().trim();

        let corrected_name = chart_manager.get_nearest_piece_name(line);
        Some(corrected_name.to_owned())
    }
}
