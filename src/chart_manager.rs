use std::{
    collections::{HashMap, HashSet},
    fs::DirEntry,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

/// Data structure providing an abstract access to a big band parts folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChartManager {
    /// Path to the directory containing the charts
    path: String,

    #[serde(skip)]
    cache: Cache,
}

#[derive(Debug, Default)]
struct Cache {
    piece_list: Option<HashSet<String>>,

    // Map of piece names to a list of parts which they support
    parts_per_piece: HashMap<String, HashMap<String, PathBuf>>,
}

impl Default for ChartManager {
    fn default() -> Self {
        Self {
            path: "/mnt/d/Music/Swing band/Current Music Library/".to_owned(),
            cache: Cache::default(),
        }
    }
}

impl ChartManager {
    pub fn get_path(&self) -> &str {
        &self.path
    }

    /// Sets a path for this ChartManager.  If this is already the same path, nothing happens.  If
    /// the path has changed, then all the caches are cleared
    pub fn update_path(&mut self, new_path: String) {
        if self.path == new_path {
            return; // No change, nothing to do
        }

        self.path = new_path;
        self.cache = Cache::default();
    }

    pub fn get_piece_list(&mut self) -> &HashSet<String> {
        // Update cache if not filled yet
        if self.cache.piece_list.is_none() {
            self.cache.piece_list = Some(self.read_piece_list_from_dir());
        }

        return self.cache.piece_list.as_ref().unwrap();
    }

    /// Gets a list of the valid parts for a given piece.  If the piece isn't found, then a default
    /// parts list is returned.
    pub fn get_parts_for_piece(&mut self, piece: &str) -> HashSet<String> {
        let piece_list = self.get_piece_list();

        if piece_list.contains(piece) {
            self.list_parts_for_piece(piece).keys().cloned().collect()
        } else {
            return Self::DEFAULT_PARTS
                .into_iter()
                .map(|s| s.to_owned())
                .collect();
        }
    }

    pub fn get_path_of_part(&mut self, piece: &str, part: &str) -> Option<PathBuf> {
        let parts = self.list_parts_for_piece(piece);
        parts.get(part).cloned()
    }

    fn list_parts_for_piece(&mut self, piece: &str) -> &HashMap<String, PathBuf> {
        // Add part list to the cache if needed
        if !self.cache.parts_per_piece.contains_key(piece) {
            let part_list = self.read_part_list_from_disk(piece);
            self.cache
                .parts_per_piece
                .insert(piece.to_owned(), part_list);
        }
        // Get it from the cache, which is now guaranteed to exist
        let part_map = &self.cache.parts_per_piece[piece];
        part_map
    }

    fn read_part_list_from_disk(&mut self, piece_name: &str) -> HashMap<String, PathBuf> {
        let piece_dir = format!("{}/{}", self.path, piece_name);

        let mut part_list = HashMap::<String, PathBuf>::new();
        if let Ok(dir_iter) = std::fs::read_dir(piece_dir) {
            for entry in dir_iter {
                if let Some((part, path)) = Self::extract_part_from_dir_entry(piece_name, &entry) {
                    part_list.insert(part, path);
                }
            }
        }
        part_list
    }

    fn extract_part_from_dir_entry(
        piece_name: &str,
        dir_entry: &std::io::Result<DirEntry>,
    ) -> Option<(String, PathBuf)> {
        // Read the required stuff from the filesystem
        let entry = dir_entry.as_ref().ok()?;
        if !entry.metadata().ok()?.is_file() {
            return None; // Skip non-files
        }
        if entry.path().extension()? != "pdf" {
            return None; // Skip non-PDF files
        }
        let path = entry.path();
        let file_stem = path.file_stem()?.to_string_lossy();

        // Split the file stem into two parts and detect which one of them is the part
        let (part_a, part_b) = file_stem.split_once(" - ")?;
        if part_a == piece_name {
            Some((part_b.to_owned(), path))
        } else if part_b == piece_name {
            Some((part_a.to_owned(), path))
        } else {
            None
        }
    }

    fn read_piece_list_from_dir(&mut self) -> HashSet<String> {
        let mut piece_list = HashSet::<String>::new();
        if let Ok(dir_iter) = std::fs::read_dir(&self.path) {
            for entry in dir_iter {
                if let Ok(entry) = entry
                    && let Ok(metadata) = entry.metadata()
                    && metadata.is_dir()
                    && let Some(file_name) = entry.path().file_name()
                {
                    piece_list.insert(file_name.to_string_lossy().into_owned());
                }
            }
        } else {
            println!("Error: Couldn't read charts directory!");
        }
        piece_list
    }

    pub const DEFAULT_PARTS: [&'static str; 18] = [
        // Saxes
        "Alto Sax 1",
        "Alto Sax 2",
        "Tenor Sax 1",
        "Tenor Sax 2",
        "Baritone Sax",
        // Trumpets
        "Trumpet 1",
        "Trumpet 2",
        "Trumpet 3",
        "Trumpet 4",
        // Trombones
        "Trombone 1",
        "Trombone 2",
        "Trombone 3",
        "Trombone 4",
        // Vocals
        "Vocal",
        // Rhythm
        "Guitar",
        "Piano",
        "Bass",
        "Drums",
    ];
}
