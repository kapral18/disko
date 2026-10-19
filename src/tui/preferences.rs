//! The last browsing view, saved outside the scan history.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum BrowseView {
    #[default]
    Chart,
    List,
}

pub(super) fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("disko").join("view"))
}

pub(super) fn load(path: &Path) -> BrowseView {
    match fs::read_to_string(path).as_deref().map(str::trim) {
        Ok("list") => BrowseView::List,
        _ => BrowseView::Chart,
    }
}

pub(super) fn save(path: &Path, view: BrowseView) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Replace the preference only after the whole value has been written.
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let value = match view {
        BrowseView::Chart => "chart\n",
        BrowseView::List => "list\n",
    };
    let result = file.write_all(value.as_bytes());
    drop(file);
    let result = result.and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
