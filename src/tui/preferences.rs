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

#[cfg(test)]
mod tests {
    use super::*;

    struct Dir(PathBuf);

    impl Dir {
        fn new(tag: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("disko-preferences-{tag}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            Self(path)
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn no_saved_view_or_one_disko_cannot_read_opens_the_chart() {
        let dir = Dir::new("unreadable");
        let path = dir.0.join("view");
        assert_eq!(load(&path), BrowseView::Chart);

        fs::create_dir_all(&dir.0).unwrap();
        fs::write(&path, "sideways\n").unwrap();
        assert_eq!(load(&path), BrowseView::Chart);
    }

    #[test]
    fn a_saved_view_is_read_back_and_leaves_nothing_else_behind() {
        let dir = Dir::new("round-trip");
        let path = dir.0.join("disko").join("view");

        save(&path, BrowseView::List).unwrap();
        assert_eq!(load(&path), BrowseView::List);
        save(&path, BrowseView::Chart).unwrap();
        assert_eq!(load(&path), BrowseView::Chart);

        let names: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["view"]);
    }
}
