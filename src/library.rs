use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Library {
    path: PathBuf,
    files: Vec<PathBuf>,
}

impl Library {
    pub fn scan(path: PathBuf) -> Self {
        let mut files = Vec::new();
        for entry in WalkDir::new(path.clone()).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()) {
                    if matches!(ext.as_str(),
                        "mp3" | "flac" | "wav" | "aac" | "m4a" | "ogg" | "opus" | "wma" | "aif" | "aiff" | "alac" | "pcm" | "mp2" | "mpga" | "ape"
                    ) {
                        files.push(p.to_path_buf());
                    }
                }
            }
        }
        Library { path, files }
    }

    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }
}
