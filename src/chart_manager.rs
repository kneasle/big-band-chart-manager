use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

/// Data structure providing an abstract access to a big band parts folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChartManager {
    /// Path to the directory containing the charts
    charts_dir: String,

    #[serde(skip)]
    cache: cache::Cache,
}

impl Default for ChartManager {
    fn default() -> Self {
        let chart_dir = "/mnt/d/Music/Swing band/Current Music Library/";
        Self {
            cache: cache::Cache::new(chart_dir),
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
        self.cache = cache::Cache::new(&self.charts_dir);
    }

    pub fn get_piece_list(&mut self) -> &HashSet<String> {
        return &self.cache.piece_list;
    }

    pub fn has_piece(&mut self, piece: &str) -> bool {
        self.get_piece_list().contains(piece)
    }

    pub fn does_piece_have_arrangements(&mut self, piece: &str) -> bool {
        self.cache
            .pieces_with_arrangements
            .get(piece)
            .is_some_and(|arrs| arrs.len() > 1)
    }

    pub fn has_part(&mut self, piece: &str, part: &str) -> bool {
        self.get_part_list(piece)
            .is_some_and(|parts| parts.contains_key(part))
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
    pub struct Cache {
        pub piece_list: HashSet<String>,
        pub pieces_with_arrangements: HashMap<String, Vec<String>>,

        /// Maps a punctuation-free, lowercase version of each piece's name to a list of every
        /// arrangement of this piece.
        // tags_to_piece: HashMap<String, String>,

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

            Self {
                piece_list,
                pieces_with_arrangements,

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
        fn piece_name_to_tag(name: &str) -> String {
            let piece_without_arranger = name.split(" (arr ").next().unwrap();

            // TODO: Do this properly

            piece_without_arranger.to_lowercase()
        }
    }
}
