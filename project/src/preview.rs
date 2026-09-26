//! What the pane is showing, and every rule that decides it.
//!
//! [`Preview`] is the state a compile writes — the text the pane holds, the
//! last good PDF bytes, how long they took, the asset list, whether the page
//! still belongs to that text, and the error when there is one — and since
//! `ltr-001` Phase 1 it is also where the refusals, the receipts and the
//! counters the window reads are decided. It came out of `app/src/preview.rs`
//! so the web session answers through the same code: **every sentence the
//! window shows and every counter it reads is produced here**, except the tail
//! an I/O error's own `Display` contributes, which is the [`Files`]
//! implementation's.
//!
//! **What stayed behind is time and threads.** The desktop's `Session` owns the
//! watch, the typing debounce, the fetch worker and its claim channel, and the
//! clock; it calls in here under its lock and announces, arms or compiles off
//! the lock afterwards, on the outcome each method returns. A host that has none
//! of those — the browser's worker — calls the same methods in order.
//!
//! **The buffer is what compiles.** The file beside it need never have held
//! that text, and the two are compared rather than conflated: [`external_change`]
//! is the whole of what an event naming the open document now means.

use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::document;
use crate::files::Files;
use crate::remote::{self, Web};

/// What the window says about the last compile.
///
/// Four states, and the app held one bit until this became four. What separates
/// *stale* from *failed* is whether there are bytes to keep, because
/// [`Preview::compile`] sets the stale mark on **every** failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// No document has been opened. The app launches into this and holds it
    /// until the first Open.
    Empty,
    /// The last compile succeeded, and the page belongs to it.
    Current,
    /// The last compile failed, and an older page is still drawn.
    Stale,
    /// The last compile failed with no page to keep — the open that never
    /// compiled.
    Failed,
}

/// Which palette the window wears, as the author asked for it.
///
/// **Three states, and [`Appearance::System`] is one of them rather than the
/// absence of the other two.** Following the system is what this app did before
/// there was a choice, and a control that could not get back to it would be a
/// regression on a machine that switches at sunset.
///
/// It is spelled the way [`State`] is, so the page reads one convention off the
/// status and not two. **Nothing about the document is here**: the page Typst
/// compiles is white in either palette, which is why `--paper` does not move,
/// and `specs/desktop_app_spec.md` §1.1 draws that line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    /// Follow `prefers-color-scheme`, which is what the page does with no
    /// `data-theme` attribute on it at all.
    #[default]
    System,
    /// Light, whatever the system says.
    Light,
    /// Dark, whatever the system says.
    Dark,
}

/// What an external change to the open document did.
///
/// The three are exhaustive over three strings, and they need no dirty flag:
/// the file equal to the buffer is decided first, whatever the last-saved text
/// is, and the rest splits on whether the buffer holds unsaved edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum External {
    /// The file already says what the buffer says — the app's own save
    /// arriving back, or a change that changed nothing.
    Unchanged,
    /// The buffer was clean, so nothing could be lost. The disk copy is the
    /// pane's text now. **This is the loop the app has shipped since Phase 2.**
    Taken,
    /// The buffer held unsaved edits and the disk moved under them. The work
    /// is kept and the divergence is named.
    Diverged,
}

/// The report a refused external change leaves for the author.
///
/// It names both ways out and takes neither: saving overwrites the disk,
/// reopening takes it. **The app does not merge** — a three-way merge is an
/// editor project, and this one is not that.
pub const DIVERGED: &str = "this file changed on disk, and the pane holds unsaved edits. \
    Save to write the pane over the file, or open the file again to take it.";

/// The report a refused *switch* leaves for the author.
///
/// **Its own sentence, and not [`DIVERGED`]'s.** That one opens *"this file
/// changed on disk"*, which is false on this occasion — nothing moved, the
/// author asked to put another file in the pane — so reusing the constant would
/// put a lie in the window. The shape is the same and deliberately so: name both
/// ways out, and take neither.
///
/// Both ride `Preview::divergence`, whose meaning is therefore *a refused
/// change* rather than *a refused external change*. **One field means one
/// occasion at a time**: a switch refused while a real divergence stands
/// overwrites this sentence and is overwritten by the next, which costs nothing
/// — the two name the same two exits, and both are cleared by the same two
/// actions.
pub const SWITCHING: &str = "the pane holds unsaved edits, so it is still holding this file. \
    Save to keep them, or discard them to open the other file.";

/// The report a refused *delete* leaves for the author.
///
/// **Its own sentence, and not [`SWITCHING`]'s.** That one closes *"discard
/// them to open the other file"*, and **nothing is being opened by a delete** —
/// so reusing it would put a lie in the window, verbatim the argument that made
/// `SWITCHING` not [`DIVERGED`]. The shape is the same for the third time: name
/// both ways out, and take neither.
///
/// It rides `Preview::divergence` with the other two, so one refusal does not
/// arrive in the window twice.
pub const TRASHING: &str = "the pane holds unsaved edits, and this is the file it is holding. \
    Save to keep them, or discard them to move it to the Trash.";

/// The receipt a plain save leaves in the bar.
///
/// **No path, because the bar already carries one.** `⌘S` writes the file
/// `Status::edited` names two cells to the left, so a receipt spelling it again
/// would repeat its neighbour. [`Session::save_as`] composes the other sentence,
/// which does carry a path, because that gesture's whole question is *where*.
///
/// **Transient, and it rides the command's return rather than [`Status`].** It
/// is an event and not state: a field would re-arrive on every render, need
/// clearing, and cost the page's typedef block a property null in almost every
/// status. `app/dist/index.html` owns the timer and nothing else.
///
/// **It is composed here and not in the command**, which is forced rather than
/// stylistic: `app/src/main.rs` has no test module — the crate is bin-only and
/// `tauri::State` has a private field and no public constructor — so a sentence
/// written in the command is a sentence no test in this repository can reach,
/// which `crate::document::asset_bytes`'s own comment records for a different
/// rule. `mpdf-003` Phase 19.
pub const SAVED: &str = "saved";

/// OQ-5's rule: what an event naming the open document means.
///
/// Three strings and two comparisons. `file` is what the disk holds now,
/// `buffer` is what the pane holds, and `saved` is the text as it stood at the
/// last open or save.
///
/// **Refusing every external change would have been the wrong answer**, and
/// the condition is what makes this one rule rather than a compromise: an
/// author who is not typing has a clean buffer, so a save in another editor
/// still redraws the page with no action taken in the window, which is the
/// loop Phase 2 shipped and the README documents.
///
/// Two limits it accepts. An author who keeps typing between a save and that
/// save's event lands in [`External::Diverged`], so the app can name a
/// divergence that was really its own write — it loses nothing, and the next
/// save clears it. And an external writer that happens to write exactly the
/// author's unsaved text takes [`External::Unchanged`], which leaves the
/// last-saved text unrefreshed. Both err toward keeping work.
pub fn external_change(file: &str, buffer: &str, saved: &str) -> External {
    if file == buffer {
        External::Unchanged
    } else if buffer == saved {
        External::Taken
    } else {
        External::Diverged
    }
}

