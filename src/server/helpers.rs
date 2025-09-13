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
