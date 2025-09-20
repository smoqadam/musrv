use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use lofty::{Accessor, AudioFile, TaggedFileExt};
use walkdir::{DirEntry, WalkDir};

use blake3::Hasher;

#[derive(Clone, Debug, Default)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<f64>,
    pub artwork_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub size: Option<u64>,
    pub metadata: TrackMetadata,
}

#[derive(Debug)]
pub struct Library {
    #[allow(dead_code)]
    root: PathBuf,
    tracks: Vec<Track>,
    folders: HashMap<String, FolderEntry>,
    artworks: HashMap<String, Artwork>,
}

#[derive(Clone, Debug, Default)]
pub struct FolderEntry {
    pub subfolders: BTreeSet<String>,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug)]
pub struct Artwork {
    pub mime: String,
    pub data: Arc<[u8]>,
}

impl Library {
    pub fn scan(root: PathBuf) -> Self {
        let mut tracks = Vec::new();
        let mut artworks: HashMap<String, Artwork> = HashMap::new();
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
                let (metadata, artwork_blob) = read_metadata(p);
                if let Some(blob) = artwork_blob {
                    artworks.entry(blob.id.clone()).or_insert_with(|| Artwork {
                        mime: blob.mime,
                        data: blob.data.into(),
                    });
                }
                tracks.push(Track {
                    path: rel,
                    size,
                    metadata,
                });
            }
        }

        let mut folders: HashMap<String, FolderEntry> = HashMap::new();
        folders.entry(String::new()).or_default();
        for t in &tracks {
            // Folder tree population
            match t.path.parent() {
                None => {
                    folders
                        .entry(String::new())
                        .or_default()
                        .tracks
                        .push(t.clone());
                }
                Some(parent) => {
                    let rel_parent = parent.to_string_lossy().to_string();
                    folders
                        .entry(rel_parent.clone())
                        .or_default()
                        .tracks
                        .push(t.clone());

                    // Build chain of subfolder links from root to this parent
                    let parts: Vec<String> = parent
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().to_string())
                        .collect();
                    let mut prev = String::new();
                    for i in 0..parts.len() {
                        let current = parts[0..=i].join("/");
                        folders.entry(current.clone()).or_default();
                        // link prev -> current
                        folders
                            .entry(prev.clone())
                            .or_default()
                            .subfolders
                            .insert(current.clone());
                        prev = current;
                    }
                }
            }
        }

        tracks.sort_by(|a, b| a.path.cmp(&b.path));

        Library {
            root,
            tracks,
            folders,
            artworks,
        }
    }

    #[allow(dead_code)]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn folder(&self, rel: &str) -> Option<&FolderEntry> {
        self.folders.get(rel)
    }

    pub fn collect_tracks_recursive(&self, rel: &str) -> Vec<Track> {
        let mut v = Vec::new();
        self.collect_tracks_recursive_inner(rel, &mut v);
        v
    }

    fn collect_tracks_recursive_inner(&self, rel: &str, out: &mut Vec<Track>) {
        if let Some(entry) = self.folders.get(rel) {
            out.extend(entry.tracks.clone());
            for child in &entry.subfolders {
                self.collect_tracks_recursive_inner(child, out);
            }
        }
    }

    pub fn artwork(&self, id: &str) -> Option<Artwork> {
        self.artworks.get(id).cloned()
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

fn read_metadata(path: &Path) -> (TrackMetadata, Option<ArtworkBlob>) {
    let mut metadata = TrackMetadata::default();
    let mut artwork_blob = None;
    if let Ok(tagged) = lofty::read_from_path(path) {
        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            if let Some(title) = tag.title() {
                metadata.title = Some(title.to_string());
            }
            if let Some(artist) = tag.artist() {
                metadata.artist = Some(artist.to_string());
            }
            if let Some(album) = tag.album() {
                metadata.album = Some(album.to_string());
            }
            if let Some(picture) = tag.pictures().first() {
                let mime = picture
                    .mime_type()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "image/jpeg".to_string());
                let data = picture.data().to_vec();
                let mut hasher = Hasher::new();
                hasher.update(&data);
                let id = hasher.finalize().to_hex().to_string();
                metadata.artwork_id = Some(id.clone());
                artwork_blob = Some(ArtworkBlob { id, mime, data });
            }
        }
        let duration = tagged.properties().duration().as_secs_f64();
        if duration.is_finite() && duration > 0.0 {
            metadata.duration = Some(duration);
        }
    }
    (metadata, artwork_blob)
}

#[derive(Debug)]
struct ArtworkBlob {
    id: String,
    mime: String,
    data: Vec<u8>,
}