/// The status line, as a value rather than as chrome.
///
/// Every word in it is chosen here and the page only places it. A window that
/// worded its own status would be checkable by eye alone, and the spec keeps
/// that list to the one claim no test can hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Status {
    /// Which of the four states the pane is in.
    pub state: State,
    /// How long the compile that produced the drawn page took, worded for the
    /// window: `"28 ms"`. `None` when no page is drawn.
    pub time: Option<String>,
    /// The message from the last compile, if it failed.
    pub error: Option<String>,
    /// Is a page drawn under the message?
    pub page: bool,
    /// The report a refused external change left, if there is one.
    ///
    /// **A divergence is not [`State::Stale`]**: nothing failed to compile,
    /// and the page on screen belongs to the text in the pane. It is the file
    /// that has gone elsewhere.
    pub divergence: Option<String>,
    /// How many times a compile has succeeded, so the page can tell new bytes
    /// from a status that merely arrived.
    ///
    /// It is what stops a signal carrying no new page from redrawing the frame
    /// and throwing the reader back to page 1 — which the app's own save now
    /// does, since its event compiles nothing.
    pub revision: u64,
    /// How many times the buffer has been replaced from disk.
    ///
    /// The page re-reads the text on this and on nothing else, so a keystroke
    /// in flight can never lose a race with a fetch of text it just sent.
    pub reloaded: u64,
    /// Where each heading landed in the page the pane is showing.
    ///
    /// The page picks the last one at or above its caret and opens the frame
    /// there. It rides the status because the status is already fetched on the
    /// path that draws, so this needs no command of its own.
    pub anchors: Vec<document::Anchor>,
    /// The project's files, in the order the panel draws them.
    ///
    /// It rides the status for the reason the anchors do: the status is already
    /// fetched on the path that draws, so the panel needs no command of its own.
    /// Empty exactly when no document is open.
    pub entries: Vec<document::Entry>,
    /// Which of them compiles, root-relative with `/` separators.
    ///
    /// **Spelled the way an entry is**, and not as the bare file *name* this
    /// field carried while the panel listed one document's parts — the page has
    /// to match it against a row to mark it, and two files of that name in
    /// different folders must not both light up.
    pub main: Option<String>,
    /// Which of them the pane is holding, spelled the same way.
    ///
    /// It rides beside [`Status::main`] because **the page cannot derive one
    /// from the other**: they are equal at every open and differ the moment a
    /// row is clicked, and the panel draws a mark for each. Equal to `main`
    /// exactly while the pane holds the file that compiles.
    pub edited: Option<String>,
    /// Which palette the window is wearing.
    ///
    /// **It is not about the last compile, and it rides here anyway** — as
    /// [`Status::entries`], [`Status::main`] and [`Status::edited`] do, and for
    /// their reason: the status is already fetched on the path that draws, so a
    /// field the footer places needs no command of its own to arrive on.
    ///
    /// **[`Preview`] does not know it.** Only [`Session::status`] fills this
    /// with the value the author chose; [`Preview::status`] fills
    /// [`Appearance::System`] and says so where it does it.
    pub appearance: Appearance,
    /// What the page says about the images the document names by URL, and
    /// the button beside it. `None` when there is nothing to say, which
    /// includes a URL only waiting out [`remote::SETTLE`].
    ///
    /// **Worded in Rust and only placed by the page**, as the status line is.
    /// `mpdf-003` Phase 25.
    pub web: Option<remote::WebLine>,
}

/// One compile's three inputs, owned, the order it started in, and the images
/// fetched by URL that it may read.
///
/// **Nothing that runs it borrows a [`Preview`]**, and that is the structural
/// half of `mpdf-003` Phase 22: [`Compile::render`] cannot hold the state lock
/// because it cannot reach the state the lock guards. [`Preview::plan`] builds
/// one under the lock, the lock is dropped, the render runs, and
/// [`Preview::absorb`] takes the answer back under it — so a keystroke arriving
/// mid-compile waits on nothing.
///
/// **The files are not in here, and the clock is not either**, since `ltr-001`
/// Phase 1. The render is handed the project's [`Files`] by whoever runs it —
/// the desktop clones its disk handle out from under the lock — and the host
/// times it, since `std::time::Instant::now` panics where the browser runs this.
///
/// **No derived `PartialEq`.** Two of these fields are compared and two are not:
/// [`Preview::current`] tests the three inputs and the serial answers a
/// different question entirely.
pub struct Compile {
    /// The file that compiles, root-relative — [`Preview::main`]'s.
    main: String,
    /// The file the pane holds, whose text the buffer below stands in for.
    edited: String,
    /// The pane's text as it stood when this compile was planned.
    buffer: String,
    /// Which compile of this [`Preview`] this is, from [`Preview::started`].
    serial: u64,
    /// The fetches that came back, on sites the open project allows. The bytes
    /// are shared, so a plan copies no image.
    fetched: remote::Fetched,
    /// Which landing of each of those this compile read, for
    /// [`Preview::absorb`] to promote exactly those and no newer one.
    read: Vec<(String, u64)>,
}

impl Compile {
    /// Which compile this is, in the order they started.
    pub fn serial(&self) -> u64 {
        self.serial
    }

    /// Compile. **This is the whole of what runs outside the lock**, and it
    /// is untimed: the host measures it and hands [`Preview::absorb`] the
    /// duration.
    pub fn render(&self, files: &impl Files) -> Result<document::Render, String> {
        document::render_project(files, &self.main, &self.edited, &self.buffer, &self.fetched)
    }
}

/// How a host times one compile: it runs the closure and answers how long that
/// took. The desktop's reads `std::time::Instant`; the browser's reads
/// `performance.now()`; [`untimed`] is the default and reads nothing.
pub type Timer = fn(&mut dyn FnMut()) -> Duration;

/// The timer a [`Preview`] has until its host gives it one: it runs the compile
/// and answers zero.
pub fn untimed(run: &mut dyn FnMut()) -> Duration {
    run();
    Duration::ZERO
}

/// Where a URL the text newly names goes, if anywhere: the host's fetch worker.
pub type OnClaim = Box<dyn Fn(String) + Send>;

/// What a gesture the pane's unsaved work can block came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// The pane holds unsaved edits: the divergence sentence is set, nothing
    /// moved, and the host announces so the window draws it.
    Refused,
    /// Nothing stood in the way.
    Granted,
}

/// What a delete came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trashed {
    /// The file is the one in the pane and the pane holds unsaved edits.
    Refused,
    /// The file is gone. `holding` says whether the pane was holding it, in
    /// which case it now holds the main.
    Removed { holding: bool },
}

/// What a Save-as came to: whether the pane followed the write, and the
/// receipt the window shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedAs {
    pub moved: bool,
    pub receipt: String,
}

