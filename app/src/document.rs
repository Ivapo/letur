//! The disk half of what this app decides without a window.
//!
//! **The rules left this file for `letur-project` in `ltr-001` Phase 1**: the
//! compile's two read passes, the panel's entries and their order, the main a
//! project opens on and every sentence a command refuses with are
//! `project/src/document.rs`'s, and the browser answers through the same ones.
//! What is here is what only a disk has — the climb from an opened file to its
//! project's root, the walk that lists it and the links it will not follow,
//! the confinement that canonicalizes, the Trash, and the three files in
//! Application Support — and [`Disk`], which answers `letur_project`'s
//! [`Files`] over a folder.
//!
//! Ordinary functions with ordinary tests, still: a GUI whose logic is
//! reachable only by clicking has no exit gate but a screenshot.

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

pub use letur_project::document::{Kind, kind_of, spell};
#[cfg(test)]
pub use letur_project::document::{Anchor, Entry, merge};
use letur_project::files::{Files, Refused};
use letur_project::preview::Appearance;
use serde::{Deserialize, Serialize};

/// The window's title for an open document: the file's own name.
///
/// A path with no file name at all cannot be opened, so the full path is a
/// fallback that no dialog reaches, not a second title format.
pub fn title(document: &Path) -> String {
    document
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| document.display().to_string())
}

// ---------------------------------------------------------------------------
// The project on disk: its root, the walk that lists it, the confinement a
// command asks, the Trash, and the facts this app remembers about it.
// `mpdf-010` Phases 1 to 5.
// ---------------------------------------------------------------------------

/// Where the project the author opened begins.
///
/// **The opened file's parent is where the search starts, not where it stops.**
/// [`crate::watch::root`] is `document.parent()`, so a double-click on
/// `showcase/sections/text.md` would root the project at `showcase/sections` —
/// below the master that names it, which the panel would then never list and
/// discovery would never find. Taking that parent as the root makes the whole
/// point of the panel unreachable, which is why this exists rather than being
/// left to `watch::root`.
///
/// **The rule**: start at the opened file's parent; if any `.md` in *that
/// directory's* parent names the opened file as one of its sections, the root is
/// the parent instead. `md2pdf_core::section_paths` is what answers "names it",
/// reading text and constructing `Ok` unconditionally, so the test is total over
/// every markdown file in that one directory. Which of them matched is not
/// returned, so the unordered `read_dir` costs no determinism: every match gives
/// the same answer.
///
/// **One level is a cap, and it is chosen rather than derived.** An earlier
/// draft argued it was a property of `mpdf-008`'s refusal of an include inside
/// an included section; that refusal is about a master naming a *master*, where
/// this looks for the master of a *file*, which a deeper relative path reaches
/// with no nesting at all — `[](parts/ch1/text.md)` is a supported marker. So
/// `parts/ch1/text.md` roots at `parts/ch1`, below its own master, which is
/// verbatim the failure the paragraph above says this prevents, surviving one
/// level down. **That cost is asserted in this file's own tests** rather than
/// left as a defect nobody noticed, and `specs/file_panel_spec.md` OQ-7 carries
/// the question. The cap is argued on cost instead: climbing further means
/// reading markdown in `~/Documents` and above to guess where the project is,
/// and this app has never opened a file the author did not name or a document
/// did not name.
///
/// Two edges answer the same way, and neither is an error in the window: a
/// parent with no parent, and a grandparent that will not `read_dir`. Both are
/// *no candidate found*, so the root is the opened file's own parent, which is
/// [`crate::watch::root`]'s answer unchanged and is every single-file document.
pub fn project_root(opened: &Path) -> PathBuf {
    let here = crate::watch::root(opened);
    let Some(above) = here.parent().filter(|up| !up.as_os_str().is_empty()) else {
        return here;
    };
    let Ok(entries) = std::fs::read_dir(above) else {
        return here;
    };

    let opened = crate::watch::resolve(opened);
    for entry in entries.flatten() {
        let candidate = entry.path();
        if kind_of(&candidate.to_string_lossy()) != Some(Kind::Markdown) || !candidate.is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&candidate) else {
            continue;
        };
        let names_it = md2pdf_core::section_paths(&text).is_ok_and(|sections| {
            sections
                .into_iter()
                .any(|section| crate::watch::resolve(&above.join(&section.path)) == opened)
        });
        if names_it {
            return above.to_path_buf();
        }
    }

    here
}

/// Every file under `root`, root-relative, in the panel's order.
///
/// **The order is total and computed here**, so the panel cannot reorder itself
/// between two compiles of the same tree: within each directory, files
/// alphabetically first, then subdirectories alphabetically, each expanded where
/// it sits. Byte-wise on the segment and not a locale collation — a
/// locale-dependent order is not reproducible by a second person, which is what
/// the exit gate needs it to be.
///
/// **The walk obeys the confinement rule, and that is not only a later phase's
/// concern.** A symlink under the root pointing at a directory elsewhere would
/// otherwise put that directory's files in the panel as though they were the
/// project's, and the phase after this one would open one of them in the pane.
/// Links are resolved *before* the check rather than after, and a directory
/// already visited is not visited again, so a link that loops back inside the
/// root costs one skip rather than a stack.
fn walk(root: &Path) -> Vec<String> {
    let real = crate::watch::resolve(root);
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    descend(&real, &real, "", &mut seen, &mut found);
    found
}

fn descend(
    root: &Path,
    here: &Path,
    prefix: &str,
    seen: &mut HashSet<PathBuf>,
    found: &mut Vec<String>,
) {
    if !seen.insert(here.to_path_buf()) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(here) else {
        return;
    };

    let mut files: Vec<String> = Vec::new();
    let mut directories: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let landed = crate::watch::resolve(&entry.path());
        if !landed.starts_with(root) {
            continue;
        }
        if landed.is_dir() {
            directories.push(name);
        } else {
            files.push(name);
        }
    }

    files.sort_unstable();
    directories.sort_unstable();

    for name in files {
        found.push(format!("{prefix}{name}"));
    }
    for name in directories {
        let below = crate::watch::resolve(&here.join(&name));
        descend(root, &below, &format!("{prefix}{name}/"), seen, found);
    }
}

/// Which file under `root` compiles, when nothing is remembered about it:
/// `letur_project::document::discover_main` over the disk, with the opened file
/// spelled root-relatively here.
///
/// A file the root cannot spell — which no open reaches, since the root is the
/// opened file's own folder or the one above it — is named by its bare name,
/// which is what this function has always fallen back to.
pub fn discover_main(root: &Path, opened: &Path) -> String {
    let here = relative(root, opened).unwrap_or_else(|| {
        opened
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });
    letur_project::document::discover_main(&Disk::new(root), &here)
}

/// A path under `root`, spelled the way an [`Entry`] is: root-relative, `/`
/// separators, and `None` when it is not under the root at all.
///
/// Both sides are canonicalized, for [`crate::watch::classify`]'s reason: on
/// macOS the Open dialog hands over `/var/…` where the filesystem spells it
/// `/private/var/…`, so comparing them as they arrive matches nothing.
pub fn relative(root: &Path, path: &Path) -> Option<String> {
    spell(&crate::watch::resolve(root), &crate::watch::resolve(path))
}

