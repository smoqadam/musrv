use serde::Serialize;

#[derive(Serialize)]
pub struct JsonFolderAlbum {
    pub name: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct JsonFolderResp {
    pub name: String,
    pub path: String,
    pub m3u8: String,
    pub albums: Vec<JsonFolderAlbum>,
}