/// The pane's state, as Rust holds it.
///
/// The bytes live here rather than only in the page, because the loop is what
/// compiled them and the export has to write the same bytes the pane is
/// showing — a file and a page that disagree would be worse than neither.
///
/// **The text lives here too**, for the same kind of reason: the rule that
/// decides what an external change does is three comparisons over strings, and
/// a buffer that lived only in the page would put that rule in the window,
/// where no test could reach it.
pub struct Preview<F> {
    /// The project the panel lists: its files, which are its root.
    ///
    /// **It does not move when a row is clicked.** The desktop's `Session::open`
    /// re-roots the watch on every open, so a click that re-rooted would strand
    /// the author below their own project with no way back up. The project
    /// changes on an explicit Open and at no other time — [`Preview::open`].
    files: Option<F>,
    /// Which file under the root compiles, root-relative with `/` separators.
    main: Option<String>,
    /// Which file the pane holds and `⌘S` writes.
    ///
    /// Equal to [`Preview::main`] resolved against the root at every open, and
    /// free to differ from it from the first row click on. **`main` is what
    /// compiles and this is what is edited**: [`Preview::compile`] reads the
    /// first and [`Preview::save`], [`Preview::load`] and [`Preview::reload`]
    /// the second.
    ///
    /// Root-relative, spelled as [`Preview::main`] is, since `ltr-001` Phase 1:
    /// the project names its own root, so the pane can hold nothing outside it
    /// by construction.
    edited: Option<String>,
    /// The files under the root, as the last walk of the disk found them.
    ///
    /// **Refreshed on two occasions only** — an open, and a `Change::Tree`
    /// event — and never recomputed in [`Preview::status`], which the page calls
    /// on every render and which would then walk the disk on every keystroke.
    /// The marked-missing rows are not in here: they come off the text in
    /// [`Preview::status`], which is why the disk half of the panel is stable
    /// and only the missing half moves while a marker is half-typed.
    tree: Vec<document::Entry>,
    buffer: String,
    saved: String,
    assets: Vec<String>,
    sections: Vec<String>,
    pdf: Option<Vec<u8>>,
    anchors: Vec<document::Anchor>,
    elapsed: Option<Duration>,
    revision: u64,
    reloaded: u64,
    /// How many compiles have been planned, and how many have landed.
    ///
    /// **[`Preview::plan`] stamps the first onto a [`Compile`] and
    /// [`Preview::absorb`] advances the second**, and together they are what
    /// decides between two renders in flight: the newest-*started* one wins, and
    /// an answer older than the one already written is dropped. Start order is
    /// the right order because every change to a file a compile reads is
    /// followed by its own filesystem event, and that event schedules a render
    /// which starts *after* the change — so the newest-started render that lands
    /// is never the one missing a change that preceded it.
    ///
    /// **They are the one pair the desktop's `Session::open_at` must carry across the
    /// `Preview` it replaces**, where `revision` and `reloaded` are deliberately
    /// reset: an Open drops both loops without joining their threads, so a render
    /// planned before it can still be waiting on the lock, and zeroing these
    /// would let that orphan win. `mpdf-003` Phase 22.
    started: u64,
    landed: u64,
    stale: bool,
    error: Option<String>,
    divergence: Option<String>,
    /// The images the document names by URL, off the last walk that answered.
    urls: Vec<String>,
    /// The URL the last compile was refused on, if it was one.
    ///
    /// **Cleared on every error written that did not come from a render** —
    /// [`Preview::absorb`]'s early `Err` arm and [`Preview::load`]'s — or a stale
    /// one would hide a *"cannot read"* about the master behind a fetch.
    refused: Option<String>,
    /// The sites the open project allows, and every URL asked about since
    /// launch. **Carried across the desktop's `Session::open_at`'s rebuild**, with only the
    /// sites replaced: the bytes are the process's, consent is the folder's.
    web: Web,
    /// Where a claim goes: the host's fetch worker, or nowhere.
    ///
    /// **A `Preview` with none claims nothing and marks nothing**: a URL marked
    /// waiting with nobody to take it would hide its refusal for good. The
    /// desktop's owns a sender on its worker's channel, so a dropped session
    /// drops it with the `Preview` and the worker's `recv` ends; the browser's
    /// session has no worker and sets none.
    on_claim: Option<OnClaim>,
    /// How the host times a compile this struct runs itself.
    timer: Timer,
}

impl<F> Default for Preview<F> {
    fn default() -> Self {
        Self {
            files: None,
            main: None,
            edited: None,
            tree: Vec::new(),
            buffer: String::new(),
            saved: String::new(),
            assets: Vec::new(),
            sections: Vec::new(),
            pdf: None,
            anchors: Vec::new(),
            elapsed: None,
            revision: 0,
            reloaded: 0,
            started: 0,
            landed: 0,
            stale: false,
            error: None,
            divergence: None,
            urls: Vec::new(),
            refused: None,
            web: Web::default(),
            on_claim: None,
            timer: untimed,
        }
    }
}

/// The sentence every command that needs an open project refuses with.
fn nothing_open() -> String {
    "no document is open".to_string()
}

impl<F: Files> Preview<F> {
    /// A preview with nothing open, timed by `timer`, claiming URLs through
    /// `on_claim` when it has one.
    pub fn new(timer: Timer, on_claim: Option<OnClaim>) -> Self {
        Self {
            timer,
            on_claim,
            ..Self::default()
        }
    }

    /// The last good bytes, whether or not they are still current.
    pub fn pdf(&self) -> Option<&[u8]> {
        self.pdf.as_deref()
    }

    /// The project the panel is listing, if one is open.
    pub fn files(&self) -> Option<&F> {
        self.files.as_ref()
    }

    /// The same, to write through.
    pub fn files_mut(&mut self) -> Option<&mut F> {
        self.files.as_mut()
    }

    /// Which file compiles, root-relative.
    pub fn main(&self) -> Option<&str> {
        self.main.as_deref()
    }

    /// Which file the pane holds, root-relative.
    pub fn edited(&self) -> Option<&str> {
        self.edited.as_deref()
    }

    /// The text the pane holds, which is the text that compiles.
    pub fn text(&self) -> &str {
        &self.buffer
    }

    /// The text as it stood at the last open or save.
    pub fn saved(&self) -> &str {
        &self.saved
    }

    /// Does the page belong to older text than the file on disk?
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// The message from the last compile, if it failed.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Every path the last compile that answered names: the host's watch
    /// filter.
    pub fn assets(&self) -> &[String] {
        &self.assets
    }

    /// The sections the master names, off the last compile.
    pub fn sections(&self) -> &[String] {
        &self.sections
    }

    /// Where the drawn page's headings landed.
    pub fn anchors(&self) -> &[document::Anchor] {
        &self.anchors
    }

    /// The images the document names by URL.
    pub fn urls(&self) -> &[String] {
        &self.urls
    }

    /// The URL the last compile was refused on, if it was one.
    pub fn refused(&self) -> Option<&str> {
        self.refused.as_deref()
    }

    /// How many compiles have produced a page.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// How many compiles have been planned.
    pub fn started(&self) -> u64 {
        self.started
    }

    /// The serial of the newest compile written.
    pub fn landed(&self) -> u64 {
        self.landed
    }

