//! Images the document names by URL: which sites the author allowed, the
//! fetch, and what came back.
//!
//! `mpdf-003` Phase 25. **`core` fetches nothing**, since `md2pdf-core` 0.3 made a
//! URL a name the caller fills, and this app fills it only for a site the
//! author allowed by pressing the button beside the refusal. Nothing here is
//! fetched before that press, and nothing else in the app reaches the network.
//!
//! **The fetch is `cli/src/main.rs:fetch_agent` and `fetch_image` again**, with
//! the same limits and the same words, so a URL that will not fetch is refused
//! at the window in the sentence the terminal prints under `--fetch`. They are
//! copied rather than shared for `crate::document::read_assets_with`'s reason:
//! the two wrappers report their errors differently, and a helper crate for
//! forty lines would buy less than it costs.
//!
//! **The bytes live in memory, for the process, and never on disk.** One fetch
//! per URL per launch is cheap, and a disk cache would be a second store with
//! its own staleness, eviction and location, keeping a site's bytes after the
//! author stopped trusting it. The CLI caches nothing either, so an export and
//! `md2pdf --fetch` agree whenever the site serves the same bytes.
//!
//! **The rules left for `letur-project` in `ltr-001` Phase 1**: which sites
//! are allowed, where each URL is, what a compile reads and the line the window
//! draws are `project/src/remote.rs`'s, and a browser with no fetch answers
//! through them too. What is here is what reaches the network — the limits, the
//! settle, the agent and the fetch.

use std::sync::OnceLock;
use std::time::Duration;

/// The CLI's own limits, value for value: a global timeout that bounds the
/// body read as well as the connect, the size cap, and the redirect count set
/// although it is also `ureq`'s default, so a version bump cannot move it.
const FETCH_TIMEOUT: Duration = Duration::from_secs(30);
const FETCH_LIMIT: u64 = 20 * 1024 * 1024;
const MAX_REDIRECTS: u32 = 10;

/// How long a URL newly named on an allowed site waits before it is fetched.
///
/// **It is what made eight in-place edits of a URL one request** in the window
/// where Phase 25 was prototyped: a URL that stops being named inside it is
/// dropped rather than fetched, so a path the author is still typing is never
/// sent to the site. **Only the press skips it.**
pub const SETTLE: Duration = Duration::from_secs(1);

/// `cli/src/main.rs:fetch_agent`, built once for the process where the CLI
/// builds one per run. No cookie store and no `gzip` are the crate's features,
/// set in `app/Cargo.toml`; every other guard is here.
///
/// **A proxy the environment names is honoured**, and one set in System
/// Settings is not: `ureq` reads `ALL_PROXY`, `HTTPS_PROXY` and `HTTP_PROXY`,
/// and a window launched from Finder has none of them. That is a recorded limit
/// of `mpdf-003` Phase 25.
fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(FETCH_TIMEOUT))
            .max_redirects(MAX_REDIRECTS)
            .http_status_as_error(false)
            .build();
        ureq::Agent::new_with_config(config)
    })
}

/// `cli/src/main.rs:fetch_image`: the bytes, or words that finish the sentence
/// `cannot fetch {url} for the image {location}: …`.
///
/// A status outside 2xx is its code and canonical reason, `404 Not Found`; the
/// cap is `larger than 20 MB`; anything else is `ureq::Error` as it displays
/// itself. `Content-Type` is not read, because `core`'s check of the bytes is
/// the authority on what they hold.
pub fn fetch(url: &str) -> Result<Vec<u8>, String> {
    let mut response = agent().get(url).call().map_err(|e| e.to_string())?;

    let status = response.status();
    if !status.is_success() {
        return Err(match status.canonical_reason() {
            Some(reason) => format!("{} {reason}", status.as_u16()),
            None => status.as_u16().to_string(),
        });
    }

    response
        .body_mut()
        .with_config()
        // `ureq`'s `LimitReader` refuses a body that *reaches* its limit, so a
        // body of exactly `FETCH_LIMIT` bytes needs one byte of headroom.
        .limit(FETCH_LIMIT + 1)
        .read_to_vec()
        .map_err(|e| match e {
            ureq::Error::BodyExceedsLimit(_) => {
                format!("larger than {} MB", FETCH_LIMIT / (1024 * 1024))
            }
            other => other.to_string(),
        })
}