/// The file `path` names under `root`, or `None` when it is not one.
///
/// **The one confinement rule three commands share.** A path from the panel
/// came from this app's own listing, but a command is a command, and
/// `root.join("../../secrets.png")` names a real file on plenty of machines.
/// So the path is resolved and its target must land under the resolved root —
/// which refuses a `..`, an absolute path, and a symlink whose target is
/// elsewhere, each by the same comparison.
///
/// **It is [`walk`]'s test and not a spelling round-trip, and that is a
/// correction.** The three commands used to ask [`relative`] to answer the same
/// spelling back, which is a *stricter* question than confinement and answers
/// no to something confinement allows: [`descend`] lists a symlink under its own
/// name, so a `cover.jpg` pointing at `figures/cover.jpg` inside the project was
/// listed as a row and then refused as though it were outside — while the
/// compile rendered it perfectly well. The two halves now ask one question, and
/// the panel cannot offer a row no command will take.
///
/// The path it returns is the **join and not the resolution**, so a read, a
/// write and a title all go through the link the author made rather than behind
/// it, and `crate::preview::Preview::status` goes on spelling it with [`spell`]
/// the way the row is spelled.
///
/// **A file being created goes through [`landing`] instead**, and the line
/// above is why: this refuses every path that is not already a file, which is
/// the whole input domain of a create. An implementer who reads "the one
/// confinement rule" and reuses it here would refuse every one.
pub fn confined(root: &Path, path: &str) -> Option<PathBuf> {
    let landed = root.join(path);
    if !landed.is_file() {
        return None;
    }

    // Resolved explicitly rather than through `crate::watch::resolve`, which
    // answers with its *input* when canonicalization fails: `root.join("../b")`
    // survives a component-wise `starts_with` textually, `..` and all. Existence
    // is settled above, so a failure here is a race or a permission, and either
    // is a refusal.
    let real = landed.canonicalize().ok()?;
    real.starts_with(crate::watch::resolve(root))
        .then_some(landed)
}

/// The bytes of one file under `root`, for the window to draw:
/// `letur_project::document::asset_bytes` over the disk, so the refusal is
/// [`confined`]'s and its words are the crate's.
///
/// **It is a function and not the command**: `app/src/main.rs` has no test
/// module, so a rule written into the command is a rule no test in this
/// repository can reach. `mpdf-010` Phase 5.
pub fn asset_bytes(root: &Path, path: &str) -> Result<Vec<u8>, String> {
    letur_project::document::asset_bytes(&Disk::new(root), path)
}

/// Where a path from the panel would land, for a file that does not exist yet.
///
/// **[`confined`]'s sibling for a write**, and it exists because that one opens
/// on `is_file`. `crate::watch::resolve` is no help either: it answers with its
/// *input* when canonicalization fails, and a file being created never
/// canonicalizes, so `root.join("../escape.md")` would survive a component-wise
/// `starts_with` textually, `..` and all.
///
/// **So the parent is canonicalized, which does exist, and the final component
/// is joined onto it.** A parent that will not canonicalize is a refusal too,
/// which is how `newdir/x.md` is refused with no clause of its own —
/// `specs/file_panel_spec.md` §1.2 makes folder creation a non-goal and this is
/// the rule that keeps it one.
///
/// It answers with the **join and not the resolution**, for [`confined`]'s own
/// reason: the write goes through the link the author made, and the sentence a
/// refusal carries spells the path the way the row does.
fn landing(root: &Path, path: &str) -> Option<PathBuf> {
    let landed = root.join(path);
    let real = landed.parent()?.canonicalize().ok()?;

    real.starts_with(crate::watch::resolve(root))
        .then_some(landed)
}

/// Make one empty file under `root`, named by the panel:
/// `letur_project::document::create_file` over the disk. The kind is refused
/// there, before anything here is asked; [`Disk`]'s `create` is [`landing`] and
/// `create_new`. `mpdf-010` Phase 3.
pub fn create_file(root: &Path, path: &str) -> Result<(), String> {
    letur_project::document::create_file(&mut Disk::new(root), path)
}

/// Move one file under `root` to the Trash, named by the panel.
///
/// **A third confinement question, and neither shipped rule serves it.**
/// [`confined`] opens on `is_file`, so it refuses a `missing: true` row and a
/// dangling symlink the walk lists; [`landing`] canonicalizes only the parent,
/// so it accepts a `secret.png` that is a link out of the project — which
/// `confined` refuses and
/// [`tests::a_link_out_of_the_project_is_refused_however_it_is_spelled`] pins as
/// a refusal for every other command. A delete wants both halves: **the name is
/// under the root, and something is at it.** So [`landing`] answers the first,
/// genuinely unchanged, and `symlink_metadata` the second — `is_file`'s question
/// widened to *anything at that name*.
///
/// **It acts on the name and not the resolution**, which [`landing`]'s join
/// decides and which settles the symlink row: a `cover.jpg` pointing at
/// `figures/cover.jpg` trashes the link and leaves the figure. That is the
/// opposite of [`confined`]'s reading for a *read*, and deliberately — a read
/// wants the bytes the author meant, where a delete wants the row the author
/// clicked, and the target of a link out of the project is not the project's to
/// move. It is also what makes the widened existence test safe: a dangling link
/// is a row the panel draws, and trashing it removes exactly that row.
///
/// **The call is a parameter, and that is deliberately not [`create_file`]'s
/// answer.** That one writes with a plain `std::fs::File::create_new` because
/// [`tests::scratch_dir`] can check the result and this file's rule already
/// canonicalizes a real parent. Neither holds here: this call's whole effect is
/// **outside the repository**, in the developer's own `~/.Trash`, which nothing
/// cleans and *"the repository stays clean"* has no reach over. So the suite
/// hands in a double and [`move_to_trash`] is what `crate::main` hands in.
///
/// **It is a function and not the command**, per this file's own header, for
/// [`asset_bytes`]'s reason. `mpdf-010` Phase 4.
pub fn trash_file(
    root: &Path,
    path: &str,
    trash: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), String> {
    trash_at(root, path, trash).map_err(|refused| refused.sentence(path, &root.join(path)))
}

/// [`trash_file`] before its refusal is a sentence, which is what [`Disk`]'s
/// `remove` answers with.
fn trash_at(
    root: &Path,
    path: &str,
    trash: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), Refused> {
    let landed = landing(root, path).ok_or(Refused::Outside)?;

    // `symlink_metadata` and not `exists`, which follows the link and reports a
    // dangling one absent — and a dangling link is a row this panel draws, so
    // reporting it absent would leave a row no gesture could remove.
    if landed.symlink_metadata().is_err() {
        return Err(Refused::Absent);
    }

    trash(&landed).map_err(Refused::Host)
}

/// The platform's own undo for a delete.
///
/// **The one function in this file no test in this repository calls**, and the
/// reason [`trash_file`] takes it as a parameter rather than calling it: every
/// clause of the gate would otherwise leave a file in whoever ran it.
///
/// `NSFileManager` is `trash` 5.2.6's own macOS implementation, reached
/// directly. `mpdf-010` Phase 4 records the measurement behind that pick.
pub fn move_to_trash(path: &Path) -> Result<(), String> {
    let url = objc2_foundation::NSURL::fileURLWithPath(&objc2_foundation::NSString::from_str(
        &path.to_string_lossy(),
    ));

    objc2_foundation::NSFileManager::defaultManager()
        .trashItemAtURL_resultingItemURL_error(&url, None)
        .map_err(|e| format!("cannot move {} to the Trash: {e}", path.display()))
}