    /// The images fetched by URL, and the sites allowed.
    pub fn web(&self) -> &Web {
        &self.web
    }

    /// The same, for the host's fetch worker to move a URL along.
    pub fn web_mut(&mut self) -> &mut Web {
        &mut self.web
    }

    /// Replace the refresh of the panel's stored half: the host saw the tree
    /// move.
    pub fn refresh_tree(&mut self) {
        if let Some(files) = &self.files {
            self.tree = document::files_under(files);
        }
    }

    /// Which of the four states the pane is in.
    ///
    /// *Empty* is exactly "no document has been opened", and that is the right
    /// boundary rather than one more condition: [`Preview::compile`] returns
    /// early with no document, and [`Session::open`] sets the document and
    /// compiles inside one lock scope, so no observable state sits between
    /// [`Preview::default`] and the first outcome.
    ///
    /// The last arm also absorbs the pair that enumeration leaves — a document
    /// set with no bytes and no failure — and calling it *failed* is the safe
    /// direction, because a *failed* pane refuses an export.
    pub fn state(&self) -> State {
        match (self.edited.is_some(), self.stale, self.pdf.is_some()) {
            (false, _, _) => State::Empty,
            (true, false, true) => State::Current,
            (true, true, true) => State::Stale,
            (true, _, false) => State::Failed,
        }
    }

    /// Everything the window says about the last compile, in one value.
    ///
    /// **The panel's two halves are put together here and nothing is read off
    /// the disk.** `Preview::tree` is the walk, taken at an open and when the
    /// host says the tree moved; the marked-missing rows come off `sections`, which
    /// every compile assigns from the master's own text. So a keystroke that
    /// half-types a marker moves one row and walks no directory, which is what
    /// makes this cheap enough to call on every render.
    ///
    /// **The error is left out exactly while it is `core`'s refusal of an image
    /// that is on its way** — waiting, being fetched, or back and not yet
    /// compiled, on a site the open project allows. The line says so, and
    /// *"no image fetched"* beside *"Fetching"* would be two sentences
    /// contradicting each other. Every other error shows as it always has: a
    /// URL on a site not allowed, which is the refusal the button sits beside;
    /// a fetch that landed and failed; and a URL left on its way by a project
    /// that has since closed. `mpdf-003` Phase 25.
    pub fn status(&self) -> Status {
        let hidden = self
            .refused
            .as_deref()
            .is_some_and(|url| self.web.on_its_way(url));
        Status {
            state: self.state(),
            time: self.elapsed.map(|took| format!("{} ms", took.as_millis())),
            error: if hidden { None } else { self.error.clone() },
            page: self.pdf.is_some(),
            divergence: self.divergence.clone(),
            revision: self.revision,
            reloaded: self.reloaded,
            anchors: self.anchors.clone(),
            entries: self.entries(),
            main: self.main.clone(),
            edited: self.edited.clone(),
            // **A `Preview` does not know the appearance**, which the host
            // holds beside its settings because it is global and this struct
            // is per-document — an open rebuilds it whole, so a preference
            // kept here would go back to `System` on every `⌘O`. The desktop's
            // `Session::status` is what fills this with the author's choice.
            appearance: Appearance::System,
            web: self.web.line(&self.urls),
        }
    }

    /// The panel's rows: the disk walk, plus the sections the master names that
    /// the walk did not find.
    fn entries(&self) -> Vec<document::Entry> {
        let Some(main) = self.main.as_deref() else {
            return Vec::new();
        };
        let named: Vec<String> = self
            .sections
            .iter()
            .map(|section| document::beside(main, section))
            .collect();
        document::merge(self.tree.clone(), &named)
    }

    /// The bytes an export may write, or why it may not.
    ///
    /// Only a *current* pane has them. **The two refusals are two sentences
    /// because they are two problems**: an *empty* pane holds no bytes at all,
    /// where a *stale* or *failed* one holds bytes that are known to belong to
    /// older text. A caller that reported one for the other would send the
    /// reader looking for the wrong thing.
    pub fn exportable(&self) -> Result<&[u8], String> {
        match (self.state(), self.pdf.as_deref()) {
            (State::Current, Some(pdf)) => Ok(pdf),
            (State::Empty, _) => Err(nothing_open()),
            _ => Err("the last compile failed, so the page is out of date".to_string()),
        }
    }

    /// Where a Save-a-copy dialog opens, or why it does not open at all.
    ///
    /// It refuses before the dialog rather than after it, so a pane that cannot
    /// be exported never asks the user for a path it will not use.
    ///
    /// **It names the file that compiles and not the file in the pane**, which
    /// are two different files since `mpdf-010` Phase 2. The bytes it offers to
    /// write are the master's, so `showcase.pdf` is the honest default where
    /// `mathematics.pdf` would name a section for a PDF holding the whole book.
    ///
    /// **Root-relative**, the main with a `.pdf` extension, which is
    /// `cli/src/main.rs:default_output`'s rule; the desktop joins it onto its
    /// root and a browser offers it as the download's name.
    pub fn export_path(&self) -> Result<String, String> {
        self.exportable()?;
        self.main
            .as_deref()
            .map(|main| Path::new(main).with_extension("pdf").to_string_lossy().into_owned())
            .ok_or_else(nothing_open)
    }

    /// Take the pane's text.
    ///
    /// It compiles nothing. The typing debounce decides when a compile falls
    /// due, because one keystroke is not a document.
    pub fn edit(&mut self, text: String) {
        if self.edited.is_some() {
            self.buffer = text;
        }
    }

    /// Hold a project: these files, this main compiling, this file in the pane,
    /// and these sites allowed. **It reads and compiles nothing**; the caller
    /// follows it with [`Preview::load`].
    ///
    /// **Everything about the previous document goes**, and three things stay.
    ///
    /// **`started` and `landed` survive, and the sentence they state is "an Open
    /// discards every answer in flight".** The desktop drops the old watch and
    /// typing loop without joining their threads, so a render planned before
    /// this open can still be waiting on the lock. Zeroes would let it win on
    /// `plan.serial > landed`, and [`Preview::current`] would not save us for an
    /// open inside the same project with a clean buffer. Worse, it would leave
    /// `landed` hundreds of compiles above `started`, after which every later
    /// compile of the newly opened document is silently dropped. `revision` and
    /// `reloaded` are still reset; those the page resets alongside, in
    /// `clear()`. `mpdf-003` Phase 22.
    ///
    /// **The fetches and the claim survive too**, and the sites are replaced
    /// *before* the load compiles: that compile's absorb claims and its plan
    /// reads against them, so installing them after would let the previous
    /// project's consent decide the first page of this one. `mpdf-003` Phase 25.
    /// The timer is the host's and survives with them.
    pub fn open(&mut self, files: F, main: String, edited: String, allowed: BTreeSet<String>) {
        let started = self.started;
        let mut web = std::mem::take(&mut self.web);
        web.install(allowed);
        let on_claim = self.on_claim.take();
        let timer = self.timer;
        let tree = document::files_under(&files);
        *self = Preview {
            files: Some(files),
            main: Some(main),
            edited: Some(edited),
            tree,
            started,
            landed: started,
            web,
            on_claim,
            timer,
            ..Preview::default()
        };
    }

