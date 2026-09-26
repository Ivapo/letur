//! `letur-project` — the rules Letur's window answers through.
//!
//! `ltr-001` Phase 1 moved them here from the desktop crate, so the web
//! session answers the same window through the same code rather than a second
//! implementation of every refusal, receipt and counter. **Nothing in this
//! crate touches a file, reads a clock or starts a thread**; `clippy.toml`
//! beside it enumerates what that means, and every file is reached through
//! [`files::Files`].
//!
//! - [`files`] — the one door to a project's files, and [`files::MemFiles`].
//! - [`document`] — one compile's reads and the sentences they refuse in, the
//!   panel's entries, and the main a project opens on.
//! - [`preview`] — the pane's state, its status, and every decision a command
//!   makes about it.
//! - [`remote`] — the images named by URL: which sites are allowed, and what
//!   came back.

pub mod document;
pub mod files;
pub mod preview;
pub mod remote;
