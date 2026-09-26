//! The rules of one compile and of the project it reads, over any [`Files`].
//!
//! Moved from `app/src/document.rs` by `ltr-001` Phase 1, with what that file
//! decides and none of what it opens: the compile's two read passes and the
//! sentences they refuse in, the panel's entries and their order, the main a
//! project opens on, and the words the three commands that move a path refuse
//! with. `app/src/document.rs` keeps the disk half — the climb to a root, the
//! walk, the Trash and the three files in Application Support — and answers
//! [`Files`] over it.

use std::collections::HashSet;
use std::path::Path;

use md2pdf_core::Asset;
use serde::Serialize;

use crate::files::Files;
use crate::remote::Fetched;

/// One heading, and the page its typeset form landed on.
///
/// This is `md2pdf_core::Anchor` again, and the duplication is deliberate for
/// the same reason [`read_assets_with`] duplicates its counterpart: this one
/// crosses to the page inside `crate::preview::Status`, so it must serialize,
/// and giving `core` a serde dependency for two `usize` fields would widen what
/// the app asks of it well past the one function it gained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Anchor {
    /// The 1-based line of the markdown heading.
    pub line: usize,
    /// The 1-based page its compiled form landed on.
    pub page: usize,
}

/// What one compile produced, and what the document named while producing it.
pub struct Render {
    /// The paths the document names: its sections in the order the master
    /// reads them, then the bibliography it declares, then the images in
    /// reader order.
    ///
    /// **The sections go in first and unconditionally**, and that is the one
    /// thing this list could not be built without.
    /// `md2pdf_core::section_paths` reads the master's own text and cannot
    /// fail, where the other two answer about the assembled document and now
    /// fail with `MissingSection` for a section that does not exist yet — and
    /// `crate::preview::Preview::compile` replaces this list only when it is
    /// `Some`. A list built the way it was built before this phase would stay
    /// empty, the desktop's `watch::classify` would drop the section's creation
    /// event, and the app would never recover.
    ///
    /// **Three answers, and each says a different thing.**
    ///
    /// - `Some(sections ++ bibliography ++ images)` when both walks answer.
    ///   For a document naming no section that is the vector this returned
    ///   before the sections existed, in the same order, with an empty list in
    ///   front of it.
    /// - `Some(sections)` when they do not and the master names any. This
    ///   *replaces* the list with a shorter one, so a multi-file document with
    ///   a missing section stops watching its figures until that section comes
    ///   back. The trade is deliberate: recovering the section beats watching
    ///   figures through a window in which nothing compiles anyway.
    /// - `None` when neither answers and no section is named. The caller keeps
    ///   the list it already had, which is what stops a transient
    ///   out-of-dialect edit from dropping the images the app knows about.
    ///
    /// Either `Some` arrives even when the compile failed, because emission
    /// reads the text and not the disk: a document whose figures are all
    /// missing still names them. That is what keeps the watch filter working
    /// while the compile does not.
    pub assets: Option<Vec<String>>,

    /// The sections the master names, in the order it reads them.
    ///
    /// The same names [`Render::assets`] puts in front of everything else, kept
    /// on their own because the panel names *files* where that list names paths
    /// to watch — and because that list is `None` exactly when the caller must
    /// keep the one it has, which is not a thing a panel can draw.
    ///
    /// Empty for a document that names no section, and empty while the marker
    /// naming them is mid-edit: `md2pdf_core::section_paths` reads the text, so
    /// this says what the buffer names now and not what is on the disk.
    pub sections: Vec<String>,

    /// The bytes, or the sentence the terminal would print.
    pub pdf: Result<Vec<u8>, String>,

    /// Where each heading landed, for the pane to open on.
    ///
    /// **Only the headings written in the file the pane holds**, which is
    /// [`Pane`]'s whole job and is no longer the same file as the one that
    /// compiles. A line means something in exactly one buffer and the pane
    /// holds one, so an anchor from another file is not a worse match — it is
    /// a number about a document the pane is not showing, and
    /// `app/dist/index.html:caretPage` walks a flat list and breaks at the
    /// first anchor past the caret. Left in, three sections numbered 1, 4 and 1
    /// would open the frame on whatever page the last of them landed on.
    ///
    /// A pure manifest whose own text is in the pane therefore yields none and
    /// the frame opens at page 1, which `caretPage` already documents as its
    /// no-anchor case; a master carrying a preface syncs on its own headings,
    /// and a section in the pane syncs on that section's, which is the state
    /// `mpdf-010` Phase 2 exists to reach.
    ///
    /// Empty when the compile failed, and empty when `core`'s own count guard
    /// declined to answer. Unlike [`Render::assets`] this describes the *page*
    /// rather than the text, so it is only ever as good as the bytes beside it.
    pub anchors: Vec<Anchor>,

    /// The images the document names by URL, once each, in the order it first
    /// names them. `mpdf-003` Phase 25.
    ///
    /// **`None` exactly when `md2pdf_core::image_paths` fails**, so a walk that
    /// cannot answer leaves the caller's list alone — which is not the same
    /// condition as [`Render::assets`] being `None`: a master whose section is
    /// missing answers `Some(sections)` there and `None` here, because it names
    /// its sections from its own text and its images only once they are joined.
    pub urls: Option<Vec<String>>,

    /// The URL this compile was refused on, if it was refused on one.
    ///
    /// **Either the one `md2pdf_core::Error::UnfetchedImage` names**, read off
    /// the typed error before it becomes a sentence, **or the one
    /// [`read_assets_with`] refused in the CLI's `cannot fetch` sentence.** It
    /// is what lets `crate::preview::Preview::status` hide exactly one error —
    /// the one about an image that is on its way — and no other, without
    /// matching words.
    pub refused: Option<String>,
}

/// Which file's headings become anchors: the one the pane is holding.
///
/// `md2pdf_core::Location`'s `file` is `None` for a heading written in the
/// master's own text and `Some(path)` for one written in a section, **spelled
/// as the master names it** — so the first two arms below are exactly the two
/// shapes that comparison can take, and the third is the one it cannot.
///
/// **Three arms rather than an `Option`, and the third is why.** An `edited`
/// that does not sit under `main`'s own directory has no master-relative
/// spelling at all. Answering `Master` for it would put the master's heading
/// lines against a buffer whose lines mean something else entirely, which is
/// the defect this filter exists to prevent, arriving through the case that
/// looks like an absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane<'a> {
    /// The pane holds `main` itself.
    Master,
    /// The pane holds this path, spelled from the master's own directory.
    ///
    /// It matches nothing when the master does not name that file — a
    /// `README.md` opened beside a master contributes no anchors and the page
    /// opens at page 1. Correct rather than special-cased.
    Beside(&'a str),
    /// The pane holds a file the master's directory does not reach, so no
    /// heading in this document is a number about it.
    Away,
}

impl Pane<'_> {
    /// Does this anchor belong to the file the pane is holding?
    fn holds(&self, file: Option<&str>) -> bool {
        match self {
            Pane::Master => file.is_none(),
            Pane::Beside(path) => file == Some(*path),
            Pane::Away => false,
        }
    }
}

