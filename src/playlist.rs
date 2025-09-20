use std::path::Path;
use std::sync::Arc;

use crate::library::Track;

pub fn encode_path(rel: &str) -> String {
    rel.split('/')
        .map(|s| urlencoding::encode(s).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn render_m3u8(base: &str, _root: &Path, tracks: &[Arc<Track>]) -> String {
    let mut body = String::from("#EXTM3U\r\n");
    for t in tracks {
        let file_name = t.path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let mut display = t.metadata.title.as_deref().unwrap_or(file_name).to_string();
        if let Some(artist) = t
            .metadata
            .artist
            .as_deref()
            .filter(|artist| !artist.is_empty())
        {
            display = format!("{artist} - {display}");
        }
        let duration = t.metadata.duration.map(|d| d.round() as i64).unwrap_or(0);
        let rel = t.path.to_string_lossy().replace('\\', "/");
        let encoded = encode_path(&rel);
        body.push_str(&format!(
            "#EXTINF:{duration},{display}\r\n{base}{encoded}\r\n"
        ));
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
            Arc::new(Track {
                path: PathBuf::from("Album/song one.mp3"),
                size: None,
                metadata: crate::library::TrackMetadata::default(),
            }),
            Arc::new(Track {
                path: PathBuf::from("Root.mp3"),
                size: Some(123),
                metadata: crate::library::TrackMetadata::default(),
            }),
        ];
        let out = render_m3u8("http://h/", Path::new("/"), &tracks);
        assert!(out.starts_with("#EXTM3U\r\n"));
        assert!(out.contains("#EXTINF:0,Root.mp3\r\nhttp://h/Root.mp3\r\n"));
        assert!(out.contains("http://h/Album/song%20one.mp3"));
    }
}
