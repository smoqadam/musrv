use crate::path_utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError {
    Invalid,
}

#[allow(dead_code)]
pub fn parse_album_name(mut name: String) -> Result<String, InputError> {
    if let Some(stripped) = name.strip_suffix(".m3u8") {
        name = stripped.to_string();
    }
    if let Ok(decoded) = urlencoding::decode(&name) {
        name = decoded.into_owned();
    }
    if name.is_empty() || name.contains('\0') {
        return Err(InputError::Invalid);
    }
    for seg in name.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." {
            return Err(InputError::Invalid);
        }
        if path_utils::is_hidden_name(seg) {
            return Err(InputError::Invalid);
        }
        if seg.contains('\\') {
            return Err(InputError::Invalid);
        }
    }
    Ok(name)
}

pub fn validate_request_path(path: &str) -> Result<String, InputError> {
    if path.is_empty() || path == "/" {
        return Err(InputError::Invalid);
    }
    let decoded = urlencoding::decode(path)
        .map_err(|_| InputError::Invalid)?
        .into_owned();
    if decoded.starts_with('/') || decoded.contains('\0') {
        return Err(InputError::Invalid);
    }
    for seg in decoded.split('/') {
        if seg.is_empty() {
            continue;
        }
        if seg == "." || seg == ".." {
            return Err(InputError::Invalid);
        }
        if path_utils::is_hidden_name(seg) {
            return Err(InputError::Invalid);
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
        // nested album paths are allowed now but sanitized
        assert!(parse_album_name("a/b".to_string()).is_ok());
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
