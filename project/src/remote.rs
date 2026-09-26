//! Images the document names by URL: which sites the author allowed, and what
//! came back — every rule of it, and none of the network.
//!
//! `mpdf-003` Phase 25, moved from `app/src/remote.rs` by `ltr-001` Phase 1.
//! **`core` fetches nothing, and neither does this crate.** The fetch, its
//! limits, the settle it waits out and the worker that carries it are the
//! desktop's, in `app/src/remote.rs` and `app/src/preview.rs`; what is here is
//! the state a compile reads and the line the window draws, which a host with no
//! fetch at all still answers — `ltr-001`'s web session installs no site, so
//! every URL is refused in the engine's own sentence and no line appears.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use serde::Serialize;

/// What a compile is handed: each finished fetch's bytes, or the reason in the
/// CLI's words. The bytes are shared, so building one of these copies no image.
pub type Fetched = HashMap<String, Result<Arc<Vec<u8>>, String>>;

/// The button beside a site not yet allowed.
const ALLOW: &str = "Fetch images from the web";

/// The button beside a fetch that failed, once every site is allowed.
const RETRY: &str = "Try again";

/// A fetch that came back, and which landing it was.
#[derive(Debug, Clone)]
struct Landed {
    bytes: Result<Arc<Vec<u8>>, String>,
    /// Bumped on every landing, so a compile can say which one it read.
    generation: u64,
}

/// Where one URL is, from the moment a compile names it on an allowed site.
///
/// **`Arrived` and `Done` both hold what came back, and the difference is the
/// page.** `Arrived` is bytes no compile has put on the page yet; `Done` is bytes
/// a compile read and landed. The line says *"Fetching"* through `Arrived` so it
/// does not clear a compile before the page it is for.
#[derive(Debug, Clone)]
enum Fetch {
    /// Named on an allowed site, and waiting out the host's settle or its worker.
    Waiting,
    /// A request is out.
    Fetching,
    /// Back, and not yet on the page.
    Arrived(Landed),
    /// Back, and read by a compile that landed.
    Done(Landed),
}

/// The line the page draws above the error, worded here and only placed there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WebLine {
    pub sentence: String,
    /// The button's label, when there is something to press.
    pub action: Option<String>,
}

/// The sites the open project allows, and one state per URL asked about.
///
/// **Consent and the bytes are held apart.** `allowed` is the open project's and
/// is replaced at every open, from `sites.json`; `fetches` is the process's and
/// survives every open. A compile reads only the bytes on sites the *open*
/// project allows, so a second project that never allowed a site does not draw
/// that site's image out of memory while its line says the image is not fetched.
#[derive(Debug, Default)]
pub struct Web {
    allowed: BTreeSet<String>,
    fetches: HashMap<String, Fetch>,
    generation: u64,
}

impl Web {
    /// The open project's sites, replacing the last project's.
    pub fn install(&mut self, allowed: BTreeSet<String>) {
        self.allowed = allowed;
    }

    /// The sites allowed now, and the sites these URLs are on.
    ///
    /// **Computed rather than applied**, so the press can write the set to disk
    /// before it wears it, and a write that fails leaves memory and disk agreeing.
    pub fn widened(&self, urls: &[String]) -> BTreeSet<String> {
        let mut allowed = self.allowed.clone();
        allowed.extend(urls.iter().map(|url| host(url)));
        allowed
    }

    /// Does the open project allow the site this URL is on?
    pub fn allows(&self, url: &str) -> bool {
        self.allowed.contains(&host(url))
    }

    /// Mark every URL on an allowed site that nothing has asked about yet as
    /// [`Fetch::Waiting`], and answer those URLs.
    ///
    /// With `retry`, a URL whose fetch failed and landed is asked about again,
    /// which is the press. **Every claim starts `Waiting`**, the press's
    /// included: the worker's one check in [`Web::begin`] is what moves it on, so
    /// a claim that skipped the settle passes through the same door.
    pub fn claim(&mut self, urls: &[String], retry: bool) -> Vec<String> {
        let mut claimed: Vec<String> = Vec::new();
        for url in urls {
            if !self.allows(url) || claimed.contains(url) {
                continue;
            }
            let start = match self.fetches.get(url) {
                None => true,
                Some(Fetch::Done(Landed { bytes: Err(_), .. })) => retry,
                Some(_) => false,
            };
            if start {
                self.fetches.insert(url.clone(), Fetch::Waiting);
                claimed.push(url.clone());
            }
        }
        claimed
    }

