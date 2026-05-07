use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use self::cache::Cache;

/// Data structure providing an abstract access to a big band parts folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChartManager {
    /// Path to the directory containing the charts
    charts_dir: String,

    #[serde(skip)]
    cache: Cache,
}

impl Default for ChartManager {
    fn default() -> Self {
        let chart_dir = "/mnt/d/Music/Swing band/Current Music Library/";
        Self {
            cache: Cache::new(chart_dir),
            charts_dir: chart_dir.to_owned(),
        }
    }
}

impl ChartManager {
    pub fn get_path(&self) -> &str {
        &self.charts_dir
    }

    /// Sets a path for this ChartManager.  If this is already the same path, nothing happens.  If
    /// the path has changed, then the cache is also updated
    pub fn update_path(&mut self, new_path: String) {
        if self.charts_dir == new_path {
            return; // No change, nothing to do
        }

        self.charts_dir = new_path;
        self.refresh_cache();
    }

    pub fn refresh_cache(&mut self) {
        self.cache = Cache::new(&self.charts_dir);
    }

    pub fn get_piece_list(&mut self) -> &HashSet<String> {
        return &self.cache.piece_list;
    }

    pub fn get_nearest_piece_name<'s>(&'s self, uncorrected_name: &'s str) -> &'s str {
        // Return an exact match if one exists
        let tag = Cache::piece_name_to_tag(uncorrected_name);
        if let Some(corrected_name) = self.cache.pieces_by_tag.get(&tag) {
            return corrected_name;
        }

        // TODO: Allow a little bit of edit distance to correct for typos

        // Otherwise, keep the existing name and let the user see the error and correct it
        uncorrected_name
    }

    pub fn does_piece_have_arrangements(&mut self, piece: &str) -> bool {
        self.cache
            .pieces_with_arrangements
            .get(piece)
            .is_some_and(|arrs| arrs.len() > 1)
    }

    pub fn get_path_of_part(&mut self, piece: &str, part: &str) -> Option<PathBuf> {
        self.get_part_list(piece)?.get(part).cloned()
    }

    pub fn get_part_list<'s>(&'s mut self, piece: &str) -> Option<&'s HashMap<String, PathBuf>> {
        self.cache.get_part_list(&self.charts_dir, piece)
    }

    pub fn get_part_list_or_default(&mut self, piece: &str) -> HashSet<String> {
        match self.cache.get_part_list(&self.charts_dir, piece) {
            Some(part_list) => part_list.keys().cloned().collect(),
            None => Self::default_part_list(),
        }
    }

    pub fn default_part_list() -> HashSet<String> {
        let default_parts = [
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
            // Rhythm
            "Guitar",
            "Piano",
            "Bass",
            "Drums",
            // Other
            "Vocal",
            "Conductor",
        ];

        default_parts.into_iter().map(|s| s.to_owned()).collect()
    }
}

// CACHE OVER THE DISK ----------------------------------------------------------------------------

mod cache {
    use std::{
        collections::{HashMap, HashSet},
        fs::DirEntry,
        path::PathBuf,
    };

    #[derive(Debug, Default)]
    pub(super) struct Cache {
        pub piece_list: HashSet<String>,
        pub pieces_with_arrangements: HashMap<String, Vec<String>>,

        /// Maps a punctuation-free, lowercase version of each piece's name to the name of the
        /// piece without the arranger name.  This is used to perform fuzzy lookup of piece names
        /// from a setlist document.
        pub pieces_by_tag: HashMap<String, String>,

        /// Map of piece names to a map of part names to the path of that PDF.  This is calculated
        /// lazily - if a piece exists in piece_list but does not have an entry here, it means that
        /// piece hasn't been requested before.
        parts_per_piece: HashMap<String, HashMap<String, PathBuf>>,
    }

    impl Cache {
        pub fn new(charts_dir: &str) -> Self {
            let piece_list = Self::read_piece_list_from_disk(charts_dir);

            // Build arrangement map
            let mut pieces_with_arrangements = HashMap::<String, Vec<String>>::new();
            for piece in &piece_list {
                let piece_name_without_arranger = piece.split(" (arr ").next().unwrap();
                pieces_with_arrangements
                    .entry(piece_name_without_arranger.to_owned())
                    .or_default()
                    .push(piece.to_owned());
            }

            // Build map of pieces by tag
            let mut pieces_by_tag = HashMap::<String, String>::new();
            for piece_name_without_arranger in pieces_with_arrangements.keys() {
                let tag = Self::piece_name_to_tag(piece_name_without_arranger);
                pieces_by_tag.insert(tag, piece_name_without_arranger.to_owned());
            }

            Self {
                piece_list,
                pieces_with_arrangements,
                pieces_by_tag,

                parts_per_piece: HashMap::new(),
            }
        }

        fn read_piece_list_from_disk(charts_dir: &str) -> HashSet<String> {
            let mut piece_list = HashSet::<String>::new();
            let Ok(dir_iter) = std::fs::read_dir(charts_dir) else {
                println!("Error: Couldn't read charts directory!");
                return HashSet::new();
            };

            for entry in dir_iter {
                if let Ok(entry) = entry
                    && let Ok(metadata) = entry.metadata()
                    && metadata.is_dir()
                    && let Some(file_name) = entry.path().file_name()
                {
                    piece_list.insert(file_name.to_string_lossy().into_owned());
                }
            }
            piece_list
        }

        pub fn get_part_list<'s>(
            &'s mut self,
            charts_dir: &str,
            piece: &str,
        ) -> Option<&'s HashMap<String, PathBuf>> {
            if !self.piece_list.contains(piece) {
                return None; // Can't list parts for a piece that doesn't exist
            }

            // Add part list to the cache if needed
            let part_list = self
                .parts_per_piece
                .entry(piece.to_owned())
                .or_insert_with(|| Self::read_part_list_from_disk(charts_dir, piece));
            return Some(part_list);
        }

        fn read_part_list_from_disk(
            charts_dir: &str,
            piece_name: &str,
        ) -> HashMap<String, PathBuf> {
            let piece_dir = format!("{}/{}", charts_dir, piece_name);

            let mut part_list = HashMap::<String, PathBuf>::new();
            if let Ok(dir_iter) = std::fs::read_dir(piece_dir) {
                for entry in dir_iter {
                    if let Some((part, path)) =
                        Self::extract_part_from_dir_entry(piece_name, &entry)
                    {
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

        /// Normalize a piece name into a punctuation-free, lowercase version of that name with no
        /// arranger.
        pub fn piece_name_to_tag(name: &str) -> String {
            let name_without_arranger = name.split(" (arr ").next().unwrap();

            let mut tag = String::new();
            for c in name_without_arranger.chars() {
                if c == '&' {
                    tag.push_str("and");
                } else if c.is_alphanumeric() {
                    tag.extend(c.to_lowercase());
                } else if c.is_whitespace() {
                    tag.push(' '); // Normalise all whitespace to ' '
                }
            }
            tag.replace("opus 1", "opus one")
                .replace("chattanooga", "chatanooga")
        }
    }
}
