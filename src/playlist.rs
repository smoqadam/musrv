use std::path::Path;

use crate::library::Track;

pub fn encode_path(rel: &str) -> String {
    rel.split('/')
        .map(|s| urlencoding::encode(s).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn render_m3u8(base: &str, root: &Path, tracks: &[Track]) -> String {
    let mut body = String::from("#EXTM3U\n");
    for t in tracks {
        let name = t.path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let rel = t.path.to_string_lossy().replace('\\', "/");
        let encoded = encode_path(&rel);
        body.push_str(&format!("#EXTINF:-1,{name}\n{base}{encoded}\n"));
    }
    body
}
