//! The pane kept current: the watch, the keyboard and the fetch worker.
//!
//! [`Session`] is one open document's state plus the loops that keep it up to
//! date — the watch, the keyboard, and since `mpdf-003` Phase 25 a worker that
//! fetches images named by URL. None of them needs a window, so all are tested
//! by ordinary tests rather than by a screenshot.
//!
//! **The state and every rule about it are `letur-project`'s since `ltr-001`
//! Phase 1.** [`Preview`] is `letur_project::preview::Preview` over [`Disk`],
//! and each command here asks it first — it decides, refuses and composes the
//! receipt — then does what only this crate can: take the lock, announce, arm
//! the loops, write Application Support, and time a compile. So what a refusal
//! says, and what a counter reads, are decided once, for this window and the
//! browser's alike.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

pub use letur_project::preview::{Appearance, Asked, External, Status, Trashed};

use crate::document::{self, Disk};
use crate::remote;
use crate::watch::{self, Change, Changed, Watch};

/// The pane's state, over the disk.
pub type Preview = letur_project::preview::Preview<Disk>;

/// What the desktop asks of a [`Preview`] that only a disk can answer: where
/// the project is, and where its files are, as absolute paths.
pub trait OnDisk {
    /// The project the panel is listing, if one is open.
    fn root(&self) -> Option<&Path>;
    /// The file the pane is showing, if one is open.
    fn document(&self) -> Option<PathBuf>;
    /// Where a Save-a-copy dialog opens, or why it does not open at all: the
    /// crate's `export_path`, joined onto the root.
    fn export_file(&self) -> Result<PathBuf, String>;
    /// Write the page's own bytes where the user asked.
    ///
    /// **Nothing here compiles.** The export writes what the pane is already
    /// showing, so the file and the page cannot disagree.
    fn export(&self, path: &Path) -> Result<(), String>;
}

impl OnDisk for Preview {
    fn root(&self) -> Option<&Path> {
        self.files().map(Disk::root)
    }

    fn document(&self) -> Option<PathBuf> {
        Some(self.root()?.join(self.edited()?))
    }

    fn export_file(&self) -> Result<PathBuf, String> {
        let name = self.export_path()?;
        self.root()
            .map(|root| root.join(name))
            .ok_or_else(|| "no document is open".to_string())
    }

    fn export(&self, path: &Path) -> Result<(), String> {
        let pdf = self.exportable()?;
        std::fs::write(path, pdf).map_err(|e| format!("cannot write {}: {e}", path.display()))
    }
}

/// How this app times the compiles a [`Preview`] runs itself.
fn timed(run: &mut dyn FnMut()) -> Duration {
    let started = Instant::now();
    run();
    started.elapsed()
}

/// One compile, planned under the lock, with the disk it reads.
///
/// **The crate's `Compile` carries no files**, since the browser's runs over a
/// map it must not copy per compile; this one carries a clone of the root, so
/// the render can run with no lock held. **Public, with every field private**,
/// for one reason: `crate::main` hands [`Compile::run`] to [`Session::new`] as
/// the fetch worker's render, the seam a test replaces.
pub struct Compile {
    plan: letur_project::preview::Compile,
    disk: Disk,
}

impl Compile {
    /// Compile, and time it. **This is the whole of what runs outside the lock.**
    ///
    /// The duration is measured around the failed path too, which costs one
    /// `Instant` and changes nothing observable: `Preview::absorb` reads it in
    /// the success arm alone.
    pub fn run(&self) -> Rendered {
        let started = Instant::now();
        let outcome = self.plan.render(&self.disk);
        (outcome, started.elapsed())
    }

    /// Which compile this is, in the order they started — for the suite's
    /// gate, which releases a render by serial rather than by arrival.
    #[cfg(test)]
    pub fn serial(&self) -> u64 {
        self.plan.serial()
    }
}

/// `Preview::plan`, with the disk cloned out beside it.
fn plan(preview: &mut Preview) -> Option<Compile> {
    let plan = preview.plan()?;
    let disk = preview.files()?.clone();
    Some(Compile { plan, disk })
}

/// `Preview::absorb`, for a plan made by [`plan`].
fn absorb(preview: &mut Preview, compile: &Compile, rendered: Rendered) {
    let (outcome, took) = rendered;
    preview.absorb(&compile.plan, outcome, took);
}

/// One open document: its preview, and the two loops that keep it current.
///
/// Opening a second document moves both, because the old [`Watch`] and the old
/// typing channel are dropped before the new ones start.
pub struct Session {
    state: Arc<Mutex<Preview>>,
    on_render: Announce,
    /// The first of the three files this app writes outside the author's own
    /// folders: which root is remembered as compiling which file.
    ///
    /// **It is a parameter and not a call to the platform**, so a test hands in
    /// a scratch directory and the rule that reads it stays on the testable
    /// side of the window. `crate::main` resolves the real one from Tauri's own
    /// path resolver, which is the authority on the bundle identifier.
    store: PathBuf,
    /// The second of the three files this app writes outside the author's own
    /// folders: which palette the window wears.
    ///
    /// **A second file and not a second key in the first**, and the reason is
    /// checkable rather than aesthetic: `projects.json` deserializes as a
    /// `BTreeMap<String, String>` and `crate::document::read_store` swallows a
    /// parse failure, so reshaping that file around a `theme` member would make
    /// every existing one malformed — and malformed means forgotten. Every
    /// author's remembered main would be dropped by the upgrade.
    ///
    /// It is a parameter for [`Session::store`]'s reason, and the same test
    /// hands in the same scratch directory.
    settings: PathBuf,
    /// The third: which sites each folder's author allowed images to be
    /// fetched from. Read at every open, written at every press, and a
    /// parameter for [`Session::store`]'s reason. `mpdf-003` Phase 25.
    sites: PathBuf,
    /// The palette the author chose, read from [`Session::settings`] at launch.
    ///
    /// **It is here and not on [`Preview`]** because it is global and a
    /// `Preview` is one document's: [`Session::open_at`] rebuilds that struct
    /// from `Preview::default()`, so a preference kept there would reset on
    /// every open, after which the footer's mark flips and the settings file
    /// disagrees with the running window.
    appearance: Appearance,
    watch: Option<Watch>,
    typing: Option<mpsc::Sender<()>>,
    /// The fetch worker's channel, for the press to send on.
    ///
    /// **The second sender, beside the one the [`Preview`]'s claim callback
    /// owns**, since `ltr-001` Phase 1 moved that struct into a crate with no
    /// channels. Both drop with this `Session`, so the worker's `recv` still
    /// ends when it does.
    claims: mpsc::Sender<Claim>,
}

impl Session {
    /// A session that calls `on_render` after every compile, its own included.
    ///
    /// The callback carries no payload. The window's copy of it emits an event
    /// and the page then asks for the bytes, because handing them through the
    /// event would serialize them as a JSON array of numbers, one per byte.
    ///
    /// **It starts the fetch worker**, `mpdf-003` Phase 25, and hands it the
    /// fetch and the render as seams: production passes `remote::fetch` and
    /// [`Compile::run`], and a test passes fakes, so no case in the suite
    /// touches the network and a case can hold a compile open. The worker
    /// holds a [`Weak`] to the state, and the new [`Preview`] holds the
    /// channel's senders, one in its claim callback and one here — so a dropped
    /// `Session` takes both, the worker's `recv` ends, and the thread exits. The suite builds
    /// hundreds of these.
    pub fn new(
        store: PathBuf,
        settings: PathBuf,
        sites: PathBuf,
        appearance: Appearance,
        fetch: impl Fn(&str) -> Result<Vec<u8>, String> + Send + Sync + 'static,
        render: impl Fn(&Compile) -> Rendered + Send + Sync + 'static,
        on_render: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let (claims, claimed) = mpsc::channel();
        let claiming = claims.clone();
        let state = Arc::new(Mutex::new(Preview::new(
            timed,
            Some(Box::new(move |url| {
                let _ = claiming.send(Claim::Fetch { url, settle: true });
            })),
        )));
        let on_render: Announce = Arc::new(on_render);

        let worker = Worker {
            state: Arc::downgrade(&state),
            fetch: Arc::new(fetch),
            render: Arc::new(render),
            on_render: Arc::clone(&on_render),
        };
        std::thread::spawn(move || {
            for claim in claimed {
                let worker = worker.clone();
                std::thread::spawn(move || worker.carry(claim));
            }
        });

        Self {
            state,
            on_render,
            store,
            settings,
            sites,
            appearance,
            watch: None,
            typing: None,
            claims,
        }
    }