/// Compile one markdown string, reading the files it names from beside the
/// document.
///
/// **The markdown is a parameter and not a path**, and that is the whole of
/// what the text pane needed: the string the pane holds is what compiles, and
/// the file beside it need never have held that text. `md2pdf_core::md_to_pdf`
/// already took a `&str`, so `core` gained nothing for this.
///
/// Every failure arrives as the sentence the CLI prints after its `error: `
/// prefix, and the two classes are not the same type. A construct outside the
/// dialect is a `md2pdf_core::Error` and reaches the page through its
/// `Display`; a file that will not read is no `Error` at all, and
/// [`read_assets_with`] builds the plain sentence for it. So a document this
/// app refuses is refused in the same words at the window and at the terminal.
///
/// **The file read is the caller's**, which is the seam Phase 1 opened at
/// [`read_assets_with`] one level up, and it exists for the same reason: a
/// caller that counts its own reads can check a claim about them rather than
/// argue it from the loop. `mpdf-010` Phase 2 is the second thing it bought —
/// the pane's buffer standing in for one file of the document — and
/// [`render_project`] is where that rides.
///
/// **One closure serves both passes**, and that is what keeps both claims worth
/// checking. [`read_sections_with`] borrows it and [`read_assets_with`] takes
/// what is left, so every file this app opens for one compile goes through the
/// one closure — a second would leave half the reads unwatched.
///
/// **The images already fetched by URL are a parameter too**, since
/// `mpdf-003` Phase 25: a URL's bytes are supplied under the URL itself, a
/// failed fetch is refused in the CLI's sentence, and a URL with neither is left
/// for `core` to refuse, as it was before anything was fetched.
pub fn render_with(
    directory: &Path,
    markdown: &str,
    pane: Pane<'_>,
    fetched: &Fetched,
    mut read: impl FnMut(&Path) -> std::io::Result<Vec<u8>>,
) -> Render {
    // **The sections come first**, exactly as `cli/src/main.rs:run` orders
    // them: a master is not a document until they are joined in, so neither
    // shopping list can be asked anything before they are read. The names are
    // taken separately from the bytes because the list below needs them whether
    // or not the files are there yet.
    let named: Vec<String> = md2pdf_core::section_paths(markdown)
        .map(|sections| sections.into_iter().map(|section| section.path).collect())
        .unwrap_or_default();

    let sections = read_sections_with(markdown, directory, &mut read);
    let supplied: &[Asset] = match &sections {
        Ok(sections) => sections.as_slice(),
        Err(_) => &[],
    };

    // The path travels separately from the bytes, and this is the only place
    // it is built: `read_assets_with` below returns a `Vec<Asset>` that reaches
    // the compile and nothing else, where this list reaches the watch filter.
    // The two walks answer or fail together — the bibliography first, as
    // `cli/src/main.rs:read_assets` orders them — and the sections go in front
    // of both whatever they answer. [`Render::assets`] argues the three
    // branches.
    let images = md2pdf_core::image_paths(markdown, supplied).ok();
    let urls: Option<Vec<String>> = images.as_ref().map(|images| {
        let mut urls: Vec<String> = Vec::new();
        for image in images.iter().filter(|image| image.is_url()) {
            if !urls.contains(&image.path) {
                urls.push(image.path.clone());
            }
        }
        urls
    });
    let assets: Option<Vec<String>> = images
        .map(|images| {
            named
                .iter()
                .cloned()
                .chain(
                    md2pdf_core::bibliography_path(markdown, supplied)
                        .ok()
                        .flatten()
                        .map(|named| named.path),
                )
                // A URL is a name and not a file, so no event can change it.
                .chain(
                    images
                        .into_iter()
                        .filter(|image| !image.is_url())
                        .map(|image| image.path),
                )
                .collect()
        })
        .or_else(|| (!named.is_empty()).then(|| named.clone()));

    // **The refused URL is read here, off the two places a refusal about a URL
    // can come from, and before either becomes a sentence.** Every other
    // failure leaves it `None`.
    let mut refused: Option<String> = None;
    let rendered = sections
        .and_then(|sections| {
            read_assets_with(markdown, sections, directory, fetched, read).map_err(|unread| {
                refused = unread.url;
                unread.message
            })
        })
        .and_then(|supplied| {
            md2pdf_core::md_to_pdf_with_anchors(markdown, &supplied).map_err(|e| {
                if let md2pdf_core::Error::UnfetchedImage { url, .. } = &e {
                    refused = Some(url.clone());
                }
                e.to_string()
            })
        });

    // The anchors describe the bytes, so a failure has none — where `assets`
    // above survives one, because it describes the text. An anchor written in
    // a file the pane is not showing is a number about a document the reader
    // cannot see; [`Pane`] is the one comparison that decides which those are,
    // and [`Render::anchors`] argues why dropping the rest is the only answer
    // that is true by construction.
    let (pdf, anchors) = match rendered {
        Ok(rendered) => (
            Ok(rendered.pdf),
            rendered
                .anchors
                .into_iter()
                .filter(|anchor| pane.holds(anchor.location.file.as_deref()))
                .map(|anchor| Anchor {
                    line: anchor.location.line,
                    page: anchor.page,
                })
                .collect(),
        ),
        Err(message) => (Err(message), Vec::new()),
    };

    Render {
        assets,
        sections: named,
        pdf,
        anchors,
        urls,
        refused,
    }
}

/// Compile the project: `main`'s own text, with the pane's buffer standing in
/// for the one file the pane is holding.
///
/// **The override rides the closure [`render_with`] already takes**, which is
/// one rule instead of a branch. The closure answers `edited` from the buffer
/// and every other path from the disk, and **`main`'s own text is read through
/// it too** — so it returns the buffer exactly when the pane holds the master,
/// and the disk copy otherwise, with nothing here having to ask which case it
/// is in. That the markdown is `main`'s and the directory is `main`'s is the
/// whole of what separates the file that compiles from the file that is edited:
/// every path the document names resolves against the master, wherever the pane
/// happens to be.
///
/// **The closure yields bytes where the compile wants a string**, so this read
/// decodes as UTF-8 and a `main` that is not text fails here. It fails in
/// [`read_document`]'s own sentence, built by wrapping the decode in the
/// `std::io::Error` `read_to_string` would have raised, so a main that will not
/// read reads the same in the window whichever path reached it.
pub fn render_project(
    files: &impl Files,
    main: &str,
    edited: &str,
    buffer: &str,
    fetched: &Fetched,
) -> Result<Render, String> {
    let (main_at, edited_at) = (files.locate(main), files.locate(edited));
    let read = |file: &Path| -> std::io::Result<Vec<u8>> {
        if files.same(file, &edited_at) {
            Ok(buffer.as_bytes().to_vec())
        } else {
            files.read(file)
        }
    };

    let markdown = read(&main_at)
        .and_then(|bytes| {
            String::from_utf8(bytes).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "stream did not contain valid UTF-8",
                )
            })
        })
        .map_err(|e| format!("cannot read {}: {e}", main_at.display()))?;

    let spelled = under(&main_at, &edited_at);
    let pane = if main == edited {
        Pane::Master
    } else {
        match spelled.as_deref() {
            Some(path) => Pane::Beside(path),
            None => Pane::Away,
        }
    };

    Ok(render_with(directory(&main_at), &markdown, pane, fetched, read))
}

/// Read a document's text, in the words the terminal uses for a file it cannot
/// read.
///
/// The read left `render` when the pane's text became what compiles, and it
/// landed here rather than at the caller because this sentence is one of the
/// two the app owes the CLI. Phase 1 built it inside the compile; the same
/// string reaches the page from one function further out.
pub fn read_document(files: &impl Files, document: &str) -> Result<String, String> {
    let located = files.locate(document);
    files
        .read(&located)
        .and_then(|bytes| {
            String::from_utf8(bytes).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "stream did not contain valid UTF-8",
                )
            })
        })
        .map_err(|e| format!("cannot read {}: {e}", located.display()))
}

/// The directory a document's assets resolve against: the one it sits in.
///
/// An empty parent is a document named with no directory at all, and joining an
/// asset onto `""` resolves it against the working directory, which is what the
/// CLI does for the same input.
pub fn directory(document: &Path) -> &Path {
    document.parent().unwrap_or(Path::new(""))
}

// ---------------------------------------------------------------------------
// The project: its root, its files, the main among them, the bytes of one of
// them, and the one fact this app remembers about it. `mpdf-010` Phases 1
// and 5.
//
// Ordinary functions, because a panel that could only be checked by opening a
// window would have no exit gate but a screenshot — this file's own header,
// applied to the newest thing in it.
// ---------------------------------------------------------------------------

/// What one row of the panel is.
///
/// **A directory is never an entry.** The page derives the folder headings and
/// the indentation from the path's own segments, which is a thing a page can do
/// and a thing a nested node type would make `crate::preview::Status` carry
/// twice.
///
/// This crosses to the page inside that `Status`, so it serializes — the same
/// reason [`Anchor`] is declared here rather than borrowed from `core`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// Root-relative, with `/` separators on every platform, so the page can
    /// split it into segments without knowing what a path is here.
    pub path: String,
    /// Which of the three channels of the pipeline would read this file.
    pub kind: Kind,
    /// True for a path the master names that the disk does not hold.
    ///
    /// It is the row the author most needs to see: it is the state
    /// `md2pdf_core::Error::MissingSection` refuses on, and a panel built from
    /// the disk alone would be silent about exactly the file that broke the
    /// document.
    pub missing: bool,
}

/// The three kinds of file the pipeline reads, which are the three the panel
/// lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Markdown,
    Bibliography,
    Image,
}

/// Which channel would read this path, or `None` for a file the pipeline has no
/// use for.
///
/// **Each channel is compared the way that channel compares it**, and the
/// asymmetry is inherited rather than invented, so nobody "fixes" it into a
/// panel that disagrees with the compiler:
///
/// - markdown is `eq_ignore_ascii_case("md")`, as `core/src/emit.rs`'s
///   `lone_markdown_link` reads an include marker;
/// - a bibliography is folded to lower case and matched against `bib`, `yml`
///   and `yaml`, as `core/src/bibliography.rs` does. **All three, not `.bib`
///   alone**: a document whose frontmatter names `refs.yml` compiles, and a
///   panel blind to it would tell the author this app could not read it —
///   which is `mpdf-010` §2's argument for taking the image list off `core`
///   rather than hand-writing a subset, applied to the channel beside it;
/// - an image is matched case-*sensitively* against
///   [`md2pdf_core::IMAGE_EXTENSIONS`], as `core/src/emit.rs`'s `check_image`
///   does, reading `VirtualPath::extension` — which is the function Typst's own
///   format detection reads.
pub fn kind_of(path: &str) -> Option<Kind> {
    let name = path.rsplit('/').next()?;
    let (_, extension) = name.rsplit_once('.')?;

    if extension.eq_ignore_ascii_case("md") {
        return Some(Kind::Markdown);
    }
    if matches!(extension.to_lowercase().as_str(), "bib" | "yml" | "yaml") {
        return Some(Kind::Bibliography);
    }
    if md2pdf_core::IMAGE_EXTENSIONS.contains(&extension) {
        return Some(Kind::Image);
    }
    None
}