/// The file the store lives in, inside the directory the platform gives this
/// app. [`settings_file`] and [`sites_file`] are its siblings and the only
/// others.
///
/// **Not a dotfile in the author's own folder**, and that was refused for two
/// reasons either of which is sufficient: it is the manifest
/// `specs/desktop_app_spec.md` §1.1 parks, arriving by another name; and it
/// writes a file into a directory the author may have under version control,
/// which this app has never done and should not start doing as a side effect of
/// a panel. The cost is accepted and stated: **the choice does not travel with
/// the files.**
pub fn store_file(support: &Path) -> PathBuf {
    support.join("projects.json")
}

/// What one root is remembered as compiling, keyed by the root's canonical path.
///
/// A `BTreeMap` so two writes of the same map produce the same bytes.
type Store = std::collections::BTreeMap<String, String>;

/// The store as it stands, or an empty one.
///
/// **A missing, unreadable or malformed store is nothing remembered, never an
/// error in the window.** The one fact in here is a convenience; a window that
/// refused to open a document because a JSON file in Application Support had
/// been truncated would be trading the whole app for it.
fn read_store(store: &Path) -> Store {
    std::fs::read_to_string(store)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Which file this root is remembered as compiling, if any.
pub fn read_override(store: &Path, root: &Path) -> Option<String> {
    read_store(store).remove(&key(root))
}

/// Remember that this root compiles `main`.
///
/// **A failed write is reported**, where a failed read is not: a set-main that
/// silently does not stick is worse than one that says why, and the author has
/// just asked for it in as many words.
pub fn write_override(store: &Path, root: &Path, main: &str) -> Result<(), String> {
    let mut held = read_store(store);
    held.insert(key(root), main.to_string());

    let text = serde_json::to_string_pretty(&held)
        .map_err(|e| format!("cannot write {}: {e}", store.display()))?;

    if let Some(parent) = store.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(store, text).map_err(|e| format!("cannot write {}: {e}", store.display()))
}

/// A root as the store keys it: the path the filesystem really spells.
fn key(root: &Path) -> String {
    crate::watch::resolve(root).to_string_lossy().into_owned()
}

/// The second file this app writes, beside [`store_file`] and in the same
/// directory: what the author asked for that is not about a folder.
///
/// **A second file and not a second key in the first, and the reason is
/// checkable rather than aesthetic.** [`Store`] is a `BTreeMap<String, String>`
/// keyed by canonical root and [`read_store`] swallows a parse failure, so
/// reshaping `projects.json` into an object with a member beside the mains
/// would make every file already on disk malformed — and malformed means
/// forgotten. Every author's remembered main would be dropped, silently, by the
/// upgrade that added a toggle to the footer.
pub fn settings_file(support: &Path) -> PathBuf {
    support.join("settings.json")
}

/// What this app remembers that is not about one folder.
///
/// One member today. It is a struct rather than a bare value so a second
/// preference is a field and not a second file, which is the shape
/// `projects.json` cannot have and this one can: `serde` fills a missing member
/// from `Default`, so an older file stays readable rather than becoming
/// malformed.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Settings {
    #[serde(default)]
    appearance: Appearance,
}

/// The palette the author chose, or [`Appearance::System`].
///
/// **[`read_store`]'s rule, inherited whole: a missing, unreadable or malformed
/// settings file is `System` and never an error in the window.** Following the
/// system is what this app did before there was a choice, so the failure state
/// and the default state are the same state, and nothing about a truncated file
/// in Application Support should reach a window that is trying to open a
/// document.
pub fn read_appearance(settings: &Path) -> Appearance {
    std::fs::read_to_string(settings)
        .ok()
        .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
        .unwrap_or_default()
        .appearance
}

/// Remember the palette the author chose.
///
/// **A failed write is reported**, where a failed read is not, for
/// [`write_override`]'s reason: the author has just pressed the control.
///
/// It writes this file and reads nothing else, so `projects.json` is not opened,
/// not rewritten and not at risk from a phase about a colour.
pub fn write_appearance(
    settings: &Path,
    appearance: Appearance,
) -> Result<(), String> {
    let text = serde_json::to_string_pretty(&Settings { appearance })
        .map_err(|e| format!("cannot write {}: {e}", settings.display()))?;

    if let Some(parent) = settings.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(settings, text).map_err(|e| format!("cannot write {}: {e}", settings.display()))
}

/// The third file this app writes, beside [`store_file`] and [`settings_file`]
/// and in the same directory: which sites each folder's author allowed images
/// to be fetched from. `mpdf-003` Phase 25.
///
/// **A third file and not a member of `projects.json`**, for [`settings_file`]'s
/// reason, which `writing_the_appearance_does_not_touch_the_store` holds: a
/// member beside the mains would make every store on disk malformed, and
/// malformed means forgotten.
///
/// **Per folder and per site, and remembered across launches.** Asking on every
/// launch trains a click nobody reads, and per *site* keeps it narrow: a `git
/// pull` that brings in a tracker's URL asks again rather than being fetched
/// silently.
pub fn sites_file(support: &Path) -> PathBuf {
    support.join("sites.json")
}

/// Which sites each root allows, keyed by the root as [`key`] spells it and
/// each list sorted, so two writes of the same consent produce the same bytes.
type Sites = std::collections::BTreeMap<String, Vec<String>>;

