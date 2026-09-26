//! Letur's window, answered in a browser: the session `web/host/worker.mjs`
//! holds, over a project in memory.
//!
//! **Every rule here is `letur-project`'s.** `ltr-001` Phase 1 moved the
//! panel's order, the main a project opens on, the compile's reads, the pane's
//! state and every refusal and receipt out of the desktop crate so that this
//! file could answer the same window through the same code. What is left here is
//! what the desktop's `app/src/main.rs` commands and `app/src/preview.rs`'s
//! `Session` add around a `Preview`, restated for a host with no disk, no watch,
//! no threads and no clock of its own — one method per command `ltr-001` §2's
//! table gives the session.
//!
//! **Three conventions cross this boundary**, and each is the desktop's reached
//! another way:
//!
//! - a refusal is thrown as the sentence itself, a plain string, because the
//!   window's `fail` prints `String(problem)` and the desktop's
//!   `Result<_, String>` reaches it as exactly that;
//! - a structured answer is a JSON string from `serde_json`, the serializer
//!   Tauri's IPC uses, so `None` is `null` and never `undefined`;
//! - bytes are `Vec<u8>`, which reaches JavaScript as a `Uint8Array`.
//!
//! **A path the window sends is virtual**: `/project/<root-relative path>`. The
//! root is the project, always — the desktop's climb from one opened file
//! exists to guess a root, and a browser project names its own.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use letur_project::document;
use letur_project::files::{Files, MemFiles};
use letur_project::preview::{Appearance, Asked, Preview, Status};
use wasm_bindgen::prelude::*;

/// Where every project of this phase lives. `ltr-001` Phase 3 makes it the
/// project's own name.
const PROJECT: &str = "/project/";

/// What marks a Save-as or an export the host will deliver as a download.
const DOWNLOAD: &str = "download:";

/// Route a Rust panic to the console instead of an unhelpful `unreachable`.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    /// The worker's clock. `std::time::Instant::now` panics on
    /// `wasm32-unknown-unknown`, which is why `letur-project` reads none.
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

/// How this host times a compile: `performance.now()` either side of it.
///
/// **Handed to `Preview::new`**, so the compiles that run *inside* an open, a
/// row click or a discard are timed exactly as the desktop's `Instant` timer
/// times them — a duration passed in by the host could never reach those.
fn timed(run: &mut dyn FnMut()) -> Duration {
    let started = now();
    run();
    Duration::from_secs_f64(((now() - started) / 1000.0).max(0.0))
}

/// The refusal as the window's `fail` prints it.
fn refused(sentence: String) -> JsValue {
    JsValue::from_str(&sentence)
}

fn nothing_open() -> JsValue {
    refused("no document is open".to_string())
}

/// A virtual path as the project spells it.
fn relative(path: &str) -> &str {
    path.strip_prefix(PROJECT).unwrap_or(path)
}

/// The name a download token carries, or the refusal for a path without one.
///
/// **Recognised before any crate call**: a `download:notes.md` handed to
/// `Preview::save_as` would be taken for a root-relative name, written into
/// the project and followed by the pane.
fn downloaded(path: &str) -> Result<&str, JsValue> {
    path.strip_prefix(DOWNLOAD)
        .ok_or_else(|| refused("a browser saves a copy only as a download".to_string()))
}

/// What a Save-as or an export hands the host to deliver.
#[wasm_bindgen(getter_with_clone)]
pub struct Download {
    pub bytes: Vec<u8>,
    pub receipt: String,
}

/// The window's backend: one project in memory, the pane over it, and the
/// appearance the host hands over.
#[wasm_bindgen]
pub struct Session {
    preview: Preview<MemFiles>,
    /// The files the next open reads, as the host fetched or was handed them.
    staged: BTreeMap<String, Vec<u8>>,
    /// **Held here and not in `Preview`**, for the desktop `Session`'s reason:
    /// it is global, and an open rebuilds the `Preview` whole.
    appearance: Appearance,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl Session {
    /// A session with nothing open. **No claim channel**: images by URL are not
    /// fetched in a browser (`ltr-001` §1.2), and a `Preview` with no claim
    /// marks nothing waiting.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Session {
        Session {
            preview: Preview::new(timed, None),
            staged: BTreeMap::new(),
            appearance: Appearance::System,
        }
    }

    /// One file into the next open.
    pub fn stage(&mut self, path: &str, bytes: Vec<u8>) {
        self.staged.insert(relative(path).to_string(), bytes);
    }

    /// Forget whatever was staged, so the next stage is the next open alone.
    pub fn unstage(&mut self) {
        self.staged.clear();
    }

    /// Open the staged files as a project, on the main `discover_main` picks.
    ///
    /// **With nothing staged, the open project is opened again**, which is what
    /// the desktop's Open of a file in the folder already open does: read, main
    /// rediscovered, counters reset.
    pub fn open_document(&mut self, path: &str) -> Result<(), JsValue> {
        let here = relative(path);
        let files = if self.staged.is_empty() {
            self.preview.files().cloned().ok_or_else(nothing_open)?
        } else {
            MemFiles::new(std::mem::take(&mut self.staged))
        };
        if !files.holds(here) {
            return Err(refused(document::not_a_file(here)));
        }
        let main = document::discover_main(&files, here);
        self.preview.open(files, main.clone(), main, BTreeSet::new());
        self.preview.load();
        Ok(())
    }

