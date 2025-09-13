use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Clone, Debug)]
pub struct Track {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub size: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct Album {
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(Debug)]
pub struct Library {
    #[allow(dead_code)]
    root: PathBuf,
    tracks: Vec<Track>,
    albums: Vec<Album>,
}

impl Library {
    #[allow(dead_code)]
    pub fn scan(root: PathBuf) -> Self {
        Self::scan_with_depth(root, 1)
    }

    pub fn scan_with_depth(root: PathBuf, album_depth: usize) -> Self {
        let mut tracks = Vec::new();
        let iter = WalkDir::new(root.clone())
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_hidden_entry(e));
        for entry in iter.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file()
                && let Some(ext) = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_ascii_lowercase())
                && matches!(
                    ext.as_str(),
                    "mp3"
                        | "flac"
                        | "wav"
                        | "aac"
                        | "m4a"
                        | "ogg"
                        | "opus"
                        | "wma"
                        | "aif"
                        | "aiff"
                        | "alac"
                        | "pcm"
                        | "mp2"
                        | "mpga"
                        | "ape"
                )
            {
                if is_hidden_path(p) {
                    continue;
                }
                let rel = p.strip_prefix(&root).unwrap_or(p).to_path_buf();
                let size = fs::metadata(p).ok().map(|m| m.len());
                tracks.push(Track { path: rel, size });
            }
        }

        let mut by_album: HashMap<String, Vec<Track>> = HashMap::new();
        for t in &tracks {
            let parent = t.path.parent();
            let Some(parent) = parent else { continue }; // root-level handled as Singles later
            let comps: Vec<_> = parent.components().collect();
            if comps.is_empty() { continue; }
            let key = if album_depth == 0 {
                parent.to_string_lossy().to_string()
            } else {
                let take = comps.len().min(album_depth);
                let parts: Vec<String> = comps
                    .iter()
                    .take(take)
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect();
                parts.join("/")
            };
            if key.is_empty() { continue; }
            by_album.entry(key).or_default().push(t.clone());
        }
        let mut albums: Vec<Album> = by_album
            .into_iter()
            .map(|(name, mut ts)| {
                ts.sort_by(|a, b| a.path.cmp(&b.path));
                Album { name, tracks: ts }
            })
            .collect();

        let singles: Vec<Track> = tracks
            .iter()
            .filter(|t| t.path.components().count() == 1)
            .cloned()
            .collect();
        if !singles.is_empty() {
            let mut singles_sorted = singles;
            singles_sorted.sort_by(|a, b| a.path.cmp(&b.path));
            let mut singles_name = "Singles".to_string();
            if albums.iter().any(|a| a.name == singles_name) {
                singles_name = "Singles (root)".to_string();
            }
            albums.push(Album { name: singles_name, tracks: singles_sorted });
        }

        albums.sort_by(|a, b| a.name.cmp(&b.name));

        tracks.sort_by(|a, b| a.path.cmp(&b.path));

        Library {
            root,
            tracks,
            albums,
        }
    }

    #[allow(dead_code)]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }
    pub fn albums(&self) -> &[Album] {
        &self.albums
    }
    pub fn album_by_name(&self, name: &str) -> Option<&Album> {
        self.albums.iter().find(|a| a.name == name)
    }
}

fn is_hidden_entry(e: &DirEntry) -> bool {
    e.path()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| {
            s.starts_with('.') || s.starts_with("._") || s == "Thumbs.db" || s == "desktop.ini"
        })
        .unwrap_or(false)
}

fn is_hidden_path(p: &Path) -> bool {
    p.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        s.starts_with('.') || s.starts_with("._") || s == "Thumbs.db" || s == "desktop.ini"
    })
}