    /// The wait is over: `true` if this URL is to be fetched now.
    ///
    /// It is, if it is still waiting, the text still names it, and the open
    /// project allows its site. Otherwise its state is dropped — so a URL the
    /// author typed past, or one claimed under a project that has since closed,
    /// is neither fetched nor left looking as though it were on its way.
    pub fn begin(&mut self, url: &str, named: bool) -> bool {
        if !matches!(self.fetches.get(url), Some(Fetch::Waiting)) {
            return false;
        }
        if named && self.allows(url) {
            self.fetches.insert(url.to_string(), Fetch::Fetching);
            true
        } else {
            self.fetches.remove(url);
            false
        }
    }

    /// A fetch came back.
    pub fn land(&mut self, url: &str, result: Result<Vec<u8>, String>) {
        self.generation += 1;
        self.fetches.insert(
            url.to_string(),
            Fetch::Arrived(Landed {
                bytes: result.map(Arc::new),
                generation: self.generation,
            }),
        );
    }

    /// What one compile reads: every fetch that came back on a site the open
    /// project allows, and the generation of each, so the compile can say
    /// afterwards exactly what it put on the page.
    pub fn finished(&self) -> (Fetched, Vec<(String, u64)>) {
        let mut fetched = Fetched::new();
        let mut read = Vec::new();
        for (url, fetch) in &self.fetches {
            let (Fetch::Arrived(landed) | Fetch::Done(landed)) = fetch else {
                continue;
            };
            if !self.allows(url) {
                continue;
            }
            fetched.insert(url.clone(), landed.bytes.clone());
            read.push((url.clone(), landed.generation));
        }
        (fetched, read)
    }

    /// A compile that read these landed: what it read is on the page now.
    ///
    /// **Only at the generation it read.** A retry lands a newer generation over
    /// the same URL, and an older compile absorbed after it must not clear the
    /// line for bytes it never compiled.
    pub fn promote(&mut self, read: &[(String, u64)]) {
        for (url, generation) in read {
            if let Some(Fetch::Arrived(landed)) = self.fetches.get(url)
                && landed.generation == *generation
            {
                let landed = landed.clone();
                self.fetches.insert(url.clone(), Fetch::Done(landed));
            }
        }
    }

    /// Is this URL on its way to the page, on a site the open project allows?
    ///
    /// It is what `crate::preview::Preview::status` hides `core`'s refusal on:
    /// while it answers `true` the line says what is happening, and a sentence
    /// saying the image is not fetched would contradict it.
    pub fn on_its_way(&self, url: &str) -> bool {
        matches!(
            self.fetches.get(url),
            Some(Fetch::Waiting | Fetch::Fetching | Fetch::Arrived(_))
        ) && self.allows(url)
    }

    /// Where this URL is, for a test to read without reaching into the state.
    ///
    /// **Public and not `#[cfg(test)]`**, since `ltr-001` Phase 1: the desktop's
    /// fetch-worker cases read it from another crate, where this crate's own
    /// `cfg(test)` is off.
    #[doc(hidden)]
    pub fn stage(&self, url: &str) -> Option<&'static str> {
        self.fetches.get(url).map(|fetch| match fetch {
            Fetch::Waiting => "waiting",
            Fetch::Fetching => "fetching",
            Fetch::Arrived(_) => "arrived",
            Fetch::Done(_) => "done",
        })
    }

    /// What the page says about the images this document names by URL.
    ///
    /// **The first of four that applies**, and the order is what keeps consent
    /// narrow. Both buttons run the same command, which allows every site the
    /// document names — so **Try again** can only appear once no site is
    /// waiting to be allowed, and pressing it allows nothing new.
    ///
    /// 1. A URL on a site not allowed: its count and its sites, and the button.
    /// 2. A URL being fetched, or back and not yet on the page: *"Fetching"*.
    /// 3. A fetch that failed: its count, and **Try again**.
    /// 4. Otherwise nothing — including a URL that is only waiting, which is
    ///    what keeps a URL being typed from flashing a sentence per keystroke.
    pub fn line(&self, urls: &[String]) -> Option<WebLine> {
        let blocked: Vec<&String> = urls.iter().filter(|url| !self.allows(url)).collect();
        if !blocked.is_empty() {
            return Some(WebLine {
                sentence: format!(
                    "{} on {} {} not fetched.",
                    capital(&noun(blocked.len())),
                    sites(&blocked),
                    if blocked.len() == 1 { "is" } else { "are" }
                ),
                action: Some(ALLOW.to_string()),
            });
        }

        let moving: Vec<&String> = urls
            .iter()
            .filter(|url| {
                matches!(
                    self.fetches.get(*url),
                    Some(Fetch::Fetching | Fetch::Arrived(_))
                )
            })
            .collect();
        if !moving.is_empty() {
            return Some(WebLine {
                sentence: format!("Fetching {} from {}…", noun(moving.len()), sites(&moving)),
                action: None,
            });
        }

        let failed = urls
            .iter()
            .filter(|url| {
                matches!(
                    self.fetches.get(*url),
                    Some(Fetch::Done(Landed { bytes: Err(_), .. }))
                )
            })
            .count();
        (failed > 0).then(|| WebLine {
            sentence: format!("{} could not be fetched.", capital(&noun(failed))),
            action: Some(RETRY.to_string()),
        })
    }
}

