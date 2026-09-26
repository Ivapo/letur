//! Where a project's files are, and the one door every rule reaches them by.
//!
//! **The rules in this crate never open a file.** They ask a [`Files`], and the
//! host decides what that means: the desktop app answers from a folder on disk,
//! and `ltr-001`'s web session answers from [`MemFiles`], a map it filled when
//! the project opened. So a refusal, a receipt or a counter is decided once,
//! here, and reaches the window in the same words on either host.
//!
//! **Paths are root-relative, `/`-separated, and the root is the project's own
//! directory, always.** The one exception is [`Files::locate`]'s answer, which
//! is the path a sentence spells and [`Files::read`] takes back: the desktop's
//! is absolute, so every sentence it builds names the file exactly as it did
//! before this crate existed, and the web's is the root-relative path itself.
//! That is `ltr-001` §2's one priced difference — the frame of the sentence is
//! this crate's, and the path and the `io::Error` at its tail are the host's.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

/// Why a create or a delete did not happen, before it is a sentence.
///
/// **Typed here and worded by [`Refused::sentence`]**, so the two hosts cannot
/// drift: a `MemFiles` refusing a create says what the disk's refusal says.
#[derive(Debug)]
pub enum Refused {
    /// A file being created would land outside the project, or in a folder
    /// that does not exist.
    Landing,
    /// A file being created is already there.
    Exists,
    /// A file being deleted is not under the project.
    Outside,
    /// A file being deleted is not there at all.
    Absent,
    /// A create the store itself failed, with its own reason.
    Unwritten(io::Error),
    /// A host's own sentence, whole: the desktop's Trash, which the web has no
    /// counterpart for.
    Host(String),
}

impl Refused {
    /// The words the window shows, for a `path` the panel named and the place
    /// [`Files::locate`] says it is.
    pub fn sentence(self, path: &str, located: &Path) -> String {
        match self {
            Refused::Landing => format!("{path} would land outside this project"),
            Refused::Exists => format!("{path} already exists"),
            Refused::Outside => format!("{path} is outside this project"),
            Refused::Absent => format!("{path} is not there"),
            Refused::Unwritten(e) => format!("cannot create {}: {e}", located.display()),
            Refused::Host(sentence) => sentence,
        }
    }
}

/// One project's files, as a host keeps them.
pub trait Files {
    /// Where a root-relative path is, as a sentence spells it and as
    /// [`Files::read`] takes it back.
    fn locate(&self, path: &str) -> PathBuf;

    /// The bytes at a path [`Files::locate`] produced, or joined onto one.
    fn read(&self, located: &Path) -> io::Result<Vec<u8>>;

    /// Are these two located paths one file?
    ///
    /// Lexical by default. The desktop resolves both, which is what
    /// `document::render_project`'s buffer override has always compared by.
    fn same(&self, a: &Path, b: &Path) -> bool {
        normal(&a.to_string_lossy()) == normal(&b.to_string_lossy())
    }

    /// Every file in the project, root-relative, in the panel's order.
    fn list(&self) -> Vec<String>;

    /// Is this path a file in the project? `document::confined`'s question.
    fn holds(&self, path: &str) -> bool;

    /// Write a file the project already holds.
    fn write(&mut self, path: &str, bytes: &[u8]) -> io::Result<()>;

    /// Make one empty file, refusing one that is there already.
    fn create(&mut self, path: &str) -> Result<(), Refused>;

    /// Delete one file. The desktop's is a move to the Trash; the web's is
    /// permanent.
    fn remove(&mut self, path: &str) -> Result<(), Refused>;

    /// Write where a Save-as dialog pointed, and answer the root-relative
    /// spelling when that is inside the project. A desktop dialog answers an
    /// absolute path, which may be anywhere.
    fn save_as(&mut self, path: &str, bytes: &[u8]) -> io::Result<Option<String>>;
}