    /// Read the document from disk into the buffer, and compile it.
    ///
    /// A file that will not read leaves the same message and the same *failed*
    /// state a compile failure leaves, because that is what the author needs
    /// to see either way and it is the sentence the terminal prints.
    pub fn load(&mut self) {
        let (Some(files), Some(document)) = (&self.files, &self.edited) else {
            return;
        };

        match document::read_document(files, document) {
            Ok(text) => {
                self.take(text);
                self.compile();
            }
            Err(message) => {
                self.stale = true;
                self.error = Some(message);
                self.refused = None;
            }
        }
    }

    /// Write the buffer to the open document's path.
    ///
    /// The last-saved text moves with it, which is what makes the buffer clean
    /// again — and what makes this save's own filesystem event take
    /// [`External::Unchanged`] a moment later, with no second compile and no
    /// suppression that would have to win a race.
    ///
    /// It answers the receipt the bar shows, which is [`SAVED`].
    pub fn save(&mut self) -> Result<String, String> {
        let (Some(files), Some(document)) = (&mut self.files, &self.edited) else {
            return Err(nothing_open());
        };

        files
            .write(document, self.buffer.as_bytes())
            .map_err(|e| format!("cannot write {}: {e}", files.locate(document).display()))?;

        self.saved = self.buffer.clone();
        self.divergence = None;
        Ok(SAVED.to_string())
    }

    /// Write the buffer to a path the author picked, and hold that file if it is
    /// one of the project's.
    ///
    /// It answers **where the write landed, and whether the pane followed it
    /// there** — the second half being what [`Session::save_as`] decides its
    /// compile and its re-arm on.
    ///
    /// **The pane follows a save inside the project and does not follow one
    /// outside it**, `mpdf-003` Phase 19. A save outside is a copy: the write
    /// still goes wherever the author pointed it, which Phase 18 settled, but
    /// `edited` never leaves the root. That removes the state Phase 18 could
    /// only mitigate — a pane holding a file with no row and no watch — rather
    /// than living with it.
    ///
    /// **The predicate wants both halves, which is `document::trash_file`'s
    /// recorded shape**: *the name is under the root, and something is at it*.
    /// [`document::spell`] alone is **not** it — it is a component-wise
    /// `strip_prefix`, so `root.join("../escape.md")` strips to `../escape.md`,
    /// answers `Some` and would be judged inside. [`document::confined`] alone
    /// is not it either: under a symlinked root it resolves both sides and says
    /// inside where `spell` cannot produce a spelling at all. So a canonicalized
    /// comparison decides the confinement and `spell` decides the spelling, and
    /// keying the move to `spell` succeeding is what makes
    /// [`Preview::edited_relative`]'s missing fallback unreachable.
    ///
    /// **`confined` is asked *after* the write and that order is forced**: it
    /// opens on `is_file`, so asked before it would answer `None` for every
    /// save-as to a name that does not exist yet. This is a command taking the
    /// page's `path` verbatim, so the standard is [`Session::set_main`]'s and
    /// not the dialog's.
    ///
    /// **Inside, this is unchanged from Phase 17, and the order is the
    /// load-bearing part.** `saved` moves to the buffer *before* `edited` does,
    /// for two reasons that both bite: it is what makes the buffer clean, so
    /// this write's own filesystem event takes [`External::Unchanged`] a moment
    /// later with no second compile; and it is why [`Session::save_as`] cannot
    /// be `save` followed by [`Session::set_edited`], which would meet
    /// `refused_while_dirty` and answer `Ok(())` having moved nothing — a silent
    /// success, on the one gesture an author makes *because* they have unsaved
    /// work.
    ///
    /// **Outside, none of those three moves, and that is the dangerous half.**
    /// Leaving `saved == buffer` while the pane keeps a file whose disk copy is
    /// older would make [`Session::refused_while_dirty`] answer *clean*: the next
    /// row click, `set_main` or trash would [`Preview::load`] over the author's
    /// text with no divergence sentence — and [`external_change`] would fall to
    /// its `buffer == saved` arm and answer [`External::Taken`], taking the disk
    /// copy over that work on any event touching the file. So an outside save
    /// leaves `saved`, `divergence` and `edited` exactly as it found them: the
    /// buffer is still dirty against the file the pane holds, which is the truth.
    ///
    /// **It does not `load`**, where `set_edited` does: the file was just
    /// written from this buffer, so a read would answer the text already held.
    ///
    /// `main` does not follow. The file that compiles and the file that is
    /// edited are two — `mpdf-010` Phase 2 — and a Save-as of a section is not a
    /// claim about which master compiles.
    ///
    /// `mpdf-003` Phase 17, narrowed by Phase 19.
    ///
    /// **Since `ltr-001` Phase 1 the write and the question *inside?* are one
    /// call, [`Files::save_as`]**, and the rest is here: the kind refused before
    /// anything is written, the three moves, the listing refreshed, the compile
    /// when the pane moved, and the receipt — `saved as <name> in <folder>`,
    /// the folder being the containing directory spelled as the dialog spelled
    /// it, not canonicalized and not abbreviated to `~`, since a save outside
    /// the project is a copy and the sentence says where it went.
    pub fn save_as(&mut self, path: &str) -> Result<SavedAs, String> {
        if self.edited.is_none() {
            return Err(nothing_open());
        }
        let Some(files) = &mut self.files else {
            return Err(nothing_open());
        };

        document::creatable(path)?;
        let inside = files
            .save_as(path, self.buffer.as_bytes())
            .map_err(|e| format!("cannot write {}: {e}", Path::new(path).display()))?;

        let moved = inside.is_some();
        if let Some(spelled) = inside {
            self.saved = self.buffer.clone();
            self.divergence = None;
            self.edited = Some(spelled);
        }

        // The listing is refreshed on both paths: a no-op outside, and the
        // same call is what lists the file when it landed inside. The compile
        // is only for the pane that moved — outside it would read exactly what
        // it read before and bump `revision` for nothing.
        self.refresh_tree();
        if moved {
            self.compile();
        }

        let landed = Path::new(path);
        Ok(SavedAs {
            moved,
            receipt: format!(
                "saved as {} in {}",
                landed.file_name().unwrap_or_default().to_string_lossy(),
                landed.parent().unwrap_or(Path::new("")).to_string_lossy()
            ),
        })
    }

    /// The disk moved under the open document: decide what that means.
    ///
    /// This is [`external_change`] with the file read for it and its answer
    /// carried out. A document that will not read at this instant — one caught
    /// mid-write — counts as [`External::Unchanged`]: the app keeps what it
    /// has, and the write's next event decides.
    pub fn reload(&mut self) -> External {
        let (Some(files), Some(document)) = (&self.files, &self.edited) else {
            return External::Unchanged;
        };
        let Ok(file) = document::read_document(files, document) else {
            return External::Unchanged;
        };

        let outcome = external_change(&file, &self.buffer, &self.saved);
        match outcome {
            External::Unchanged => {}
            External::Taken => {
                self.take(file);
                self.compile();
            }
            External::Diverged => self.divergence = Some(DIVERGED.to_string()),
        }
        outcome
    }

