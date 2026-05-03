use std::{borrow::Cow, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistManager {
    path: String,

    #[serde(skip)]
    cache: Cache,
}

#[derive(Debug)]
pub struct Playlist {
    pub path: PathBuf,
}

#[derive(Debug, Default)]
struct Cache {
    playlists: Option<Vec<Playlist>>,
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
    pub fn read_setlist(&self) -> Vec<String> {
        // TODO: Do this properly
        vec![
            "Sing Sing Sing".to_owned(),
            "The Jazz Police".to_owned(),
            "A String of Pearls".to_owned(),
        ]
    }
}