/// The panel's order, as a comparison, so the union below sorts by the same rule
/// the walk emitted.
///
/// At the first segment the two paths differ on: a path with nothing left after
/// it is a file *in this directory* and sorts before one that still has
/// segments to go, which is a file in a subdirectory beside it. Otherwise the
/// segments are compared as bytes.
pub(crate) fn order(left: &str, right: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let mut left = left.split('/');
    let mut right = right.split('/');

    loop {
        return match (left.next(), right.next()) {
            (Some(here), Some(there)) => {
                let (last_here, last_there) = (
                    left.clone().next().is_none(),
                    right.clone().next().is_none(),
                );
                if here == there && last_here == last_there {
                    continue;
                }
                match (last_here, last_there) {
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    _ => here.as_bytes().cmp(there.as_bytes()),
                }
            }
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        };
    }
}

/// What the panel lists is [`files_under`] and [`merge`], and it is two
/// functions rather than one because the app runs them at two different rates.
///
/// **The union is the point.** A tree built from the store alone loses the one
/// thing the panel this replaces was good at: a section the master names and the
/// store does not hold is exactly the row the author needs to see. And splitting
/// it is what keeps the panel honest while a marker is half-typed — the stored
/// half is stable and only the marked-missing half moves.
///
/// The stored half: every file in the project the pipeline can read.
///
/// **This is the half that costs a walk** on the desktop, where
/// [`Files::list`] reads every directory. `crate::preview::Preview` holds its
/// answer and refreshes it at an open and when the host says the tree moved,
/// where [`merge`] below runs on every status.
pub fn files_under(files: &impl Files) -> Vec<Entry> {
    files
        .list()
        .into_iter()
        .filter_map(|path| {
            kind_of(&path).map(|kind| Entry {
                path,
                kind,
                missing: false,
            })
        })
        .collect()
}

/// The union: the files found, plus the paths `named` holds that they do not.
///
/// Pure, and cheap enough to run on every render — which is what lets the disk
/// half stay still while the marked-missing half follows the text.
///
/// `named` is root-relative, which is `crate::preview::Preview`'s job to make
/// it: `md2pdf_core::section_paths` answers relative to the master, and the
/// master need not sit at the root.
pub fn merge(found: Vec<Entry>, named: &[String]) -> Vec<Entry> {
    let mut entries = found;

    for path in named {
        if entries.iter().all(|entry| &entry.path != path) {
            entries.push(Entry {
                path: path.clone(),
                kind: Kind::Markdown,
                missing: true,
            });
        }
    }

    entries.sort_by(|left, right| order(&left.path, &right.path));
    entries
}

/// Every `.md` **directly in** `root` that names a section, in the panel's order.
///
/// **Discovery is total, so the common case needs no configuration.**
/// `md2pdf_core::section_paths` reads the master's own text and its body cannot
/// fail — it returns `Result` for signature symmetry with the two walks beside
/// it and constructs `Ok` unconditionally — so *"a `.md` here whose text names
/// section markers"* is a decidable test over every markdown file in one
/// directory.
///
/// **It does not recurse, and the reason is a property rather than a
/// preference.** A master cannot name a section above itself:
/// `core/src/emit.rs:landed_path` refuses a marker that climbs out of the
/// document's own folder, so `[](../a.md)` is not an include at all. Every
/// section therefore sits at or below its master's directory, which means the
/// master of the opened file is at the root or *above* it and never in a
/// subdirectory of it — [`project_root`]'s climb answers "above", and this
/// answers "at".
///
/// **A recursive walk got this wrong in the window, which is what the exit
/// gate's `samples/article.md` case is now here to catch.** `samples/` holds a
/// single-file document beside the whole `showcase/` project, so recursion
/// found `showcase/showcase.md`, called it the one master, and compiled it for
/// an author who had opened `article.md`. A `.md` in a subdirectory that names
/// sections is another project's master, not this root's.
pub fn masters(files: &impl Files) -> Vec<String> {
    files
        .list()
        .into_iter()
        .filter(|path| !path.contains('/'))
        .filter(|path| kind_of(path) == Some(Kind::Markdown))
        .filter(|path| {
            read_document(files, path)
                .ok()
                .and_then(|text| md2pdf_core::section_paths(&text).ok())
                .is_some_and(|sections| !sections.is_empty())
        })
        .collect()
}

/// Which file under `root` compiles, when nothing is remembered about it.
///
/// - **exactly one master → that file**, whatever the author opened;
/// - **no master → the file the author opened**, which is every single-file
///   document and is this app's behaviour before the panel existed;
/// - **more than one master → the opened file if it is itself one of them,
///   otherwise the byte-wise alphabetically first**, and the panel marks which
///   it landed on.
///
/// **This never leaves the main unset.** An empty pane and no page is a worse
/// answer than a guess the author can see and correct in one action, and the
/// mark in the panel is what makes the guess visible. Alphabetical is not a
/// claim about which is right — it is a claim that the same folder opens the
/// same way twice, which a set iteration order would not be.
///
/// `here` is the opened file spelled root-relatively, which is the host's to
/// make: the desktop resolves a path a dialog handed it, and a browser project
/// names its own files.
pub fn discover_main(files: &impl Files, here: &str) -> String {
    let mut masters = masters(files);
    match masters.len() {
        0 => here.to_string(),
        1 => masters.remove(0),
        _ if masters.iter().any(|master| master == here) => here.to_string(),
        _ => masters.remove(0),
    }
}

/// [`relative`] without the canonicalization, for two paths this app built
/// from one root.
///
/// **It reads nothing off the disk, and that is why it is a function of its
/// own.** `relative` resolves both sides because it answers about a path that
/// came from outside — a command, a filesystem event — where the two callers
/// here already hold `root` and `root.join(…)` and have nothing to resolve.
/// `crate::preview::Preview::status` is one of them, and it runs on every
/// render: a `canonicalize` in there would put two syscalls in front of every
/// status and falsify that function's own stated invariant.
pub fn spell(root: &Path, path: &Path) -> Option<String> {
    let rest = path.strip_prefix(root).ok()?;

    let mut spelled = String::new();
    for part in rest.components() {
        if !spelled.is_empty() {
            spelled.push('/');
        }
        spelled.push_str(&part.as_os_str().to_string_lossy());
    }
    (!spelled.is_empty()).then_some(spelled)
}

/// How the master would name this file: [`beside`] run backwards.
///
/// **It is `beside`'s inverse and not a call to it.** That one takes a path the
/// master names to a root-relative one; this takes a root-relative path back to
/// the spelling `md2pdf_core::Location` carries, which is what the anchor filter
/// compares against. `None` is a file the master's own directory does not reach,
/// which has no such spelling at all — [`Pane::Away`], not [`Pane::Master`].
pub fn under(main: &Path, edited: &Path) -> Option<String> {
    spell(directory(main), edited)
}

/// One root-relative path joined onto the directory another sits in.
///
/// The master need not sit at the root, and `md2pdf_core::section_paths`
/// answers relative to the master — so a master at `parts/book.md` naming
/// `text.md` means the root-relative `parts/text.md`. A `..` is resolved
/// lexically, which is enough: `core/src/emit.rs`'s `landed_path` has already
/// refused any path that climbs out of the master's own folder.
pub fn beside(main: &str, named: &str) -> String {
    let mut segments: Vec<&str> = match main.rsplit_once('/') {
        Some((directory, _)) => directory.split('/').collect(),
        None => Vec::new(),
    };
    for part in named.split('/') {
        match part {
            "." | "" => {}
            ".." => {
                segments.pop();
            }
            part => segments.push(part),
        }
    }
    segments.join("/")
}

/// The sentence three commands refuse a path with when it is not one of the
/// project's files: [`Files::holds`] said no.
///
/// **One sentence and not three**, so a figure read, a switch and a set-main
/// that refuse the same path refuse it in the same words — on the desktop, where
/// `app/src/document.rs:confined` answers, and in the browser alike.
pub fn not_a_file(path: &str) -> String {
    format!("{path} is not a file in this project")
}