    /// Take a text from disk as both the buffer and the last-saved text.
    ///
    /// The count it bumps is how the page knows to re-read: it replaces its
    /// own text on this and on nothing else, so text the author is typing is
    /// never overwritten by a fetch that raced it.
    fn take(&mut self, text: String) {
        self.saved = text.clone();
        self.buffer = text;
        self.reloaded += 1;
        self.divergence = None;
    }

    /// Compile the pane's text and take in what came back.
    ///
    /// The three steps are [`Preview::plan`], [`Compile::run`] and
    /// [`Preview::absorb`], and what each writes is argued where it is written.
    ///
    /// **It compiles [`Preview::main`], not the file in the pane.** The pane's
    /// text reaches it through the closure `document::render_project` builds,
    /// which answers `edited` from this buffer and everything else from the
    /// disk — so the page shows the whole document while the author edits one
    /// file of it, and shows exactly what the pane says while the two are the
    /// same file. A `main` this app cannot read at all leaves the message and
    /// the *failed* state [`Preview::load`] leaves, which is where a document
    /// that will not read has always landed.
    ///
    /// **Split into three since `mpdf-003` Phase 22**, and it stays whole for
    /// its three synchronous callers — [`Preview::load`], [`Preview::reload`]
    /// and [`Preview::save_as`] — each of which is one user action that has just
    /// moved the document wholesale, already holds the desktop's
    /// `Mutex<Session>` for its duration, and may as well hold this one too. The
    /// two closures that fire while a hand is on the keys take the three steps
    /// apart instead, and so does the fetch worker, which fires on the network's
    /// time rather than the author's.
    ///
    /// **The one compile this struct times itself**, through the host's
    /// [`Timer`], since `ltr-001` Phase 1: nothing here may read a clock.
    pub fn compile(&mut self) {
        let Some(plan) = self.plan() else {
            return;
        };
        let timer = self.timer;
        let mut outcome = None;
        let took = {
            let files = self.files.as_ref().expect("a plan is only made with files");
            timer(&mut || outcome = Some(plan.render(files)))
        };
        if let Some(outcome) = outcome {
            self.absorb(&plan, outcome, took);
        }
    }

    /// What the next compile would read, and the number that orders it.
    ///
    /// `None` on the two absences [`Preview::compile`] has always returned early
    /// on. **`&mut` because it stamps the serial**: the count of compiles ever
    /// started is bumped here and nowhere else, and the plan carries the number
    /// out to whichever thread runs it.
    ///
    /// The buffer is cloned, which is this phase's whole per-compile cost: a
    /// copy of the document, four to five orders below the compile it is handed
    /// to. The fetched images are not: their bytes are shared.
    pub fn plan(&mut self) -> Option<Compile> {
        let (main, edited) = (self.main.clone()?, self.edited.clone()?);
        self.files.as_ref()?;
        let (fetched, read) = self.web.finished();

        self.started += 1;
        Some(Compile {
            main,
            edited,
            buffer: self.buffer.clone(),
            serial: self.started,
            fetched,
            read,
        })
    }

    /// Would this plan still read what it read, if it were made now?
    ///
    /// **Derived and not maintained, deliberately.** The alternative was a
    /// serial bumped on every write to `main`, `edited` or `buffer` — cheaper
    /// per compile and wrong by construction, because those inputs are written
    /// in six places ([`Preview::edit`], [`Preview::take`], [`Preview::save_as`],
    /// [`Session::set_edited`] and [`Session::trash`] reaching through this
    /// guard, and [`Session::open_at`] assigning a fresh `Preview`). A counter
    /// over writers is a bump a future author must remember; a comparison is one
    /// they cannot forget.
    ///
    /// It builds no second [`Compile`]: the cost is one `String` equality, and
    /// it stops at the first differing byte.
    fn current(&self, plan: &Compile) -> bool {
        self.main.as_deref() == Some(plan.main.as_str())
            && self.edited.as_deref() == Some(plan.edited.as_str())
            && self.buffer == plan.buffer
    }

    /// Take in what a compile came back with — if it is still wanted.
    ///
    /// A success replaces the bytes and clears both the error and the stale
    /// mark. **A failure keeps the bytes**, records the message and sets the
    /// mark: an author mid-edit passes through broken states constantly, and
    /// blanking the pane on each one would lose their place and make the loop
    /// worse than the command it replaces. The mark is what stops the kept
    /// page from silently claiming to be the current text.
    ///
    /// **The duration and the anchors travel with the bytes**, replaced on a
    /// success and kept on a failure exactly as they are, so the time the window
    /// shows and the page the pane opens on always describe the page on screen
    /// rather than the last attempt at one.
    ///
    /// **Two things decide whether any of that happens, and the order is the
    /// cheap test first.** The serial: a render older than the one already
    /// written is dropped, because start order is the order a filesystem event
    /// puts its own render in. Then the inputs: text that has moved on since the
    /// plan was made is a page of something the author has since changed.
    ///
    /// **Dropping an answer is safe because a fresher one is always already
    /// coming.** Each of the six writers [`Preview::current`] lists is followed
    /// by a compile — [`Preview::edit`] by the typing channel's own nudge, which
    /// a `settle` thread mid-compile buffers and re-touches afterwards;
    /// [`Preview::take`] by the `compile` on the next line of both its callers;
    /// [`Session::set_edited`], [`Session::trash`] and [`Session::discard`] by
    /// [`Preview::load`]; [`Session::open_at`] by replacing this struct whole.
    /// So the page is never left holding bytes with nothing on the way.
    ///
    /// **The guard stands in front of every write, the failed arm included.** A
    /// stale render also carries an asset list and a section list, so a dropped
    /// compile that wrote only its shopping lists would re-arm the watch against
    /// a document that has moved on; and a render that read a file mid-write
    /// returns `Err`, which absorbed out of order would set the mark and the
    /// message over a newer good page. One guard, one `return`, no partial
    /// absorb — and `landed` advances before the outcome is looked at, so an
    /// older answer cannot overwrite a newer failure either.
    ///
    /// `mpdf-003` Phase 22.
    ///
    /// **Since Phase 25 it is also the one place a fetch is claimed.** Every
    /// compile path reaches it — the typing loop, the watch loop, an open, a
    /// reload, a save-as, the fetch worker's own compile — so none of them can
    /// strand a URL the text newly names on an allowed site. And past the guard
    /// it marks what the plan read as on the page, **at the landing it read**,
    /// whatever the outcome: a plan that read a failure and lands after a retry
    /// has landed newer bytes leaves those bytes on their way.
    pub fn absorb(
        &mut self,
        plan: &Compile,
        outcome: Result<document::Render, String>,
        took: Duration,
    ) {
        if plan.serial <= self.landed || !self.current(plan) {
            return;
        }
        self.landed = plan.serial;
        self.web.promote(&plan.read);

        let render = match outcome {
            Ok(render) => render,
            Err(message) => {
                self.stale = true;
                self.error = Some(message);
                self.refused = None;
                return;
            }
        };

        if let Some(assets) = render.assets {
            self.assets = assets;
        }
        self.refused = render.refused;
        if let Some(urls) = render.urls {
            self.urls = urls;
            self.claim();
        }

        // Taken whether or not the compile succeeded, as the asset list above
        // is and for the same reason: it is read off the text rather than the
        // page, so a document that will not compile still names its sections.
        self.sections = render.sections;

        match render.pdf {
            Ok(pdf) => {
                self.pdf = Some(pdf);
                self.anchors = render.anchors;
                self.elapsed = Some(took);
                self.revision += 1;
                self.stale = false;
                self.error = None;
            }
            Err(message) => {
                self.stale = true;
                self.error = Some(message);
            }
        }
    }

