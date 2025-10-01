use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::library::Library;
use arc_swap::ArcSwap;

#[derive(Clone)]
pub struct AppState {
    pub lib: Arc<ArcSwap<Library>>,
    pub base: String,
    pub root: PathBuf,
    pub scan_ready: Arc<AtomicBool>,
    pub scan_in_progress: Arc<AtomicBool>,
}

impl AppState {
    pub fn schedule_scan(&self, mark_unready: bool) -> bool {
        if self
            .scan_in_progress
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return false;
        }
        if mark_unready {
            self.scan_ready.store(false, Ordering::SeqCst);
        }
        let state = self.clone();
        tokio::spawn(async move {
            let root = state.root.clone();
            let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Library> {
                let lib = crate::library::Library::scan(root);
                lib.save_cached()?;
                Ok(lib)
            })
            .await;
            match result {
                Ok(Ok(lib)) => {
                    state.lib.store(Arc::new(lib));
                }
                Ok(Err(err)) => {
                    tracing::error!(?err, "scan failed");
                }
                Err(err) => {
                    tracing::error!("scan task join error: {}", err);
                }
            }
            state.scan_ready.store(true, Ordering::SeqCst);
            state.scan_in_progress.store(false, Ordering::SeqCst);
        });
        true
    }
}