fn noun(count: usize) -> String {
    match count {
        1 => "1 image".to_string(),
        n => format!("{n} images"),
    }
}

fn capital(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// The sites, in the order the document first names them, as a list a person
/// reads: `a`, `a and b`, `a, b and c`.
fn sites(urls: &[&String]) -> String {
    let mut seen: Vec<String> = Vec::new();
    for url in urls {
        let site = host(url);
        if !seen.contains(&site) {
            seen.push(site);
        }
    }
    match seen.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The site a URL is on: its host, lower-cased. A URL with no host to read is
/// its own site, so it is allowed by name and never by accident.
pub fn host(url: &str) -> String {
    url.parse::<http::Uri>()
        .ok()
        .and_then(|uri| uri.host().map(str::to_ascii_lowercase))
        .unwrap_or_else(|| url.to_string())
}

// A test may read a fixture: `clippy.toml`'s list is about what the crate
// ships, which is what the gate lints.
#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;

    fn urls(named: &[&str]) -> Vec<String> {
        named.iter().map(|url| url.to_string()).collect()
    }

    /// A `Web` allowing these sites, with each URL put where the case wants it.
    fn web(allowed: &[&str], states: &[(&str, &str)]) -> Web {
        let mut web = Web::default();
        web.install(allowed.iter().map(|site| site.to_string()).collect());
        for (url, stage) in states {
            let fetch = match *stage {
                "waiting" => Fetch::Waiting,
                "fetching" => Fetch::Fetching,
                "arrived" => Fetch::Arrived(Landed {
                    bytes: Ok(Arc::new(Vec::new())),
                    generation: 1,
                }),
                "failed" => Fetch::Done(Landed {
                    bytes: Err("503 Service Unavailable".to_string()),
                    generation: 1,
                }),
                other => panic!("no such stage: {other}"),
            };
            web.fetches.insert(url.to_string(), fetch);
        }
        web
    }

    fn said(line: Option<WebLine>) -> Option<(String, Option<String>)> {
        line.map(|line| (line.sentence, line.action))
    }

    /// `mpdf-003` Phase 25, case 1: the line's precedence and every word of it.
    #[test]
    fn the_line_names_the_sites_not_allowed_first_and_says_nothing_while_waiting() {
        let allow = Some(ALLOW.to_string());

        // A site not allowed: one, two and three of them, counting and naming
        // only the URLs on sites not allowed.
        let one = urls(&["https://CDN.example.com/a.png"]);
        assert_eq!(
            said(web(&[], &[]).line(&one)),
            Some((
                "1 image on cdn.example.com is not fetched.".to_string(),
                allow.clone()
            ))
        );
        let two = urls(&[
            "https://a.example/1.png",
            "https://b.example/2.png",
            "https://a.example/3.png",
            "https://allowed.example/4.png",
        ]);
        assert_eq!(
            said(web(&["allowed.example"], &[]).line(&two)),
            Some((
                "3 images on a.example and b.example are not fetched.".to_string(),
                allow.clone()
            ))
        );
        let three = urls(&[
            "https://a.example/1.png",
            "https://b.example/2.png",
            "https://c.example/3.png",
        ]);
        assert_eq!(
            said(web(&[], &[]).line(&three)),
            Some((
                "3 images on a.example, b.example and c.example are not fetched.".to_string(),
                allow.clone()
            ))
        );

        // A site not allowed beside a URL being fetched, and beside a failed
        // one: the first sentence either way, so the press is always there
        // while a site waits to be allowed.
        let mixed = urls(&["https://on.example/1.png", "https://off.example/2.png"]);
        for stage in ["fetching", "arrived", "failed"] {
            assert_eq!(
                said(web(&["on.example"], &[("https://on.example/1.png", stage)]).line(&mixed)),
                Some((
                    "1 image on off.example is not fetched.".to_string(),
                    allow.clone()
                )),
                "beside a URL that is {stage}"
            );
        }

        // Being fetched beside a failed one: "Fetching", with no button.
        let both = urls(&["https://on.example/1.png", "https://on.example/2.png"]);
        let fetching = web(
            &["on.example"],
            &[
                ("https://on.example/1.png", "fetching"),
                ("https://on.example/2.png", "failed"),
            ],
        );
        assert_eq!(
            said(fetching.line(&both)),
            Some(("Fetching 1 image from on.example…".to_string(), None))
        );
        // Back and not yet on the page is still on its way.
        let arrived = web(
            &["on.example"],
            &[
                ("https://on.example/1.png", "arrived"),
                ("https://on.example/2.png", "fetching"),
            ],
        );
        assert_eq!(
            said(arrived.line(&both)),
            Some(("Fetching 2 images from on.example…".to_string(), None))
        );

        // A failure alone: Try again.
        let failed = web(&["on.example"], &[("https://on.example/2.png", "failed")]);
        assert_eq!(
            said(failed.line(&both)),
            Some((
                "1 image could not be fetched.".to_string(),
                Some(RETRY.to_string())
            ))
        );

        // A URL only waiting: no line at all.
        let waiting = web(&["on.example"], &[("https://on.example/1.png", "waiting")]);
        assert_eq!(said(waiting.line(&both[..1])), None);
        assert_eq!(said(Web::default().line(&[])), None);
    }

    /// The site is the host, lower-cased; a URL with no host is its own site.
    #[test]
    fn the_site_is_the_lowercased_host_or_the_url_itself() {
        assert_eq!(host("https://CDN.Example.COM:8443/a/b.png?x=1"), "cdn.example.com");
        assert_eq!(host("http://127.0.0.1:4446/dot.png"), "127.0.0.1");
        assert_eq!(host("https:/example.com/x.png"), "https:/example.com/x.png");
    }

    /// A claim takes an allowed site's unasked URL once, and only a retry takes
    /// a failure back; `begin` drops what the text stopped naming.
    #[test]
    fn a_claim_waits_and_begin_drops_what_is_no_longer_named() {
        let mut web = web(&["on.example"], &[("https://on.example/f.png", "failed")]);
        let named = urls(&[
            "https://on.example/a.png",
            "https://on.example/a.png",
            "https://off.example/b.png",
            "https://on.example/f.png",
        ]);

        assert_eq!(web.claim(&named, false), urls(&["https://on.example/a.png"]));
        assert_eq!(web.stage("https://on.example/a.png"), Some("waiting"));
        assert_eq!(web.claim(&named, false), Vec::<String>::new());
        assert_eq!(web.claim(&named, true), urls(&["https://on.example/f.png"]));

        assert!(web.begin("https://on.example/a.png", true));
        assert_eq!(web.stage("https://on.example/a.png"), Some("fetching"));
        assert!(!web.begin("https://on.example/f.png", false));
        assert_eq!(web.stage("https://on.example/f.png"), None);
    }

    /// A compile reads only the allowed sites' fetches, and promotes only the
    /// generation it read.
    #[test]
    fn a_compile_reads_the_allowed_sites_and_promotes_the_generation_it_read() {
        let mut web = Web::default();
        web.install(["on.example".to_string()].into());
        web.land("https://on.example/a.png", Ok(vec![1]));
        web.land("https://off.example/b.png", Ok(vec![2]));

        let (fetched, read) = web.finished();
        assert_eq!(fetched.len(), 1);
        assert_eq!(read, vec![("https://on.example/a.png".to_string(), 1)]);

        // A retry lands a newer generation before the older compile absorbs.
        web.land("https://on.example/a.png", Ok(vec![3]));
        web.promote(&read);
        assert_eq!(web.stage("https://on.example/a.png"), Some("arrived"));

        let (_, read) = web.finished();
        web.promote(&read);
        assert_eq!(web.stage("https://on.example/a.png"), Some("done"));
        assert!(!web.on_its_way("https://on.example/a.png"));
    }
}