    /// Hand the fetch worker every URL the text names on an allowed site that
    /// nothing has asked about yet. Each waits out the host's settle first,
    /// which is what makes a URL edited in place one request and not one per
    /// compile.
    ///
    /// **A `Preview` with no worker claims nothing**, and marks nothing either:
    /// a URL marked waiting with nobody to take it would hide its refusal for
    /// good.
    fn claim(&mut self) {
        let Some(on_claim) = &self.on_claim else {
            return;
        };
        for url in self.web.claim(&self.urls, false) {
            on_claim(url);
        }
    }

    /// Is there unsaved work this gesture would throw away? Then say so and stop.
    ///
    /// **It reports through `Preview::divergence` and not through an `Err`.**
    /// The caller's `Err` is where a path outside the project goes, and the
    /// window draws that in the error bar; a refusal that arrived both ways
    /// would be one problem in two places. This is a status, and the page places
    /// it exactly as it places every other status sentence.
    ///
    /// **The sentence is the caller's**, because the three occasions are three
    /// different claims and each of the others would be false on the other two:
    /// [`SWITCHING`] names an open, [`TRASHING`] names the Trash. The field they
    /// share carries one at a time, which costs nothing — all three name the
    /// same two exits.
    ///
    /// **The host announces on `true`**, because nothing else will: no compile
    /// ran, so the page would otherwise never fetch the status carrying the
    /// sentence.
    pub fn refused_while_dirty(&mut self, sentence: &str) -> bool {
        if self.buffer == self.saved {
            return false;
        }
        self.divergence = Some(sentence.to_string());
        true
    }

    /// May the main become this file? The host remembers it and opens again on
    /// [`Asked::Granted`].
    ///
    /// **Confined, and not merely checked for existence.** The path comes from
    /// the panel, which got it from this app's own listing — but a command is a
    /// command, and `root.join("../../secrets.md")` names a real file on plenty
    /// of machines. [`Files::holds`] is the walk's own test, shared with
    /// [`Preview::set_edited`] and with the figure read.
    ///
    /// It refuses on [`SWITCHING`] while the buffer diverges from the
    /// last-saved text.
    pub fn ask_main(&mut self, main: &str) -> Result<Asked, String> {
        let files = self.files.as_ref().ok_or_else(nothing_open)?;
        if !files.holds(main) {
            return Err(document::not_a_file(main));
        }
        if self.refused_while_dirty(SWITCHING) {
            return Ok(Asked::Refused);
        }
        Ok(Asked::Granted)
    }

    /// Put another of the project's files in the pane, leaving the main alone.
    ///
    /// **This is not an open, and the difference is the counters.**
    /// [`Preview::open`] zeroes `revision` and `reloaded`, and
    /// `app/dist/index.html`'s `clear()` — which resets the counters the page
    /// compares them against — runs on an Open and not on a row click. So this
    /// sets `edited`, reads that file into the buffer, and leaves the project,
    /// the main, the listing, the bytes and both counters exactly as it found
    /// them: they *advance* here, they do not restart.
    ///
    /// It confines the path as [`Preview::ask_main`] does, and refuses on the
    /// same terms while the buffer diverges from the last-saved text.
    pub fn set_edited(&mut self, path: &str) -> Result<Asked, String> {
        let (Some(files), Some(_)) = (&self.files, &self.main) else {
            return Err(nothing_open());
        };
        if !files.holds(path) {
            return Err(document::not_a_file(path));
        }
        if self.refused_while_dirty(SWITCHING) {
            return Ok(Asked::Refused);
        }

        self.edited = Some(path.to_string());
        self.load();
        Ok(Asked::Granted)
    }

    /// Delete one of the project's files.
    ///
    /// **Three refusals, and they do not arrive the same way.** The main and
    /// the two the delete itself makes come back as `Err`, which the page draws
    /// in the error bar exactly as it draws [`Preview::set_edited`]'s — and none
    /// of the three is reachable from a row: the main row draws no button, and
    /// every other row came out of this app's own listing. The dirty-buffer one
    /// rides the divergence and answers [`Trashed::Refused`], as
    /// [`Preview::refused_while_dirty`] does, so one refusal does not arrive in
    /// the window two ways.
    ///
    /// **That one is asked only of the file the pane is holding**: deleting some
    /// *other* file throws no unsaved work away, so refusing there would be a
    /// refusal with nothing behind it.
    ///
    /// **The delete is `remove`**, which is the host's: the desktop moves the
    /// file to the Trash through a call its suite can replace, and the browser
    /// calls [`document::remove_file`]. **The panel is refreshed here and not by
    /// a watch**: this app made the change and knows it, and a deleted section
    /// the master names is in the asset list, so the desktop's watch would never
    /// classify it as a change to the tree. `document::merge` then puts the path
    /// back as `missing: true`, because the master still names it.
    ///
    /// **When the pane held the file it falls back to the main, and loads.**
    /// Moving the pane without loading would leave the buffer holding the
    /// deleted file's text while `edited` names the main, so a save would write
    /// a deleted section over the master. `mpdf-010` Phase 4.
    pub fn trash(
        &mut self,
        path: &str,
        remove: impl FnOnce(&mut F, &str) -> Result<(), String>,
    ) -> Result<Trashed, String> {
        let (Some(_), Some(main)) = (&self.files, self.main.clone()) else {
            return Err(nothing_open());
        };

        if path == main {
            return Err(format!(
                "{path} is the file this project compiles. Set another file as main first."
            ));
        }

        // Only the pane's own file can cost the author anything.
        let holding = self.edited.as_deref() == Some(path);
        if holding && self.refused_while_dirty(TRASHING) {
            return Ok(Trashed::Refused);
        }

        let files = self.files.as_mut().ok_or_else(nothing_open)?;
        remove(files, path)?;
        self.refresh_tree();

        if holding {
            self.edited = Some(main);
            self.load();
        }
        Ok(Trashed::Removed { holding })
    }

