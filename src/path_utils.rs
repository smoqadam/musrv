use std::ffi::OsStr;

pub fn is_hidden_name(name: &str) -> bool {
    name.starts_with('.') || name.starts_with("._") || name == "Thumbs.db" || name == "desktop.ini"
}

pub fn is_hidden_component(component: &OsStr) -> bool {
    component.to_str().map(is_hidden_name).unwrap_or(false)
}
