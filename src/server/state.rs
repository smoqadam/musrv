use std::path::PathBuf;
use std::sync::Arc;

use crate::library::Library;
use arc_swap::ArcSwap;

#[derive(Clone)]
pub struct AppState {
    pub lib: Arc<ArcSwap<Library>>,
    pub base: String,
    pub root: PathBuf,
    pub album_depth: usize,
}