/// The bytes of one file in the project, for the window to draw.
///
/// It confines through [`Files::holds`], which is the rule the two commands
/// that move a path share. `mpdf-010` Phase 5.
pub fn asset_bytes(files: &impl Files, path: &str) -> Result<Vec<u8>, String> {
    if !files.holds(path) {
        return Err(not_a_file(path));
    }
    let located = files.locate(path);
    files
        .read(&located)
        .map_err(|e| format!("cannot read {}: {e}", located.display()))
}

/// Refuse a new file the panel cannot list: the kind is the extension's.
///
/// **Shared by [`create_file`] and the Save-as**, with its "a new file is"
/// wording intact, so that neither caller's copy is quietly reworked and the two
/// drift apart.
pub fn creatable(path: &str) -> Result<(), String> {
    match kind_of(path) {
        Some(Kind::Markdown | Kind::Bibliography) => Ok(()),
        _ => Err(format!(
            "{path} is neither markdown nor a bibliography: \
             a new file is a .md, .bib, .yml or .yaml"
        )),
    }
}

/// Make one empty file in the project, named by the panel.
///
/// **The extension decides the kind and there is no kind parameter.** A kind
/// that supplied the extension would make an extensionless name the normal
/// input rather than a refusal, and would turn `notes.typ` into `notes.typ.md`.
/// So [`kind_of`] is the predicate, and the create is accepted exactly where it
/// answers [`Kind::Markdown`] or [`Kind::Bibliography`] — which takes the
/// `.yml` and `.yaml` bibliographies `core/src/bibliography.rs` reads, because
/// a `.bib`-only create would be the hand-written subset
/// `specs/file_panel_spec.md` §2 refuses for `.jpg`: two lists that drift, and
/// a panel listing a kind it cannot create. [`Kind::Image`] is refused with the
/// rest; the panel does not make pictures.
///
/// It is asked first because it needs no store at all — [`creatable`], which
/// the Save-as shares.
///
/// **Where the file lands and whether it is there already are the store's**,
/// through [`Files::create`]: the desktop canonicalizes the parent and makes the
/// file with `create_new`, so *already exists* is the filesystem's own answer
/// and a dangling symlink at that name is refused rather than written through;
/// a map answers the same two questions of its keys. The words are
/// [`crate::files::Refused`]'s either way. `mpdf-010` Phase 3.
pub fn create_file(files: &mut impl Files, path: &str) -> Result<(), String> {
    creatable(path)?;
    files
        .create(path)
        .map_err(|refused| refused.sentence(path, &files.locate(path)))
}

/// Delete one file in the project, named by the panel, in [`Refused`]'s words.
///
/// What a delete *is* belongs to the host: the desktop's [`Files::remove`] is a
/// move to the Trash, the web's is permanent. `mpdf-010` Phase 4.
pub fn remove_file(files: &mut impl Files, path: &str) -> Result<(), String> {
    files
        .remove(path)
        .map_err(|refused| refused.sentence(path, &files.locate(path)))
}

/// Read every file the document names, from beside the document.
///
/// This mirrors `cli/src/main.rs:read_assets`: a path resolves against the
/// directory of the open file, so a document, its figures and its bibliography
/// travel as one folder, and an asset keeps the path the markdown wrote,
/// because that is the name the generated Typst source asks for. The
/// duplication between the two wrappers is deliberate. A shared helper crate
/// for forty lines would buy less than it costs, and the two report their
/// errors differently, which is most of what those forty lines do.
///
/// The image list arrives in document order and may name one path twice, so
/// this reads each file once. The bibliography is one frontmatter value rather
/// than something the walk finds, so it comes from an export of its own — and
/// it is read first of the two, since the line it names is the earliest one in
/// the file.
///
/// **The sections are already read when this runs**, and they arrive here so
/// they ride out on the same array: neither list above can be asked for until
/// the document they belong to has been assembled, which is why
/// [`read_sections_with`] is a pass of its own and this one takes its result.
/// Their paths seed the same `seen` set, so no file is opened twice across the
/// two passes.
///
/// **Every path joins the master's directory, a section's own images
/// included.** `core` writes a section's own folder into the destination before
/// the list reaches here, so an image drawn in `sections/method.md` arrives as
/// `sections/figure.png` and is found beside the file that drew it — the rule
/// this app inherits rather than carries a copy of.
///
/// The read is a parameter for one gate. Phase 1 asks that a path the document
/// names twice is read *once*, and a caller that counts its own reads is the
/// only way to check that rather than argue it from the loop below.
///
/// **An image named by a URL is read from nowhere, and since `mpdf-003` Phase 25
/// it has three answers**, each `cli/src/main.rs:read_assets`' own under
/// `--fetch`. Bytes that were fetched are supplied under the URL itself, which is
/// the name the generated Typst source asks for. A fetch that failed is refused
/// here, in the CLI's `cannot fetch` sentence. A URL nothing has fetched gets no
/// bytes and is left for `core`'s own refusal. It is never joined onto the
/// directory in any of the three: that would hand the OS `dir/https://…` and the
/// author an OS error about a file that was never meant to exist.
fn read_assets_with(
    markdown: &str,
    sections: Vec<Asset>,
    directory: &Path,
    fetched: &Fetched,
    mut read: impl FnMut(&Path) -> std::io::Result<Vec<u8>>,
) -> Result<Vec<Asset>, Unread> {
    let images = md2pdf_core::image_paths(markdown, &sections).map_err(Unread::from)?;
    let bibliography = md2pdf_core::bibliography_path(markdown, &sections).map_err(Unread::from)?;

    let mut seen: HashSet<String> = sections.iter().map(|s| s.path.clone()).collect();
    let mut assets = sections;

    if let Some(named) = bibliography {
        let file = directory.join(&named.path);
        let bytes = read(&file).map_err(|e| {
            Unread::from(format!(
                "cannot read {} for the bibliography {}: {e}",
                file.display(),
                named.location
            ))
        })?;

        seen.insert(named.path.clone());
        assets.push(Asset {
            path: named.path,
            bytes,
        });
    }

    for image in images {
        if !seen.insert(image.path.clone()) {
            continue;
        }

        let bytes = if image.is_url() {
            match fetched.get(&image.path) {
                Some(Ok(bytes)) => bytes.to_vec(),
                Some(Err(reason)) => {
                    return Err(Unread {
                        message: format!(
                            "cannot fetch {} for the image {}: {reason}",
                            image.path, image.location
                        ),
                        url: Some(image.path),
                    });
                }
                None => continue,
            }
        } else {
            let file = directory.join(&image.path);
            read(&file).map_err(|e| {
                Unread::from(format!(
                    "cannot read {} for the image {}: {e}",
                    file.display(),
                    image.location
                ))
            })?
        };

        assets.push(Asset {
            path: image.path,
            bytes,
        });
    }
    Ok(assets)
}

/// Why [`read_assets_with`] stopped: the sentence the terminal prints, and the
/// URL when that sentence is a failed fetch's.
///
/// **The URL rides beside the words rather than being read back out of them**,
/// which is [`Render::refused`]'s whole reason for existing.
#[derive(Debug)]
struct Unread {
    message: String,
    url: Option<String>,
}

impl From<String> for Unread {
    fn from(message: String) -> Self {
        Self { message, url: None }
    }
}

impl From<md2pdf_core::Error> for Unread {
    fn from(error: md2pdf_core::Error) -> Self {
        Self::from(error.to_string())
    }
}

/// Read every section file the master names, in the order it names them.
///
/// This mirrors `cli/src/main.rs:read_sections`, and it runs **before** either
/// shopping list for the reason that function records: the markers are in the
/// master's own text, so the sections can be read with no join, where every
/// later question is about the document they assemble into. One extra round
/// trip through `core`, no recursion here, and one place that ever
/// concatenates — which is `core`, because it is the joining that builds the
/// map every message is translated through.
///
/// The read is borrowed rather than taken, so [`read_assets_with`] goes on to
/// use the same closure. A section that will not open is the third of the
/// sentence the image and the bibliography already print, and the third this
/// app owes the terminal.
fn read_sections_with(
    markdown: &str,
    directory: &Path,
    mut read: impl FnMut(&Path) -> std::io::Result<Vec<u8>>,
) -> Result<Vec<Asset>, String> {
    let named = md2pdf_core::section_paths(markdown).map_err(|e| e.to_string())?;

    let mut sections = Vec::with_capacity(named.len());
    for section in named {
        let file = directory.join(&section.path);
        let bytes = read(&file).map_err(|e| {
            format!(
                "cannot read {} for the section {}: {e}",
                file.display(),
                section.location
            )
        })?;

        sections.push(Asset {
            path: section.path,
            bytes,
        });
    }

    Ok(sections)
}