    /// What the pane is showing. Phase 3's export reads the same values.
    pub fn preview(&self) -> std::sync::MutexGuard<'_, Preview> {
        self.state.lock().expect("the preview lock was poisoned")
    }

    /// Everything the window says, from the two places that know it.
    ///
    /// **The composition is here rather than in [`Preview::status`], and the
    /// split is the point**: twelve of the thirteen fields are the last compile's
    /// and one is the author's, held for a document's lifetime rather than for
    /// a compile's. [`Preview::status`] keeps its signature because some
    /// thirty-five call sites read it, nearly all of them tests.
    ///
    /// `crate::main::status` calls **this** and not `preview().status()`, which
    /// is the one line no test in this crate reaches.
    pub fn status(&self) -> Status {
        Status {
            appearance: self.appearance,
            ..self.preview().status()
        }
    }

    /// Wear a palette, remember it, and say so.
    ///
    /// **All three, and here rather than in a command**, because
    /// [`Session::on_render`] is private to this module: a command that
    /// announced from `crate::main` would have to emit the signal itself and
    /// duplicate the one path this app has. The loose reading — a command that
    /// wrote the file and never announced — has a failure mode nothing would
    /// catch: the title bar right, the file right, and the footer's mark stale
    /// until the next compile.
    ///
    /// It announces through the compile signal for the reason
    /// [`Session::set_edited`] does, compiling nothing either: the page's
    /// `refresh` guards on the revision, so nothing recompiles and nothing
    /// redraws. A second signal for one field would be a second path doing the
    /// job of the one that exists.
    ///
    /// **The write is reported where a read is not**: the author has just asked
    /// for it, which is the rule `crate::document::write_override` already
    /// follows.
    pub fn set_appearance(&mut self, appearance: Appearance) -> Result<(), String> {
        document::write_appearance(&self.settings, appearance)?;
        self.appearance = appearance;
        (self.on_render)();
        Ok(())
    }

    /// Open what the author picked: find the project it sits in, read the file
    /// that project compiles, and watch the whole of it from now on.
    ///
    /// **This opens the file it landed on rather than the file it was handed.**
    /// The root is `crate::document::project_root`'s one-level climb, so a
    /// double-click on a section finds the master above it; the main is the
    /// store's answer for that root when it has one, and
    /// `crate::document::discover_main` when it does not.
    ///
    /// **The store is read first, on every open**, or it is a thing written and
    /// never used. An override naming a file the disk no longer holds falls
    /// through to discovery rather than opening nothing.
    ///
    /// The previous document's page and text go with it. A page kept across an
    /// open would belong to a file the window no longer names.
    pub fn open(&mut self, opened: PathBuf) -> Result<(), String> {
        let root = document::project_root(&opened);
        let main = document::read_override(&self.store, &root)
            .filter(|main| root.join(main).is_file())
            .unwrap_or_else(|| document::discover_main(&root, &opened));

        self.open_at(root, main)
    }

    /// The open, once the root and the main are decided.
    ///
    /// Shared with [`Session::set_main`], so the store and the window can never
    /// disagree about which file compiles: there is one path that puts a
    /// document in the pane, and both callers take it.
    ///
    /// **What survives the replacement is the crate's to keep** —
    /// `letur_project::preview::Preview::open` carries the serials, the fetches
    /// and the claim across it, and says why each one must. The sites are read
    /// here and handed in, since `sites.json` is this app's.
    fn open_at(&mut self, root: PathBuf, main: String) -> Result<(), String> {
        {
            let mut preview = self.preview();
            let allowed = document::read_sites(&self.sites, &root);
            preview.open(Disk::new(&root), main.clone(), main.clone(), allowed);
            preview.load();
        }
        (self.on_render)();
        self.arm(root, &main, &main)
    }

    /// Point both loops at these two files, dropping whatever they held.
    ///
    /// **Shared with [`Session::set_edited`], which is not an open.** Both
    /// closures guard on `Preview::edited` against a path captured when they
    /// were started, so a command that moved the pane and stopped there would
    /// leave the typing debounce compiling nothing and every filesystem event
    /// dropped. The guard itself stays, and since `mpdf-003` Phase 22 it is
    /// asked twice: once before [`Preview::plan`] and once after the render, on
    /// the way back in. What stops a thread mid-compile from writing its page
    /// over a newer one is no longer the guard but [`Preview::current`] and the
    /// serial [`Preview::absorb`] tests.
    ///
    /// The old loops go before the new ones start, so no two of them ever hold
    /// the same document.
    ///
    /// `main` and `edited` are root-relative, as the [`Preview`] holds them; the
    /// loops are handed them joined onto the root, and guard on that absolute
    /// path — so a loop left over from another project cannot pass for this
    /// one's on a file of the same name.
    fn arm(&mut self, root: PathBuf, main: &str, edited: &str) -> Result<(), String> {
        self.watch = None;
        self.typing = None;

        let edited = root.join(edited);
        self.typing = Some(watch::debounced(
            watch::TYPING_DEBOUNCE,
            self.recompile(edited.clone()),
        ));
        let classify = self.classifier(root.clone(), root.join(main), edited.clone());
        let on_change = self.on_change(edited);
        self.watch = Some(watch::start(&root, watch::DEBOUNCE, classify, on_change)?);

        Ok(())
    }

    /// Set which file under the open root compiles, and remember it.
    ///
    /// The store is written *before* the open, so a window that opened and then
    /// failed to remember cannot happen: the fact is on disk or the author is
    /// told why it is not.
    ///
    /// **The confinement and the refusal are the crate's**, in
    /// `letur_project::preview::Preview::ask_main`; the store and the open are
    /// this app's.
    pub fn set_main(&mut self, main: String) -> Result<(), String> {
        let asked = self.preview().ask_main(&main)?;
        if asked == Asked::Refused {
            (self.on_render)();
            return Ok(());
        }

        let root = self
            .preview()
            .root()
            .map(Path::to_path_buf)
            .ok_or_else(|| "no document is open".to_string())?;
        document::write_override(&self.store, &root, &main)?;
        self.open_at(root, main)
    }

    /// Put another of the project's files in the pane, leaving the main alone.
    ///
    /// **This is not an open, and the difference is the counters.**
    /// [`Session::open_at`] assigns `Preview { ..Preview::default() }`, which
    /// zeroes `revision` and `reloaded`, and `app/dist/index.html`'s `clear()`
    /// — which resets the counters the page compares them against — runs on an
    /// Open and not on a row click. So this sets `edited`, reads that file into
    /// the buffer, and leaves the root, the main, the listing, the bytes and
    /// both counters exactly as it found them: they *advance* here, they do not
    /// restart.
    ///
    /// It confines the path as [`Session::set_main`] does, and refuses on the
    /// same terms while the buffer diverges from the last-saved text.
    pub fn set_edited(&mut self, path: String) -> Result<(), String> {
        let (asked, root, main) = {
            let mut preview = self.preview();
            let asked = preview.set_edited(&path)?;
            let root = preview.root().map(Path::to_path_buf);
            (asked, root, preview.main().map(str::to_string))
        };
        (self.on_render)();
        if asked == Asked::Refused {
            return Ok(());
        }

        match (root, main) {
            (Some(root), Some(main)) => self.arm(root, &main, &path),
            _ => Err("no document is open".to_string()),
        }
    }

    /// Write the pane to a path the author picked, and hold that file after.
    ///
    /// **Three duties the watch will not do for this command**, which is what
    /// separates it from every other write in this file.
    ///
    /// **Since `ltr-001` Phase 1 the write, the three moves, the listing, the
    /// compile and the receipt are `letur_project::preview::Preview::save_as`'s**,
    /// and what stays here is the announce and the re-arm. What follows is why
    /// each duty is one.
    ///
    /// **It compiles.** The compile substitutes the buffer for
    /// `edited` alone and reads every other path off the disk, so moving
    /// `edited` changes what the next compile reads — onto a file the master
    /// names, or *away* from one, which is the case an author hits by default:
    /// pane on the master with unsaved edits, saved under a new name, and the
    /// master goes back to its own on-disk text. **The write's own event will
    /// not cause that compile.** [`crate::watch::classify`] answers on first
    /// match and that match is [`crate::watch::Change::Edited`], whose
    /// `reload()` answers [`External::Unchanged`] — [`Preview::save_as`] having
    /// just made the buffer and the file agree — so [`Session::on_change`] sets
    /// `announce = false` and compiles nothing.
    ///
    /// **It rebuilds the tree by hand**, exactly as [`Session::trash`] does and
    /// for the same reason: that same first match means the event never reaches
    /// `Change::Tree`, so the file just written would have no row in the panel —
    /// or would have one only depending on where the debounce fell relative to
    /// the move, which is worse than never.
    ///
    /// **It announces**, which is what carries the new [`Status`] to the footer
    /// cell, the marked row and the title the command sets beside it.
    ///
    /// **Two of the three are conditional since `mpdf-003` Phase 19, and one is
    /// not.** Where the pane did not follow the write, nothing moved: there is
    /// nothing to re-arm and nothing new to read.
    ///
    /// **Arming would be worse than pointless — it would stop the window.**
    /// [`Session::arm`]'s two closures both open by comparing `Preview::edited`
    /// against a path captured when they were built, so arming on an outside
    /// `landed` while `edited` stays inside would make every filesystem event and
    /// every typing recompile return early. **The compile is dropped for a
    /// quieter reason**: it would read exactly what it read before, and
    /// [`Preview::compile`] bumps `revision`, which the page's `refresh` treats
    /// as a reason to re-fetch and re-place the reader — a redraw `revision`
    /// exists to prevent.
    ///
    /// **The tree rebuild and the announce stay on both paths.** Both are no-ops
    /// outside — `document::files_under` finds nothing new, and an unchanged
    /// [`Status`] stops at the page's `revision === drawnRevision` guard — and
    /// the same call is what lists the file when it landed inside. Branching on
    /// where it landed would be two paths where the walk is one.
    ///
    /// **It answers a receipt**, `saved as <name> in <folder>`, for [`SAVED`]'s
    /// reasons and with one addition of its own: a save outside the project is a
    /// copy, and the sentence says which rather than leaving it to be inferred
    /// from a footer cell that names an outside file exactly as it names a
    /// project one.
    ///
    /// **The folder is the containing directory's absolute path, spelled
    /// plainly**, and it is not split root-relative-inside / absolute-outside.
    /// `document::spell` ends `(!spelled.is_empty()).then_some(spelled)`, so a
    /// destination *directly in the root* answers `None` — and saving beside the
    /// file you are editing is exactly that case, `crate::main::save_as_path`
    /// defaulting the panel there. **Nor is it abbreviated to `~`**: there is no
    /// home resolution anywhere in this workspace, every platform-resolved path
    /// reaches [`Session`] as a constructor parameter for the reason
    /// [`Session::store`]'s own comment records, and a lookup here would be the
    /// untestable platform call that rule exists to forbid.
    ///
    /// **It is spelled as landed and not canonicalized**, which is what makes the
    /// sentence the author's own path back rather than the filesystem's answer
    /// about it.
    ///
    /// `mpdf-003` Phase 17, narrowed by Phase 19.
    pub fn save_as(&mut self, path: String) -> Result<String, String> {
        let (saved, root, main, edited) = {
            let mut preview = self.preview();
            let saved = preview.save_as(&path)?;
            (
                saved,
                preview.root().map(Path::to_path_buf),
                preview.main().map(str::to_string),
                preview.edited().map(str::to_string),
            )
        };
        (self.on_render)();

        if saved.moved
            && let (Some(root), Some(main), Some(edited)) = (root, main, edited)
        {
            self.arm(root, &main, &edited)?;
        }

        Ok(saved.receipt)
    }

    /// Move one of the project's files to the Trash.
    ///
    /// **Every decision is `letur_project::preview::Preview::trash`'s** — the
    /// main refused, the pane's own unsaved work refused on `TRASHING`, the
    /// listing refreshed, and the pane falling back to the main — and this is
    /// what only a session has: the lock, the announce, and [`Session::arm`].
    /// The delete itself is `crate::document::trash_file`, handed the call that
    /// moves the file, which is a parameter so the suite can hand in a double.
    ///
    /// **Three refusals, and they do not arrive the same way.** The main and the
    /// two `crate::document::trash_file` makes come back as `Err`, which the
    /// page draws in the error bar exactly as it draws [`Session::set_edited`]'s
    /// — and none of the three is reachable from a row: the main row draws no
    /// button, and every other row came out of this app's own listing. The
    /// dirty-buffer one rides the divergence and returns `Ok(())`, so one
    /// refusal does not arrive in the window two ways.
    ///
    /// **That one is asked only of the file the pane is holding**, which is the
    /// difference from [`Session::set_edited`] and [`Session::set_main`], where
    /// it is unconditional: deleting some *other* file throws no unsaved work
    /// away, so refusing there would be a refusal with nothing behind it.
    ///
    /// **The panel is refreshed by the crate and not by the watch, and that is
    /// forced.** `crate::watch::classify` answers the **first** match and a
    /// section the master names is already in the asset list the compile builds
    /// — so deleting one answers `Change::Asset`, never `Change::Tree`, and
    /// [`Session::on_change`] refreshes the listing only under `changed.tree`.
    /// **The asymmetry with the create is real rather than an inconsistency**: a
    /// *created* file is not in the asset list, so the watch classifies that one
    /// correctly. This app made the change and knows it; the watch is for
    /// changes it did not make.
    ///
    /// `mpdf-010` Phase 4.
    pub fn trash(
        &mut self,
        path: String,
        trash: impl FnOnce(&Path) -> Result<(), String>,
    ) -> Result<(), String> {
        let (trashed, root, main) = {
            let mut preview = self.preview();
            let trashed = preview.trash(&path, |disk, path| {
                document::trash_file(disk.root(), path, trash)
            })?;
            (
                trashed,
                preview.root().map(Path::to_path_buf),
                preview.main().map(str::to_string),
            )
        };
        (self.on_render)();

        // **Arming is load-bearing where the pane held the file**: both loops
        // guard on a path captured when they were started, and the crate has
        // just moved the pane back to the main and loaded it.
        match (trashed, root, main) {
            (Trashed::Removed { holding: true }, Some(root), Some(main)) => {
                self.arm(root, &main, &main)
            }
            _ => Ok(()),
        }
    }

    /// The author pressed the button beside the line: fetch the document's
    /// images from the web.
    ///
    /// **The press is the consent**, and it is as wide as the sentence it sits
    /// beside: every site the document names is allowed for this folder, and
    /// remembered in `sites.json`. The file is written *before* the sites are
    /// worn, [`Session::set_main`]'s order, so a write that fails is reported
    /// and memory and disk never disagree about what was allowed.
    ///
    /// Then every named URL nothing has asked about, or whose fetch failed, is
    /// claimed **without the settle** — the press is the one thing that skips
    /// it. **And the worker is always sent a compile, even when no fetch
    /// starts**: the bytes may already be in memory from a project that allowed
    /// the site first, and a press that only widens consent must still redraw.
    ///
    /// **It holds no lock across the network or the render**, since
    /// `crate::main`'s command holds `Mutex<Session>` while it runs, as `edit`'s
    /// does. It does not announce: the worker announces the moment a fetch
    /// goes out, and a status read between the two would show the line empty.
    /// `mpdf-003` Phase 25.
    pub fn fetch_images(&self) -> Result<(), String> {
        let mut preview = self.preview();
        let root = preview
            .root()
            .map(Path::to_path_buf)
            .ok_or_else(|| "no document is open".to_string())?;

        let urls = preview.urls().to_vec();
        let allowed = preview.web().widened(&urls);
        document::write_sites(&self.sites, &root, &allowed)?;
        preview.web_mut().install(allowed);

        for url in preview.web_mut().claim(&urls, true) {
            let _ = self.claims.send(Claim::Fetch { url, settle: false });
        }
        let _ = self.claims.send(Claim::Compile);
        Ok(())
    }

    /// Drop what the pane holds and take the file again.
    ///
    /// **The second way out both refusals name.** `Preview::load` already reads
    /// the edited file into the buffer and the last-saved text together, which
    /// is exactly "discard"; this is that path behind a command. It clears the
    /// divergence through `Preview::take`, so one action answers a refused
    /// switch and a refused external change alike.
    pub fn discard(&self) {
        self.preview().discard();
        (self.on_render)();
    }

    /// Take the pane's text, and start the clock on the compile it will want.
    ///
    /// The keystroke crosses the IPC boundary and the debounce is Rust's,
    /// which is what puts this on the testable side of the window.
    pub fn edit(&self, text: String) {
        self.preview().edit(text);
        if let Some(typing) = &self.typing {
            let _ = typing.send(());
        }
    }

    /// Write the pane's text to the document's own path, and say so.
    ///
    /// The receipt is `letur_project::preview`'s `SAVED`, which carries no path: the file is the one the
    /// bar already names.
    pub fn save(&self) -> Result<String, String> {
        self.preview().save()
    }

    /// The filter, closed over the asset list the last successful parse left.
    ///
    /// That list follows the buffer, because the buffer is the document now: a
    /// figure named in text that has not been saved is watched for all the
    /// same.
    fn classifier(
        &self,
        root: PathBuf,
        main: PathBuf,
        edited: PathBuf,
    ) -> impl Fn(&Path) -> Option<Change> + Send + 'static {
        let state = Arc::clone(&self.state);
        move |path| {
            let assets = state
                .lock()
                .expect("the preview lock was poisoned")
                .assets()
                .to_vec();
            watch::classify(path, &root, &main, &edited, &assets)
        }
    }

    /// What one settled window of filesystem events does.
    ///
    /// **The edited file and everything else reach different code.** The file
    /// the pane holds runs [`Preview::reload`], because the pane's own buffer is
    /// what stands against it and losing that buffer is the one thing this app
    /// refuses to do quietly. Everything else the compile reads — an asset, and
    /// since `mpdf-010` Phase 2 the master itself — is a bare recompile, because
    /// nothing but the disk supplies it. `Change::Edited` is decided before
    /// both, so while the pane holds the main an event still runs the rule,
    /// exactly as it did before the two could differ.
    ///
    /// A window that took the disk copy compiled inside the rule, and it read
    /// the new assets on the way, so the two never compile twice for one
    /// window. And nothing is announced when nothing happened: the app's own
    /// save arrives here, changes nothing, and must not redraw a frame the
    /// reader has scrolled.
    fn on_change(&self, edited: PathBuf) -> impl FnMut(Changed) + Send + 'static {
        self.on_change_with(edited, Compile::run)
    }

    /// The same, with the render handed in.
    ///
    /// **The seam is for this loop's own gate**, and it is the shape
    /// `crate::document::render_with` already established one level down: the
    /// expensive call is the caller's, so a test can *check* a claim about the
    /// three steps instead of racing a real compile to argue it.
    ///
    /// **Two lock scopes, and this is what is in each.** The closure's body is
    /// longer than its compile, and "preserved exactly" stops meaning atomic the
    /// moment the lock is dropped, so the split is stated rather than left to be
    /// read off the braces:
    ///
    /// 1. the `edited` guard, [`Preview::reload`] on `changed.edited`,
    ///    [`Preview::plan`] on the bare-recompile branch, **and the
    ///    `changed.tree` walk** — moved up here so that the only thing outside
    ///    the lock is the render. Moving it costs nothing and closes a hole:
    ///    `document::files_under` and the compile write disjoint fields, so the
    ///    final state is what it was either way — but left *after* the render it
    ///    would be an unguarded write of one project's listing into whatever
    ///    `Preview` is live when the render returns;
    /// 2. the render, with no lock held;
    /// 3. the `edited` guard **re-checked**, then [`Preview::absorb`], which
    ///    applies its own two-part guard on top.
    ///
    /// **[`Preview::reload`] keeps the lock across its own compile, and that is
    /// this split's one exception.** It is a filesystem event, on the watch
    /// thread, behind no `Mutex<Session>` — so typing can still wait on a
    /// compile the window started by itself, for a document's full length, when
    /// another program rewrites the file in the pane over a clean buffer. It
    /// stays whole because taking it apart means `reload` no longer compiling
    /// and this closure's `!taken` condition inverting. `mpdf-003` Phase 22
    /// records the shape for the phase that meets it: `reload` returns its
    /// [`External`] and this closure plans on [`External::Taken`] as it plans on
    /// `changed.document`.
    ///
    /// **The announcement is exactly what it was**: the same three flags decide
    /// it, and it fires once, after the second scope, whether or not `absorb`
    /// wrote — a redundant one costs nothing, the page's own
    /// `revision === drawnRevision` comparison being what decides a re-fetch.
    /// The one case that announces nothing is a **failed re-check**: a different
    /// document is open, and its own open has already announced.
    fn on_change_with(
        &self,
        edited: PathBuf,
        render: impl Fn(&Compile) -> Rendered + Send + 'static,
    ) -> impl FnMut(Changed) + Send + 'static {
        let state = Arc::clone(&self.state);
        let on_render = Arc::clone(&self.on_render);

        move |changed: Changed| {
            let mut announce = false;
            let planned = {
                let mut preview = state.lock().expect("the preview lock was poisoned");
                if preview.document().as_deref() != Some(edited.as_path()) {
                    return;
                }

                let taken = if changed.edited {
                    let outcome = preview.reload();
                    announce = outcome != External::Unchanged;
                    outcome == External::Taken
                } else {
                    false
                };

                let planned = if (changed.document || changed.assets) && !taken {
                    announce = true;
                    plan(&mut preview)
                } else {
                    None
                };

                // **A file the document does not name moved: the panel is out
                // of date and the page is not.** This walks the disk and stops
                // — no compile, so `revision` stands still and the page draws
                // nothing again. It is announced all the same, because the
                // panel is drawn off the status the announcement fetches.
                if changed.tree {
                    preview.refresh_tree();
                    announce = true;
                }

                planned
            };

            if let Some(plan) = planned {
                let rendered = render(&plan);

                let mut preview = state.lock().expect("the preview lock was poisoned");
                if preview.document().as_deref() != Some(edited.as_path()) {
                    return;
                }
                absorb(&mut preview, &plan, rendered);
            }

            if announce {
                on_render();
            }
        }
    }

    /// What one settled pause in the typing does.
    ///
    /// It checks the document before **planning**. Dropping a [`Watch`] or a
    /// typing channel does not join its thread, so a thread that was mid-compile
    /// when a second document opened could otherwise plan against a document the
    /// window no longer holds. What stands in front of the *write* is
    /// [`Preview::current`] and the serial, one scope further down.
    fn recompile(&self, document: PathBuf) -> impl FnMut() + Send + 'static {
        self.recompile_with(document, Compile::run)
    }

    /// The same, with the render handed in.
    ///
    /// **The seam is for this loop's own gate**, for
    /// [`Session::on_change_with`]'s reason and in its shape.
    ///
    /// **Three steps, and the middle one holds no lock** — which is the whole of
    /// `mpdf-003` Phase 22 as the author feels it: this is the compile that fires
    /// 300 ms after the last keystroke, which is exactly when an author who
    /// paused to think resumes, so [`Session::edit`] and [`Session::status`]
    /// arriving inside it now wait on nothing rather than on a document-length
    /// proportional render.
    ///
    /// The announcement is what it was: it fires after the second scope whether
    /// or not [`Preview::absorb`] wrote, and whether or not there was anything to
    /// plan. A **failed re-check** is the one path that announces nothing, the
    /// document that replaced this one having announced already.
    fn recompile_with(
        &self,
        document: PathBuf,
        render: impl Fn(&Compile) -> Rendered + Send + 'static,
    ) -> impl FnMut() + Send + 'static {
        let state = Arc::clone(&self.state);
        let on_render = Arc::clone(&self.on_render);

        move || {
            let planned = {
                let mut preview = state.lock().expect("the preview lock was poisoned");
                if preview.document().as_deref() != Some(document.as_path()) {
                    return;
                }
                plan(&mut preview)
            };

            if let Some(plan) = planned {
                let rendered = render(&plan);

                let mut preview = state.lock().expect("the preview lock was poisoned");
                if preview.document().as_deref() != Some(document.as_path()) {
                    return;
                }
                absorb(&mut preview, &plan, rendered);
            }

            on_render();
        }
    }
}

/// What a render answers, and how long it took.
pub type Rendered = (Result<letur_project::document::Render, String>, Duration);

/// The signal after a compile. It carries nothing; the page asks for the rest.
type Announce = Arc<dyn Fn() + Send + Sync>;

/// The worker's fetch: `remote::fetch`, or a test's fake.
type Fetcher = Arc<dyn Fn(&str) -> Result<Vec<u8>, String> + Send + Sync>;

/// The worker's render: [`Compile::run`], or a test's gate.
type Renderer = Arc<dyn Fn(&Compile) -> Rendered + Send + Sync>;

/// One piece of work for the fetch worker.
enum Claim {
    /// Fetch this URL, waiting out [`remote::SETTLE`] first unless it was
    /// pressed for.
    Fetch { url: String, settle: bool },
    /// Compile, and announce. What the press sends whether or not it fetches.
    Compile,
}

/// What each claim's thread carries: the state, reached only when it is
/// needed, and the three things it calls.
#[derive(Clone)]
struct Worker {
    /// **Weak, and upgraded one step at a time**, so no thread holds a
    /// dropped session's state alive across a thirty-second request.
    state: Weak<Mutex<Preview>>,
    fetch: Fetcher,
    render: Renderer,
    on_render: Announce,
}

impl Worker {
    /// Carry one claim out: `mpdf-003` Phase 25's five steps, of which a
    /// [`Claim::Compile`] is the last alone.
    ///
    /// 1. Wait out the settle, unless the claim came from the press.
    /// 2. Under the lock, start the fetch if the text still names the URL and
    ///    the open project allows its site — or drop the claim — and announce.
    ///    A claim made under one project that settles after another opens is
    ///    thereby dropped, not fetched on the second project's behalf.
    /// 3. Fetch, off the lock.
    /// 4. Under the lock, record what came back, and announce.
    /// 5. The typing loop's three steps — plan under the lock, render off it,
    ///    absorb under it — and announce.
    ///
    /// **A keystroke never waits on the network**, which is `mpdf-003` Phase
    /// 22's property extended to a new source of work: the lock is taken for
    /// three short writes and a plan, and never across a request or a render.
    fn carry(&self, claim: Claim) {
        if let Claim::Fetch { url, settle } = claim {
            if settle {
                std::thread::sleep(remote::SETTLE);
            }

            let Some(state) = self.state.upgrade() else {
                return;
            };
            let begun = {
                let mut preview = state.lock().expect("the preview lock was poisoned");
                let named = preview.urls().contains(&url);
                preview.web_mut().begin(&url, named)
            };
            drop(state);
            (self.on_render)();
            if !begun {
                return;
            }

            let result = (self.fetch)(&url);

            let Some(state) = self.state.upgrade() else {
                return;
            };
            state
                .lock()
                .expect("the preview lock was poisoned")
                .web_mut()
                .land(&url, result);
            drop(state);
            (self.on_render)();
        }

        let Some(state) = self.state.upgrade() else {
            return;
        };
        let planned = plan(&mut state.lock().expect("the preview lock was poisoned"));
        if let Some(plan) = planned {
            let rendered = (self.render)(&plan);
            absorb(
                &mut state.lock().expect("the preview lock was poisoned"),
                &plan,
                rendered,
            );
        }
        drop(state);
        (self.on_render)();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use letur_project::preview::{SAVED, State};
    use letur_project::remote::WebLine;
    use std::collections::BTreeSet;
    use std::sync::Condvar;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    /// A file from the frozen copy of the engine's `samples/`.
    ///
    /// **Frozen at `mpdf-011` Phase 1, and a copy on purpose.** These tests
    /// assert what the *app* does with a document, not what the dialect does
    /// with it, so a fixture that stopped tracking the engine costs them
    /// nothing — and one that went on tracking it would move this app's
    /// measured numbers on every dialect phase.
    fn sample(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures/samples")
            .join(name)
    }

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures")
            .join(name)
    }

