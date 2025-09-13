use std::path::Path;

use crate::library::Track;

pub fn encode_path(rel: &str) -> String {
    rel.split('/')
        .map(|s| urlencoding::encode(s).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn render_m3u8(base: &str, _root: &Path, tracks: &[Track]) -> String {
    let mut body = String::from("#EXTM3U\r\n");
    for t in tracks {
        let name = t.path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let rel = t.path.to_string_lossy().replace('\\', "/");
        let encoded = encode_path(&rel);
        body.push_str(&format!("#EXTINF:0,{name}\r\n{base}{encoded}\r\n"));
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn m3u8_renders_crlf_and_urls() {
        let tracks = vec![
            Track { path: PathBuf::from("Album/song one.mp3"), size: None },
            Track { path: PathBuf::from("Root.mp3"), size: Some(123) },
        ];
        let out = render_m3u8("http://h/", Path::new("/"), &tracks);
        assert!(out.starts_with("#EXTM3U\r\n"));
        assert!(out.contains("#EXTINF:0,Root.mp3\r\nhttp://h/Root.mp3\r\n"));
        assert!(out.contains("http://h/Album/song%20one.mp3"));
    }
}