/// [`read_store`]'s rule, inherited whole: a missing, unreadable or malformed
/// file is nothing allowed, and never an error in the window. The failure state
/// is the state a first launch is in, and the button is still there to press.
fn read_all_sites(sites: &Path) -> Sites {
    std::fs::read_to_string(sites)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// The sites this root's author allowed, or none.
pub fn read_sites(sites: &Path, root: &Path) -> std::collections::BTreeSet<String> {
    read_all_sites(sites)
        .remove(&key(root))
        .unwrap_or_default()
        .into_iter()
        .collect()
}

/// Remember that this root allows exactly these sites.
///
/// **A failed write is reported**, for [`write_override`]'s reason: the author
/// has just pressed the button. It writes this file and reads nothing else, so
/// neither `projects.json` nor `settings.json` is opened.
pub fn write_sites(
    sites: &Path,
    root: &Path,
    allowed: &std::collections::BTreeSet<String>,
) -> Result<(), String> {
    let mut held = read_all_sites(sites);
    held.insert(key(root), allowed.iter().cloned().collect());

    let text = serde_json::to_string_pretty(&held)
        .map_err(|e| format!("cannot write {}: {e}", sites.display()))?;

    if let Some(parent) = sites.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(sites, text).map_err(|e| format!("cannot write {}: {e}", sites.display()))
}


/// The panel's stored half, walked off the disk — for the suite, which asks it
/// of a path where the window asks it of an open project.
#[cfg(test)]
pub fn files_under(root: &Path) -> Vec<Entry> {
    letur_project::document::files_under(&Disk::new(root))
}

/// A project on the disk: `letur_project`'s [`Files`], answered from a folder.
///
/// **The root is all it holds**, so a clone is a path and the desktop's
/// renders, which run off the session's lock, carry one each.
///
/// **Every answer keeps the sentence the desktop always gave.** [`Files::locate`]
/// is the absolute path, so a refusal names the file as it did before the rules
/// left this crate; [`Files::same`] resolves both sides, which is what the
/// pane's buffer override has compared by since `mpdf-010` Phase 2; and each
/// method below is a function this file already had.
#[derive(Debug, Clone)]
pub struct Disk {
    root: PathBuf,
}

impl Disk {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The folder the project is.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl Files for Disk {
    fn locate(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }

    fn read(&self, located: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(located)
    }

    fn same(&self, a: &Path, b: &Path) -> bool {
        crate::watch::resolve(a) == crate::watch::resolve(b)
    }

    /// [`walk`], which confines every link it meets.
    fn list(&self) -> Vec<String> {
        walk(&self.root)
    }

    /// [`confined`]'s question.
    fn holds(&self, path: &str) -> bool {
        confined(&self.root, path).is_some()
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> io::Result<()> {
        std::fs::write(self.root.join(path), bytes)
    }

    /// [`landing`], then `create_new`.
    ///
    /// **The file is made with `create_new` rather than checked and then
    /// written.** `O_EXCL` makes *already exists* the filesystem's own answer, so
    /// nothing here races with a check — and it refuses a **dangling symlink** at
    /// that name, which an `exists()` test would have reported absent and then
    /// written straight through, out of the project the parent was canonicalized
    /// to keep it in.
    fn create(&mut self, path: &str) -> Result<(), Refused> {
        let landed = landing(&self.root, path).ok_or(Refused::Landing)?;
        match std::fs::File::create_new(&landed) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Err(Refused::Exists),
            Err(e) => Err(Refused::Unwritten(e)),
        }
    }

    /// A move to the Trash, through [`trash_at`].
    fn remove(&mut self, path: &str) -> Result<(), Refused> {
        trash_at(&self.root, path, move_to_trash)
    }

    /// Write `bytes` where a save dialog pointed, and say whether that is one
    /// of the project's files.
    ///
    /// **It is not confined, and that is the decision.** `mpdf-003` Phase 17
    /// asked [`landing`] and refused anything outside the root, on the argument
    /// that `Open…` is how an author goes elsewhere. **That argument was
    /// retrofitted** — what drove it was that `landing` already existed — and a
    /// `Save as…` that cannot save where the author points it is not the gesture
    /// that name denotes. Phase 18 dropped the call.
    ///
    /// **The path is written as given**, so an absolute one from a dialog lands
    /// where it says and a *relative* one would resolve against the process
    /// working directory rather than the project. No caller passes a relative
    /// path — the dialog always answers absolute.
    ///
    /// **`create_new` is parted from, and that is the decision.** `O_EXCL` makes
    /// *already exists* a refusal, which is right for a `+` gesture that invents
    /// a name and wrong for a Save-as, whose purpose is sometimes to replace.
    /// `NSSavePanel` asks before it answers with a path that exists, so the
    /// confirmation is the platform's.
    ///
    /// **The inside test wants both halves, which is [`trash_file`]'s recorded
    /// shape**: *the name is under the root, and something is at it*. [`spell`]
    /// alone is **not** it — it is a component-wise `strip_prefix`, so
    /// `root.join("../escape.md")` strips to `../escape.md`, answers `Some` and
    /// would be judged inside. [`confined`] alone is not it either: under a
    /// symlinked root it resolves both sides and says inside where `spell`
    /// cannot produce a spelling at all. So a canonicalized comparison decides
    /// the confinement and `spell` decides the spelling. **`confined` is asked
    /// *after* the write and that order is forced**: it opens on `is_file`, so
    /// asked before it would answer `None` for every save-as to a name that does
    /// not exist yet. `mpdf-003` Phase 17, narrowed by Phase 19.
    fn save_as(&mut self, path: &str, bytes: &[u8]) -> io::Result<Option<String>> {
        let landed = PathBuf::from(path);
        std::fs::write(&landed, bytes)?;

        Ok(if confined(&self.root, path).is_some() {
            spell(&self.root, &landed)
        } else {
            None
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    /// A document's text, in the sentence the terminal prints when it will not
    /// read — `letur_project::document::read_document` over the disk, so the
    /// tail is the OS's own.
    fn read_document(document: &Path) -> Result<String, String> {
        let name = document.file_name().unwrap().to_string_lossy().into_owned();
        letur_project::document::read_document(&Disk::new(directory(document)), &name)
    }

    fn directory(document: &Path) -> &Path {
        document.parent().unwrap_or(Path::new(""))
    }

    // **The rules' own cases left with the rules**, `ltr-001` Phase 1: the
    // compile's two read passes, the anchors, discovery and the pane's buffer
    // override are `project/src/document.rs`'s tests now, by the same names.
    // What stays here is what only a disk can answer — the climb, the walk and
    // the links it will not follow, confinement, the Trash and the stores.

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures")
            .join(name)
    }

    /// A scratch directory that this test process owns, so runs do not
    /// collide and the repository stays clean.
    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("letur-test-{}", std::process::id()))
            .join(name);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A document that will not read names itself, in the sentence the
    /// terminal prints for the same file.
    #[test]
    fn a_document_that_will_not_read_names_the_path_and_the_reason() {
        let error = read_document(&fixture("no-such-document.md")).unwrap_err();

        assert!(error.contains("no-such-document.md"), "{error}");
        assert!(error.contains("os error"), "{error}");
    }

    #[test]
    fn the_title_is_the_documents_file_name() {
        assert_eq!(title(Path::new("/tmp/notes/paper.md")), "paper.md");
    }

    // -- the project: root, discovery, listing, store ----------------------
    //
    // `mpdf-010` Phase 1's exit gate, clauses 1 through 6. The fixture is
    // `tests/fixtures/panel/`, created by that phase: `samples/showcase/`
    // gains a `showcase.pdf` for any developer who has run its own README, and
    // `pdf` is in `IMAGE_EXTENSIONS`, so an exact-enumeration gate over the
    // sample tree is not reproducible by a second person.

    /// Clause 1. **The root climbs, and it climbs exactly once.**
    ///
    /// The first case is the whole phase's observable: opening a section from
    /// Finder must find the master above it, or the window compiles one section
    /// standalone as it does today.
    ///
    /// **The last case asserts the cap rather than accepting it.**
    /// `parts/ch1/deep.md` is named by `book.md` as `[](parts/ch1/deep.md)` — a
    /// supported marker, not merely an unrefused one — and it still roots at
    /// `parts/ch1`, because `parts/` holds no markdown for the climb to find.
    /// That is `specs/file_panel_spec.md` §2's stated cost, pinned here so a
    /// later change to the climb has to change this line deliberately.
    #[test]
    fn the_root_climbs_one_level_and_stops() {
        let panel = fixture("panel");

        assert_eq!(project_root(&panel.join("sections/text.md")), panel);
        assert_eq!(project_root(&panel.join("book.md")), panel);
        assert_eq!(
            project_root(&panel.join("loose/orphan.md")),
            panel.join("loose"),
            "no `.md` above `loose/` names it, so it is its own root"
        );
        assert_eq!(
            project_root(&panel.join("parts/ch1/deep.md")),
            panel.join("parts/ch1"),
            "the cap: `parts/` holds no markdown, so the climb finds nothing"
        );
    }

    /// Clause 4. **The listing, against the manifest beside the fixture.**
    ///
    /// The manifest is a `.txt` and sits in `tests/fixtures/` rather than in
    /// `tests/fixtures/panel/`, so it cannot become a row in the listing it
    /// defines.
    ///
    /// `sections/missing.md` is handed in as a path the master names. `book.md`
    /// deliberately does not name it: a master naming a file the disk lacks
    /// refuses with `MissingSection`, and the byte-for-byte claim in
    /// `crate::preview`'s own test needs `book.md` to compile.
    #[test]
    fn the_listing_is_the_disk_and_what_the_master_names() {
        let named = [
            "sections/text.md".to_string(),
            "parts/ch1/deep.md".to_string(),
            "sections/missing.md".to_string(),
        ];
        let listed = merge(files_under(&fixture("panel")), &named);

        let spelled: Vec<(&str, Kind, bool)> = listed
            .iter()
            .map(|entry| (entry.path.as_str(), entry.kind, entry.missing))
            .collect();

        assert_eq!(
            spelled,
            [
                ("book.md", Kind::Markdown, false),
                ("cover.jpg", Kind::Image, false),
                ("other.md", Kind::Markdown, false),
                ("plan.pdf", Kind::Image, false),
                ("refs.bib", Kind::Bibliography, false),
                ("refs.yml", Kind::Bibliography, false),
                ("loose/orphan.md", Kind::Markdown, false),
                ("parts/ch1/deep.md", Kind::Markdown, false),
                ("sections/mark.svg", Kind::Image, false),
                ("sections/missing.md", Kind::Markdown, true),
                ("sections/text.md", Kind::Markdown, false),
            ],
            "the listing and `tests/fixtures/panel-manifest.txt` disagree"
        );
    }

    /// Clause 5. **Confinement, tested where it can fail.**
    ///
    /// `tests/fixtures/panel/outside` is a symlink to
    /// `tests/fixtures/panel-decoy/`, a committed sibling so the link resolves
    /// the same on any clone. The target holds a `.md` and a `.png` that both
    /// match the filter — a link to a directory holding nothing it matched
    /// would pass under an implementation with no confinement at all, which is
    /// why the target holds two files that do.
    #[test]
    fn the_walk_does_not_follow_a_link_out_of_the_root() {
        let decoy = fixture("panel-decoy");
        assert!(
            fixture("panel/outside").is_dir(),
            "the fixture's symlink did not survive the checkout"
        );
        assert_eq!(
            files_under(&decoy)
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>(),
            ["decoy.md", "decoy.png"],
            "the decoy must hold rows the filter matches, or the clause below proves nothing"
        );

        assert!(
            files_under(&fixture("panel"))
                .iter()
                .all(|entry| !entry.path.starts_with("outside")),
            "a link out of the root put its target's files in the panel"
        );
    }

    /// Clause 6. The store round-trips, and a truncated one is nothing
    /// remembered rather than an error in the window.
    #[test]
    fn the_store_remembers_one_fact_per_root_and_forgives_a_bad_file() {
        let dir = scratch_dir("store");
        let store = store_file(&dir);
        let root = fixture("panel");
        let other = fixture("panel-pair");

        assert_eq!(read_override(&store, &root), None, "a store with no file");

        write_override(&store, &root, "other.md").unwrap();
        write_override(&store, &other, "beta.md").unwrap();
        assert_eq!(read_override(&store, &root).as_deref(), Some("other.md"));
        assert_eq!(read_override(&store, &other).as_deref(), Some("beta.md"));

        // A second write of the same root replaces rather than accumulates.
        write_override(&store, &root, "book.md").unwrap();
        assert_eq!(read_override(&store, &root).as_deref(), Some("book.md"));
        assert_eq!(read_override(&store, &other).as_deref(), Some("beta.md"));

        std::fs::write(&store, "{\"/some/root\": ").unwrap();
        assert_eq!(
            read_override(&store, &root),
            None,
            "a truncated store is nothing remembered, and never an error"
        );
    }

    // -- the second file, and what it must not touch -----------------------
    //
    // `mpdf-003` Phase 13. The appearance is global where the store's one fact
    // is per-root, so it is a second file rather than a second key — and the
    // third test below is what makes that a decision rather than a preference.

    /// All three appearances round-trip through `settings.json`.
    #[test]
    fn the_settings_file_remembers_each_of_the_three_appearances() {
        let dir = scratch_dir("settings-round-trip");
        let settings = settings_file(&dir);

        assert_eq!(
            read_appearance(&settings),
            Appearance::System,
            "a settings file that is not there yet"
        );

        for appearance in [
            Appearance::Light,
            Appearance::Dark,
            Appearance::System,
        ] {
            write_appearance(&settings, appearance).unwrap();
            assert_eq!(
                read_appearance(&settings),
                appearance,
                "the settings file did not give back what it was written"
            );
        }
    }

    /// A missing, a truncated and a malformed settings file each read as
    /// `System`, and none of the three is an error.
    ///
    /// **The failure state and the default state are the same state**, which is
    /// the whole of why this can be swallowed: following the system is what the
    /// app does anyway, so a broken file in Application Support costs the
    /// author their choice and never their document. `read_appearance` returns
    /// no `Result`, so "none is an error" is a fact about its signature that
    /// this test states rather than proves — what it proves is the value.
    #[test]
    fn a_settings_file_that_will_not_read_is_the_system_and_not_an_error() {
        let dir = scratch_dir("settings-forgiven");
        let settings = settings_file(&dir);
        let system = Appearance::System;

        assert_eq!(read_appearance(&settings), system, "no file at all");

        write_appearance(&settings, Appearance::Dark).unwrap();
        let whole = std::fs::read_to_string(&settings).unwrap();
        assert_eq!(
            read_appearance(&settings),
            Appearance::Dark,
            "the file this case truncates must first read as something else"
        );

        std::fs::write(&settings, &whole[..whole.len() / 2]).unwrap();
        assert_eq!(read_appearance(&settings), system, "a truncated file");

        std::fs::write(&settings, "{\"appearance\": \"chartreuse\"}").unwrap();
        assert_eq!(read_appearance(&settings), system, "a value serde refuses");

        std::fs::write(&settings, "not json at all").unwrap();
        assert_eq!(read_appearance(&settings), system, "not JSON");

        std::fs::write(&settings, "{}").unwrap();
        assert_eq!(
            read_appearance(&settings),
            system,
            "an object with no member"
        );
    }

    /// Writing the appearance leaves `projects.json` **byte-identical**.
    ///
    /// **This is the second-file decision's own check.** Had the appearance
    /// gone in beside the mains, every `projects.json` already on disk would
    /// have become malformed — and [`read_store`] reads malformed as nothing
    /// remembered, so the upgrade would have dropped every author's remembered
    /// main in silence. The clause is worded against the bytes and not against
    /// the reads, because a rewrite that happened to round-trip would still be
    /// this app touching a file it has no business in.
    #[test]
    fn writing_the_appearance_does_not_touch_the_store() {
        let dir = scratch_dir("settings-beside-store");
        let store = store_file(&dir);
        let settings = settings_file(&dir);
        let root = fixture("panel");

        write_override(&store, &root, "book.md").unwrap();
        let before = std::fs::read(&store).unwrap();

        write_appearance(&settings, Appearance::Dark).unwrap();
        write_appearance(&settings, Appearance::Light).unwrap();

        assert_eq!(
            std::fs::read(&store).unwrap(),
            before,
            "writing an appearance rewrote projects.json"
        );
        assert_eq!(
            read_override(&store, &root).as_deref(),
            Some("book.md"),
            "the remembered main did not survive an appearance being written"
        );
        assert_ne!(store, settings, "the two files must not be one file");
    }

    // -- the third file ----------------------------------------------------
    //
    // `mpdf-003` Phase 25. The sites a folder's author allowed are per-root, as
    // the store's one fact is, and they are a third file for the second's
    // reason — which the last test below holds.

    /// `sites.json` round-trips per root, and a malformed one is nothing
    /// allowed rather than an error in the window.
    #[test]
    fn the_sites_round_trip_per_root_and_a_bad_file_allows_nothing() {
        let dir = scratch_dir("sites-round-trip");
        let sites = sites_file(&dir);
        let root = fixture("panel");
        let other = fixture("panel-pair");
        let set = |named: &[&str]| -> std::collections::BTreeSet<String> {
            named.iter().map(|site| site.to_string()).collect()
        };

        assert!(read_sites(&sites, &root).is_empty(), "no file at all");

        write_sites(&sites, &root, &set(&["b.example", "a.example"])).unwrap();
        write_sites(&sites, &other, &set(&["c.example"])).unwrap();
        assert_eq!(read_sites(&sites, &root), set(&["a.example", "b.example"]));
        assert_eq!(read_sites(&sites, &other), set(&["c.example"]));

        // Sorted on disk, under the root the filesystem spells.
        let held: Sites = serde_json::from_str(&std::fs::read_to_string(&sites).unwrap()).unwrap();
        assert_eq!(
            held.get(&key(&root)),
            Some(&vec!["a.example".to_string(), "b.example".to_string()])
        );

        // A second write of the same root replaces rather than accumulates.
        write_sites(&sites, &root, &set(&["a.example"])).unwrap();
        assert_eq!(read_sites(&sites, &root), set(&["a.example"]));
        assert_eq!(read_sites(&sites, &other), set(&["c.example"]));

        for malformed in ["{\"/some/root\": ", "not json at all", "{\"/r\": \"a.example\"}"] {
            std::fs::write(&sites, malformed).unwrap();
            assert!(
                read_sites(&sites, &root).is_empty(),
                "{malformed:?} must read as nothing allowed"
            );
        }
    }

    /// Writing the sites leaves `projects.json` **and** `settings.json`
    /// byte-identical — the third-file decision's own check, worded against the
    /// bytes for `writing_the_appearance_does_not_touch_the_store`'s reason.
    #[test]
    fn writing_the_sites_does_not_touch_the_store() {
        let dir = scratch_dir("sites-beside-store");
        let store = store_file(&dir);
        let settings = settings_file(&dir);
        let sites = sites_file(&dir);
        let root = fixture("panel");

        write_override(&store, &root, "book.md").unwrap();
        write_appearance(&settings, Appearance::Dark).unwrap();
        let (mains, appearance) = (
            std::fs::read(&store).unwrap(),
            std::fs::read(&settings).unwrap(),
        );

        write_sites(&sites, &root, &["images.example".to_string()].into()).unwrap();
        write_sites(&sites, &root, &["other.example".to_string()].into()).unwrap();

        assert_eq!(
            std::fs::read(&store).unwrap(),
            mains,
            "writing the sites rewrote projects.json"
        );
        assert_eq!(
            std::fs::read(&settings).unwrap(),
            appearance,
            "writing the sites rewrote settings.json"
        );
        assert_ne!(sites, store);
        assert_ne!(sites, settings);
    }

    // -- one of the project's files, read for the window to draw ------------
    //
    // `mpdf-010` Phase 5. The panel has listed the project's figures since
    // Phase 1 and could not show one. The read is here rather than in the
    // command for this file's own reason, and these three clauses are what
    // that split buys.

    /// Clause 1. The bytes are the file's, whatever kind of figure it is.
    #[test]
    fn a_figure_reads_back_the_bytes_the_disk_holds() {
        let root = fixture("panel");
        for path in ["cover.jpg", "sections/mark.svg"] {
            assert_eq!(
                asset_bytes(&root, path).unwrap(),
                std::fs::read(root.join(path)).unwrap(),
                "{path} did not come back as the disk holds it"
            );
        }
    }

    /// Clause 2. A path that leaves the project is refused, and the second
    /// case is the one that can only be refused by confinement.
    ///
    /// `outside/decoy.png` is a file the disk really holds — through
    /// `tests/fixtures/panel/outside`, the committed symlink to
    /// `tests/fixtures/panel-decoy/` that the walk's own clause uses — so
    /// `is_file()` cannot refuse it and only the root-relative spelling can.
    #[test]
    fn a_figure_outside_the_project_is_refused_by_name() {
        let root = fixture("panel");

        assert!(
            root.join("outside/decoy.png").is_file(),
            "the decoy must be reachable through the link, or this clause proves nothing"
        );

        for path in ["/tmp/escape.png", "outside/decoy.png"] {
            assert_eq!(
                asset_bytes(&root, path).err().as_deref(),
                Some(format!("{path} is not a file in this project").as_str()),
                "{path} was not refused in the sentence the other two refusals use"
            );
        }
    }

    /// Clause 3. A `..` is refused **by the rule and not by the file being
    /// absent**, which is why the test writes the file it asks for.
    ///
    /// It runs over a scratch root rather than the fixture: `escape.png` has to
    /// exist in the root's *parent* for `is_file()` to pass and the
    /// confinement to be what refuses, and `tests/fixtures/` is tracked —
    /// [`scratch_dir`]'s own reason for existing.
    #[test]
    fn a_figure_reached_by_climbing_out_of_the_root_is_refused() {
        let above = scratch_dir("figure-escape");
        let root = above.join("project");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(above.join("escape.png"), b"not the project's").unwrap();

        assert!(
            root.join("../escape.png").is_file(),
            "the file has to be there, or `is_file()` refuses before the rule runs"
        );
        assert_eq!(
            asset_bytes(&root, "../escape.png").err().as_deref(),
            Some("../escape.png is not a file in this project")
        );
    }

    /// A symlink **into** the project is a row the panel offers, so it must be
    /// a row a command takes.
    ///
    /// `descend` lists a link under its own name once its target resolves under
    /// the root, so `cover.jpg -> figures/cover.jpg` is a row spelled
    /// `cover.jpg`. The rule these commands used to apply — ask [`relative`] for
    /// that spelling back — answered `figures/cover.jpg` and refused it, while
    /// the compile rendered the same file perfectly well. The walk and the
    /// commands ask one question now, and this is the case that told them apart.
    #[test]
    fn a_link_to_a_file_inside_the_project_is_a_file_in_this_project() {
        let root = scratch_dir("link-inside");
        std::fs::create_dir_all(root.join("figures")).unwrap();
        std::fs::write(root.join("figures/real.png"), b"the figure itself").unwrap();

        let link = root.join("cover.png");
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(root.join("figures/real.png"), &link).unwrap();

        assert!(
            files_under(&root)
                .iter()
                .any(|entry| entry.path == "cover.png"),
            "the walk must offer the row, or this case is not the one under test"
        );
        assert_eq!(
            asset_bytes(&root, "cover.png").unwrap(),
            b"the figure itself",
            "a link into the project was listed and then refused"
        );
        assert!(confined(&root, "cover.png").is_some());
    }

    /// And the link that leaves is still refused, by the same rule.
    ///
    /// Both halves in one test on purpose: a confinement that stopped refusing
    /// would pass the clause above and fail here, and the two are the same
    /// comparison read in opposite directions.
    #[test]
    fn a_link_out_of_the_project_is_refused_however_it_is_spelled() {
        let outside = scratch_dir("link-outside");
        std::fs::write(outside.join("secret.png"), b"not the project's").unwrap();

        let root = scratch_dir("link-outside-root");
        let link = root.join("secret.png");
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(outside.join("secret.png"), &link).unwrap();

        assert!(
            link.is_file(),
            "the link must resolve, or `is_file` refuses first"
        );
        assert_eq!(confined(&root, "secret.png"), None);
        assert_eq!(
            asset_bytes(&root, "secret.png").err().as_deref(),
            Some("secret.png is not a file in this project")
        );

        let elsewhere = scratch_dir("link-outside-dir");
        std::fs::write(elsewhere.join("deep.png"), b"nor this").unwrap();
        let door = root.join("door");
        let _ = std::fs::remove_file(&door);
        std::os::unix::fs::symlink(&elsewhere, &door).unwrap();
        assert!(root.join("door/deep.png").is_file());
        assert_eq!(confined(&root, "door/deep.png"), None);
    }

    // -- a file made from the panel -----------------------------------------
    //
    // `mpdf-010` Phase 3. This is the first time this app writes to a path the
    // author did not choose in a native dialog, so the confinement clauses
    // below are the point and the create is what carries them.
    //
    // **Each clause says where it runs.** The writes go to [`scratch_dir`], per
    // that helper's own reason; the symlink refusal runs over the committed
    // `tests/fixtures/panel/`, because that link resolves the same on any clone
    // and because a refusal writes nothing. The fixture does not grow, so
    // [`the_listing_is_the_disk_and_what_the_master_names`] keeps its eleven
    // rows and `tests/fixtures/panel-manifest.txt` is untouched.

    /// Clause 1. One file appears, it is empty, and nothing else in the tree
    /// moved — checked by walking the root before and after rather than by
    /// looking at the one path the create was given.
    #[test]
    fn a_create_puts_one_empty_file_in_the_project_and_nothing_else() {
        let root = scratch_dir("create-one");
        std::fs::create_dir_all(root.join("sections")).unwrap();
        // Scratch directories outlive a run whose process id is reused, and
        // `create_new` would then refuse for a reason that is not this clause.
        let _ = std::fs::remove_file(root.join("sections/note.md"));

        let before = walk(&root);
        create_file(&root, "sections/note.md").unwrap();
        let after = walk(&root);

        let appeared: Vec<&String> = after.iter().filter(|path| !before.contains(path)).collect();
        assert_eq!(
            appeared,
            [&"sections/note.md".to_string()],
            "the create put something other than the file it was asked for in the tree"
        );
        assert_eq!(
            after.len(),
            before.len() + 1,
            "the tree lost a path the create was not asked to touch"
        );
        assert_eq!(
            std::fs::read(root.join("sections/note.md")).unwrap(),
            Vec::<u8>::new(),
            "the created file is not empty"
        );
    }

    /// Clause 2. Both spellings of *outside* are refused, in the sentence that
    /// says so rather than merely in some sentence.
    ///
    /// **Neither path may exist**, and that is what makes the clause
    /// independent of the order the rules run in: a create's other refusal is
    /// *already exists*, so an implementation checking existence first would
    /// pass on the wrong rule for a path that happened to be there. It is the
    /// mirror of Phase 5's clause 3 rather than a copy — for a *read* the file
    /// has to exist or `is_file` refuses first.
    #[test]
    fn a_create_that_would_land_outside_the_project_is_refused_by_name() {
        let above = scratch_dir("create-escape");
        let root = above.join("project");
        std::fs::create_dir_all(&root).unwrap();

        for path in ["/tmp/escape.md", "../escape.md"] {
            let landed = root.join(path);
            assert!(
                !landed.exists(),
                "{} has to be absent, or this clause can pass on the exists-rule",
                landed.display()
            );
            assert_eq!(
                create_file(&root, path).err().as_deref(),
                Some(format!("{path} would land outside this project").as_str()),
                "{path} was not refused as leaving the project"
            );
            assert!(
                !landed.exists(),
                "a refused create still wrote {}",
                landed.display()
            );
        }
    }

    /// Clause 3. A link out of the project is refused, over the fixture whose
    /// link is committed.
    ///
    /// `Path::starts_with` is component-wise, so `…/fixtures/panel-decoy`
    /// genuinely fails against `…/fixtures/panel` rather than passing as a
    /// string prefix.
    #[test]
    fn a_create_through_a_link_out_of_the_project_is_refused() {
        let root = fixture("panel");

        assert!(
            root.join("outside").is_dir(),
            "the link must resolve, or the parent never canonicalizes and this proves nothing"
        );
        assert_eq!(
            create_file(&root, "outside/escape.md").err().as_deref(),
            Some("outside/escape.md would land outside this project")
        );
        assert!(
            !root.join("outside/escape.md").exists(),
            "a refused create wrote into the decoy"
        );
    }

    /// Clause 4. An existing path is refused and keeps its bytes.
    ///
    /// It is `create_new` that makes this true rather than a check the write
    /// races with, so the assertion is about the file and not about the order
    /// two statements run in.
    #[test]
    fn a_create_over_an_existing_file_is_refused_without_truncating_it() {
        let root = scratch_dir("create-exists");
        std::fs::write(root.join("held.md"), b"the author's own text").unwrap();

        assert_eq!(
            create_file(&root, "held.md").err().as_deref(),
            Some("held.md already exists")
        );
        assert_eq!(
            std::fs::read(root.join("held.md")).unwrap(),
            b"the author's own text",
            "a refused create truncated the file it refused"
        );
    }

    /// Clause 5. The kinds are the pipeline's own, read off the extension.
    ///
    /// **`.yml` and `.yaml` are the clause that matters.** They are what a
    /// hand-written `.md`-or-`.bib` subset would refuse, and
    /// `core/src/bibliography.rs` reads all three — so a panel that listed a
    /// bibliography it could not create would be the drift `mpdf-010` §2
    /// refuses for `.jpg`.
    #[test]
    fn a_create_takes_the_kinds_the_pipeline_reads_and_no_others() {
        let root = scratch_dir("create-kinds");

        for path in ["notes.typ", "notes", "figure.png"] {
            let _ = std::fs::remove_file(root.join(path));
            assert_eq!(
                create_file(&root, path).err().as_deref(),
                Some(
                    format!(
                        "{path} is neither markdown nor a bibliography: \
                         a new file is a .md, .bib, .yml or .yaml"
                    )
                    .as_str()
                ),
                "{path} is not a file the pipeline reads and was not refused as one"
            );
            assert!(!root.join(path).exists(), "{path} was created anyway");
        }

        for path in ["section.md", "refs.bib", "refs.yml", "refs.yaml"] {
            let _ = std::fs::remove_file(root.join(path));
            create_file(&root, path).unwrap_or_else(|e| panic!("{path} was refused: {e}"));
            assert!(
                root.join(path).is_file(),
                "{path} was accepted and not made"
            );
        }
    }

    // -- a file moved to the Trash ------------------------------------------
    //
    // `mpdf-010` Phase 4. **Every clause here runs against a double**, and
    // nothing below puts a file in anybody's Trash: the call's whole effect is
    // outside this repository, where [`scratch_dir`]'s "the repository stays
    // clean" has no reach at all.
    //
    // **The double removes the file as well as recording the call.** Clauses 1
    // and 3 each turn on the file being gone afterwards, and a double that only
    // counted would pass both over an unchanged tree.
    //
    // Each clause says where it runs, per Phase 3's standard.

    /// What the double was asked to move, as the clauses read it back.
    type Moved = Arc<Mutex<Vec<PathBuf>>>;

    /// A [`trash_file`] call that records what it was asked to move and moves
    /// it — to nowhere, which is what makes the clauses cheap.
    fn recording() -> (impl FnOnce(&Path) -> Result<(), String>, Moved) {
        let moved = Arc::new(Mutex::new(Vec::new()));
        let taken = Arc::clone(&moved);

        let double = move |path: &Path| {
            taken.lock().unwrap().push(path.to_path_buf());
            std::fs::remove_file(path).map_err(|e| format!("the double could not remove: {e}"))
        };
        (double, moved)
    }

    /// Clause 1. The file goes, exactly one call is made with the path
    /// [`landing`] answers, and nothing else in the tree moved.
    ///
    /// It is [`a_create_puts_one_empty_file_in_the_project_and_nothing_else`]
    /// run backwards, walking the root before and after rather than looking at
    /// the one path the delete was given.
    #[test]
    fn a_trash_takes_the_file_it_was_given_and_nothing_else() {
        let root = scratch_dir("trash-one");
        std::fs::create_dir_all(root.join("sections")).unwrap();
        std::fs::write(root.join("sections/note.md"), b"a section").unwrap();

        let before = walk(&root);
        let (double, moved) = recording();
        trash_file(&root, "sections/note.md", double).unwrap();
        let after = walk(&root);

        assert_eq!(
            *moved.lock().unwrap(),
            [root.join("sections/note.md")],
            "the call was not made once with the path `landing` answers"
        );
        let gone: Vec<&String> = before.iter().filter(|path| !after.contains(path)).collect();
        assert_eq!(
            gone,
            [&"sections/note.md".to_string()],
            "the trash took something other than the file it was asked for"
        );
        assert_eq!(
            after.len(),
            before.len() - 1,
            "the tree gained a path the trash was not asked to touch"
        );
    }

    /// Clause 2. Both spellings of *outside* are refused, over scratch.
    ///
    /// **The mirror of [`a_create_that_would_land_outside_the_project_is_refused_by_name`]
    /// and not a copy.** That one requires the path to be *absent*, because for
    /// a create an existing file is refused by the exists-rule first. For a
    /// delete that inverts exactly: the file has to be **there**, or the
    /// not-there rule refuses and the confinement rule never executes.
    ///
    /// **The absolute spelling is this test's own scratch and not a literal
    /// `/tmp/escape.md`**, which is the one difference from Phase 3's clause and
    /// is forced by the inversion: that one asserts the path is *absent*, so a
    /// shared name costs nothing, where this one has to **create** it — and a
    /// delete clause writing `/tmp/escape.md` is a delete clause breaking
    /// [`a_create_that_would_land_outside_the_project_is_refused_by_name`] on
    /// the same machine. Both spellings still name one file in the root's
    /// parent, which is what the clause is about.
    #[test]
    fn a_trash_that_would_leave_the_project_is_refused_by_name() {
        let above = scratch_dir("trash-escape");
        let root = above.join("project");
        std::fs::create_dir_all(&root).unwrap();

        let escape = above.join("escape.md");
        let absolute = escape.to_string_lossy().into_owned();

        for path in [absolute.as_str(), "../escape.md"] {
            let landed = root.join(path);
            std::fs::write(&landed, b"not the project's").unwrap();
            assert!(
                landed.exists(),
                "{} has to be there, or the not-there rule refuses first",
                landed.display()
            );

            let (double, moved) = recording();
            assert_eq!(
                trash_file(&root, path, double).err().as_deref(),
                Some(format!("{path} is outside this project").as_str()),
                "{path} was not refused as leaving the project"
            );
            assert!(
                moved.lock().unwrap().is_empty(),
                "a refused trash called the OS anyway for {path}"
            );
            assert!(
                landed.exists(),
                "a refused trash still took {}",
                landed.display()
            );
        }
    }

    /// Clause 3. A link out of the project takes the **link** and leaves its
    /// target.
    ///
    /// This is the clause that distinguishes this rule from [`confined`], which
    /// refuses the same row —
    /// [`a_link_out_of_the_project_is_refused_however_it_is_spelled`] pins that.
    /// A read wants the bytes the author meant; a delete wants the row the
    /// author clicked, and the target of a link out of the project is not the
    /// project's to move.
    #[test]
    fn a_link_out_of_the_project_is_trashed_and_its_target_is_not() {
        let outside = scratch_dir("trash-link-target");
        let target = outside.join("secret.png");
        std::fs::write(&target, b"not the project's").unwrap();

        let root = scratch_dir("trash-link-root");
        let link = root.join("secret.png");
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert_eq!(
            confined(&root, "secret.png"),
            None,
            "the read rule must refuse this row, or the clause proves nothing"
        );

        let (double, moved) = recording();
        trash_file(&root, "secret.png", double).unwrap();

        assert_eq!(
            moved.lock().unwrap().as_slice(),
            std::slice::from_ref(&link),
            "the join was not the path moved"
        );
        assert!(link.symlink_metadata().is_err(), "the link is still there");
        assert!(
            target.is_file(),
            "the trash followed the link and took the target"
        );
    }

    /// Clause 4. Nothing at that name is refused, in a sentence of its own.
    ///
    /// **One sentence and not two.** [`merge`] marks a row missing *because*
    /// [`files_under`] did not find it, so a `missing: true` path and a plain
    /// absent one are the same `symlink_metadata` failure inside a function that
    /// never sees `crate::preview::Preview`'s sections. Both inputs run all the
    /// same, since what is pinned is that the panel's marked-missing row is not
    /// a special case.
    #[test]
    fn a_trash_of_nothing_is_refused_and_the_marked_missing_row_is_no_special_case() {
        let root = scratch_dir("trash-absent");
        std::fs::create_dir_all(root.join("sections")).unwrap();

        // The second is what a `missing: true` row spells: `book.md` names it
        // and the disk does not hold it. The rule cannot tell them apart, and
        // that is the claim.
        for path in ["gone.md", "sections/missing.md"] {
            let _ = std::fs::remove_file(root.join(path));

            let (double, moved) = recording();
            assert_eq!(
                trash_file(&root, path, double).err().as_deref(),
                Some(format!("{path} is not there").as_str()),
                "{path} was not refused as absent"
            );
            assert!(
                moved.lock().unwrap().is_empty(),
                "a refused trash called the OS anyway for {path}"
            );
        }
    }
}