// A test may read a fixture: `clippy.toml`'s list is about what the crate
// ships, which is what the gate lints.
#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;
    use crate::files::MemFiles;
    use std::path::PathBuf;

    /// [`render_with`] with the disk supplying every file and the pane holding
    /// the master, which is what every test below but the project's own means.
    fn render(directory: &Path, markdown: &str) -> Render {
        // The closure is not noise: `std::fs::read` names one lifetime where
        // the parameter asks for any, so passing it directly does not compile.
        render_with(directory, markdown, Pane::Master, &Fetched::new(), |file| {
            std::fs::read(file)
        })
    }

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures")
            .join(name)
    }

    /// A scratch directory that this test process owns, so runs do not
    /// collide and the repository stays clean.
    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("letur-project-test-{}", std::process::id()))
            .join(name);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A fixture compiled the way the pane compiles: its text as a string,
    /// against the directory it sits in.
    fn render_fixture(name: &str) -> Render {
        let document = fixture(name);
        let markdown = std::fs::read_to_string(&document).unwrap();
        render(directory(&document), &markdown)
    }

    /// Every file under a fixture directory, held in memory, which is how the
    /// browser holds a project. **A link is not followed**: the desktop's walk
    /// confines what a link reaches, and a map has nothing to confine.
    fn loaded(root: &Path) -> MemFiles {
        fn under(root: &Path, here: &Path, found: &mut Vec<(String, Vec<u8>)>) {
            for entry in std::fs::read_dir(here).unwrap().flatten() {
                let path = entry.path();
                let kind = std::fs::symlink_metadata(&path).unwrap().file_type();
                if kind.is_symlink() {
                    continue;
                }
                if kind.is_dir() {
                    under(root, &path, found);
                } else {
                    let spelled = path.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                    found.push((spelled, std::fs::read(&path).unwrap()));
                }
            }
        }
        let mut found = Vec::new();
        under(root, root, &mut found);
        MemFiles::new(found)
    }

    /// A document and both the files it names, each read from beside it.
    ///
    /// `figures/mark.svg` pins that a path in a subdirectory resolves against
    /// the document's directory and not against the current one, and that the
    /// asset keeps the path the markdown wrote rather than the resolved one.
    #[test]
    fn a_document_and_its_images_read_as_two_assets() {
        let dir = scratch_dir("figure-doc");
        std::fs::copy(fixture("dot.png"), dir.join("dot.png")).unwrap();
        std::fs::create_dir_all(dir.join("figures")).unwrap();
        std::fs::copy(fixture("mark.svg"), dir.join("figures/mark.svg")).unwrap();

        let markdown = std::fs::read_to_string(fixture("figure.md")).unwrap();
        let assets =
            read_assets_with(&markdown, Vec::new(), &dir, &Fetched::new(), |file| std::fs::read(file)).unwrap();

        let paths: Vec<&str> = assets.iter().map(|a| a.path.as_str()).collect();
        assert_eq!(paths, ["dot.png", "figures/mark.svg"]);
        assert!(assets.iter().all(|a| !a.bytes.is_empty()));
    }

    /// The same document beside no `figures/` directory, which is how
    /// `tests/fixtures/` actually stands: `dot.png` sits there and
    /// `figures/mark.svg` does not, so the second reference fails.
    #[test]
    fn a_missing_image_names_the_path_the_line_and_the_reason() {
        let markdown = std::fs::read_to_string(fixture("figure.md")).unwrap();
        let error = read_assets_with(&markdown, Vec::new(), &fixture(""), &Fetched::new(), |file| {
            std::fs::read(file)
        })
        .unwrap_err()
        .message;

        assert!(error.contains("figures/mark.svg"), "{error}");
        assert!(error.contains("line 5"), "{error}");
        assert!(error.contains("os error"), "{error}");
    }

    /// One path named twice is one asset and one read.
    ///
    /// The document is written here rather than taken from
    /// `tests/fixtures/images.md`, which repeats `dot.png` on lines 3 and 10
    /// but names `fig#2.png` on line 7: a reader fails there before it ever
    /// reaches the repeat.
    #[test]
    fn a_path_named_twice_is_read_once() {
        let dir = scratch_dir("repeated-image");
        std::fs::copy(fixture("dot.png"), dir.join("dot.png")).unwrap();

        let markdown = "![the first](dot.png)\n\nText between them.\n\n![the second](dot.png)\n";

        let mut reads = Vec::new();
        let assets = read_assets_with(markdown, Vec::new(), &dir, &Fetched::new(), |file| {
            reads.push(file.to_path_buf());
            std::fs::read(file)
        })
        .unwrap();

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].path, "dot.png");
        assert_eq!(reads, [dir.join("dot.png")]);
    }

    /// An image named by a URL is read from nowhere, watched for nothing, and
    /// refused in the words the terminal uses without `--fetch`.
    ///
    /// Before `md2pdf-core` 0.3 the dialect refused the scheme itself. Since
    /// then the URL is a name on `image_paths`' list like any other, and joining
    /// it onto the directory put an OS error about `dir/https:/…` on the page.
    /// The file beside it is still read, so the refusal is the URL's and not the
    /// first image's.
    #[test]
    fn a_url_image_is_read_from_nowhere_and_refused_in_the_engine_s_words() {
        let dir = scratch_dir("url-image");
        std::fs::copy(fixture("dot.png"), dir.join("dot.png")).unwrap();

        let markdown = "![here](dot.png)\n\n![there](https://example.com/figure.png)\n";

        let mut reads = Vec::new();
        let render = render_with(&dir, markdown, Pane::Master, &Fetched::new(), |file| {
            reads.push(file.to_path_buf());
            std::fs::read(file)
        });

        assert_eq!(reads, [dir.join("dot.png")]);
        assert_eq!(
            render.pdf,
            Err("no image fetched for 'https://example.com/figure.png' at line 3".to_string())
        );
        assert_eq!(render.assets, Some(vec!["dot.png".to_string()]));
        assert_eq!(
            render.urls,
            Some(vec!["https://example.com/figure.png".to_string()])
        );
    }

    /// A fetched URL is supplied under its own name, and read from nowhere else.
    ///
    /// The bytes are `dot.png`'s, handed over as though a site had served them,
    /// and the read closure is counted: it sees the file beside the document and
    /// never the URL.
    #[test]
    fn a_fetched_url_is_supplied_under_its_own_name_and_read_from_nowhere() {
        let dir = scratch_dir("url-fetched");
        std::fs::copy(fixture("dot.png"), dir.join("dot.png")).unwrap();
        let url = "https://images.example/figure.png";
        let markdown = format!("![here](dot.png)\n\n![there]({url})\n\n![again]({url})\n");

        let fetched: Fetched = [(
            url.to_string(),
            Ok(std::sync::Arc::new(std::fs::read(fixture("dot.png")).unwrap())),
        )]
        .into();

        let mut reads = Vec::new();
        let assets = read_assets_with(&markdown, Vec::new(), &dir, &fetched, |file| {
            reads.push(file.to_path_buf());
            std::fs::read(file)
        })
        .unwrap();
        assert_eq!(reads, [dir.join("dot.png")]);
        let paths: Vec<&str> = assets.iter().map(|a| a.path.as_str()).collect();
        assert_eq!(paths, ["dot.png", url], "one asset per name, the URL's its own");

        let render = render_with(&dir, &markdown, Pane::Master, &fetched, |file| {
            std::fs::read(file)
        });
        assert!(render.pdf.is_ok(), "{:?}", render.pdf.err());
        assert_eq!(render.refused, None);
        assert_eq!(render.urls, Some(vec![url.to_string()]));
    }

    /// A fetch that failed is refused in `cli/src/main.rs:read_assets`' own
    /// sentence, word for word, and names the URL it was refused for.
    #[test]
    fn a_failed_fetch_is_refused_in_the_terminals_own_words() {
        let dir = scratch_dir("url-failed");
        let url = "https://images.example/gone.png";
        let markdown = format!("# Title\n\n![gone]({url})\n");
        let fetched: Fetched = [(url.to_string(), Err("403 Forbidden".to_string()))].into();

        let unread = read_assets_with(&markdown, Vec::new(), &dir, &fetched, |file| {
            std::fs::read(file)
        })
        .unwrap_err();
        assert_eq!(
            unread.message,
            format!("cannot fetch {url} for the image at line 3: 403 Forbidden")
        );
        assert_eq!(unread.url.as_deref(), Some(url));

        let render = render_with(&dir, &markdown, Pane::Master, &fetched, |file| {
            std::fs::read(file)
        });
        assert_eq!(render.pdf, Err(unread.message));
    }

    /// [`Render::refused`] names the URL `core` refused on and the URL a failed
    /// fetch was refused for, and nothing for any other failure. [`Render::urls`]
    /// is `None` when `image_paths` fails, while [`Render::assets`] still names
    /// the sections of a master whose section is missing.
    #[test]
    fn the_refused_url_is_read_off_the_error_and_the_urls_off_the_walk() {
        let dir = scratch_dir("url-refused");
        let url = "https://images.example/figure.png";
        let markdown = format!("# Title\n\n![there]({url})\n");

        let unfetched = render_with(&dir, &markdown, Pane::Master, &Fetched::new(), |file| {
            std::fs::read(file)
        });
        assert_eq!(unfetched.refused.as_deref(), Some(url), "core's refusal");

        let failed: Fetched = [(url.to_string(), Err("503 Service Unavailable".to_string()))].into();
        let refused = render_with(&dir, &markdown, Pane::Master, &failed, |file| {
            std::fs::read(file)
        });
        assert_eq!(refused.refused.as_deref(), Some(url), "a failed fetch");

        let missing = render_with(&dir, "![here](absent.png)\n", Pane::Master, &Fetched::new(), |file| {
            std::fs::read(file)
        });
        assert!(missing.pdf.is_err());
        assert_eq!(missing.refused, None, "a missing file is not a URL");

        // A master beside none of its sections: the walk cannot answer, so
        // there is no list of URLs, and the sections are still named.
        let master = std::fs::read_to_string(fixture("multi_file.md")).unwrap();
        let empty = scratch_dir("url-section-absent");
        let sectioned = render_with(&empty, &master, Pane::Master, &Fetched::new(), |file| {
            std::fs::read(file)
        });
        assert_eq!(sectioned.urls, None);
        assert_eq!(sectioned.refused, None);
        assert_eq!(
            sectioned.assets.as_deref(),
            Some(sectioned.sections.as_slice())
        );
        assert!(!sectioned.sections.is_empty());
    }

    /// A `mermaid` fence draws on the page, which is `md2pdf-core` 0.2's
    /// diagram reaching this app through the path the pane compiles on.
    #[test]
    fn a_mermaid_fence_compiles() {
        let dir = scratch_dir("mermaid");
        let markdown = "```mermaid\nflowchart LR\n  A[write] --> B[see the page]\n```\n";

        let render = render(&dir, markdown);

        assert!(render.pdf.is_ok(), "{:?}", render.pdf.err());
        assert_eq!(render.assets, Some(Vec::new()));
    }

    /// A document and the bibliography it declares, read from beside it.
    ///
    /// The second asset channel at this app's own seam, mirroring
    /// `cli/src/main.rs:read_assets`: the bibliography is read **first**, and
    /// the asset keeps the path the frontmatter wrote rather than the resolved
    /// one. `tests/fixtures/citations.md` names no image, so the one asset is
    /// the whole list.
    #[test]
    fn a_document_and_its_bibliography_read_as_one_asset() {
        let dir = scratch_dir("bibliography-doc");
        std::fs::copy(fixture("refs.yml"), dir.join("refs.yml")).unwrap();

        let markdown = std::fs::read_to_string(fixture("citations.md")).unwrap();
        let mut reads = Vec::new();
        let assets = read_assets_with(&markdown, Vec::new(), &dir, &Fetched::new(), |file| {
            reads.push(file.to_path_buf());
            std::fs::read(file)
        })
        .unwrap();

        assert_eq!(reads, [dir.join("refs.yml")]);
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].path, "refs.yml");
        assert!(!assets[0].bytes.is_empty());
    }

    /// The same document beside no bibliography at all, which is how
    /// `scratch_dir` stands until something copies one in.
    ///
    /// The sentence is `cli/src/main.rs:read_assets`' own, word for word, and
    /// the line is the frontmatter's rather than any the walk could reach —
    /// a bibliography is one value the walk never meets.
    #[test]
    fn a_missing_bibliography_names_the_path_the_line_and_the_reason() {
        let dir = scratch_dir("bibliography-absent");
        let _ = std::fs::remove_file(dir.join("refs.yml"));

        let markdown = std::fs::read_to_string(fixture("citations.md")).unwrap();
        let error =
            read_assets_with(&markdown, Vec::new(), &dir, &Fetched::new(), |file| std::fs::read(file)).unwrap_err().message;

        assert!(error.contains("refs.yml"), "{error}");
        assert!(error.contains("for the bibliography"), "{error}");
        assert!(error.contains("line 3"), "{error}");
        assert!(error.contains("os error"), "{error}");
    }

    /// A construct outside the dialect names itself and its line, in the words
    /// the terminal uses. The window shows this sentence; that half is read by
    /// eye, and this is the half a test can hold.
    #[test]
    fn a_construct_outside_the_dialect_names_itself_and_its_line() {
        let error = render_fixture("unsupported_html.md").pdf.unwrap_err();

        assert!(error.contains("raw HTML block"), "{error}");
        assert!(error.contains("line 5"), "{error}");
    }

    /// A document that does not parse hands back no list, and the caller keeps
    /// the one it had. A document that parses but will not compile hands one
    /// back anyway, which is what keeps the watch filter alive while a figure
    /// is missing.
    #[test]
    fn the_asset_list_survives_a_failed_compile_but_not_a_failed_parse() {
        assert_eq!(render_fixture("unsupported_html.md").assets, None);

        let render = render_fixture("figure.md");
        assert!(render.pdf.is_err());
        assert_eq!(
            render.assets,
            Some(vec!["dot.png".to_string(), "figures/mark.svg".to_string()])
        );
    }

    /// A master beside no sections at all, in the words the terminal uses for
    /// the same file.
    ///
    /// The third of this app's hand-built sentences, beside the image's and the
    /// bibliography's, and word for word the one `cli/src/main.rs:read_sections`
    /// prints. A `SectionRef`'s location never carries a file — a section may
    /// not name a section — so the phrase is `at line N` and the line is the
    /// master's own.
    #[test]
    fn a_missing_section_names_the_path_the_line_and_the_reason() {
        let dir = scratch_dir("section-absent");
        let markdown = std::fs::read_to_string(fixture("multi_file.md")).unwrap();
        let error = read_sections_with(&markdown, &dir, |file| std::fs::read(file)).unwrap_err();

        assert!(error.contains("sections/introduction.md"), "{error}");
        assert!(error.contains("for the section"), "{error}");
        assert!(error.contains("at line 7"), "{error}");
        assert!(error.contains("os error"), "{error}");
    }

    /// Every file a master names is opened once, across both passes.
    ///
    /// [`a_path_named_twice_is_read_once`] extended to the channel that added a
    /// second pass over the same directory. One closure serves both, so a
    /// second one — or a section read again as an asset — shows up as an extra
    /// entry here rather than as an argument about the loops.
    ///
    /// The two images are named bare inside the sections and reach the list as
    /// `sections/dot.png` and `sections/mark.svg`, which is Phase 2's rule
    /// arriving in this wrapper with nothing added for it.
    #[test]
    fn every_file_a_master_names_is_read_once_across_both_passes() {
        let dir = fixture("");
        let markdown = std::fs::read_to_string(fixture("multi_file.md")).unwrap();

        let mut reads = Vec::new();
        let rendered = render_with(&dir, &markdown, Pane::Master, &Fetched::new(), |file: &Path| {
            reads.push(file.to_path_buf());
            std::fs::read(file)
        });

        assert!(rendered.pdf.is_ok(), "{:?}", rendered.pdf.as_ref().err());
        assert_eq!(
            reads,
            [
                dir.join("sections/introduction.md"),
                dir.join("sections/method.md"),
                dir.join("sections/results.md"),
                dir.join("sections/dot.png"),
                dir.join("sections/mark.svg"),
            ]
        );
        assert_eq!(
            rendered.assets,
            Some(vec![
                "sections/introduction.md".to_string(),
                "sections/method.md".to_string(),
                "sections/results.md".to_string(),
                "sections/dot.png".to_string(),
                "sections/mark.svg".to_string(),
            ])
        );
    }

    /// The panel's list is the master's own, in the order the master reads it.
    ///
    /// [`every_file_a_master_names_is_read_once_across_both_passes`] already
    /// pins that order — as the *prefix of `assets`*, which is a list of paths
    /// to watch and not a list of parts to draw. This asserts the field the
    /// panel actually reads, so the two are checked to be one answer arriving
    /// twice rather than one assumed from the other. It is a plain `Vec` where
    /// `assets` is an `Option`, and the type is the claim: a panel draws what
    /// the text names, where `assets`' `None` says something about a watch
    /// filter that a panel cannot draw.
    #[test]
    fn the_sections_a_master_names_are_its_parts_in_master_order() {
        let dir = fixture("");
        let markdown = std::fs::read_to_string(fixture("multi_file.md")).unwrap();
        let rendered = render(&dir, &markdown);

        assert!(rendered.pdf.is_ok(), "{:?}", rendered.pdf.as_ref().err());
        assert_eq!(
            rendered.sections,
            [
                "sections/introduction.md",
                "sections/method.md",
                "sections/results.md",
            ]
        );
    }

    /// A master whose sections are not on the disk still names them.
    ///
    /// The list is read off the *text* and never off the read, which is what
    /// lets the panel name the parts of a document that will not compile —
    /// `md2pdf_core::section_paths` walks the markers and the asset walk only
    /// borrows the result. True by construction today and asserted here because
    /// an implementer who took the list from `read_sections_with`'s answer
    /// instead would pass every other case in this module and empty the panel
    /// exactly when the author most needs to see which file is missing.
    #[test]
    fn a_master_whose_sections_are_missing_still_names_them() {
        let dir = scratch_dir("sections-absent-still-named");
        let markdown = std::fs::read_to_string(fixture("multi_file.md")).unwrap();
        let rendered = render(&dir, &markdown);

        assert!(rendered.pdf.is_err(), "no section is on disk here");
        assert_eq!(
            rendered.sections,
            [
                "sections/introduction.md",
                "sections/method.md",
                "sections/results.md",
            ]
        );
    }

    /// A document that names no section names none, and that is an answer.
    ///
    /// `section_paths` cannot fail — its `Result` is symmetry with the two
    /// shopping lists, not a channel that carries anything — so an empty list
    /// is never a failure to answer and the panel is right to draw nothing.
    /// [`a_single_file_document_keeps_its_anchors_and_its_bytes`] asserts the
    /// `assets` analogue of this and is deliberately not the same claim: that
    /// list is `Some(Vec::new())`, an answer wrapped in the channel that can
    /// also say *no answer*, and this one has no such wrapper to be read
    /// through.
    #[test]
    fn a_document_that_names_no_section_has_no_parts() {
        let markdown = std::fs::read_to_string(fixture("basic.md")).unwrap();
        let rendered = render(&fixture(""), &markdown);

        assert!(rendered.pdf.is_ok(), "{:?}", rendered.pdf.as_ref().err());
        assert!(rendered.sections.is_empty());
    }

    /// Only the headings written in the file the pane holds become anchors.
    ///
    /// Both directions, because an unasserted absence is what
    /// `md2pdf_core::anchors_from`'s count guard punishes silently.
    /// `tests/fixtures/multi_file.md` is a pure manifest and `core` answers it
    /// with three anchors — `(sections/introduction.md, 1)`,
    /// `(sections/method.md, 4)` and `(sections/results.md, 1)`, pinned by
    /// `core/tests/golden_test.rs:an_anchor_names_the_file_its_heading_was_written_in`
    /// — so an empty list here is the filter working rather than `core`
    /// declining to answer. Left in, those three numbers would send
    /// `app/dist/index.html:caretPage`, which walks a flat list, to whatever
    /// page the last of them landed on.
    #[test]
    fn only_the_headings_the_pane_holds_become_anchors() {
        let manifest = render_fixture("multi_file.md");
        assert!(manifest.pdf.is_ok(), "{:?}", manifest.pdf.as_ref().err());
        assert!(manifest.anchors.is_empty(), "{:?}", manifest.anchors);

        let dir = scratch_dir("master-with-a-heading");
        std::fs::create_dir_all(dir.join("sections")).unwrap();
        std::fs::write(
            dir.join("sections/one.md"),
            "# A heading the pane does not hold\n\nText.\n",
        )
        .unwrap();

        let markdown = "# A preface of the master's own\n\nText.\n\n[](sections/one.md)\n";
        let rendered = render(&dir, markdown);

        assert!(rendered.pdf.is_ok(), "{:?}", rendered.pdf.as_ref().err());
        assert_eq!(rendered.anchors, [Anchor { line: 1, page: 1 }]);
    }

    /// A single-file document is what it always was: the same anchors, and the
    /// same bytes.
    ///
    /// [`render_with`] now calls `md2pdf_core::section_paths` and threads a
    /// section array on *every* compile. A document naming no section has a
    /// one-entry map, so nothing is joined, nothing is prefixed and no anchor
    /// carries a file — arithmetic rather than a branch, and asserted here
    /// rather than assumed. The bytes are compared against a compile this test
    /// asked for itself, so an app that quietly agreed with itself could not
    /// pass.
    #[test]
    fn a_single_file_document_keeps_its_anchors_and_its_bytes() {
        let markdown = std::fs::read_to_string(fixture("basic.md")).unwrap();
        let rendered = render(&fixture(""), &markdown);

        let lines: Vec<usize> = rendered.anchors.iter().map(|anchor| anchor.line).collect();
        assert_eq!(lines, [1, 5, 10, 14, 16, 18]);
        assert_eq!(rendered.assets, Some(Vec::new()));
        assert_eq!(
            rendered.pdf.unwrap(),
            md2pdf_core::md_to_pdf(&markdown, &[]).unwrap()
        );
    }

    /// Clause 2. Discovery is the `.md` files whose text names sections.
    #[test]
    fn discovery_is_every_markdown_that_names_a_section() {
        assert_eq!(masters(&loaded(&fixture("panel"))), ["book.md"]);
        assert_eq!(masters(&loaded(&fixture("panel/loose"))), Vec::<String>::new());
        assert_eq!(masters(&loaded(&fixture("panel-pair"))), ["alpha.md", "beta.md"]);
    }

    /// **A master in a subdirectory belongs to another project, not this root.**
    ///
    /// `samples/` is the case that proves it and the case that caught it: a
    /// single-file document sits there beside the whole `showcase/` project, so
    /// a discovery that recursed found `showcase/showcase.md`, called it the one
    /// master, and compiled the showcase for an author who opened
    /// `article.md` — which the window gate reported and no fixture had.
    ///
    /// It is a property rather than a preference. A master cannot name a
    /// section above itself, `landed_path` refusing a marker that climbs out of
    /// the document's folder, so every section sits at or below its master and
    /// a master is never below its own sections. `project_root` answers
    /// "above"; this answers "at"; below is somebody else's document.
    ///
    /// **The shape of a real tree, not a tidy fixture**, because the defect was
    /// that the fixtures were all tidier than the repository the app is
    /// developed in. It was the engine's own `samples/` until `mpdf-011`
    /// Phase 1; it is now a frozen copy of it under
    /// `tests/fixtures/samples/`, holding exactly what makes the case — one
    /// single-file document at the top and one whole project one level down.
    #[test]
    fn a_master_in_a_subdirectory_is_not_this_roots_master() {
        let samples =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/samples");

        assert!(
            samples.join("showcase/showcase.md").is_file(),
            "the tree this case is about has moved"
        );
        assert_eq!(
            masters(&loaded(&samples)),
            Vec::<String>::new(),
            "a project one directory down was taken for this root's own"
        );
        assert_eq!(
            discover_main(&loaded(&samples), "article.md"),
            "article.md",
            "opening a single-file document compiled somebody else's master"
        );

        // And the showcase, opened as itself, still finds its own.
        let showcase = samples.join("showcase");
        assert_eq!(masters(&loaded(&showcase)), ["showcase.md"]);
        assert_eq!(
            discover_main(&loaded(&showcase), "sections/text.md"),
            "showcase.md"
        );
    }

    /// **`mpdf-011` Phase 1's exit gate, clause 4: the page does not move.**
    ///
    /// The split carried this app into a repository of its own and made the
    /// engine a dependency by revision rather than a sibling by path. What that
    /// could silently break is the *bytes* — a font that stops being found, a
    /// look whose bytes the crate no longer embeds — and nothing else in this
    /// suite would say so, because every other test here asserts what the app
    /// does with a document rather than what the document compiles to.
    ///
    /// So this writes the showcase's PDF where a shell can hash it, and the
    /// gate compares that hash against the engine's own `md2pdf` over the
    /// engine's own `samples/showcase/showcase.md` at the split commit. The
    /// identity is sound to key a gate to: `PdfOptions::default()` carries no
    /// timestamp and an `Auto` document id, and nothing in the looks sets a
    /// date.
    ///
    /// **`md_to_pdf`, and the two passes in `cli/src/main.rs:run`'s order** —
    /// sections first, then the assets — because the CLI on the other side of
    /// the comparison is what this has to agree with. [`render_with`] calls
    /// `md_to_pdf_with_anchors` instead, which is a different question.
    ///
    /// `#[ignore]`d and run deliberately, as the page test's own blessing
    /// counterpart is: it writes a file, and a gate rather than a suite is what
    /// reads it.
    #[test]
    #[ignore = "writes the showcase PDF for mpdf-011 Phase 1's gate; run with --ignored"]
    fn the_showcase_compiles_to_the_bytes_the_engine_writes() {
        let showcase = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures/samples/showcase");
        let document = showcase.join("showcase.md");
        let markdown = std::fs::read_to_string(&document).unwrap();

        let sections =
            read_sections_with(&markdown, &showcase, |file| std::fs::read(file)).unwrap();
        let assets =
            read_assets_with(&markdown, sections, &showcase, &Fetched::new(), |file| std::fs::read(file)).unwrap();
        let pdf = md2pdf_core::md_to_pdf(&markdown, &assets).unwrap();

        let out = std::env::temp_dir().join("letur-mpdf-011-phase1-showcase.pdf");
        std::fs::write(&out, &pdf).unwrap();
        println!("{}", out.display());
    }

    /// Clause 3's discovery half. The store is read by the session, not here;
    /// what this pins is the answer when nothing is remembered.
    #[test]
    fn the_main_is_the_one_master_or_the_first_of_several() {
        let panel = fixture("panel");
        let pair = fixture("panel-pair");

        // One master: whatever the author opened, the master compiles.
        assert_eq!(
            discover_main(&loaded(&panel), "sections/text.md"),
            "book.md"
        );
        assert_eq!(discover_main(&loaded(&panel), "book.md"), "book.md");

        // None: the file the author opened, which is every single-file document
        // and is this app's behaviour before the panel existed.
        let loose = panel.join("loose");
        assert_eq!(discover_main(&loaded(&loose), "orphan.md"), "orphan.md");

        // Several: the opened file when it is one of them, else the byte-wise
        // first — a claim that the same folder opens the same way twice, not a
        // claim about which is right.
        assert_eq!(discover_main(&loaded(&pair), "note.md"), "alpha.md");
        assert_eq!(discover_main(&loaded(&pair), "beta.md"), "beta.md");
    }

    /// The path arithmetic the panel's union needs: a master that does not sit
    /// at the root names its sections relative to itself.
    #[test]
    fn a_section_is_named_beside_the_master_that_reads_it() {
        assert_eq!(beside("book.md", "sections/text.md"), "sections/text.md");
        assert_eq!(beside("parts/book.md", "text.md"), "parts/text.md");
        assert_eq!(
            beside("parts/book.md", "../shared/text.md"),
            "shared/text.md"
        );
    }

    /// `beside` run backwards, and the file it cannot reach.
    #[test]
    fn a_file_in_the_pane_is_named_the_way_the_master_would_name_it() {
        let root = Path::new("/p");

        assert_eq!(
            under(&root.join("book.md"), &root.join("sections/text.md")).as_deref(),
            Some("sections/text.md")
        );
        assert_eq!(
            under(&root.join("parts/book.md"), &root.join("parts/text.md")).as_deref(),
            Some("text.md"),
            "a master in a subdirectory names its neighbour by its bare name"
        );
        assert_eq!(
            under(&root.join("parts/book.md"), &root.join("README.md")),
            None,
            "a file the master's own directory does not reach has no such spelling"
        );
        assert_eq!(
            spell(root, &root.join("sections/text.md")).as_deref(),
            Some("sections/text.md")
        );
        assert_eq!(spell(root, root), None, "the root is not under itself");
    }

    /// Whose headings become anchors, all three answers.
    ///
    /// **The third is why [`Pane`] is not an `Option`.** `Master` and `Away`
    /// would collapse into one absence, and the master's own heading lines would
    /// then be handed to a buffer whose line 1 is a different sentence
    /// altogether — which is the exact defect this filter exists to prevent,
    /// arriving through the case that looks like nothing.
    #[test]
    fn the_anchors_are_the_headings_of_the_file_the_pane_is_holding() {
        let dir = scratch_dir("pane-arms");
        std::fs::create_dir_all(dir.join("sections")).unwrap();
        std::fs::write(
            dir.join("sections/one.md"),
            "\n# The section's own heading\n\nText.\n",
        )
        .unwrap();

        let markdown = "# A preface of the master's own\n\nText.\n\n[](sections/one.md)\n";
        let lines = |pane| {
            render_with(&dir, markdown, pane, &Fetched::new(), |file: &Path| std::fs::read(file))
                .anchors
                .into_iter()
                .map(|anchor| anchor.line)
                .collect::<Vec<usize>>()
        };

        assert_eq!(lines(Pane::Master), [1], "the master's own heading");
        assert_eq!(
            lines(Pane::Beside("sections/one.md")),
            [2],
            "the section's own heading, at its own line inside its own file"
        );
        assert!(
            lines(Pane::Away).is_empty(),
            "no heading in this document is a number about a file it does not reach"
        );
    }

    /// The pane's buffer stands in for the one file it is holding, and the rest
    /// of the document comes off the disk.
    ///
    /// The claim is checked against a compile this test asked for itself, after
    /// putting the buffer on the disk — so an app that quietly agreed with
    /// itself could not pass.
    #[test]
    fn the_panes_buffer_stands_in_for_the_file_it_is_holding() {
        let mut files = MemFiles::new([
            ("book.md".to_string(), b"# Book\n\n[](sections/one.md)\n".to_vec()),
            (
                "sections/one.md".to_string(),
                b"# On disk\n\nThe disk's own text.\n".to_vec(),
            ),
        ]);

        let buffer = "# In the pane\n\nText nobody has saved.\n";
        let unsaved = render_project(&files, "book.md", "sections/one.md", buffer, &Fetched::new())
            .expect("the master would not read");

        files.write("sections/one.md", buffer.as_bytes()).unwrap();
        let master = read_document(&files, "book.md").unwrap();
        let saved = render_project(&files, "book.md", "book.md", &master, &Fetched::new())
            .expect("the master would not read");

        assert_eq!(
            unsaved.pdf.expect("the unsaved compile failed"),
            saved.pdf.expect("the saved compile failed"),
            "the buffer reached the compile, or it did not"
        );
    }

    /// A `main` this app cannot read fails in [`read_document`]'s own sentence.
    ///
    /// Both classes, because the closure yields bytes where the compile wants a
    /// string and only one of the two is an `io::Error` to begin with. The
    /// messages are compared against `read_document`'s rather than spelled out
    /// again, which is the claim: a main that will not read reads the same in
    /// the window whichever path reached it.
    ///
    /// **Only reachable while the pane holds another file**, and that is the
    /// mechanism working rather than a gap in the test. With the pane on the
    /// main the closure answers from the buffer and never touches the disk, so
    /// there is nothing there to fail — the buffer is the document, which is
    /// what this app has meant since `mpdf-003` Phase 2.
    #[test]
    fn a_main_that_will_not_read_says_so_in_the_terminals_own_words() {
        let files = MemFiles::new([
            ("held.md".to_string(), b"# Held\n".to_vec()),
            ("binary.md".to_string(), vec![0xff, 0xfe, 0x00]),
        ]);

        assert_eq!(
            render_project(&files, "nothing.md", "held.md", "# Held\n", &Fetched::new()).err(),
            read_document(&files, "nothing.md").err()
        );
        assert_eq!(
            render_project(&files, "binary.md", "held.md", "# Held\n", &Fetched::new()).err(),
            read_document(&files, "binary.md").err()
        );

        assert!(
            render_project(
                &files,
                "nothing.md",
                "nothing.md",
                "# Text the disk never held\n",
                &Fetched::new()
            )
            .is_ok(),
            "the pane holds the main, so the store is not consulted at all"
        );
    }

    /// **Phase 8's whole observable.** The pane's unsaved bibliography reaches
    /// the *citation* pass, and not merely the section reads.
    ///
    /// [`read_assets_with`] takes the same closure [`read_sections_with`]
    /// borrowed and asks it for `directory.join(&named.path)`, so the
    /// bibliography is one more path the override answers — there is no branch
    /// here to test and that is the finding. What is tested is the outcome:
    /// against a compile of the same tree, at the same absolute paths, after
    /// the buffer has been put on the disk.
    ///
    /// **A scratch copy and not the tracked fixture**, because this clause
    /// writes. Only the files `book.md` names or declares are copied — the rest
    /// of `tests/fixtures/panel/` is the listing's concern and not the
    /// compile's.
    #[test]
    fn the_panes_unsaved_bibliography_reaches_the_compile() {
        let panel = fixture("panel");
        let mut files = MemFiles::new(
            [
                "book.md",
                "refs.bib",
                "sections/text.md",
                "sections/mark.svg",
                "parts/ch1/deep.md",
            ]
            .map(|path| (path.to_string(), std::fs::read(panel.join(path)).unwrap())),
        );

        // The one field the reference list prints, so a compile that ignored
        // the buffer would draw the tracked title instead.
        let on_disk = read_document(&files, "refs.bib").unwrap();
        let buffer = on_disk.replace("A Book the Panel Lists", "A Title Nobody Has Saved");
        assert_ne!(
            buffer, on_disk,
            "the buffer matches the store, so this clause proves nothing"
        );

        let unsaved = render_project(&files, "book.md", "refs.bib", &buffer, &Fetched::new())
            .expect("the master would not read");

        files.write("refs.bib", buffer.as_bytes()).unwrap();
        let master = read_document(&files, "book.md").unwrap();
        let saved = render_project(&files, "book.md", "book.md", &master, &Fetched::new())
            .expect("the master would not read");

        assert_eq!(
            unsaved.pdf.expect("the unsaved compile failed"),
            saved.pdf.expect("the saved compile failed"),
            "the pane's bibliography did not reach the citation pass"
        );
    }
}