    /// A scratch directory this test owns.
    ///
    /// It sits under `std::env::temp_dir()` deliberately: on macOS that
    /// resolves through a symlink, so a filter that forgets to canonicalize
    /// fails these cases loudly rather than passing under some directory that
    /// happens not to be symlinked.
    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("md2pdf-preview-test-{}", std::process::id()))
            .join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_ne!(
            dir.canonicalize().unwrap(),
            dir,
            "the scratch directory is not symlinked, so these cases prove less than they claim"
        );
        dir
    }

    /// A session whose compiles can be counted, which is the seam every case
    /// below is read through.
    ///
    /// Its store is a scratch file this process owns, so a test never reads or
    /// writes the store the installed app keeps — and a case that wants an
    /// override writes one into it and says so.
    /// **A store directory per session, and the counter is load-bearing.**
    /// Every caller shared one `scratch_dir("store-of-the-session")` until
    /// `mpdf-003` Phase 17 added eight more of them, and [`scratch_dir`]
    /// `remove_dir_all`s before it creates: two tests reaching it at once left
    /// one of them between `create_dir_all` and `canonicalize` with its
    /// directory deleted under it, which is a `NotFound` on a line that reads
    /// like an assertion about symlinks. The race was always there; the eighth
    /// caller is what made it show. Distinct names cost nothing and remove it.
    fn counted() -> (Session, Arc<AtomicUsize>) {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::SeqCst);
        counted_with(document::store_file(&scratch_dir(&format!(
            "store-of-the-session-{n}"
        ))))
    }

    /// The same, with the store named, for the cases that put something in it.
    ///
    /// **The settings file is derived rather than passed**, and it is derived
    /// the way the real app derives it: beside the store, in the same
    /// directory. So a case that wants to read what an appearance wrote spells
    /// `document::settings_file` over the same scratch directory and gets the
    /// same path, and the twelve call sites of this helper do not move.
    ///
    /// **The worker's two seams are fakes here**, and for every case that does
    /// not ask for others: a fetch that refuses, so no case in the suite can
    /// touch the network, and the real render. `sites.json` is derived beside
    /// the store the way `settings.json` is.
    fn counted_with(store: PathBuf) -> (Session, Arc<AtomicUsize>) {
        seamed(
            store,
            |_: &str| Err("the suite touches no network".to_string()),
            Compile::run,
        )
    }

    /// The same, with the fetch worker's fetch and render named.
    fn seamed(
        store: PathBuf,
        fetch: impl Fn(&str) -> Result<Vec<u8>, String> + Send + Sync + 'static,
        render: impl Fn(&Compile) -> Rendered + Send + Sync + 'static,
    ) -> (Session, Arc<AtomicUsize>) {
        let support = store.parent().map(Path::to_path_buf).unwrap_or_default();
        let compiles = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&compiles);
        let session = Session::new(
            store,
            document::settings_file(&support),
            document::sites_file(&support),
            Appearance::System,
            fetch,
            render,
            move || {
                counter.fetch_add(1, Ordering::SeqCst);
            },
        );
        (session, compiles)
    }

    /// Wait for the loop to reach a compile count, or give up.
    ///
    /// The bound is generous because FSEvents' own coalescing sits under the
    /// debounce. It is a bound on wiring, not a measurement: the count itself
    /// is pinned by `watch::tests`, which needs no filesystem.
    fn wait_for(compiles: &AtomicUsize, target: usize) -> usize {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            let seen = compiles.load(Ordering::SeqCst);
            if seen >= target {
                return seen;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        compiles.load(Ordering::SeqCst)
    }

    /// Give a change that should *not* compile long enough to prove it.
    ///
    /// It clears both intervals several times over: a keystroke's compile
    /// falls due after [`watch::TYPING_DEBOUNCE`], which is the longer of the
    /// two, and a filesystem event's after [`watch::DEBOUNCE`].
    fn settle() {
        std::thread::sleep(watch::TYPING_DEBOUNCE * 4);
    }

    /// A copy of `samples/article.md` and both the figures it names.
    fn article_in(dir: &Path) -> PathBuf {
        let document = dir.join("article.md");
        std::fs::copy(sample("article.md"), &document).unwrap();
        std::fs::copy(sample("pipeline.svg"), dir.join("pipeline.svg")).unwrap();
        std::fs::copy(sample("check.svg"), dir.join("check.svg")).unwrap();
        document
    }

    /// A copy of `tests/fixtures/citations.md` and the bibliography it names.
    fn citing_document_in(dir: &Path) -> PathBuf {
        let document = dir.join("citations.md");
        std::fs::copy(fixture("citations.md"), &document).unwrap();
        std::fs::copy(fixture("refs.yml"), dir.join("refs.yml")).unwrap();
        document
    }

    /// A copy of `tests/fixtures/multi_file.md`, the three sections it names
    /// and the two figures those sections name.
    ///
    /// The figures go into `sections/` and not beside the master, which is the
    /// layout Phase 2 shipped: `introduction.md` writes a bare `dot.png` and
    /// the emitter resolves it against the folder that file lives in.
    fn multi_file_in(dir: &Path) -> PathBuf {
        let document = dir.join("multi_file.md");
        std::fs::copy(fixture("multi_file.md"), &document).unwrap();

        std::fs::create_dir_all(dir.join("sections")).unwrap();
        for name in [
            "introduction.md",
            "method.md",
            "results.md",
            "dot.png",
            "mark.svg",
        ] {
            std::fs::copy(
                fixture(&format!("sections/{name}")),
                dir.join("sections").join(name),
            )
            .unwrap();
        }
        document
    }

    /// A preview holding one document, read from disk and compiled, built
    /// without a session.
    ///
    /// The root is the document's own directory and the main is the document,
    /// which is what a single-file open lands on and is what every case using
    /// this is about.
    fn compiled(document: &Path) -> Preview {
        let root = watch::root(document);
        let name = document::title(document);
        let mut preview = Preview::default();
        preview.open(Disk::new(root), name.clone(), name, BTreeSet::new());
        preview.load();
        preview
    }

    /// The two figures `samples/article.md` names, read by this test rather
    /// than by the reader under test.
    ///
    /// It names each of them once, so there is no dedup subtlety to mirror.
    /// Reading them here is what makes the assertion independent: an
    /// `md_to_pdf` fed by `app`'s own reader would only prove that reader
    /// agrees with itself.
    fn article_assets(dir: &Path) -> Vec<md2pdf_core::Asset> {
        ["pipeline.svg", "check.svg"]
            .into_iter()
            .map(|name| md2pdf_core::Asset {
                path: name.to_string(),
                bytes: std::fs::read(dir.join(name)).unwrap(),
            })
            .collect()
    }

    /// A compile error keeps the bytes and sets the mark. Fixing the text
    /// clears both.
    ///
    /// The broken states are typed rather than written to the file, because
    /// typing is how an author reaches them: a half-typed table, a fence not
    /// yet closed.
    #[test]
    fn a_failed_compile_keeps_the_last_good_page_and_marks_it_stale() {
        let dir = scratch_dir("stale");
        let mut preview = compiled(&article_in(&dir));

        let good = preview.pdf().unwrap().to_vec();
        assert!(good.starts_with(b"%PDF"));
        assert!(!preview.is_stale());
        assert_eq!(preview.error(), None);

        preview.edit("# Broken\n\n<div>raw HTML</div>\n".to_string());
        preview.compile();

        assert_eq!(preview.pdf(), Some(good.as_slice()));
        assert!(preview.is_stale());
        assert!(preview.error().unwrap().contains("raw HTML block"));

        preview.edit("# Fixed\n\nOrdinary text.\n".to_string());
        preview.compile();

        assert!(!preview.is_stale());
        assert_eq!(preview.error(), None);
        assert_ne!(preview.pdf(), Some(good.as_slice()));
    }

    /// The document and a figure it names each redraw the page. This is the
    /// real watcher, on a real directory, through the code the window runs.
    ///
    /// The document's half is the loop Phase 2 shipped, and it survives the
    /// text pane exactly because the buffer here is clean: nobody has typed,
    /// so nothing can be lost, and the rule takes the disk copy.
    #[test]
    fn a_saved_document_and_a_replaced_figure_each_compile_again() {
        let dir = scratch_dir("watch-article");
        let document = article_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document.clone()).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1, "opening compiles once");

        let markdown = std::fs::read_to_string(&document).unwrap();
        std::fs::write(&document, markdown.replace("Introduction", "The start")).unwrap();
        assert_eq!(wait_for(&compiles, 2), 2, "saving the document compiles");

        std::fs::write(
            dir.join("pipeline.svg"),
            std::fs::read(sample("check.svg")).unwrap(),
        )
        .unwrap();
        assert_eq!(wait_for(&compiles, 3), 3, "replacing a figure compiles");

        assert!(session.preview().pdf().unwrap().starts_with(b"%PDF"));
        assert!(!session.preview().is_stale());
    }

    /// A figure the document names minutes before anyone creates it.
    ///
    /// This is the case a watch set of files could not have held — `notify`'s
    /// macOS backend refuses to register a path that does not exist — and the
    /// one the directory answer exists for.
    #[test]
    fn a_figure_that_does_not_exist_yet_is_watched_and_then_compiles() {
        let dir = scratch_dir("figure-to-come");
        let document = dir.join("paper.md");
        std::fs::write(&document, "# Paper\n\n![a mark to come](figures/new.svg)\n").unwrap();

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);
        assert!(
            session
                .preview()
                .error()
                .unwrap()
                .contains("figures/new.svg")
        );
        assert!(session.preview().pdf().is_none());

        std::fs::create_dir_all(dir.join("figures")).unwrap();
        std::fs::copy(fixture("mark.svg"), dir.join("figures/new.svg")).unwrap();

        assert!(wait_for(&compiles, 2) >= 2, "creating the figure compiles");
        assert!(session.preview().pdf().unwrap().starts_with(b"%PDF"));
        assert!(!session.preview().is_stale());
    }

    /// The bibliography moves and nothing else does, and the page that comes
    /// back is a real one.
    ///
    /// **The compile count is not the assertion here.** [`Session::on_change`]
    /// calls `on_render()` whenever the asset mark is set, a failed compile
    /// included, so a counter alone passes an app that publishes the path to
    /// the filter and never supplies the bytes: it would go 1 → 2 while every
    /// compile of the two errored `MissingBibliography`. The bytes and the
    /// stale mark are what tell the two apart.
    #[test]
    fn a_replaced_bibliography_compiles_again_and_the_page_is_good() {
        let dir = scratch_dir("watch-bibliography");
        let document = citing_document_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1, "opening compiles once");
        assert!(session.preview().pdf().unwrap().starts_with(b"%PDF"));

        // The same key under a different record, so the citation still
        // resolves: a compile that reached these bytes succeeds, and one that
        // reached nothing fails on the file rather than on the key.
        std::fs::write(
            dir.join("refs.yml"),
            concat!(
                "\"DBLP:books/lib/Knuth86a\":\n",
                "  type: book\n",
                "  title: The TeXbook\n",
                "  author: Knuth, Donald E.\n",
                "  date: 1984\n",
                "  publisher: Addison-Wesley\n",
            ),
        )
        .unwrap();
        assert_eq!(
            wait_for(&compiles, 2),
            2,
            "rewriting the bibliography compiles"
        );

        assert!(session.preview().pdf().unwrap().starts_with(b"%PDF"));
        assert!(!session.preview().is_stale());
    }

    /// A bibliography the document names minutes before anyone creates it.
    ///
    /// The sibling of
    /// [`a_figure_that_does_not_exist_yet_is_watched_and_then_compiles`], and
    /// the case that fails for an app publishing the path only out of a
    /// successful read: the first compile has no bytes at all, so the list the
    /// filter holds can only have come from the text.
    #[test]
    fn a_bibliography_that_does_not_exist_yet_is_watched_and_then_compiles() {
        let dir = scratch_dir("bibliography-to-come");
        let document = dir.join("citations.md");
        std::fs::copy(fixture("citations.md"), &document).unwrap();

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        let error = session.preview().error().unwrap().to_string();
        assert!(error.contains("refs.yml"), "{error}");
        assert!(error.contains("line 3"), "{error}");
        assert!(session.preview().pdf().is_none());

        std::fs::copy(fixture("refs.yml"), dir.join("refs.yml")).unwrap();

        assert!(
            wait_for(&compiles, 2) >= 2,
            "creating the bibliography compiles"
        );
        assert!(session.preview().pdf().unwrap().starts_with(b"%PDF"));
        assert!(!session.preview().is_stale());
    }

    // ---------------------------------------------------------- Save as
    //
    // `mpdf-003` Phase 17's exit gate, clauses 1 and 2. **Clause 1's four tests
    // compare the PDF byte-wise against the compile before the save**, which §2
    // of the spec licenses — five identical compiles across five processes —
    // and **each states its starting buffer**, because the fourth case makes the
    // starting state load-bearing rather than incidental.

    /// A Save-as onto a path the master names moves the PDF. Clean buffer.
    ///
    /// **A path the master names other than the one the pane holds**: saving
    /// onto the pane's own path from a clean buffer writes identical bytes and
    /// moves nothing, which would pass this assertion for the wrong reason.
    /// **Its project is built here rather than taken from
    /// [`multi_file_in`]**, and that is a finding rather than a preference:
    /// every section of that fixture is entangled with the others — one
    /// declares `#fig:pipeline`, one declares `#fig:mark` and cites a footnote,
    /// one defines that footnote and references both figures — so overwriting
    /// *any* of them fails the compile, the last good page survives by design,
    /// and `pdf()` answers the same bytes for a reason that has nothing to do
    /// with this clause. Two plain files instead, and the page moves because the
    /// section's text moved.
    #[test]
    fn a_save_as_onto_a_path_the_master_names_moves_the_page() {
        let dir = scratch_dir("save-as-onto-a-named-path");
        std::fs::write(
            dir.join("book.md"),
            "---\ntitle: A Book\n---\n\n[](part.md)\n",
        )
        .unwrap();
        std::fs::write(dir.join("part.md"), "# Part\n\nThe text the section had.\n").unwrap();
        std::fs::write(dir.join("swap.md"), "# Swap\n\nQuite another paragraph.\n").unwrap();

        let (mut session, compiles) = counted();
        session.open(dir.join("book.md")).unwrap();
        wait_for(&compiles, 1);

        // The pane holds a file the master does not name, and holds it clean.
        session.set_edited("swap.md".to_string()).unwrap();
        session.preview().compile();
        let before = session.preview().pdf().unwrap().to_vec();

        let onto = dir.join("part.md");
        session
            .save_as(onto.to_string_lossy().into_owned())
            .unwrap();

        let after = session.preview().pdf().unwrap().to_vec();
        assert!(
            session.preview().error().is_none(),
            "{:?}",
            session.preview().error()
        );
        assert_ne!(
            before, after,
            "overwriting a section the master names moves the page"
        );
    }

    /// A Save-as onto a path the master does not name leaves the PDF alone.
    ///
    /// **From a clean buffer, and that qualification is the clause.** From a
    /// dirty one this is false — see
    /// [`a_save_as_off_a_dirty_named_file_moves_the_page`], which is the case an
    /// author hits by default.
    #[test]
    fn a_save_as_onto_a_path_the_master_does_not_name_leaves_the_page() {
        let dir = scratch_dir("save-as-onto-an-unnamed-path");
        let document = multi_file_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);
        let before = session.preview().pdf().unwrap().to_vec();

        let onto = dir.join("notes.md");
        session
            .save_as(onto.to_string_lossy().into_owned())
            .unwrap();

        let after = session.preview().pdf().unwrap().to_vec();
        assert_eq!(
            before, after,
            "a file nothing names does not reach the compile"
        );
    }

    /// The fourth case: `edited` leaving a dirty file the master names.
    ///
    /// **The default configuration, and the one the first draft of Phase 17 got
    /// backwards.** `document::render_project` substitutes the buffer for
    /// `edited` alone, so the moment a Save-as moves `edited` off the master,
    /// the master goes back to its own on-disk text — and the page changes even
    /// though the file written is one nothing names. **It is a state this app
    /// has never been in**: [`Session::set_edited`]'s `refused_while_dirty`
    /// blocks exactly it, and [`Session::save_as`] bypasses that guard
    /// deliberately.
    #[test]
    fn a_save_as_off_a_dirty_named_file_moves_the_page() {
        let dir = scratch_dir("save-as-off-a-dirty-file");
        let document = multi_file_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);

        // Dirty, and visibly so in the typeset page rather than in a comment.
        let text = format!(
            "{}\n\nA paragraph the file on disk does not have.\n",
            session.preview().text()
        );
        session.preview().edit(text);
        session.preview().compile();
        let before = session.preview().pdf().unwrap().to_vec();

        let onto = dir.join("draft.md");
        session
            .save_as(onto.to_string_lossy().into_owned())
            .unwrap();

        let after = session.preview().pdf().unwrap().to_vec();
        assert_ne!(
            before, after,
            "the master stops being read from the buffer and reverts to its own disk text"
        );
    }

    /// A Save-as outside the project writes, and nothing about the pane moves.
    ///
    /// **This test has been inverted twice and deleted neither time.** It
    /// asserted a refusal until Phase 17, asserted that the pane held the
    /// outside file under Phase 18, and asserts under Phase 19 that the pane
    /// **keeps** what it was holding. What survives both reversals is the half a
    /// deletion would have dropped: `main` and `root` are unmoved, which is
    /// Phase 18's own decision about re-rooting and is not the PDF's. Whether
    /// the page moves is clause 2's, and deliberately not here.
    ///
    /// Its destination has a name of its own, as every outside destination in
    /// these tests does: they share one `<pid>` parent and cargo runs them in
    /// parallel.
    #[test]
    fn a_save_as_outside_the_project_writes_and_the_project_stays() {
        let dir = scratch_dir("save-as-outside-writes");
        let document = multi_file_in(&dir);
        let outside = dir.parent().unwrap().join("outside-writes.md");
        let _ = std::fs::remove_file(&outside);

        let (mut session, compiles) = counted();
        session.open(document.clone()).unwrap();
        wait_for(&compiles, 1);
        let root = session.preview().root().unwrap().to_path_buf();

        session
            .save_as(outside.to_string_lossy().into_owned())
            .unwrap();

        assert!(
            outside.exists(),
            "the file is written where it was asked for"
        );
        assert_eq!(
            session.preview().document().as_deref(),
            Some(document.as_path()),
            "the pane keeps the file it was holding: a save outside the project is a copy"
        );
        assert_eq!(
            session.preview().root(),
            Some(root.as_path()),
            "the root does not move"
        );
        assert_eq!(
            session.preview().status().main.as_deref(),
            Some("multi_file.md"),
            "the window keeps compiling the project it had"
        );
    }

    /// Clause 2, the clean half: an outside save leaves the page alone.
    #[test]
    fn a_save_as_outside_from_a_clean_buffer_leaves_the_page() {
        let dir = scratch_dir("save-as-outside-clean");
        let document = multi_file_in(&dir);
        let outside = dir.parent().unwrap().join("outside-clean.md");
        let _ = std::fs::remove_file(&outside);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);
        let before = session.preview().pdf().unwrap().to_vec();

        session
            .save_as(outside.to_string_lossy().into_owned())
            .unwrap();

        assert_eq!(before, session.preview().pdf().unwrap().to_vec());
    }

    /// Clause 2, the dirty half: **Phase 17's fourth case, no longer reachable
    /// outside the project.**
    ///
    /// **Renamed and inverted rather than deleted, and the name is the point.**
    /// Under Phase 18 this was `a_save_as_outside_from_a_dirty_buffer_moves_the_page`
    /// and asserted `assert_ne!`: the pane held a file the master named, the
    /// save moved `edited` out, `render_project` stopped substituting the buffer
    /// for that file, and the section reverted to its own on-disk text. That is
    /// the observable Phase 19 exists to stop moving. `edited` never leaves the
    /// root now, so the buffer still stands in for the file the master names and
    /// the page is byte-identical.
    #[test]
    fn a_save_as_outside_from_a_dirty_buffer_leaves_the_page() {
        let dir = scratch_dir("save-as-outside-dirty");
        let document = multi_file_in(&dir);
        let outside = dir.parent().unwrap().join("outside-dirty.md");
        let _ = std::fs::remove_file(&outside);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);

        let text = format!(
            "{}\n\nA paragraph the file on disk does not have.\n",
            session.preview().text()
        );
        session.preview().edit(text);
        session.preview().compile();
        let before = session.preview().pdf().unwrap().to_vec();

        session
            .save_as(outside.to_string_lossy().into_owned())
            .unwrap();

        assert_eq!(
            before,
            session.preview().pdf().unwrap().to_vec(),
            "the pane never left the file the master names, so the buffer still compiles for it"
        );
    }

    /// The outside file gets no row, and the bar names the file the pane
    /// **kept**.
    ///
    /// **Renamed with the assertion.** Under Phase 18 this was
    /// `a_file_saved_outside_has_no_row_but_the_bar_still_names_it` and pinned
    /// the absolute-path fallback `Preview::edited_relative` carried: the bar
    /// named the file that had just been written, by a bare name that said
    /// nothing about where it was. That ambiguity is what this phase was found
    /// by. The cell now names a file that still has a row, spelled
    /// root-relative, because that is the file the pane is still holding.
    #[test]
    fn a_file_saved_outside_gets_no_row_and_the_bar_names_the_file_the_pane_kept() {
        let dir = scratch_dir("save-as-outside-named");
        let document = multi_file_in(&dir);
        let outside = dir.parent().unwrap().join("outside-named.md");
        let _ = std::fs::remove_file(&outside);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);

        session
            .save_as(outside.to_string_lossy().into_owned())
            .unwrap();

        let status = session.preview().status();
        assert!(
            !status
                .entries
                .iter()
                .any(|entry| entry.path.ends_with("outside-named.md")),
            "the panel walks the root, so a file outside it has no row"
        );
        assert_eq!(
            status.edited.as_deref(),
            Some("multi_file.md"),
            "the bar names what the pane holds, which the save did not move"
        );
    }

    // -- the pane is kept, so the dirty state must be kept with it -----------
    //
    // `mpdf-003` Phase 19's gate clause 2. **Three cases for one paragraph of
    // `Preview::save_as`, because a green gate is how the other build would have
    // shipped**: moving `saved` to the buffer on the outside path leaves the
    // pane holding a file whose disk copy is older while every rule in this file
    // reads the buffer as clean.

    /// After a copy goes out, the buffer is still dirty against the file the
    /// pane kept — so the next switch is refused rather than silently obeyed.
    ///
    /// **The gesture a `saved == buffer` build would answer *clean*.**
    /// [`Session::refused_while_dirty`] would see no divergence, `set_edited`
    /// would call [`Preview::load`], and the author's unsaved text would go
    /// under the file's own — with no sentence anywhere saying so.
    #[test]
    fn a_save_as_outside_leaves_the_buffer_dirty_against_the_file_the_pane_kept() {
        let dir = scratch_dir("save-as-outside-still-dirty");
        let document = multi_file_in(&dir);
        let outside = dir.parent().unwrap().join("outside-still-dirty.md");
        let _ = std::fs::remove_file(&outside);

        let (mut session, compiles) = counted();
        session.open(document.clone()).unwrap();
        wait_for(&compiles, 1);

        let text = format!(
            "{}\n\nA paragraph the file on disk does not have.\n",
            session.preview().text()
        );
        session.preview().edit(text.clone());

        session
            .save_as(outside.to_string_lossy().into_owned())
            .unwrap();

        session
            .set_edited("sections/method.md".to_string())
            .unwrap();

        assert_eq!(
            session.preview().document().as_deref(),
            Some(document.as_path()),
            "the switch met the dirty buffer and was refused"
        );
        assert!(
            session.preview().status().divergence.is_some(),
            "and it said so, rather than moving in silence"
        );
        assert_eq!(
            session.preview().text(),
            text,
            "the author's work is still in the pane"
        );
    }

    /// And an event on that file answers [`External::Diverged`], not
    /// [`External::Taken`].
    ///
    /// **The other half of the same unsafe state, and the one that loses work
    /// without any gesture at all.** With `saved == buffer`, [`external_change`]
    /// falls to its middle arm and takes the disk copy over the author's text on
    /// the next event touching that file — the outcome §2's *"Why an external
    /// change waits for a clean buffer"* names as the one that loses.
    #[test]
    fn a_save_as_outside_leaves_an_external_change_diverged_and_not_taken() {
        let dir = scratch_dir("save-as-outside-diverged");
        let document = multi_file_in(&dir);
        let outside = dir.parent().unwrap().join("outside-diverged.md");
        let _ = std::fs::remove_file(&outside);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);

        let text = format!(
            "{}\n\nA paragraph the file on disk does not have.\n",
            session.preview().text()
        );
        session.preview().edit(text);

        session
            .save_as(outside.to_string_lossy().into_owned())
            .unwrap();

        assert_eq!(session.preview().reload(), External::Diverged);
    }

    /// And the save itself compiled nothing.
    ///
    /// **`revision` is the reading, because it is what the page guards its
    /// redraw on.** [`Preview::compile`] bumps it and `app/dist/index.html`'s
    /// `refresh` treats a new one as a reason to re-fetch the bytes and re-place
    /// the reader — so a compile here would throw the author back up the page to
    /// show them exactly what was already on it.
    ///
    /// **It cannot be read at the window, and that is worth recording here
    /// rather than in a gate that tries.** `saveDocumentAs` sends the pane's
    /// text over with `invoke('edit')` *before* it opens the dialog, and
    /// [`Session::edit`] kicks the [`crate::watch::TYPING_DEBOUNCE`] — so a
    /// compile falls due while the native panel is still open, on both paths and
    /// for a reason that has nothing to do with the save. Nothing intervenes
    /// here.
    #[test]
    fn a_save_as_outside_the_project_compiles_nothing() {
        let dir = scratch_dir("save-as-outside-quiet");
        let document = multi_file_in(&dir);
        let outside = dir.parent().unwrap().join("outside-quiet.md");
        let _ = std::fs::remove_file(&outside);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);
        let before = session.preview().status().revision;

        session
            .save_as(outside.to_string_lossy().into_owned())
            .unwrap();

        assert_eq!(
            session.preview().status().revision,
            before,
            "it reads what it read before, so a compile would only move the reader"
        );

        // The announce is kept all the same, which is the half a reader would
        // otherwise assume went with the compile.
        assert!(
            compiles.load(Ordering::SeqCst) > 1,
            "the tree rebuild and the announce stay on both paths"
        );
    }

    /// And both loops are still running, because nothing was re-armed.
    ///
    /// **The failure this rules out is the whole window going still.**
    /// [`Session::arm`]'s two closures open by comparing `Preview::edited`
    /// against a path captured when they were built, so arming on the outside
    /// destination while `edited` stayed inside would make every filesystem
    /// event and every typing recompile return early.
    ///
    /// **Counted relatively and not absolutely**, because the announce on the
    /// outside path is kept: the save itself moves the count once.
    #[test]
    fn the_loops_still_run_after_a_save_as_outside_the_project() {
        let dir = scratch_dir("save-as-outside-loops");
        let document = multi_file_in(&dir);
        let outside = dir.parent().unwrap().join("outside-loops.md");
        let _ = std::fs::remove_file(&outside);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);

        session
            .save_as(outside.to_string_lossy().into_owned())
            .unwrap();
        let after_the_save = compiles.load(Ordering::SeqCst);

        session.edit("Typed after the copy went out.\n".to_string());

        assert!(
            wait_for(&compiles, after_the_save + 1) > after_the_save,
            "typing still reaches a compile"
        );
    }

    // -- both halves of the predicate ----------------------------------------
    //
    // `mpdf-003` Phase 19's gate clause 4. **A gate that tested one half would
    // pass the regression this phase was rewritten to close**, so each half has
    // a case that the other half's predicate alone gets wrong.

    /// A destination the root cannot **spell** is outside, however `confined`
    /// resolves it.
    ///
    /// **The half a `confined`-only predicate gets wrong**, and the case that
    /// killed the first replacement. On macOS the scratch parent is reached
    /// through a symlink, so the same directory has two spellings:
    /// `document::confined` canonicalizes both sides and answers *inside*, while
    /// `document::spell`'s bare `strip_prefix` cannot produce a root-relative
    /// name at all. Moving the pane there would leave
    /// [`Preview::edited_relative`] with nothing to answer — the state whose
    /// fallback this phase removed.
    #[test]
    fn a_destination_the_root_cannot_spell_is_treated_as_outside() {
        let dir = scratch_dir("save-as-unspellable");
        let document = multi_file_in(&dir);
        let dest = dir.canonicalize().unwrap().join("copy.md");
        let _ = std::fs::remove_file(&dest);

        let (mut session, compiles) = counted();
        session.open(document.clone()).unwrap();
        wait_for(&compiles, 1);
        let root = session.preview().root().unwrap().to_path_buf();

        session
            .save_as(dest.to_string_lossy().into_owned())
            .unwrap();

        // Asserted after the write, because `confined` opens on `is_file`.
        assert!(
            document::confined(&root, &dest.to_string_lossy()).is_some(),
            "a canonicalized comparison alone would judge this inside"
        );
        assert_eq!(
            document::spell(&root, &dest),
            None,
            "and it cannot be spelled"
        );

        assert!(dest.is_file(), "the file is written where it was asked for");
        assert_eq!(
            session.preview().document().as_deref(),
            Some(document.as_path()),
            "so the pane stays"
        );
    }

    /// A destination that climbs out of the root is outside, however it is
    /// spelled.
    ///
    /// **The half a `spell`-only predicate gets wrong**, and the regression
    /// round 17 of this phase's review found. `document::spell` is a
    /// component-wise `strip_prefix`, so this path strips to
    /// `../escape-from-save-as.md`, answers `Some`, and reads as *inside* — after
    /// which the pane follows the file out of the project and, where a master
    /// names that spelling, the very case this phase removes fires anyway.
    /// `document::confined` canonicalizes and refuses it.
    #[test]
    fn a_destination_that_climbs_out_of_the_root_is_treated_as_outside() {
        let dir = scratch_dir("save-as-escape");
        let document = multi_file_in(&dir);
        let escaped = dir.parent().unwrap().join("escape-from-save-as.md");
        let _ = std::fs::remove_file(&escaped);

        let (mut session, compiles) = counted();
        session.open(document.clone()).unwrap();
        wait_for(&compiles, 1);
        let root = session.preview().root().unwrap().to_path_buf();

        let climbing = format!("{}/../escape-from-save-as.md", root.display());
        session.save_as(climbing.clone()).unwrap();

        assert!(
            document::spell(&root, Path::new(&climbing)).is_some(),
            "a spelling alone would judge this inside, `..` and all"
        );
        assert_eq!(document::confined(&root, &climbing), None, "and it is not");

        assert!(
            escaped.is_file(),
            "the write resolves the `..`, so the file lands above the root"
        );
        assert_eq!(
            session.preview().document().as_deref(),
            Some(document.as_path()),
            "so the pane stays"
        );
    }

    /// A link out of the project is written **through** and does not take the
    /// pane with it.
    ///
    /// **The inside set is narrower than Phase 17's**, and this is where.
    /// `std::fs::write` follows a symlink, so the bytes land outside the root
    /// either way — Phase 17 accepted that destination, `document::landing`
    /// canonicalizing only the parent, where `document::confined` resolves the
    /// name itself and refuses it. `document::tests::a_link_out_of_the_project_is_refused_however_it_is_spelled`
    /// pins the same refusal for every other command.
    #[test]
    fn a_link_out_of_the_project_does_not_take_the_pane_with_it() {
        let elsewhere = scratch_dir("save-as-link-target");
        let target = elsewhere.join("real.md");
        std::fs::write(&target, "# Not the project's\n").unwrap();

        let dir = scratch_dir("save-as-link-root");
        let document = multi_file_in(&dir);
        let link = dir.join("link.md");
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let (mut session, compiles) = counted();
        session.open(document.clone()).unwrap();
        wait_for(&compiles, 1);

        session
            .save_as(link.to_string_lossy().into_owned())
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            session.preview().text(),
            "the write followed the link, so the bytes are outside the project"
        );
        assert_eq!(
            session.preview().document().as_deref(),
            Some(document.as_path()),
            "and the pane did not follow them"
        );
    }

    // -- the receipt ---------------------------------------------------------
    //
    // `mpdf-003` Phase 19's gate clause 3. **Here and not in `app/src/main.rs`,
    // which is exactly why the sentences are composed in `Session`**: that file
    // has no test module — the crate is bin-only and `tauri::State` has a private
    // field and no public constructor — so a sentence written in the command is
    // one nothing in this repository can reach.

    /// `⌘S` answers `saved`, and carries no path.
    #[test]
    fn a_plain_save_answers_saved() {
        let dir = scratch_dir("save-receipt-plain");
        let document = multi_file_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);

        assert_eq!(session.save().unwrap(), SAVED);
    }

    /// `Save as…` answers with the name it wrote and the folder it wrote it in.
    ///
    /// **The folder is compared against the destination's own `parent()`,
    /// un-resolved**, and that is the one thing that decides whether this case
    /// can be written at all: a scratch directory sits under `/var/folders/…`
    /// while its canonical form is `/private/var/folders/…`, so only the
    /// spelling as landed matches.
    ///
    /// **Both destinations, because the two are the sentence's whole point.**
    /// Inside, the pane moves and the receipt is a confirmation; outside, the
    /// pane stays and the receipt is the only thing that says where the bytes
    /// went.
    #[test]
    fn a_save_as_answers_with_the_name_and_the_folder_it_landed_in() {
        let dir = scratch_dir("save-receipt-as");
        let document = multi_file_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);

        // Directly in the root, which is the commonest destination — the panel
        // defaults there — and the case that ended the root-relative/absolute
        // split: `document::spell` of that *folder* is the empty string, which
        // it answers `None` for, so a root-relative folder would be no folder.
        assert_eq!(document::spell(&dir, &dir), None);

        let beside = dir.join("draft.md");
        assert_eq!(
            session
                .save_as(beside.to_string_lossy().into_owned())
                .unwrap(),
            format!("saved as draft.md in {}", dir.display())
        );
        assert_eq!(
            session.preview().document().as_deref(),
            Some(beside.as_path()),
            "an inside save still moves the pane"
        );

        let outside = dir.parent().unwrap().join("receipt-outside.md");
        let _ = std::fs::remove_file(&outside);
        assert_eq!(
            session
                .save_as(outside.to_string_lossy().into_owned())
                .unwrap(),
            format!(
                "saved as receipt-outside.md in {}",
                dir.parent().unwrap().display()
            )
        );
        assert_eq!(
            session.preview().document().as_deref(),
            Some(beside.as_path()),
            "and the outside one leaves it where the inside one put it"
        );
    }

    /// A Save-as with a dirty buffer moves the pane rather than diverging.
    ///
    /// **The failure `Preview::save` followed by [`Session::set_edited`] would
    /// have**: that path meets `refused_while_dirty`, sets a divergence and
    /// answers `Ok(())` having moved nothing — a silent success, on the one
    /// gesture an author makes *because* they have unsaved work.
    #[test]
    fn a_save_as_with_a_dirty_buffer_moves_the_pane() {
        let dir = scratch_dir("save-as-dirty");
        let document = multi_file_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);
        session
            .preview()
            .edit("Rewritten in the pane.\n".to_string());

        let onto = dir.join("moved.md");
        session
            .save_as(onto.to_string_lossy().into_owned())
            .unwrap();

        assert_eq!(session.preview().document().as_deref(), Some(onto.as_path()));
        assert!(
            session.preview().status().divergence.is_none(),
            "no divergence"
        );
        assert_eq!(
            std::fs::read_to_string(&onto).unwrap(),
            "Rewritten in the pane.\n"
        );
    }

    /// A Save-as onto a file that already exists overwrites it.
    ///
    /// **Deliberately not `create_file`'s rule.** `File::create_new` makes
    /// *already exists* a refusal, which is right for a `+` gesture that invents
    /// a name and wrong for a Save-as, whose purpose is sometimes to replace.
    #[test]
    fn a_save_as_onto_an_existing_file_overwrites_it() {
        let dir = scratch_dir("save-as-overwrite");
        let document = multi_file_in(&dir);
        let onto = dir.join("already.md");
        std::fs::write(&onto, "What was there before.\n").unwrap();

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);
        session.preview().edit("What the pane says.\n".to_string());

        session
            .save_as(onto.to_string_lossy().into_owned())
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(&onto).unwrap(),
            "What the pane says.\n"
        );
    }

    /// A Save-as to a kind the panel cannot list is refused.
    ///
    /// `document::files_under` filters every row through `kind_of`, so a
    /// `notes.txt` would be a file the panel can never show while the pane holds
    /// it — `Status::edited` naming a path with no row, and no gesture back.
    #[test]
    fn a_save_as_to_a_kind_the_panel_cannot_list_is_refused() {
        let dir = scratch_dir("save-as-wrong-kind");
        let document = multi_file_in(&dir);
        let onto = dir.join("notes.txt");

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);

        let refusal = session
            .save_as(onto.to_string_lossy().into_owned())
            .unwrap_err();

        assert!(
            refusal.contains("neither markdown nor a bibliography"),
            "{refusal}"
        );
        assert!(!onto.exists());
    }

    /// The three duties the watch will not do for a Save-as.
    ///
    /// One test and not three, because they are one call's postconditions and a
    /// build that omits any of them fails here for its own reason: the page is
    /// stale, the file has no row, or nothing announced.
    #[test]
    fn a_save_as_compiles_lists_the_new_file_and_announces() {
        let dir = scratch_dir("save-as-duties");
        let document = multi_file_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        wait_for(&compiles, 1);

        let revision = session.preview().status().revision;
        let announced = compiles.load(Ordering::SeqCst);

        let onto = dir.join("listed.md");
        session
            .save_as(onto.to_string_lossy().into_owned())
            .unwrap();

        assert!(
            session.preview().status().revision > revision,
            "it compiles — the watch will not, `classify` answering Change::Edited on first match"
        );
        assert!(
            compiles.load(Ordering::SeqCst) > announced,
            "it announces, which is what carries the new Status to the bar"
        );
        assert!(
            session
                .preview()
                .status()
                .entries
                .iter()
                .any(|entry| entry.path == "listed.md"),
            "it rebuilds the tree by hand, as `Session::trash` does"
        );
    }

    /// A section changes and the page comes back, and it is a real one.
    ///
    /// The multi-file sibling of
    /// [`a_replaced_bibliography_compiles_again_and_the_page_is_good`], and its
    /// warning applies here too: [`Session::on_change`] announces whenever the
    /// asset mark is set, a failed compile included, so a counter alone would
    /// pass an app that published the section's path to the filter and never
    /// supplied its bytes. The bytes and the stale mark tell the two apart.
    ///
    /// **It is a bounded wait and not a latency.** Two intervals sit under it —
    /// [`watch::DEBOUNCE`] at 100 ms for the filesystem and
    /// [`watch::TYPING_DEBOUNCE`] at 300 ms for the pane — and neither is
    /// asserted end to end anywhere, as [`wait_for`]'s own comment says.
    #[test]
    fn a_section_that_changes_compiles_again_and_the_page_is_good() {
        let dir = scratch_dir("watch-section");
        let document = multi_file_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1, "opening compiles once");
        assert!(session.preview().pdf().unwrap().starts_with(b"%PDF"));

        // Appended rather than replaced: this section declares `#fig:mark` and
        // cites the footnote the third file defines, so a rewrite that dropped
        // either would fail on the name and never reach the bytes.
        let section = dir.join("sections/method.md");
        let mut text = std::fs::read_to_string(&section).unwrap();
        text.push_str("\nA paragraph this test added to the second file.\n");
        std::fs::write(&section, text).unwrap();

        assert_eq!(
            wait_for(&compiles, 2),
            2,
            "editing a section compiles again"
        );
        assert!(session.preview().pdf().unwrap().starts_with(b"%PDF"));
        assert!(!session.preview().is_stale());
    }

    /// An error inside a section names that section's own file and its own
    /// line, on the exact string.
    ///
    /// That string is `md2pdf_core::Error`'s `Display`, so this is the
    /// `in FILE at line N` phrase Phase 1 shipped, arriving at the window
    /// unaltered. The author wrote the block on line 3 of a file of their own;
    /// the joined document it was refused in exists nowhere.
    #[test]
    fn an_error_in_a_section_names_that_file_and_its_own_line() {
        let dir = scratch_dir("error-in-a-section");
        let document = dir.join("report.md");
        std::fs::write(&document, "[](sections/one.md)\n").unwrap();
        std::fs::create_dir_all(dir.join("sections")).unwrap();
        std::fs::write(
            dir.join("sections/one.md"),
            "# A section\n\n<div>a raw HTML block</div>\n",
        )
        .unwrap();

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        assert_eq!(
            session.preview().error(),
            Some("unsupported markdown construct 'raw HTML block' in sections/one.md at line 3")
        );
    }

    /// A section the master names minutes before anyone creates it.
    ///
    /// The sibling of
    /// [`a_bibliography_that_does_not_exist_yet_is_watched_and_then_compiles`],
    /// and the case the unconditional section list in `document::Render::assets`
    /// exists for. Both shopping lists fail with `MissingSection` here, and
    /// [`Preview::compile`] replaces the list only when it is `Some` — so an app
    /// that published the path out of a successful read would leave the list
    /// empty, `watch::classify` would drop the creation event, and the window
    /// would never recover on its own.
    #[test]
    fn a_section_that_does_not_exist_yet_is_watched_and_then_compiles() {
        let dir = scratch_dir("section-to-come");
        let document = dir.join("report.md");
        std::fs::write(&document, "[](sections/one.md)\n").unwrap();

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        let error = session.preview().error().unwrap().to_string();
        assert!(error.contains("sections/one.md"), "{error}");
        assert!(error.contains("for the section"), "{error}");
        assert!(error.contains("at line 1"), "{error}");
        assert!(session.preview().pdf().is_none());

        std::fs::create_dir_all(dir.join("sections")).unwrap();
        std::fs::write(dir.join("sections/one.md"), "# One\n\nText.\n").unwrap();

        assert!(wait_for(&compiles, 2) >= 2, "creating the section compiles");
        assert!(session.preview().pdf().unwrap().starts_with(b"%PDF"));
        assert!(!session.preview().is_stale());
    }

    /// Opening a second document moves the watch. An implementer who set the
    /// watcher up once rather than per document passes every case above and
    /// fails this one.
    #[test]
    fn opening_a_second_document_moves_the_watch() {
        let first_dir = scratch_dir("first");
        let second_dir = scratch_dir("second");
        let first = article_in(&first_dir);
        let second = article_in(&second_dir);

        let (mut session, compiles) = counted();
        session.open(first.clone()).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        session.open(second.clone()).unwrap();
        assert_eq!(wait_for(&compiles, 2), 2);

        std::fs::write(&first, "# The first, edited\n").unwrap();
        settle();
        assert_eq!(
            compiles.load(Ordering::SeqCst),
            2,
            "the first document is no longer watched"
        );

        std::fs::write(&second, "# The second, edited\n").unwrap();
        assert_eq!(wait_for(&compiles, 3), 3, "the second document is watched");
    }

    /// **The pane's text is what compiles, and the file beside it need never
    /// have held that text.**
    ///
    /// This is the whole of what Phase 4 changed one layer down, and the file
    /// is asserted to be untouched so that a compile which quietly went back
    /// to reading the disk could not pass.
    #[test]
    fn the_pane_compiles_text_that_is_not_on_disk() {
        let dir = scratch_dir("buffer-compiles");
        let document = article_in(&dir);

        let mut preview = compiled(&document);
        let from_disk = preview.pdf().unwrap().to_vec();

        let typed = "# Typed, never saved\n\nThis text is in the pane and nowhere else.\n";
        preview.edit(typed.to_string());
        preview.compile();

        assert_eq!(
            std::fs::read_to_string(&document).unwrap(),
            std::fs::read_to_string(sample("article.md")).unwrap(),
            "the file moved, so this proves nothing about the buffer"
        );
        assert_ne!(preview.pdf().unwrap(), from_disk.as_slice());
        assert_eq!(
            preview.pdf().unwrap(),
            md2pdf_core::md_to_pdf(typed, &[]).unwrap()
        );
    }

    /// A save writes the buffer, and **the save's own event compiles nothing**
    /// — the rule's first outcome, reached by comparing content rather than by
    /// winning a race against a 12 ms event.
    ///
    /// The figure at the end is the half that proves the filter narrowed
    /// rather than stopped: an implementer who dropped the watch while the
    /// pane owns the document passes everything above it and fails here.
    #[test]
    fn a_save_writes_the_buffer_and_the_loop_compiles_no_second_time() {
        let dir = scratch_dir("save");
        let document = article_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document.clone()).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1, "opening compiles once");

        let typed = format!(
            "{}\nA paragraph the file has never held.\n",
            std::fs::read_to_string(&document).unwrap()
        );
        session.edit(typed.clone());
        assert_eq!(wait_for(&compiles, 2), 2, "a pause in the typing compiles");

        session.save().unwrap();
        assert_eq!(std::fs::read_to_string(&document).unwrap(), typed);

        settle();
        assert_eq!(
            compiles.load(Ordering::SeqCst),
            2,
            "the save's own event compiled a second time"
        );

        std::fs::write(
            dir.join("pipeline.svg"),
            std::fs::read(sample("check.svg")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            wait_for(&compiles, 3),
            3,
            "a figure no longer redraws the page"
        );
    }

    /// The first outcome: the file already says what the pane says.
    ///
    /// This is the app's own save arriving back, and **nothing at all
    /// happens** — which the whole status is compared to prove, because a
    /// compile here would redraw a page the reader has scrolled.
    #[test]
    fn an_external_change_matching_the_buffer_does_nothing() {
        let dir = scratch_dir("external-unchanged");
        let document = article_in(&dir);

        let mut preview = compiled(&document);
        let before = preview.status();

        std::fs::write(&document, preview.text()).unwrap();
        assert_eq!(preview.reload(), External::Unchanged);

        assert_eq!(preview.status(), before);
    }

    /// The second outcome: the buffer is clean, so nothing can be lost.
    ///
    /// **This is Phase 2's shipped loop**, and it is the case an unconditional
    /// refusal would have broken — an author who is not typing saves in
    /// another editor and the page redraws, with no action at the window.
    #[test]
    fn an_external_change_over_a_clean_buffer_is_taken_and_recompiled() {
        let dir = scratch_dir("external-taken");
        let document = article_in(&dir);

        let mut preview = compiled(&document);
        let first = preview.pdf().unwrap().to_vec();
        let before = preview.status();

        let theirs = "# Edited elsewhere\n\nBy another program, while nobody typed.\n";
        std::fs::write(&document, theirs).unwrap();
        assert_eq!(preview.reload(), External::Taken);

        assert_eq!(preview.text(), theirs);
        assert_ne!(preview.pdf().unwrap(), first.as_slice());

        let status = preview.status();
        assert_eq!(status.state, State::Current);
        assert_eq!(status.divergence, None);
        assert!(status.revision > before.revision, "it did not recompile");
        assert!(
            status.reloaded > before.reloaded,
            "it did not take the text"
        );
    }

    /// The third outcome: the buffer holds unsaved edits, so the disk copy is
    /// refused and the divergence is named.
    ///
    /// An implementer who tests only this one ships a pane that stops redrawing
    /// on an external save. It is here as one of three for that reason.
    #[test]
    fn an_external_change_over_a_dirty_buffer_is_refused_and_reported() {
        let dir = scratch_dir("external-diverged");
        let document = article_in(&dir);

        let mut preview = compiled(&document);
        preview.edit("# Mine, unsaved\n\nStill being written.\n".to_string());
        preview.compile();

        let mine = preview.pdf().unwrap().to_vec();
        let before = preview.status();

        std::fs::write(&document, "# Theirs\n\nWritten by another program.\n").unwrap();
        assert_eq!(preview.reload(), External::Diverged);

        assert_eq!(preview.text(), "# Mine, unsaved\n\nStill being written.\n");
        assert_eq!(preview.pdf().unwrap(), mine.as_slice());

        let status = preview.status();
        assert_eq!(status.revision, before.revision, "it compiled anyway");
        assert_eq!(status.reloaded, before.reloaded, "it took the disk copy");
        // A divergence is not staleness: nothing failed to compile, and the
        // page belongs to the text in the pane.
        assert_eq!(status.state, State::Current);
        assert!(!preview.is_stale());
        assert!(
            status.divergence.as_deref().unwrap().contains("unsaved"),
            "{:?}",
            status.divergence
        );
    }

    /// A document opened, edited, saved and reopened round-trips byte for byte
    /// **against the buffer at save**, which an edit has already made unequal
    /// to the original.
    ///
    /// The text carries a CRLF and no trailing newline, which are the two a
    /// text pane is likeliest to normalise away.
    #[test]
    fn an_edited_document_round_trips_byte_for_byte_through_a_save() {
        let dir = scratch_dir("round-trip");
        let document = article_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document.clone()).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        let typed = "# Edited\r\n\r\nA line under a CRLF, and no newline at the end.";
        session.edit(typed.to_string());
        session.save().unwrap();

        assert_eq!(std::fs::read(&document).unwrap(), typed.as_bytes());

        session.open(document).unwrap();
        assert_eq!(session.preview().text(), typed);
    }

    /// The export writes the page itself, and the page is what the core crate
    /// makes of this document.
    ///
    /// This is one half of the byte-identity claim; the other half lives in
    /// `cli/tests/cli_test.rs`, because `CARGO_BIN_EXE_md2pdf` reaches only
    /// integration tests of the package that defines that binary, and nothing
    /// in `app/src/` is importable from there. The middle leg — the in-test
    /// `md_to_pdf` call — is what composes them.
    #[test]
    fn the_export_writes_the_bytes_the_pane_is_showing() {
        let dir = scratch_dir("export");
        let document = article_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document.clone()).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        // The default path is `cli/src/main.rs:default_output`'s rule.
        let output = document.with_extension("pdf");
        assert_eq!(session.preview().export_file().unwrap(), output);

        session.preview().export(&output).unwrap();
        let written = std::fs::read(&output).unwrap();
        assert_eq!(written, session.preview().pdf().unwrap());

        // Without this assertion the two halves of the gate would meet only
        // through a reading of the two asset readers, and a later divergence
        // in either would pass both while the wrappers disagreed.
        let markdown = std::fs::read_to_string(&document).unwrap();
        let expected = md2pdf_core::md_to_pdf(&markdown, &article_assets(&dir)).unwrap();
        assert_eq!(written, expected);

        // **Nothing recompiled**, not for the export and not for the PDF it
        // left in the very directory the loop watches — `revision` is what
        // "recompiled" means, and it has not moved.
        //
        // The *announcement* did happen, and that is `mpdf-010` Phase 1's
        // `Change::Tree` arriving on a real filesystem rather than in a
        // constructed case: a file appeared in the project, so the panel is out
        // of date and the page is fetched again to redraw it. The counter
        // beside `revision` is what tells the two apart.
        settle();
        let after = session.preview().status();
        assert_eq!(
            after.revision, 1,
            "a file appearing in the project compiled"
        );
        assert!(
            compiles.load(Ordering::SeqCst) > 1,
            "the panel was never told the export had landed"
        );

        // And it is listed, PDF and all. `pdf` is in
        // `md2pdf_core::IMAGE_EXTENSIONS` because a PDF is a legal figure in
        // this dialect, so a document's own export appears beside its figures.
        // That is `specs/file_panel_spec.md` OQ-3, pinned rather than
        // special-cased on a guess.
        assert!(
            after
                .entries
                .iter()
                .any(|entry| entry.path == "article.pdf" && entry.kind == document::Kind::Image),
            "the exported PDF is not in the panel: {:?}",
            after.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );
    }

    /// The first of two refusals: the bytes exist and are known to be old.
    #[test]
    fn export_is_refused_while_the_pane_is_stale() {
        let dir = scratch_dir("export-stale");
        let document = article_in(&dir);

        let mut preview = compiled(&document);
        assert_eq!(preview.state(), State::Current);

        preview.edit("# Broken\n\n<div>raw HTML</div>\n".to_string());
        preview.compile();
        assert_eq!(preview.state(), State::Stale);

        let output = dir.join("refused.pdf");
        let refusal = preview.export(&output).unwrap_err();
        assert!(refusal.contains("out of date"), "{refusal}");
        assert!(preview.export_path().is_err(), "the dialog would open");
        assert!(!output.exists(), "a refused export wrote a file");
    }

    /// The second refusal is not the first. `Preview::default()` has the stale
    /// mark clear and no bytes at all, and an implementer who tests only the
    /// case above leaves the launch state to panic or to write nothing.
    #[test]
    fn export_is_refused_while_no_document_is_open() {
        let dir = scratch_dir("export-empty");
        let preview = Preview::default();
        assert_eq!(preview.state(), State::Empty);

        let output = dir.join("refused.pdf");
        let refusal = preview.export(&output).unwrap_err();
        assert!(refusal.contains("no document is open"), "{refusal}");
        assert!(preview.export_path().is_err(), "the dialog would open");
        assert!(!output.exists(), "a refused export wrote a file");
    }

    /// The state the app launches into and holds until the first Open.
    #[test]
    fn the_empty_status_names_no_document_and_no_time() {
        let status = Preview::default().status();

        assert_eq!(status.state, State::Empty);
        assert_eq!(status.time, None);
        assert_eq!(status.error, None);
        assert!(!status.page);
    }

    #[test]
    fn the_current_status_names_the_compile_time() {
        let dir = scratch_dir("status-current");
        let status = compiled(&article_in(&dir)).status();

        assert_eq!(status.state, State::Current);
        assert_eq!(status.error, None);
        assert!(status.page);
        assert!(
            status.time.as_deref().unwrap().ends_with(" ms"),
            "{:?}",
            status.time
        );
    }

    #[test]
    fn the_stale_status_names_the_error_and_keeps_the_page() {
        let dir = scratch_dir("status-stale");
        let mut preview = compiled(&article_in(&dir));

        preview.edit("# Broken\n\n<div>raw HTML</div>\n".to_string());
        preview.compile();
        let status = preview.status();

        assert_eq!(status.state, State::Stale);
        assert!(status.page);
        assert!(
            status.error.as_deref().unwrap().contains("raw HTML block"),
            "{:?}",
            status.error
        );
        // The time belongs to the page still drawn, not to the attempt that
        // failed, because the duration travels with the bytes.
        assert!(status.time.is_some());
    }

    #[test]
    fn the_failed_status_names_the_error_and_has_no_page() {
        let dir = scratch_dir("status-failed");
        let document = dir.join("paper.md");
        std::fs::write(&document, "# Paper\n\n![a mark to come](figures/new.svg)\n").unwrap();

        let status = compiled(&document).status();

        assert_eq!(status.state, State::Failed);
        assert!(!status.page);
        assert_eq!(status.time, None);
        assert!(
            status.error.as_deref().unwrap().contains("figures/new.svg"),
            "{:?}",
            status.error
        );
    }

    /// A master that stops naming sections stops having parts.
    ///
    /// **This belongs at `Preview::compile` and nowhere else.**
    /// `document::render_with` is stateless, so at *that* surface a master
    /// whose markers were just deleted and a document that never had one are
    /// the same call, and the assertion would hold however this list were kept.
    /// It is [`Preview::sections`] — retained across compiles, replaced
    /// unconditionally — that the question was ever about: under a rule that
    /// kept the last non-empty answer, a master whose markers are genuinely
    /// deleted would hold a phantom panel for the life of the open document,
    /// because no later compile could restore an empty list.
    ///
    /// **The disk half does not move with it, and that is the point.** Before
    /// `mpdf-010` the whole panel was this list, so deleting a marker emptied
    /// the panel; now the files are still on the disk and still listed, and
    /// what the deletion costs is the one marked-missing row. That is the
    /// flicker `mpdf-008` §2 accepted, reduced to the half that has to move.
    ///
    /// `main` rides along, being the other field the panel reads and the other
    /// one nothing else asserts. It is root-relative and not a bare file name,
    /// because the page matches it against a row.
    #[test]
    fn a_master_that_stops_naming_sections_loses_its_marked_missing_rows() {
        let dir = scratch_dir("parts-deleted");
        std::fs::create_dir_all(dir.join("sections")).unwrap();
        std::fs::write(dir.join("sections/one.md"), "# One\n\nText.\n").unwrap();
        let document = dir.join("report.md");
        std::fs::write(&document, "[](sections/one.md)\n\n[](sections/two.md)\n").unwrap();

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1, "opening compiles once");

        let opened = session.preview().status();
        assert_eq!(opened.main.as_deref(), Some("report.md"));
        assert_eq!(
            opened
                .entries
                .iter()
                .map(|entry| (entry.path.as_str(), entry.missing))
                .collect::<Vec<_>>(),
            [
                ("report.md", false),
                ("sections/one.md", false),
                ("sections/two.md", true),
            ],
            "the section that is named and absent is the row the author needs"
        );

        session.edit("# A master that names nothing now.\n".to_string());
        assert_eq!(wait_for(&compiles, 2), 2, "a pause in the typing compiles");

        let after = session.preview().status();
        assert_eq!(
            after
                .entries
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>(),
            ["report.md", "sections/one.md"],
            "the deleted marker left its row behind"
        );
        assert!(
            after.entries.iter().all(|entry| !entry.missing),
            "nothing is named and absent once the markers are gone"
        );
        assert_eq!(
            after.main.as_deref(),
            Some("report.md"),
            "the document is still the one that is open"
        );
    }

    // -- the project, at the session -------------------------------------
    //
    // `mpdf-010` Phase 1's exit gate, clauses 3 and 7. The pieces they are
    // built from are pinned in `crate::document`'s own tests; these are the
    // claims about what the window does with them.

    /// **Clause 3, and the phase's whole observable.**
    ///
    /// Opening a section from Finder compiles the master above it. Before this
    /// phase the same double-click compiled that one section standalone, so the
    /// two openings produce the same PDF now and produced different ones then —
    /// which is the difference a reader of this test is meant to see.
    ///
    /// The bytes are compared rather than the paths, and `is_some` is asserted
    /// beside them: two failures are equal too, and a fixture that stopped
    /// compiling would otherwise leave this passing and proving nothing.
    #[test]
    fn opening_a_section_compiles_the_master_that_names_it() {
        let panel = fixture("panel");
        let store = document::store_file(&scratch_dir("nothing-remembered"));

        let (mut session, compiles) = counted_with(store.clone());
        session.open(panel.join("sections/text.md")).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1, "opening compiles once");

        let (main, edited, bytes) = {
            let preview = session.preview();
            (
                preview.status().main,
                preview.document(),
                preview.pdf().map(<[u8]>::to_vec),
            )
        };
        assert_eq!(main.as_deref(), Some("book.md"));
        assert_eq!(
            edited,
            Some(panel.join("book.md")),
            "the pane holds the main, which is where Phase 1 leaves the two"
        );

        let (mut direct, direct_compiles) = counted_with(store);
        direct.open(panel.join("book.md")).unwrap();
        assert_eq!(wait_for(&direct_compiles, 1), 1);

        assert!(
            bytes.is_some(),
            "the fixture master did not compile, so the comparison below is vacuous"
        );
        assert_eq!(
            bytes,
            direct.preview().pdf().map(<[u8]>::to_vec),
            "opening the section and opening the master produced different pages"
        );
    }

    /// Clause 3's second half: the store is read first, and discovery is not
    /// consulted when it answers.
    ///
    /// `other.md` names no section, so discovery would never land on it — which
    /// is what makes this case tell the two apart.
    #[test]
    fn a_stored_override_decides_which_file_compiles() {
        let panel = fixture("panel");
        let store = document::store_file(&scratch_dir("override"));
        document::write_override(&store, &panel, "other.md").unwrap();

        let (mut session, compiles) = counted_with(store);
        session.open(panel.join("sections/text.md")).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        assert_eq!(
            session.preview().status().main.as_deref(),
            Some("other.md"),
            "discovery answered where the store already had"
        );
    }

    /// An override naming a file the disk no longer holds falls through to
    /// discovery, rather than opening nothing.
    ///
    /// The store is keyed by root and a wrong root cannot be corrected from
    /// inside the panel, so a stale key must not be able to strand a window.
    #[test]
    fn an_override_naming_nothing_falls_back_to_discovery() {
        let panel = fixture("panel");
        let store = document::store_file(&scratch_dir("stale-override"));
        document::write_override(&store, &panel, "deleted.md").unwrap();

        let (mut session, compiles) = counted_with(store);
        session.open(panel.join("sections/text.md")).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        assert_eq!(session.preview().status().main.as_deref(), Some("book.md"));
    }

    /// Setting the main moves the pane, marks the new row, and is remembered
    /// the next time that folder is opened.
    ///
    /// The reopen is what makes the last claim worth asserting: a field written
    /// into `Preview` and not onto the disk would pass every line above it.
    #[test]
    fn setting_the_main_moves_the_pane_and_is_remembered() {
        let dir = scratch_dir("set-main");
        std::fs::write(dir.join("first.md"), "# First\n\nText.\n").unwrap();
        std::fs::write(dir.join("second.md"), "# Second\n\nText.\n").unwrap();

        let store = document::store_file(&scratch_dir("set-main-store"));
        let (mut session, compiles) = counted_with(store.clone());
        session.open(dir.join("first.md")).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);
        assert_eq!(session.preview().status().main.as_deref(), Some("first.md"));

        session.set_main("second.md".to_string()).unwrap();
        assert_eq!(wait_for(&compiles, 2), 2, "the switch compiles once");
        let after = session.preview().status();
        assert_eq!(after.main.as_deref(), Some("second.md"));
        assert!(
            session.preview().text().contains("# Second"),
            "the pane is still holding the file it was told to leave"
        );

        // **A path out of the project is refused, and nothing moves** — both
        // the one that names nothing and the one that names a real file
        // somewhere else, which is the case an existence check alone would let
        // through.
        std::fs::write(dir.parent().unwrap().join("escape.md"), "# Elsewhere\n").unwrap();
        for asked in ["../escape.md", "nothing.md", "/etc/hosts"] {
            assert!(
                session.set_main(asked.to_string()).is_err(),
                "{asked} was accepted as this project's main"
            );
            assert_eq!(
                session.preview().status().main.as_deref(),
                Some("second.md")
            );
        }

        // And a second window over the same folder lands on it without asking.
        let (mut again, again_compiles) = counted_with(store);
        again.open(dir.join("first.md")).unwrap();
        assert_eq!(wait_for(&again_compiles, 1), 1);
        assert_eq!(again.preview().status().main.as_deref(), Some("second.md"));
    }

    /// **Clause 7 on a real filesystem.** A file appearing in the project
    /// refreshes the panel and compiles nothing.
    ///
    /// `revision` is what "compiles nothing" means as an assertion: it moves
    /// only on a compile that produced bytes. The announcement does happen, and
    /// must — the panel is drawn off the status that announcement fetches.
    #[test]
    fn a_file_appearing_in_the_project_refreshes_the_panel_and_compiles_nothing() {
        let dir = scratch_dir("tree-event");
        let document = article_in(&dir);

        let (mut session, compiles) = counted();
        session.open(document).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        let before = session.preview().status();
        assert!(
            !before.entries.iter().any(|entry| entry.path == "notes.md"),
            "the file under test was already listed"
        );

        std::fs::write(dir.join("notes.md"), "# A file nobody names\n").unwrap();
        assert_eq!(wait_for(&compiles, 2), 2, "the panel was never told");
        settle();

        let after = session.preview().status();
        assert_eq!(
            after.revision, before.revision,
            "a file the document does not name caused a compile"
        );
        assert!(
            after.entries.iter().any(|entry| entry.path == "notes.md"),
            "the new file is not in the panel: {:?}",
            after.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );
    }

    // -- the pane and the main are two files -------------------------------
    //
    // `mpdf-010` Phase 2's exit gate. It runs over a **writable copy of
    // `samples/showcase/`**: two of the clauses write, `samples/` is tracked,
    // and a suite that left the repository dirty would also destroy the first
    // clause's own premise the second time it was run.

    /// A writable copy of `samples/showcase/`, and its own store.
    fn showcase_in(name: &str) -> (PathBuf, PathBuf) {
        let root = scratch_dir(name);
        copy_tree(&sample("showcase"), &root);
        (
            root,
            document::store_file(&scratch_dir(&format!("{name}-store"))),
        )
    }

    fn copy_tree(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap().flatten() {
            let (source, target) = (entry.path(), to.join(entry.file_name()));
            if source.is_dir() {
                copy_tree(&source, &target);
            } else {
                std::fs::copy(&source, &target).unwrap();
            }
        }
    }

    /// A loaded [`Preview`] with the two files named separately.
    ///
    /// [`compiled`] is its single-file counterpart and cannot express this: it
    /// derives the main from the document, which is the conflation this phase
    /// ends.
    fn project(root: &Path, main: &str, edited: &str) -> Preview {
        let mut preview = Preview::default();
        preview.open(
            Disk::new(root),
            main.to_string(),
            edited.to_string(),
            BTreeSet::new(),
        );
        preview.load();
        preview
    }

    fn lines_of(preview: &Preview) -> Vec<usize> {
        preview.anchors().iter().map(|anchor| anchor.line).collect()
    }

    /// **Clause 1, and the phase's whole observable.** The page is the master's,
    /// and the pane's unsaved text is in it.
    ///
    /// Checked against a compile of the same tree after the buffer has been put
    /// on the disk, in the same directory and at the same absolute paths — so
    /// nothing here turns on a compile being reproducible across two locations.
    #[test]
    fn the_panes_unsaved_section_reaches_the_master_that_compiles() {
        let (root, _) = showcase_in("phase2-override");

        let mut preview = project(&root, "showcase.md", "sections/mathematics.md");
        let typed = format!("{}\nA paragraph nobody has saved.\n", preview.text());
        preview.edit(typed.clone());
        preview.compile();
        let unsaved = preview.pdf().expect("the unsaved compile failed").to_vec();

        std::fs::write(root.join("sections/mathematics.md"), &typed).unwrap();
        let saved = project(&root, "showcase.md", "showcase.md");

        assert_eq!(
            unsaved,
            saved.pdf().expect("the saved compile failed"),
            "the buffer did not reach the compile"
        );
    }

    /// **Clause 2. The anchors are lines, not files.**
    ///
    /// `document::Anchor` is `{ line, page }` and the filter under test is
    /// precisely what drops `location.file`, so the claim is keyed to what
    /// survives it. The two sets are disjoint by construction: `mathematics.md`
    /// has one heading, on its own first line, where the master's own three sit
    /// at lines 32, 47 and 94 of `showcase.md` — below its thirteen-line
    /// frontmatter and the `::: abstract` and `::: keywords` blocks under that,
    /// and nowhere near line 1.
    #[test]
    fn the_anchors_are_the_headings_of_whichever_file_the_pane_holds() {
        let (root, _) = showcase_in("phase2-anchors");

        let section = project(&root, "showcase.md", "sections/mathematics.md");
        let master = project(&root, "showcase.md", "showcase.md");

        let (theirs, its_own) = (lines_of(&section), lines_of(&master));
        assert_eq!(theirs, [1], "the section's own heading");
        assert_eq!(its_own, [32, 47, 94], "the master's own three");
        assert!(
            theirs.iter().all(|line| !its_own.contains(line)),
            "the two sets are not disjoint, so this clause proves nothing"
        );
    }

    /// **Clause 3.** `⌘S` writes the file in the pane and nothing else.
    #[test]
    fn the_save_writes_the_file_in_the_pane_and_leaves_the_master_alone() {
        let (root, _) = showcase_in("phase2-save");
        let master = std::fs::read(root.join("showcase.md")).unwrap();

        let mut preview = project(&root, "showcase.md", "sections/mathematics.md");
        let typed = format!(
            "{}\nA paragraph the author means to keep.\n",
            preview.text()
        );
        preview.edit(typed.clone());
        preview.save().unwrap();

        assert_eq!(
            std::fs::read_to_string(root.join("sections/mathematics.md")).unwrap(),
            typed
        );
        assert_eq!(
            std::fs::read(root.join("showcase.md")).unwrap(),
            master,
            "the save wrote the master too"
        );
    }

    /// **Clause 4.** The switch refuses over unsaved work, names two ways out
    /// without claiming the file moved, and the discard is the second of them.
    #[test]
    fn a_switch_refuses_over_unsaved_work_and_the_discard_lets_it_through() {
        let (root, store) = showcase_in("phase2-refusal");
        let (mut session, compiles) = counted_with(store);
        session.open(root.join("showcase.md")).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        let mine = "# Mine, unsaved\n\nStill being written.\n".to_string();
        session.edit(mine.clone());
        settle();

        session
            .set_edited("sections/mathematics.md".to_string())
            .unwrap();

        let refused = session.preview().status();
        assert_eq!(
            refused.edited.as_deref(),
            Some("showcase.md"),
            "the switch moved the pane anyway"
        );
        assert_eq!(session.preview().text(), mine, "it took the work");

        let sentence = refused.divergence.expect("the refusal said nothing");
        assert!(
            !sentence.contains("changed on disk"),
            "the refusal claims the file moved, which it did not: {sentence:?}"
        );
        assert!(
            sentence.contains("Save") && sentence.contains("discard"),
            "the refusal names fewer than two ways out: {sentence:?}"
        );

        session
            .set_main("sections/mathematics.md".to_string())
            .unwrap();
        assert_eq!(
            session.preview().status().main.as_deref(),
            Some("showcase.md"),
            "set_main took the same work the switch refused to lose"
        );

        session.discard();
        let clean = session.preview().status();
        assert_eq!(clean.divergence, None, "the discard left the refusal up");
        assert_ne!(session.preview().text(), mine, "the discard kept the work");

        session
            .set_edited("sections/mathematics.md".to_string())
            .unwrap();
        assert_eq!(
            session.preview().status().edited.as_deref(),
            Some("sections/mathematics.md"),
            "the switch was refused twice"
        );
        assert_eq!(
            session.preview().status().main.as_deref(),
            Some("showcase.md"),
            "the switch moved the main"
        );
    }

    /// **`mpdf-010` Phase 8.** `⌘S` writes the bibliography in the pane and
    /// nothing else — Phase 2's clause 3 asked of a file that is not markdown.
    ///
    /// [`Preview::save`] writes `edited` whatever its extension and this phase
    /// changed none of that, which is the point: the kind gate was in the page
    /// and never here. It is asserted rather than argued because it is the
    /// half of the gesture the compile above cannot see.
    #[test]
    fn the_save_writes_a_bibliography_in_the_pane_and_leaves_the_master_alone() {
        let root = scratch_dir("phase8-save");
        copy_tree(&fixture("panel"), &root);
        let master = std::fs::read(root.join("book.md")).unwrap();

        let mut preview = project(&root, "book.md", "refs.bib");
        let typed = preview
            .text()
            .replace("A Book the Panel Lists", "A Title Nobody Has Saved");
        assert_ne!(typed, preview.text(), "the buffer matches the file on disk");

        preview.edit(typed.clone());
        preview.save().unwrap();

        assert_eq!(
            std::fs::read_to_string(root.join("refs.bib")).unwrap(),
            typed,
            "the save did not reach the bibliography"
        );
        assert_eq!(
            std::fs::read(root.join("book.md")).unwrap(),
            master,
            "the save wrote the master too"
        );
    }

    /// **Clause 5, first half.** The master moving on disk is a bare recompile,
    /// and it is one even while the pane holds work of its own.
    #[test]
    fn an_external_write_to_the_master_recompiles_and_does_not_run_the_rule() {
        let (root, store) = showcase_in("phase2-master-moved");
        let (mut session, compiles) = counted_with(store);
        session.open(root.join("showcase.md")).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        session
            .set_edited("sections/mathematics.md".to_string())
            .unwrap();
        settle();
        let seen = compiles.load(Ordering::SeqCst);
        let before = session.preview().status();

        session.edit(format!("{}\nUnsaved.\n", session.preview().text()));
        assert_eq!(wait_for(&compiles, seen + 1), seen + 1);

        let master = std::fs::read_to_string(root.join("showcase.md")).unwrap();
        std::fs::write(
            root.join("showcase.md"),
            format!("{master}\nA line another program added.\n"),
        )
        .unwrap();
        assert_eq!(
            wait_for(&compiles, seen + 2),
            seen + 2,
            "it never recompiled"
        );
        settle();

        let after = session.preview().status();
        assert_eq!(
            after.divergence, None,
            "the master's own event ran the divergence rule"
        );
        assert!(
            after.revision > before.revision,
            "the master moved and nothing recompiled"
        );
    }

    /// **Clause 5, second half.** The file in the pane moving runs the rule —
    /// and it still does when that file is the master, which is Phase 1's
    /// behaviour and every single-file document's.
    #[test]
    fn an_external_write_to_the_file_in_the_pane_runs_the_rule() {
        for (name, edited) in [
            ("phase2-section-moved", "sections/mathematics.md"),
            ("phase2-only-file-moved", "showcase.md"),
        ] {
            let (root, store) = showcase_in(name);
            let (mut session, compiles) = counted_with(store);
            session.open(root.join("showcase.md")).unwrap();
            assert_eq!(wait_for(&compiles, 1), 1);

            if edited != "showcase.md" {
                session.set_edited(edited.to_string()).unwrap();
                settle();
            }
            let seen = compiles.load(Ordering::SeqCst);

            session.edit("# Mine, unsaved\n\nStill being written.\n".to_string());
            assert_eq!(wait_for(&compiles, seen + 1), seen + 1);

            std::fs::write(root.join(edited), "# Theirs\n\nBy another program.\n").unwrap();
            assert_eq!(
                wait_for(&compiles, seen + 2),
                seen + 2,
                "{edited}: the change was never noticed"
            );
            settle();

            let status = session.preview().status();
            assert!(
                status
                    .divergence
                    .as_deref()
                    .is_some_and(|said| said.contains("changed on disk")),
                "{edited}: the rule did not run — {:?}",
                status.divergence
            );
            assert_eq!(
                session.preview().text(),
                "# Mine, unsaved\n\nStill being written.\n",
                "{edited}: the work was taken"
            );
        }
    }

    /// **Clause 6.** A single-file document is what it always was.
    ///
    /// "Equal to what it produced before this phase" has nothing committed to
    /// compare against — `tests/golden/` holds `.typ` and no PDF — so the
    /// reproducible form of the claim is a compile this test asks the library
    /// for itself, which is how
    /// `document::tests::a_single_file_document_keeps_its_anchors_and_its_bytes`
    /// already makes it one level down.
    #[test]
    fn a_document_that_is_its_own_project_compiles_to_the_librarys_own_bytes() {
        let dir = scratch_dir("phase2-article");
        article_in(&dir);

        let preview = project(&dir, "article.md", "article.md");
        let markdown = std::fs::read_to_string(dir.join("article.md")).unwrap();

        assert_eq!(
            preview.pdf().expect("the compile failed"),
            md2pdf_core::md_to_pdf(&markdown, &article_assets(&dir)).unwrap()
        );
    }

    // -- a file moved to the Trash ------------------------------------------
    //
    // `mpdf-010` Phase 4. **Nothing here puts a file in anybody's Trash**: the
    // call is a parameter all the way up to the command, so the three clauses
    // below hand in a double — which removes the file as well as recording the
    // call, since clause 7 turns on it being gone.

    /// What the double was asked to move, as the clauses read it back.
    type Moved = Arc<Mutex<Vec<PathBuf>>>;

    /// A `Session::trash` call that records what it was asked to move and
    /// removes it. `document::tests` has its own; this one is here because
    /// that module is private to that file.
    fn recording() -> (impl FnOnce(&Path) -> Result<(), String>, Moved) {
        let moved = Arc::new(Mutex::new(Vec::new()));
        let taken = Arc::clone(&moved);

        let double = move |path: &Path| {
            taken.lock().unwrap().push(path.to_path_buf());
            std::fs::remove_file(path).map_err(|e| format!("the double could not remove: {e}"))
        };
        (double, moved)
    }

    /// **Clause 5.** The file that compiles is refused while it compiles.
    #[test]
    fn the_main_is_refused_and_nothing_is_moved() {
        let (root, store) = showcase_in("phase4-main");
        let (mut session, compiles) = counted_with(store);
        session.open(root.join("showcase.md")).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        let (double, moved) = recording();
        let refused = session
            .trash("showcase.md".to_string(), double)
            .expect_err("the main was trashed");

        assert!(
            refused.contains("compiles"),
            "the refusal does not say why: {refused:?}"
        );
        assert!(
            moved.lock().unwrap().is_empty(),
            "a refused delete called the OS anyway"
        );
        assert!(root.join("showcase.md").is_file(), "the master is gone");
    }

    /// **Clause 6.** The file the pane holds: refused over unsaved work, and
    /// over a clean buffer it falls back to the main — **three things and not
    /// one**.
    ///
    /// The middle assertion is the clause. A build that armed the loops without
    /// reading the main into the buffer passes the first and the third, and
    /// then writes the *trashed* file's text over the master on the next `⌘S`.
    #[test]
    fn trashing_the_file_in_the_pane_refuses_over_work_and_otherwise_falls_back_to_the_main() {
        let (root, store) = showcase_in("phase4-edited");
        let (mut session, compiles) = counted_with(store);
        session.open(root.join("showcase.md")).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);

        session
            .set_edited("sections/mathematics.md".to_string())
            .unwrap();
        settle();

        let mine = "# Mine, unsaved\n\nStill being written.\n".to_string();
        session.edit(mine.clone());
        settle();

        let (double, moved) = recording();
        session
            .trash("sections/mathematics.md".to_string(), double)
            .unwrap();

        let refused = session.preview().status();
        assert!(
            moved.lock().unwrap().is_empty(),
            "the refusal moved the file anyway"
        );
        assert_eq!(
            refused.edited.as_deref(),
            Some("sections/mathematics.md"),
            "the refusal moved the pane"
        );
        assert_eq!(session.preview().text(), mine, "it took the work");

        let sentence = refused.divergence.expect("the refusal said nothing");
        assert!(
            !sentence.contains("open"),
            "the refusal claims a file is being opened, which it is not: {sentence:?}"
        );
        assert!(
            sentence.contains("Save") && sentence.contains("discard"),
            "the refusal names fewer than two ways out: {sentence:?}"
        );

        // The second way out, and then the delete goes through.
        session.discard();
        let master = std::fs::read_to_string(root.join("showcase.md")).unwrap();
        let seen = compiles.load(Ordering::SeqCst);

        let (double, moved) = recording();
        session
            .trash("sections/mathematics.md".to_string(), double)
            .unwrap();

        assert_eq!(
            *moved.lock().unwrap(),
            [root.join("sections/mathematics.md")],
            "the delete did not move the file the pane held"
        );

        // One: the pane falls back to the file that compiles.
        assert_eq!(
            session.preview().status().edited.as_deref(),
            Some("showcase.md"),
            "the pane is still holding a file that is gone"
        );
        // Two, **and this is the clause**: it was read, not assigned.
        assert_eq!(
            session.preview().text(),
            master,
            "the buffer still holds the trashed file's text"
        );
        assert_eq!(
            session.preview().saved(),
            master,
            "the last-saved text still belongs to the trashed file"
        );
        // Three: both loops survived the move, asserted as clause 5 of Phase 2
        // asserts them — a subsequent edit compiles, and a subsequent external
        // write runs the rule.
        session.edit(mine.clone());
        assert_eq!(
            wait_for(&compiles, seen + 1),
            seen + 1,
            "the typing loop is dead"
        );

        std::fs::write(
            root.join("showcase.md"),
            "# Theirs\n\nBy another program.\n",
        )
        .unwrap();
        assert_eq!(
            wait_for(&compiles, seen + 2),
            seen + 2,
            "the watch loop is dead"
        );
        settle();
        assert!(
            session
                .preview()
                .status()
                .divergence
                .as_deref()
                .is_some_and(|said| said.contains("changed on disk")),
            "the rule did not run over the file the pane fell back to"
        );
    }

    /// **Clause 7.** A section the master names goes, the panel keeps it as a
    /// marked-missing row, and the next compile refuses by its name.
    ///
    /// **Over a scratch copy and not the committed fixture**, which Phase 3's
    /// gate could use because a refusal writes nothing — a delete does not have
    /// that property.
    ///
    /// The refusal is `document::read_sections_with`'s sentence and **not
    /// `md2pdf_core::Error::MissingSection`**, which this app never reaches: `?`
    /// short-circuits `render_with`'s chain before `core/src/sections.rs`'s only
    /// raising site is called at all.
    ///
    /// *A note so it is not rediscovered:* [`copy_tree`] reads through
    /// `std::fs::copy`, which follows a symlink, so `<copy>/outside` arrives as
    /// a real directory holding the decoy's two files. No assertion here
    /// depends on the row count.
    #[test]
    fn trashing_a_named_section_leaves_its_row_and_refuses_the_next_compile() {
        let root = scratch_dir("phase4-section");
        copy_tree(&fixture("panel"), &root);
        let store = document::store_file(&scratch_dir("phase4-section-store"));

        let (mut session, compiles) = counted_with(store);
        session.open(root.join("book.md")).unwrap();
        assert_eq!(wait_for(&compiles, 1), 1);
        assert_eq!(session.preview().status().main.as_deref(), Some("book.md"));

        let (double, moved) = recording();
        session
            .trash("sections/text.md".to_string(), double)
            .unwrap();
        assert_eq!(*moved.lock().unwrap(), [root.join("sections/text.md")]);
        assert!(!root.join("sections/text.md").exists());

        // The panel is the command's own re-walk: the watch answers `Asset` for
        // a path in the asset list and would never refresh the listing.
        let listed = session.preview().status().entries;
        assert!(
            listed
                .iter()
                .any(|entry| entry.path == "sections/text.md" && entry.missing),
            "the deleted section is not the marked-missing row the master still names: {:?}",
            listed
                .iter()
                .map(|entry| (entry.path.as_str(), entry.missing))
                .collect::<Vec<_>>()
        );

        session.preview().compile();
        let error = session
            .preview()
            .error()
            .map(str::to_string)
            .expect("the compile succeeded without a section it names");
        assert!(
            error.contains("cannot read") && error.contains("for the section"),
            "the refusal is not the app's own sentence: {error:?}"
        );
        assert!(
            error.contains("sections/text.md"),
            "the refusal does not name the file: {error:?}"
        );
    }

    /// The `@property` names of one `@typedef {object} …` block in the page.
    ///
    /// Plain string operations, and no HTML parser enters this crate for it:
    /// the markers are fixed literals and the block ends at the first `*/`
    /// after one. The type is skipped by brace depth rather than by the first
    /// `}`, so an inline object type would not silently truncate the name.
    fn typedef_properties(page: &str, name: &str) -> Vec<String> {
        let marker = format!("@typedef {{object}} {name}\n");
        let at = page
            .find(&marker)
            .unwrap_or_else(|| panic!("the page declares no `@typedef {{object}} {name}`"));
        let block = &page[at + marker.len()..];
        let block = &block[..block.find("*/").expect("an unterminated JSDoc block")];

        block
            .lines()
            .filter_map(|line| {
                let rest = line.trim_start().strip_prefix("* @property ")?;
                let rest = rest.strip_prefix('{')?;
                let mut depth = 1usize;
                let close = rest.char_indices().find_map(|(at, glyph)| {
                    match glyph {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(at);
                            }
                        }
                        _ => {}
                    }
                    None
                })?;
                Some(rest[close + 1..].split_whitespace().next()?.to_string())
            })
            .collect()
    }

    /// The page's typedefs and this crate's `Status` name the same fields.
    ///
    /// **This is the narrow edge `specs/desktop_app_spec.md` OQ-10 names.**
    /// `invoke` answers with an untyped value and `app/dist/index.html` reads
    /// thirteen fields off it by name, so a field renamed here and not there breaks
    /// the window silently, at runtime, with no console anyone reads. The type
    /// check over that file (`app/typecheck.mjs`) makes the typedef bind on the
    /// page's side; this makes it bind on Rust's. **Two declarations compared
    /// against each other**, rather than usage compared against a declaration.
    ///
    /// `anchors` holds one, and must: `Anchor` is not reachable in the JSON at
    /// all while the list is empty. `entries` holds one for the same reason.
    ///
    /// **The count moved from ten to eleven in `mpdf-010` Phase 2**, which
    /// added `edited` beside `main` so the panel can mark the row the pane is
    /// holding, and **from eleven to twelve in `mpdf-003` Phase 13**, which
    /// added `appearance` so the footer's toggle places what Rust decided, and
    /// **from twelve to thirteen in `mpdf-003` Phase 25**, which added `web` —
    /// the line about images named by URL — and joined `WebLine` to the
    /// declarations checked here, `Some` in the literal so there is a line to
    /// compare. Phase 1 left it at ten by coincidence — it removed `sections` and
    /// `master` and added `entries` and `main` — and said so here, in a note
    /// each later phase's own scope is the authority for rewriting. A literal
    /// that has now moved twice is still not a thing to "fix" to silence a
    /// failure: a failure here is about a field.
    #[test]
    fn the_page_typedefs_name_exactly_the_fields_status_serializes() {
        const PAGE: &str = include_str!("../dist/index.html");

        let status = Status {
            state: State::Current,
            time: Some("28 ms".to_string()),
            error: None,
            page: true,
            divergence: None,
            revision: 3,
            reloaded: 1,
            anchors: vec![document::Anchor { line: 1, page: 1 }],
            entries: vec![document::Entry {
                path: "sections/one.md".to_string(),
                kind: document::Kind::Markdown,
                missing: false,
            }],
            main: Some("report.md".to_string()),
            edited: Some("sections/method.md".to_string()),
            appearance: Appearance::Dark,
            web: Some(WebLine {
                sentence: "1 image on images.example is not fetched.".to_string(),
                action: Some("Fetch images from the web".to_string()),
            }),
        };
        let sent = serde_json::to_value(&status).expect("a `Status` that will not serialize");

        let mut carried: Vec<String> = sent
            .as_object()
            .expect("a `Status` that is not a JSON object")
            .keys()
            .cloned()
            .collect();
        let mut declared = typedef_properties(PAGE, "Status");
        carried.sort_unstable();
        declared.sort_unstable();
        // Both counts, not just the equality: two empty lists are equal, and a
        // marker that stopped matching should fail loudly rather than pass.
        assert_eq!(
            declared.len(),
            13,
            "the page's `Status` typedef declares {} properties: {:?}",
            declared.len(),
            declared
        );
        assert_eq!(
            declared, carried,
            "the page's `Status` typedef and the serialized `Status` name different fields"
        );

        let mut riding: Vec<String> = sent["anchors"][0]
            .as_object()
            .expect("an `Anchor` that is not a JSON object")
            .keys()
            .cloned()
            .collect();
        let mut named = typedef_properties(PAGE, "Anchor");
        riding.sort_unstable();
        named.sort_unstable();
        assert_eq!(
            named.len(),
            2,
            "the page's `Anchor` typedef declares {} properties: {:?}",
            named.len(),
            named
        );
        assert_eq!(
            named, riding,
            "the page's `Anchor` typedef and the anchors a `Status` carries name different fields"
        );

        let mut listed: Vec<String> = sent["entries"][0]
            .as_object()
            .expect("an `Entry` that is not a JSON object")
            .keys()
            .cloned()
            .collect();
        let mut drawn = typedef_properties(PAGE, "Entry");
        listed.sort_unstable();
        drawn.sort_unstable();
        assert_eq!(
            drawn.len(),
            3,
            "the page's `Entry` typedef declares {} properties: {:?}",
            drawn.len(),
            drawn
        );
        assert_eq!(
            drawn, listed,
            "the page's `Entry` typedef and the entries a `Status` carries name different fields"
        );

        let mut worded: Vec<String> = sent["web"]
            .as_object()
            .expect("a `WebLine` that is not a JSON object")
            .keys()
            .cloned()
            .collect();
        let mut placed = typedef_properties(PAGE, "WebLine");
        worded.sort_unstable();
        placed.sort_unstable();
        assert_eq!(
            placed.len(),
            2,
            "the page's `WebLine` typedef declares {} properties: {:?}",
            placed.len(),
            placed
        );
        assert_eq!(
            placed, worded,
            "the page's `WebLine` typedef and the line a `Status` carries name different fields"
        );

        // The kind crosses as a bare lowercase word, which is what the page
        // switches on. A `rename_all` dropped here would send `"Markdown"` and
        // every row would draw as the fallback.
        assert_eq!(sent["entries"][0]["kind"], "markdown");
    }

    /// A `Session` told an appearance reports it; a bare `Preview` reports
    /// `System`.
    ///
    /// **The seam this reaches is the two-place composition, and it is silent
    /// when it is wrong.** `Preview::status` deliberately fills
    /// `Appearance::System` and only `Session::status` corrects it, so a
    /// composition that dropped the override would report `System` for ever
    /// while everything else went on working: `set_theme` flips the native
    /// appearance directly, so the title bar and the palette would follow the
    /// author's choice and only the footer's own mark would stall. The harness
    /// stubs Rust entirely and would read as passing.
    ///
    /// **Both halves are load-bearing.** Without the second, an implementation
    /// that guessed a value in `Preview::status` instead of filling `System`
    /// would pass; without the first, one that never overrode would.
    ///
    /// What it does **not** reach is the call site — whether
    /// `crate::main::status` was moved onto `Session::status` at all is outside
    /// every test in this repository, by the division `main.rs` records for
    /// itself, and the window gate is where that line gets eyes.
    #[test]
    fn the_session_carries_the_appearance_and_a_bare_preview_does_not() {
        let (mut session, compiles) = counted();

        assert_eq!(
            Preview::default().status().appearance,
            Appearance::System,
            "a `Preview` knows no appearance and must say so"
        );
        assert_eq!(
            session.status().appearance,
            Appearance::System,
            "a session nobody has told follows the system"
        );

        for appearance in [Appearance::Dark, Appearance::Light, Appearance::System] {
            session.set_appearance(appearance).unwrap();
            assert_eq!(
                session.status().appearance,
                appearance,
                "the session did not report the appearance it was told"
            );
            assert_eq!(
                session.preview().status().appearance,
                Appearance::System,
                "the preview answered an appearance it cannot know"
            );
        }

        // It announces, and that is the whole of how the footer's mark moves:
        // `set_appearance` compiles nothing, so this counts announcements and
        // not compiles.
        assert_eq!(
            compiles.load(Ordering::SeqCst),
            3,
            "each appearance must announce exactly once"
        );

        // And the twelve fields that are the last compile's are still the
        // preview's own, so the override is one field and not a second status.
        let empty = Preview::default().status();
        let told = session.status();
        assert_eq!(told.state, empty.state);
        assert_eq!(told.revision, empty.revision);
        assert_eq!(told.entries, empty.entries);
    }

    /// The footer's brand cell says exactly what the bundle is called.
    ///
    /// `mpdf-003` Phase 11 puts the product name along the foot of the window
    /// because the title bar stops carrying it at the first open — `set_edited`
    /// ends in `window.set_title`, handed the document's own file name. A name
    /// that is on screen for the rest of the session is a name that has to be
    /// right: `tauri.conf.json` is what the bundler reads and
    /// `app/dist/index.html` is what the reader sees, and nothing else makes the
    /// two agree.
    ///
    /// **This is the one string a later rename could falsify in silence.** Phase
    /// 10 moved seven of them across five files and every other one is reachable
    /// from a build or a test; the page is outside both. Phase 10's own text
    /// says a brand cell must not name a product the bundle does not carry, and
    /// this is what makes that mechanical rather than a promise.
    ///
    /// Two declarations compared against each other, as
    /// `the_page_typedefs_name_exactly_the_fields_status_serializes` compares
    /// two — and, as there, the extraction is asserted non-empty as well as
    /// equal, so a marker that stopped matching fails loudly instead of
    /// comparing two empty strings.
    #[test]
    fn the_brand_cell_says_exactly_the_bundles_product_name() {
        const PAGE: &str = include_str!("../dist/index.html");
        const BUNDLE: &str = include_str!("../tauri.conf.json");

        // Plain string operations, and no HTML parser enters this crate for it:
        // the id is a fixed literal, and the cell's text is what lies between
        // the first `>` after it and the `<` that closes the element.
        let at = PAGE
            .find("id=\"brand\"")
            .expect("the page declares no element with `id=\"brand\"`");
        let rest = &PAGE[at..];
        let open = rest
            .find('>')
            .expect("the `#brand` start tag is never closed");
        let close = rest
            .find('<')
            .expect("the `#brand` element is never closed");
        assert!(
            close > open,
            "the `#brand` start tag does not close before the element does"
        );
        let drawn = rest[open + 1..close].trim();

        let bundle: serde_json::Value =
            serde_json::from_str(BUNDLE).expect("a `tauri.conf.json` that will not parse");
        let product = bundle["productName"]
            .as_str()
            .expect("`tauri.conf.json` names no `productName`");

        assert!(!drawn.is_empty(), "the footer's brand cell draws nothing");
        assert_eq!(
            drawn, product,
            "the footer's brand cell and the bundle's `productName` name different products"
        );
    }

    /// How long a case waits on another thread before calling it wiring rather
    /// than slow.
    ///
    /// **A bound on wiring and not a measurement**, as [`wait_for`]'s is:
    /// nothing below is timed, and no value of this changes what any case
    /// asserts. It exists because a build that still holds the lock across a
    /// render must fail with a sentence rather than deadlock `cargo test`,
    /// which has no timeout of its own.
    const WIRING: Duration = Duration::from_secs(10);

    /// A render that reports itself and then blocks until it is let go.
    ///
    /// It is [`Compile::run`] behind a gate, and it is what the five cases for
    /// `mpdf-003` Phase 22 rest on: with a render held open, a case is *inside*
    /// the window this phase exists for and can drive a keystroke, an open or a
    /// second render through it rather than race a real compile to observe one.
    ///
    /// **A release names a serial and not an arrival**, which is the difference
    /// the ordering cases turn on: two closures plan under the lock and render
    /// outside it, so which of them *entered* first is the scheduler's answer
    /// where which of them *started* first is [`Preview::started`]'s — and start
    /// order is the thing under test.
    struct Gate {
        entered: mpsc::Receiver<u64>,
        reporting: mpsc::Sender<u64>,
        cleared: Arc<(Mutex<Vec<u64>>, Condvar)>,
    }

    impl Gate {
        fn new() -> Self {
            let (reporting, entered) = mpsc::channel();
            Self {
                entered,
                reporting,
                cleared: Arc::new((Mutex::new(Vec::new()), Condvar::new())),
            }
        }

        /// The render to hand to [`Session::recompile_with`] or
        /// [`Session::on_change_with`]. One gate can serve several.
        fn render(&self) -> impl Fn(&Compile) -> Rendered + Send + Sync + 'static {
            let reporting = self.reporting.clone();
            let cleared = Arc::clone(&self.cleared);

            move |plan: &Compile| {
                let _ = reporting.send(plan.serial());

                let (serials, waking) = &*cleared;
                let mut serials = serials.lock().expect("the gate was poisoned");
                while !serials.contains(&plan.serial()) {
                    serials = waking.wait(serials).expect("the gate was poisoned");
                }
                drop(serials);

                plan.run()
            }
        }

        /// The serial of the next render to enter, or a failure that says so.
        fn entered(&self) -> u64 {
            self.entered
                .recv_timeout(WIRING)
                .expect("no render entered the gate inside the deadline")
        }

        /// Let the render carrying this serial finish.
        fn release(&self, serial: u64) {
            let (serials, waking) = &*self.cleared;
            serials.lock().expect("the gate was poisoned").push(serial);
            waking.notify_all();
        }
    }

    /// Wait for a serial to be written, so a case can order two absorbs rather
    /// than hope for one. [`WIRING`]'s bound, and [`WIRING`]'s reason.
    fn wait_landed(session: &Session, serial: u64) {
        let deadline = Instant::now() + WIRING;
        while Instant::now() < deadline {
            if session.preview().landed() >= serial {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("the render carrying serial {serial} never landed");
    }

    /// The four fields an absorb writes that an Open resets, read together.
    fn written(session: &Session) -> (u64, Option<Vec<u8>>, Vec<String>, Vec<String>) {
        let preview = session.preview();
        (
            preview.revision(),
            preview.pdf().map(<[u8]>::to_vec),
            preview.assets().to_vec(),
            preview.sections().to_vec(),
        )
    }

    /// **The phase's own property**: a keystroke arriving while a compile is in
    /// flight is taken rather than made to wait for it.
    ///
    /// The render is held open on its own thread and the `edit` command runs on
    /// a second, answering on a channel this thread waits on **with a deadline
    /// rather than a join** — a build that still holds the lock across the
    /// render has to fail with a sentence, not hang the suite. No value of that
    /// deadline lets the code before this phase pass: it cannot return until the
    /// render does.
    #[test]
    fn a_keystroke_lands_while_a_compile_is_in_flight() {
        let dir = scratch_dir("a-keystroke-during-a-compile");
        let document = article_in(&dir);
        let (mut session, _) = counted();
        session.open(document.clone()).unwrap();
        session.watch = None;

        // **An `Arc` and not a `thread::scope`.** A scope joins on unwind, so a
        // panic raised while the render is still blocked would wait on a thread
        // that never finishes — the hang this case was written to avoid.
        let session = Arc::new(session);

        let gate = Gate::new();
        let recompile = session.recompile_with(document, gate.render());
        std::thread::spawn(recompile);
        let serial = gate.entered();

        let (finished, typed) = mpsc::channel();
        let typing = Arc::clone(&session);
        std::thread::spawn(move || {
            typing.edit("# typed while the compile was running\n".to_string());
            let _ = finished.send(());
        });

        let landed = typed.recv_timeout(WIRING);
        gate.release(serial);
        landed.expect("the keystroke waited on the compile in flight");
    }

    /// The text moved on while the compile ran, so its answer is dropped — and
    /// the next compile puts the new text on the page.
    #[test]
    fn a_render_whose_text_moved_on_is_dropped_and_the_next_one_lands() {
        let dir = scratch_dir("an-answer-whose-text-moved-on");
        let document = article_in(&dir);
        let (mut session, _) = counted();
        session.open(document.clone()).unwrap();
        session.watch = None;

        let before = written(&session);
        let error = session.preview().error().map(str::to_string);

        let gate = Gate::new();
        let recompile = session.recompile_with(document, gate.render());
        let running = std::thread::spawn(recompile);
        let serial = gate.entered();

        // **Through `Preview::edit` and not `Session::edit`.** The command
        // nudges the typing channel, which would leave a real compile of this
        // text due in 300 ms racing the assertions below; the command itself is
        // the case above. And the text is *different*, which is what a keystroke
        // is and what decides the assertion.
        session
            .preview()
            .edit("# moved on\n\nAnd this names no figure at all.\n".to_string());

        gate.release(serial);
        running.join().expect("the render thread panicked");

        assert_eq!(
            written(&session),
            before,
            "an answer to text the author had already moved on from was absorbed"
        );
        assert_eq!(
            session.preview().error().map(str::to_string),
            error,
            "a dropped answer wrote its message anyway"
        );

        // **Not awaited through the debounce**, which would put a 300 ms
        // interval inside a case that calls its own interval a bound.
        session.preview().compile();
        assert!(
            session.preview().revision() > before.0,
            "the text that replaced it never reached the page"
        );
        assert!(
            session.preview().assets().is_empty(),
            "the new text names no figure, so the asset list should have emptied"
        );
    }

    /// The later-**started** render wins, whichever of the two finishes first.
    ///
    /// Two plans through the seam over identical inputs, so nothing but the
    /// serial can tell them apart. No clock: the order they land in is this
    /// case's to choose.
    #[test]
    fn the_later_started_render_wins_whichever_lands_first() {
        let dir = scratch_dir("two-renders-in-flight");
        let document = article_in(&dir);
        let (mut session, _) = counted();
        session.open(document.clone()).unwrap();
        session.watch = None;

        let gate = Gate::new();
        let first = session.recompile_with(document.clone(), gate.render());
        let second = session.recompile_with(document.clone(), gate.render());
        let one = std::thread::spawn(first);
        let two = std::thread::spawn(second);

        let (a, b) = (gate.entered(), gate.entered());
        let (earlier, later) = (a.min(b), a.max(b));

        // Out of start order: the later-started one lands, and the earlier one,
        // arriving after it, is dropped.
        gate.release(later);
        wait_landed(&session, later);
        let after_the_later = written(&session);

        gate.release(earlier);
        one.join().expect("a render thread panicked");
        two.join().expect("a render thread panicked");

        assert_eq!(
            written(&session),
            after_the_later,
            "an earlier-started render landed over a later-started one"
        );

        // And in start order: both land, the later one last.
        let third = session.recompile_with(document.clone(), gate.render());
        let fourth = session.recompile_with(document, gate.render());
        let three = std::thread::spawn(third);
        let four = std::thread::spawn(fourth);

        let (c, d) = (gate.entered(), gate.entered());
        let (earlier, later) = (c.min(d), c.max(d));

        gate.release(earlier);
        wait_landed(&session, earlier);
        let after_the_earlier = session.preview().revision();

        gate.release(later);
        three.join().expect("a render thread panicked");
        four.join().expect("a render thread panicked");

        assert_eq!(
            session.preview().landed(),
            later,
            "the later-started render did not land behind the earlier one"
        );
        assert_eq!(
            session.preview().revision(),
            after_the_earlier + 1,
            "the later-started render was dropped when it arrived in start order"
        );
    }

    /// `on_change_with` walks the disk in its first lock scope, and re-checks
    /// its own guard in its second.
    ///
    /// It is the branch the two counters exist for: the listing is refreshed
    /// before the render rather than after it, and a pane that moved while the
    /// render ran drops the absorb without announcing.
    #[test]
    fn the_listing_lands_before_the_render_and_a_moved_pane_drops_the_absorb() {
        let dir = scratch_dir("a-change-across-two-scopes");
        let document = multi_file_in(&dir);
        let (mut session, announcements) = counted();
        session.open(document.clone()).unwrap();

        // **Quiesced first**, or the file the walk is to find announces on its
        // own against "nothing is announced" below.
        session.watch = None;

        let root = watch::root(&document);
        std::fs::write(root.join("notes.md"), "# notes\n").unwrap();

        let gate = Gate::new();
        let mut on_change = session.on_change_with(document, gate.render());
        let running = std::thread::spawn(move || {
            on_change(Changed {
                assets: true,
                tree: true,
                ..Changed::default()
            })
        });
        let serial = gate.entered();

        assert!(
            session
                .preview()
                .status()
                .entries
                .iter()
                .any(|entry| entry.path == "notes.md"),
            "the listing was not refreshed until after the render"
        );

        let before = session.preview().revision();
        let announced = announcements.load(Ordering::SeqCst);

        // **By the field, and not through `Session::set_edited`**, whose `load`
        // would compile and bump the revision this case requires unmoved.
        session.preview().hold("sections/introduction.md");

        gate.release(serial);
        running.join().expect("the change thread panicked");

        assert_eq!(
            session.preview().revision(),
            before,
            "a render whose pane had moved on was absorbed"
        );
        assert_eq!(
            announcements.load(Ordering::SeqCst),
            announced,
            "a failed re-check announced"
        );
    }

    /// An answer orphaned by an Open is dropped, and the newly opened document
    /// still compiles.
    ///
    /// **The second half is the important one**: a fix that drops the orphan but
    /// leaves `landed` above `started` passes the first assertion and freezes the
    /// window.
    #[test]
    fn an_answer_orphaned_by_an_open_is_dropped_and_the_new_document_compiles() {
        let dir = scratch_dir("an-answer-orphaned-by-an-open");
        let document = article_in(&dir);
        let (mut session, _) = counted();
        session.open(document.clone()).unwrap();
        session.watch = None;

        let gate = Gate::new();
        let recompile = session.recompile_with(document.clone(), gate.render());
        let running = std::thread::spawn(recompile);
        let orphan = gate.entered();

        // **The same file again.** A different one is dropped by
        // `Preview::current` even where the serial is wrong, so only re-opening
        // makes all three inputs match and the drop attributable to the order.
        session.open(document).unwrap();
        session.watch = None;

        // "Nothing" is a delta against the state the Open left, `open_at` having
        // reset all four of these itself.
        let after_open = written(&session);

        gate.release(orphan);
        running.join().expect("the render thread panicked");

        assert_eq!(
            written(&session),
            after_open,
            "an answer the Open orphaned was written over the document it opened"
        );

        session.preview().compile();
        assert!(
            session.preview().revision() > after_open.0,
            "the newly opened document stopped compiling: `landed` was left above `started`"
        );
    }

    // -- images fetched by URL ------------------------------------------------
    //
    // `mpdf-003` Phase 25's cases 6 to 15. **No case touches the network**: the
    // fetch is a [`Fake`] that counts its calls, serves `tests/fixtures/dot.png`
    // and can be held open, and every URL is on a reserved `.example` host. A
    // case that needs a render held open passes a [`Gate`] as the worker's
    // render, the seam [`Session::new`] takes. A case that waits for the settle
    // waits [`remote::SETTLE`] plus the suite's usual margin.

    /// The one URL most cases name, and the site it is on.
    const IMAGE: &str = "http://images.example/dot.png";
    const SITE: &str = "images.example";

    /// A fetch that records every URL it is asked for, answers from a table —
    /// `dot.png`'s bytes unless a case says otherwise — and can be held open on
    /// a [`Condvar`] until a case lets it go.
    /// What the fake answers for each URL a case has told it about.
    type Answers = Arc<Mutex<std::collections::HashMap<String, Result<Vec<u8>, String>>>>;

    #[derive(Clone)]
    struct Fake {
        calls: Arc<Mutex<Vec<String>>>,
        answers: Answers,
        held: Arc<(Mutex<bool>, Condvar)>,
    }

    impl Fake {
        fn new() -> Self {
            Self {
                calls: Arc::default(),
                answers: Arc::default(),
                held: Arc::new((Mutex::new(false), Condvar::new())),
            }
        }

        /// The fetch to hand to [`seamed`]. The call is recorded before it
        /// blocks, so a case can wait for it to have *entered*.
        fn fetch(&self) -> impl Fn(&str) -> Result<Vec<u8>, String> + Send + Sync + 'static {
            let fake = self.clone();
            move |url: &str| {
                fake.calls
                    .lock()
                    .expect("the fake was poisoned")
                    .push(url.to_string());

                let (held, waking) = &*fake.held;
                let mut holding = held.lock().expect("the fake was poisoned");
                while *holding {
                    holding = waking.wait(holding).expect("the fake was poisoned");
                }
                drop(holding);

                fake.answers
                    .lock()
                    .expect("the fake was poisoned")
                    .get(url)
                    .cloned()
                    .unwrap_or_else(|| Ok(std::fs::read(fixture("dot.png")).unwrap()))
            }
        }

        fn calls(&self) -> Vec<String> {
            self.calls.lock().expect("the fake was poisoned").clone()
        }

        fn answer(&self, url: &str, answer: Result<Vec<u8>, String>) {
            self.answers
                .lock()
                .expect("the fake was poisoned")
                .insert(url.to_string(), answer);
        }

        fn hold(&self) {
            *self.held.0.lock().expect("the fake was poisoned") = true;
        }

        fn release(&self) {
            *self.held.0.lock().expect("the fake was poisoned") = false;
            self.held.1.notify_all();
        }

        /// Wait for this many calls to have entered. [`WIRING`]'s bound.
        fn entered(&self, count: usize) -> Vec<String> {
            let deadline = Instant::now() + WIRING + remote::SETTLE;
            while Instant::now() < deadline {
                let calls = self.calls();
                if calls.len() >= count {
                    return calls;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            panic!("{count} fetches never entered: {:?}", self.calls());
        }
    }

    /// A session over its own Application Support directory, with this fake as
    /// its fetch and `render` as the worker's render.
    fn fetching(
        name: &str,
        fake: &Fake,
        render: impl Fn(&Compile) -> Rendered + Send + Sync + 'static,
    ) -> (Session, PathBuf) {
        let support = scratch_dir(&format!("support-{name}"));
        let (session, _) = seamed(document::store_file(&support), fake.fetch(), render);
        (session, support)
    }

    /// A single-file document naming these images by URL, the first at line 3.
    fn web_document_in(dir: &Path, urls: &[&str]) -> PathBuf {
        let document = dir.join("web.md");
        let mut text = "# Images from the web\n".to_string();
        for (n, url) in urls.iter().enumerate() {
            text.push_str(&format!("\n![figure {n}]({url})\n"));
        }
        std::fs::write(&document, text).unwrap();
        document
    }

    /// Remember, before any open, that this document's folder allows `site`.
    fn allow(support: &Path, document: &Path, site: &str) {
        document::write_sites(
            &document::sites_file(support),
            &document::project_root(document),
            &[site.to_string()].into(),
        )
        .unwrap();
    }

    /// Past the settle, by the suite's usual margin: long enough to prove a
    /// fetch that should not happen did not.
    fn past_the_settle() {
        std::thread::sleep(remote::SETTLE + watch::TYPING_DEBOUNCE * 4);
    }

    /// Poll the status until it says what a case waits for. [`WIRING`]'s bound,
    /// plus the settle, which some of these wait out first.
    fn until(session: &Session, why: &str, done: impl Fn(&Status) -> bool) -> Status {
        let deadline = Instant::now() + WIRING + remote::SETTLE;
        loop {
            let status = session.status();
            if done(&status) {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "{why}: state {:?}, error {:?}, web {:?}",
                status.state,
                status.error,
                status.web
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// The line a case expects, spelled out.
    fn line(sentence: &str, action: Option<&str>) -> Option<WebLine> {
        Some(WebLine {
            sentence: sentence.to_string(),
            action: action.map(str::to_string),
        })
    }

    fn drawn(status: &Status) -> bool {
        status.state == State::Current && status.error.is_none() && status.web.is_none()
    }

    /// Case 6. **Nothing is fetched before the press**, however long the window
    /// is left open: the refusal is `core`'s, and the line offers the button.
    #[test]
    fn nothing_is_fetched_before_the_press() {
        let dir = scratch_dir("web-before-the-press");
        let document = web_document_in(&dir, &[IMAGE]);
        let fake = Fake::new();
        let (mut session, _) = fetching("before-the-press", &fake, Compile::run);
        session.open(document).unwrap();
        session.watch = None;

        past_the_settle();

        assert_eq!(fake.calls(), Vec::<String>::new(), "a fetch before the press");
        let status = session.status();
        assert_eq!(
            status.error,
            Some(format!("no image fetched for '{IMAGE}' at line 3"))
        );
        assert_eq!(
            status.web,
            line(
                "1 image on images.example is not fetched.",
                Some("Fetch images from the web")
            )
        );
    }

    /// Case 7. **The press fetches each URL once, remembers the site under the
    /// root, and the page goes current.**
    #[test]
    fn the_press_fetches_each_url_once_remembers_the_site_and_draws() {
        let dir = scratch_dir("web-the-press");
        let second = "http://Images.Example/again.png";
        let document = web_document_in(&dir, &[IMAGE, second, IMAGE]);
        let fake = Fake::new();
        let (mut session, support) = fetching("the-press", &fake, Compile::run);
        session.open(document.clone()).unwrap();
        session.watch = None;

        session.fetch_images().unwrap();
        until(&session, "the press never drew the page", drawn);

        let mut calls = fake.calls();
        calls.sort_unstable();
        assert_eq!(calls, [second, IMAGE], "each URL once");
        assert_eq!(
            document::read_sites(
                &document::sites_file(&support),
                &document::project_root(&document)
            ),
            std::collections::BTreeSet::from([SITE.to_string()]),
            "the site is remembered under the root, lower-cased"
        );

        past_the_settle();
        assert_eq!(fake.calls().len(), 2, "a URL already on the page was fetched again");
    }

    /// Case 8. **Consent carries across launches, and the bytes do not** — and
    /// in between, a project that never allowed the site does not draw it from
    /// memory, which is decision 2's "consent and the bytes held apart".
    #[test]
    fn consent_carries_across_launches_and_the_bytes_do_not() {
        let dir = scratch_dir("web-across-launches");
        let document = web_document_in(&dir, &[IMAGE]);
        let elsewhere = web_document_in(&scratch_dir("web-never-allowed"), &[IMAGE]);

        let fake = Fake::new();
        let (mut session, support) = fetching("across-launches", &fake, Compile::run);
        session.open(document.clone()).unwrap();
        session.watch = None;
        session.fetch_images().unwrap();
        until(&session, "the press never drew the page", drawn);
        assert_eq!(fake.calls(), [IMAGE]);

        // A second project that never allowed the site: the bytes are in
        // memory, and it does not draw them.
        session.open(elsewhere).unwrap();
        session.watch = None;
        let never = session.status();
        assert_eq!(never.state, State::Failed, "a project drew bytes it never allowed");
        assert_eq!(
            never.error,
            Some(format!("no image fetched for '{IMAGE}' at line 3"))
        );
        assert_eq!(
            never.web,
            line(
                "1 image on images.example is not fetched.",
                Some("Fetch images from the web")
            )
        );

        // A second open of the allowed one, in the same session: at once.
        session.open(document.clone()).unwrap();
        session.watch = None;
        assert!(drawn(&session.status()), "a second open did not draw at once");
        assert_eq!(fake.calls(), [IMAGE], "a second open fetched again");
        drop(session);

        // A fresh session over the same `sites.json`: no press, one fetch, after
        // the settle.
        let later = Fake::new();
        let (relaunched, _) = seamed(document::store_file(&support), later.fetch(), Compile::run);
        let mut relaunched = relaunched;
        relaunched.open(document).unwrap();
        relaunched.watch = None;
        let opened = relaunched.status();
        assert_eq!(later.calls(), Vec::<String>::new(), "fetched before the settle");
        assert_eq!(opened.error, None, "the refusal of an image on its way was shown");
        assert_eq!(opened.web, None, "a URL waiting out the settle says nothing");

        until(&relaunched, "a relaunch never drew the page", drawn);
        assert_eq!(later.calls(), [IMAGE]);
    }

    /// Case 9. **A URL on a site not allowed asks again**, while its neighbour on
    /// an allowed site is fetched without one.
    #[test]
    fn a_site_not_allowed_asks_again_while_its_neighbour_is_fetched() {
        let dir = scratch_dir("web-asks-again");
        let first = "http://one.example/dot.png";
        let neighbour = "http://one.example/second.png";
        let stranger = "http://two.example/dot.png";
        let document = web_document_in(&dir, &[first]);
        let fake = Fake::new();
        let (mut session, support) = fetching("asks-again", &fake, Compile::run);
        session.open(document.clone()).unwrap();
        session.watch = None;
        session.fetch_images().unwrap();
        until(&session, "the press never drew the page", drawn);

        {
            let mut preview = session.preview();
            preview.edit(format!(
                "# Images\n\n![a]({first})\n\n![b]({neighbour})\n\n![c]({stranger})\n"
            ));
            preview.compile();
        }

        fake.entered(2);
        past_the_settle();
        assert_eq!(fake.calls(), [first, neighbour], "only the allowed site is fetched");

        let status = until(&session, "the neighbour never reached the page", |status| {
            status.error.as_deref() == Some(&*format!("no image fetched for '{stranger}' at line 7"))
        });
        assert_eq!(
            status.web,
            line(
                "1 image on two.example is not fetched.",
                Some("Fetch images from the web")
            )
        );
        assert_eq!(
            document::read_sites(
                &document::sites_file(&support),
                &document::project_root(&document)
            ),
            std::collections::BTreeSet::from(["one.example".to_string()])
        );
    }

    /// Case 10. **The watch loop claims too**: an external write to the master
    /// that names a URL on an allowed site is fetched after the settle, with no
    /// press — through `Session::on_change_with`, the path the draft forgot.
    #[test]
    fn the_watch_loop_claims_too() {
        let dir = scratch_dir("web-the-watch-loop");
        let master = multi_file_in(&dir);
        let fake = Fake::new();
        let (mut session, support) = fetching("the-watch-loop", &fake, Compile::run);
        allow(&support, &master, SITE);
        session.open(master.clone()).unwrap();
        session.set_edited("sections/introduction.md".to_string()).unwrap();
        session.watch = None;

        let text = std::fs::read_to_string(&master).unwrap();
        std::fs::write(&master, format!("{text}\n![from the web]({IMAGE})\n")).unwrap();

        let edited = session.preview().document().unwrap();
        let mut on_change = session.on_change_with(edited, Compile::run);
        on_change(Changed {
            document: true,
            ..Changed::default()
        });
        assert_eq!(fake.calls(), Vec::<String>::new(), "fetched inside the settle");

        until(&session, "the watch loop's URL never reached the page", drawn);
        assert_eq!(fake.calls(), [IMAGE]);
    }

    /// Case 11. **The settle drops a URL the text stopped naming**: two edits
    /// inside it make one call, for the second.
    ///
    /// **Each edit is compiled, and has landed, before the next is made** —
    /// through `Preview::edit` and `Preview::compile` under the lock rather than
    /// the typing debounce, which would fold the two into one compile and let
    /// this case pass with no settle at all. Under the lock, too, the first
    /// URL's settle cannot end between the second compile's plan and its absorb.
    #[test]
    fn the_settle_drops_a_url_the_text_stopped_naming() {
        let dir = scratch_dir("web-the-settle");
        let document = web_document_in(&dir, &[]);
        let (typed, kept) = ("http://images.example/a.png", "http://images.example/b.png");
        let fake = Fake::new();
        let (mut session, support) = fetching("the-settle", &fake, Compile::run);
        allow(&support, &document, SITE);
        session.open(document).unwrap();
        session.watch = None;

        for url in [typed, kept] {
            let mut preview = session.preview();
            preview.edit(format!("# Images\n\n![figure]({url})\n"));
            preview.compile();
            assert_eq!(preview.urls(), [url], "the edit's compile has not landed");
        }

        fake.entered(1);
        past_the_settle();
        assert_eq!(fake.calls(), [kept], "a URL the text stopped naming was fetched");
        assert_eq!(session.preview().web().stage(typed), None);
    }

    /// Case 12. **While an image is on its way the line says so, and nothing
    /// contradicts it** — until the compile that read the bytes has landed.
    #[test]
    fn the_line_says_fetching_until_the_page_it_was_for_has_landed() {
        let dir = scratch_dir("web-on-its-way");
        let document = web_document_in(&dir, &[IMAGE]);
        let fake = Fake::new();
        let gate = Gate::new();
        let (mut session, _) = fetching("on-its-way", &fake, gate.render());
        session.open(document).unwrap();
        session.watch = None;

        fake.hold();
        session.fetch_images().unwrap();
        // The press's own compile, which reads nothing new.
        let pressed = gate.entered();
        gate.release(pressed);
        wait_landed(&session, pressed);
        fake.entered(1);

        let fetching = session.status();
        assert_eq!(
            fetching.web,
            line("Fetching 1 image from images.example…", None)
        );
        assert_eq!(fetching.error, None, "the refusal was shown beside the line");

        // Back, and the compile that reads it held open.
        fake.release();
        let reading = gate.entered();
        let arrived = session.status();
        assert_eq!(session.preview().web().stage(IMAGE), Some("arrived"));
        assert_eq!(arrived.web, line("Fetching 1 image from images.example…", None));
        assert_eq!(arrived.error, None);
        assert_ne!(arrived.state, State::Current);

        gate.release(reading);
        wait_landed(&session, reading);
        assert!(drawn(&session.status()), "the page never went current");
    }

    /// Case 13. **A retry's line survives an older plan.** A compile that read
    /// the failure and absorbs after the retry has landed leaves *"Fetching"* up
    /// and no error — the failure's own *"cannot fetch"* included.
    #[test]
    fn a_retry_survives_an_older_plan_that_read_the_failure() {
        let dir = scratch_dir("web-a-retry");
        let document = web_document_in(&dir, &[IMAGE]);
        let fake = Fake::new();
        fake.answer(IMAGE, Err("503 Service Unavailable".to_string()));
        let gate = Gate::new();
        let (mut session, _) = fetching("a-retry", &fake, gate.render());
        session.open(document.clone()).unwrap();
        session.watch = None;

        // The press and the failed fetch each compile; let both land.
        session.fetch_images().unwrap();
        let (a, b) = (gate.entered(), gate.entered());
        gate.release(a);
        gate.release(b);
        wait_landed(&session, a.max(b));
        let failed = session.status();
        assert_eq!(
            failed.error,
            Some(format!(
                "cannot fetch {IMAGE} for the image at line 3: 503 Service Unavailable"
            ))
        );
        assert_eq!(failed.web, line("1 image could not be fetched.", Some("Try again")));

        // A plan that reads the failure, held open.
        let older = session.recompile_with(document, gate.render());
        let running = std::thread::spawn(older);
        let stale = gate.entered();

        // Try again, and let the retry land, with its compiles held.
        fake.answer(IMAGE, Ok(std::fs::read(fixture("dot.png")).unwrap()));
        session.fetch_images().unwrap();
        let (c, d) = (gate.entered(), gate.entered());
        assert_eq!(session.preview().web().stage(IMAGE), Some("arrived"));

        gate.release(stale);
        running.join().expect("the older render panicked");
        assert_eq!(session.preview().landed(), stale, "the older plan was not absorbed");
        assert!(
            session
                .preview()
                .error()
                .is_some_and(|error| error.starts_with("cannot fetch")),
            "the older plan should have written the failure it read"
        );
        let retrying = session.status();
        assert_eq!(retrying.web, line("Fetching 1 image from images.example…", None));
        assert_eq!(retrying.error, None, "the failure showed beside the retry");

        gate.release(c);
        gate.release(d);
        wait_landed(&session, c.max(d));
        until(&session, "the retry never drew the page", drawn);
        assert_eq!(fake.calls(), [IMAGE, IMAGE]);
    }

    /// Case 14. **A master that stops reading while a fetch is out says
    /// *"cannot read"***: the refused URL is cleared on that write, so the fetch
    /// on its way does not hide it.
    #[test]
    fn a_master_that_stops_reading_while_a_fetch_is_out_says_so() {
        let dir = scratch_dir("web-cannot-read");
        let master = multi_file_in(&dir);
        let text = std::fs::read_to_string(&master).unwrap();
        std::fs::write(&master, format!("{text}\n![from the web]({IMAGE})\n")).unwrap();

        let fake = Fake::new();
        fake.hold();
        let (mut session, support) = fetching("cannot-read", &fake, Compile::run);
        allow(&support, &master, SITE);
        session.open(master.clone()).unwrap();
        session.set_edited("sections/introduction.md".to_string()).unwrap();
        session.watch = None;
        fake.entered(1);
        assert_eq!(session.status().error, None, "the refusal was shown beside the line");

        std::fs::remove_file(&master).unwrap();
        session.preview().compile();

        let error = session.status().error;
        assert!(
            error
                .as_deref()
                .is_some_and(|error| error.starts_with("cannot read") && error.contains("multi_file.md")),
            "{error:?}"
        );
        assert_eq!(session.preview().refused(), None);
        fake.release();
    }

    /// Case 15. **`edit` returns while the fetch is held, and while the fetch's
    /// own render is held** — `mpdf-003` Phase 22's property, over the new
    /// source of work. A deadline rather than a join, for
    /// `a_keystroke_lands_while_a_compile_is_in_flight`'s reason.
    #[test]
    fn a_keystroke_lands_while_a_fetch_and_its_render_are_held() {
        let dir = scratch_dir("web-a-keystroke");
        let document = web_document_in(&dir, &[IMAGE]);
        let fake = Fake::new();
        let gate = Gate::new();
        let (mut session, _) = fetching("a-keystroke", &fake, gate.render());
        session.open(document).unwrap();
        session.watch = None;
        let session = Arc::new(session);

        fake.hold();
        session.fetch_images().unwrap();
        let pressed = gate.entered();
        gate.release(pressed);
        fake.entered(1);

        let type_one = |text: &'static str| {
            let (finished, typed) = mpsc::channel();
            let typing = Arc::clone(&session);
            std::thread::spawn(move || {
                typing.edit(text.to_string());
                let _ = finished.send(());
            });
            typed.recv_timeout(WIRING)
        };

        let during_the_fetch = type_one("# typed while the fetch was out\n");
        fake.release();
        during_the_fetch.expect("the keystroke waited on the fetch");

        let reading = gate.entered();
        let during_the_render = type_one("# typed while its render was held\n");
        gate.release(reading);
        during_the_render.expect("the keystroke waited on the fetch's render");
    }
}