    /// The text the pane should be holding.
    pub fn document_text(&self) -> String {
        self.preview.text().to_string()
    }

    /// Take what the author has typed. **It compiles nothing**: the host's
    /// debounce sends [`Session::compile`] once the typing stops.
    pub fn edit(&mut self, text: String) {
        self.preview.edit(text);
    }

    /// Compile what the pane holds, timed by [`timed`].
    pub fn compile(&mut self) {
        self.preview.compile();
    }

    /// Everything the window says about the last compile, as JSON.
    ///
    /// **`web` is `null`, always**: there is no fetch in a browser, so the line
    /// that offers one never appears, and a URL image is refused in
    /// `md2pdf-core`'s own sentence.
    pub fn status(&self) -> Result<String, JsValue> {
        let status = Status {
            appearance: self.appearance,
            web: None,
            ..self.preview.status()
        };
        serde_json::to_string(&status).map_err(|e| refused(e.to_string()))
    }

    /// The page's bytes, or the reason the page on screen is out of date.
    ///
    /// The fallback sentence is `app/src/main.rs:current_pdf`'s, restated word
    /// for word — one of the two literals `ltr-001` §2 records as the host's on
    /// both hosts.
    pub fn current_pdf(&self) -> Result<Vec<u8>, JsValue> {
        if self.preview.is_stale() {
            let error = self.preview.error().unwrap_or("the page is out of date");
            return Err(refused(error.to_string()));
        }
        self.preview.pdf().map(<[u8]>::to_vec).ok_or_else(nothing_open)
    }

    /// The bytes of one of the project's files, for the page to draw.
    pub fn asset_bytes(&self, path: &str) -> Result<Vec<u8>, JsValue> {
        let files = self.preview.files().ok_or_else(nothing_open)?;
        document::asset_bytes(files, relative(path)).map_err(refused)
    }

    /// Make another file the one that compiles, and open the project on it.
    ///
    /// Granted, it is an open — the same files, this main — which resets the
    /// counters as the desktop's reopen does and as the window's `setMain`
    /// `clear()` expects. Refused, the divergence carries `SWITCHING` and
    /// nothing moved.
    pub fn set_main(&mut self, path: &str) -> Result<(), JsValue> {
        let main = relative(path).to_string();
        if self.preview.ask_main(&main).map_err(refused)? == Asked::Refused {
            return Ok(());
        }
        let files = self.preview.files().cloned().ok_or_else(nothing_open)?;
        self.preview.open(files, main.clone(), main, BTreeSet::new());
        self.preview.load();
        Ok(())
    }

    /// Put another of the project's files in the pane.
    pub fn set_edited(&mut self, path: &str) -> Result<(), JsValue> {
        self.preview.set_edited(relative(path)).map_err(refused)?;
        Ok(())
    }

    /// Drop the pane's edits and take the file again.
    pub fn discard(&mut self) {
        self.preview.discard();
    }

    /// Write the pane into the project, and answer the receipt.
    pub fn save(&mut self) -> Result<String, JsValue> {
        self.preview.save().map_err(refused)
    }

    /// Make one empty file in the project.
    ///
    /// **The listing is refreshed here**, where the desktop leaves it to the
    /// watch that sees the file appear: there is no watch in a browser.
    pub fn create_file(&mut self, path: &str) -> Result<(), JsValue> {
        let files = self.preview.files_mut().ok_or_else(nothing_open)?;
        document::create_file(files, relative(path)).map_err(refused)?;
        self.preview.refresh_tree();
        Ok(())
    }

    /// Delete one of the project's files. **Permanently**: the map has no
    /// Trash, and the window's sentence that says otherwise is `ltr-001`
    /// OQ-7's.
    pub fn trash_file(&mut self, path: &str) -> Result<(), JsValue> {
        self.preview
            .trash(relative(path), document::remove_file)
            .map_err(refused)?;
        Ok(())
    }

    /// Where the Save-as prompt starts: the file the pane holds.
    pub fn save_as_path(&self) -> Result<String, JsValue> {
        self.preview
            .edited()
            .map(|edited| format!("{PROJECT}{edited}"))
            .ok_or_else(nothing_open)
    }

    /// The pane's text as a download: `Preview::download`, nothing moved.
    pub fn save_as(&self, path: &str) -> Result<Download, JsValue> {
        let name = downloaded(path)?;
        let downloaded = self.preview.download(name).map_err(refused)?;
        Ok(Download {
            bytes: downloaded.bytes,
            receipt: downloaded.receipt,
        })
    }

    /// The export's suggested name, or why there is no export.
    pub fn export_path(&self) -> Result<String, JsValue> {
        self.preview.export_path().map_err(refused)
    }

    /// The page's bytes as a download, refused in `exportable`'s words.
    pub fn export(&self, path: &str) -> Result<Download, JsValue> {
        downloaded(path)?;
        let pdf = self.preview.exportable().map_err(refused)?;
        Ok(Download {
            bytes: pdf.to_vec(),
            receipt: String::new(),
        })
    }

    /// Which palette the window wears: `"system"`, `"light"` or `"dark"`,
    /// spelled as the status spells it.
    pub fn set_appearance(&mut self, value: &str) -> Result<(), JsValue> {
        self.appearance = serde_json::from_value(serde_json::Value::String(value.to_string()))
            .map_err(|e| refused(e.to_string()))?;
        Ok(())
    }
}