    /// Move the pane and read nothing.
    ///
    /// **Not a command**: every gesture that moves the pane goes through
    /// [`Preview::set_edited`], [`Preview::trash`] or [`Preview::save_as`],
    /// each of which reads what it moved to. This is for a host's suite that
    /// has to move the pane *without* the compile a load runs, to catch an
    /// answer in flight landing on a pane that has moved on.
    #[doc(hidden)]
    pub fn hold(&mut self, edited: &str) {
        self.edited = Some(edited.to_string());
    }

    /// Drop what the pane holds and take the file again.
    ///
    /// **The second way out every refusal names.** [`Preview::load`] already
    /// reads the edited file into the buffer and the last-saved text together,
    /// which is exactly "discard"; this is that path behind a command. It clears
    /// the divergence through `take`, so one action answers a refused switch
    /// and a refused external change alike.
    pub fn discard(&mut self) {
        self.load();
    }
}

// A test may read a fixture: `clippy.toml`'s list is about what the crate
// ships, which is what the gate lints.
#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;
    use crate::files::MemFiles;

    /// `ltr-001` Phase 1's own cases: the rules the web session will rely on,
    /// run over memory, where no desktop test exercises them.
    ///
    /// A two-file project — a master naming one section — opened on the master,
    /// which is what an open lands on.
    fn opened() -> Preview<MemFiles> {
        let files = MemFiles::new([
            (
                "book.md".to_string(),
                b"# Book\n\n[](sections/one.md)\n".to_vec(),
            ),
            (
                "sections/one.md".to_string(),
                b"# One\n\nText.\n".to_vec(),
            ),
        ]);
        let mut preview = Preview::default();
        preview.open(
            files,
            "book.md".to_string(),
            "book.md".to_string(),
            BTreeSet::new(),
        );
        preview.load();
        assert_eq!(preview.state(), State::Current, "{:?}", preview.error());
        preview
    }

    fn read(preview: &Preview<MemFiles>, path: &str) -> String {
        document::read_document(preview.files().unwrap(), path).unwrap()
    }

    #[test]
    fn a_switch_over_unsaved_work_is_refused_in_switching_s_words() {
        let mut preview = opened();
        preview.edit("# Book, unsaved\n".to_string());

        assert_eq!(preview.set_edited("sections/one.md"), Ok(Asked::Refused));
        assert_eq!(preview.edited(), Some("book.md"), "the pane moved");
        assert_eq!(preview.status().divergence.as_deref(), Some(SWITCHING));

        assert_eq!(preview.ask_main("sections/one.md"), Ok(Asked::Refused));

        preview.discard();
        assert_eq!(preview.status().divergence, None);
        assert_eq!(preview.set_edited("sections/one.md"), Ok(Asked::Granted));
        assert_eq!(preview.edited(), Some("sections/one.md"));
        assert_eq!(preview.text(), "# One\n\nText.\n");
    }

    #[test]
    fn a_delete_of_the_file_in_the_pane_over_unsaved_work_is_refused_in_trashing_s_words() {
        let mut preview = opened();
        assert_eq!(preview.set_edited("sections/one.md"), Ok(Asked::Granted));
        preview.edit("# One, unsaved\n".to_string());

        let removed = |files: &mut MemFiles, path: &str| document::remove_file(files, path);
        assert_eq!(preview.trash("sections/one.md", removed), Ok(Trashed::Refused));
        assert_eq!(preview.status().divergence.as_deref(), Some(TRASHING));
        assert!(preview.files().unwrap().holds("sections/one.md"), "the file went");

        // Saved, the same delete goes through and the pane falls back to the
        // main — which still names the file, so its row comes back missing.
        preview.save().unwrap();
        assert_eq!(
            preview.trash("sections/one.md", removed),
            Ok(Trashed::Removed { holding: true })
        );
        assert_eq!(preview.edited(), Some("book.md"));
        assert!(!preview.files().unwrap().holds("sections/one.md"));
        let row = preview
            .status()
            .entries
            .into_iter()
            .find(|entry| entry.path == "sections/one.md")
            .expect("the master's missing section has no row");
        assert!(row.missing);

        assert_eq!(
            preview.trash("book.md", removed),
            Err("book.md is the file this project compiles. Set another file as main first.".to_string())
        );
    }

    #[test]
    fn a_save_writes_the_buffer_and_answers_saved() {
        let mut preview = opened();
        preview.edit("# Book, kept\n\n[](sections/one.md)\n".to_string());

        assert_eq!(preview.save(), Ok(SAVED.to_string()));
        assert_eq!(read(&preview, "book.md"), "# Book, kept\n\n[](sections/one.md)\n");
        assert_eq!(read(&preview, "sections/one.md"), "# One\n\nText.\n");
        assert_eq!(preview.saved(), preview.text());

        assert_eq!(Preview::<MemFiles>::default().save(), Err(nothing_open()));
    }

    #[test]
    fn an_export_is_refused_in_one_of_two_sentences() {
        let empty = Preview::<MemFiles>::default();
        assert_eq!(empty.exportable().err().as_deref(), Some("no document is open"));
        assert!(empty.export_path().is_err());

        let mut preview = opened();
        assert_eq!(preview.export_path().as_deref(), Ok("book.pdf"));

        preview.edit("# Broken\n\n<div>raw HTML</div>\n".to_string());
        preview.compile();
        assert_eq!(preview.state(), State::Stale);
        assert_eq!(
            preview.exportable().err().as_deref(),
            Some("the last compile failed, so the page is out of date")
        );
        assert!(preview.export_path().is_err());
    }

    #[test]
    fn revision_rises_on_a_page_and_not_on_a_failure() {
        let mut preview = opened();
        let first = preview.revision();
        assert_eq!(first, 1, "the open's compile drew a page");

        preview.edit("# Broken\n\n<div>raw HTML</div>\n".to_string());
        preview.compile();
        assert_eq!(preview.revision(), first, "a failed compile counted");
        assert!(preview.is_stale());

        preview.edit("# Fixed\n\nText.\n".to_string());
        preview.compile();
        assert_eq!(preview.revision(), first + 1);
        assert_eq!(preview.status().time.as_deref(), Some("0 ms"), "the default timer");
    }

    #[test]
    fn a_path_that_climbs_out_of_the_project_is_refused() {
        let mut preview = opened();

        assert_eq!(
            preview.set_edited("../escape.md"),
            Err("../escape.md is not a file in this project".to_string())
        );
        assert_eq!(
            preview.ask_main("../escape.md"),
            Err("../escape.md is not a file in this project".to_string())
        );
        let files = preview.files_mut().unwrap();
        assert_eq!(
            document::asset_bytes(files, "../escape.png"),
            Err("../escape.png is not a file in this project".to_string())
        );
        assert_eq!(
            document::create_file(files, "../escape.md"),
            Err("../escape.md would land outside this project".to_string())
        );
        assert_eq!(
            document::create_file(files, "newdir/x.md"),
            Err("newdir/x.md would land outside this project".to_string()),
            "a folder is not created by a create"
        );
        assert_eq!(
            document::remove_file(files, "../escape.md"),
            Err("../escape.md is outside this project".to_string())
        );
    }
}
