use serde::Serialize;

#[derive(Serialize)]
pub struct JsonFolderAlbum {
    pub name: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct JsonFolderTrack {
    pub name: String,
    pub display_name: String,
    pub relative_path: String,
    pub url: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<f64>,
    pub artwork_url: Option<String>,
}

#[derive(Serialize)]
pub struct JsonFolderResp {
    pub name: String,
    pub path: String,
    pub m3u8: String,
    pub albums: Vec<JsonFolderAlbum>,
    pub tracks: Vec<JsonFolderTrack>,
    pub scanning: bool,
}