/// A path spelled root-relatively with every `.` and empty segment gone, or
/// `None` when it is absolute or climbs out of the root.
///
/// **Lexical, because a map has no links to follow.** The desktop's
/// confinement canonicalizes; this is what the same question is where every
/// file is a key.
pub fn normal(path: &str) -> Option<String> {
    if path.starts_with('/') {
        return None;
    }
    let mut segments: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            part => segments.push(part),
        }
    }
    (!segments.is_empty()).then(|| segments.join("/"))
}

/// A project held in memory: every file's bytes under its root-relative path.
///
/// **What `ltr-001` Phase 2's web session opens a project into**, and what this
/// crate's own tests run the rules over. A directory is never a key: it exists
/// exactly while some file sits in it, which is also how the panel draws one.
#[derive(Debug, Clone, Default)]
pub struct MemFiles {
    files: BTreeMap<String, Vec<u8>>,
}

impl MemFiles {
    /// A project of these files, keyed as given after [`normal`] spells them.
    /// A path that climbs out of the root is not a file of the project and is
    /// dropped.
    pub fn new(files: impl IntoIterator<Item = (String, Vec<u8>)>) -> Self {
        Self {
            files: files
                .into_iter()
                .filter_map(|(path, bytes)| normal(&path).map(|path| (path, bytes)))
                .collect(),
        }
    }

    fn missing() -> io::Error {
        io::Error::new(io::ErrorKind::NotFound, "No such file or directory")
    }

    /// Does a folder hold anything? The root always does.
    fn folder(&self, path: &str) -> bool {
        match path.rsplit_once('/') {
            None => true,
            Some((folder, _)) => {
                let prefix = format!("{folder}/");
                self.files.keys().any(|key| key.starts_with(&prefix))
            }
        }
    }
}

impl Files for MemFiles {
    fn locate(&self, path: &str) -> PathBuf {
        PathBuf::from(path)
    }

    fn read(&self, located: &Path) -> io::Result<Vec<u8>> {
        normal(&located.to_string_lossy())
            .and_then(|key| self.files.get(&key).cloned())
            .ok_or_else(Self::missing)
    }

    fn list(&self) -> Vec<String> {
        let mut listed: Vec<String> = self.files.keys().cloned().collect();
        listed.sort_by(|left, right| crate::document::order(left, right));
        listed
    }

    fn holds(&self, path: &str) -> bool {
        normal(path).is_some_and(|key| self.files.contains_key(&key))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> io::Result<()> {
        let key = normal(path).ok_or_else(Self::missing)?;
        self.files.insert(key, bytes.to_vec());
        Ok(())
    }

    fn create(&mut self, path: &str) -> Result<(), Refused> {
        let key = normal(path).ok_or(Refused::Landing)?;
        if !self.folder(&key) {
            return Err(Refused::Landing);
        }
        if self.files.contains_key(&key) {
            return Err(Refused::Exists);
        }
        self.files.insert(key, Vec::new());
        Ok(())
    }

    fn remove(&mut self, path: &str) -> Result<(), Refused> {
        let key = normal(path).ok_or(Refused::Outside)?;
        self.files.remove(&key).map(|_| ()).ok_or(Refused::Absent)
    }

    fn save_as(&mut self, path: &str, bytes: &[u8]) -> io::Result<Option<String>> {
        let key = normal(path).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "outside this project")
        })?;
        self.files.insert(key.clone(), bytes.to_vec());
        Ok(Some(key))
    }
}

// A test may read a fixture: `clippy.toml`'s list is about what the crate
// ships, which is what the gate lints.
#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;

    #[test]
    fn a_path_is_spelled_once_and_never_leaves_the_root() {
        assert_eq!(normal("a/./b//c.md").as_deref(), Some("a/b/c.md"));
        assert_eq!(normal("a/../b.md").as_deref(), Some("b.md"));
        assert_eq!(normal("../b.md"), None);
        assert_eq!(normal("/etc/passwd"), None);
        assert_eq!(normal(""), None);
    }
}
