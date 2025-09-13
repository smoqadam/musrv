pub fn parse_album_name(mut name: String) -> Result<String, ()> {
    if let Some(stripped) = name.strip_suffix(".m3u8") {
        name = stripped.to_string();
    }
    if let Ok(decoded) = urlencoding::decode(&name) {
        name = decoded.into_owned();
    }
    if name.is_empty() || name.starts_with('.') || name.contains('/') || name.contains('\\') {
        return Err(());
    }
    Ok(name)
}

pub fn validate_request_path(path: &str) -> Result<String, ()> {
    if path.is_empty() || path == "/" {
        return Err(());
    }
    let decoded = urlencoding::decode(path).map_err(|_| ())?.into_owned();
    if decoded.contains("..") || decoded.starts_with('/') || decoded.contains('\0') {
        return Err(());
    }
    for seg in decoded.split('/') {
        if seg.is_empty() {
            continue;
        }
        if seg.starts_with('.')
            || seg == "Thumbs.db"
            || seg == "desktop.ini"
            || seg.starts_with("._")
        {
            return Err(());
        }
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_album_name_accepts_simple() {
        let s = parse_album_name("MyAlbum.m3u8".to_string()).unwrap();
        assert_eq!(s, "MyAlbum");
        let s2 = parse_album_name("Another".to_string()).unwrap();
        assert_eq!(s2, "Another");
    }

    #[test]
    fn parse_album_name_rejects_bad() {
        assert!(parse_album_name("".to_string()).is_err());
        assert!(parse_album_name("../hack".to_string()).is_err());
        assert!(parse_album_name("a/b".to_string()).is_err());
        assert!(parse_album_name(".hidden".to_string()).is_err());
    }

    #[test]
    fn validate_request_path_ok() {
        let p = validate_request_path("Album/song.mp3").unwrap();
        assert_eq!(p, "Album/song.mp3");
    }

    #[test]
    fn validate_request_path_rejects() {
        assert!(validate_request_path("").is_err());
        assert!(validate_request_path("/abs").is_err());
        assert!(validate_request_path("..%2Fescape").is_err());
        assert!(validate_request_path(".hidden/file").is_err());
    }
}
