---
title: desktop-autosave
sources:
  - app/src/watch.rs
  - app/src/preview.rs
  - app/src/main.rs
  - project/src/preview.rs
  - app/dist/index.html
covers: >
  the buffer that writes itself: the interval and the channel it runs on, the
  order Preview::autosave decides in and the five answers it gives, the flag a
  Save-as outside sets, the hold a dialog sets and the flush that ignores it, the
  quit that flushes, the two events the page hears, and the three commands and
  swallowed rejections the page and the browser host meet
max_lines: 60
# **Split out of `rules/desktop-compile.md` on 2026-11-05**, at `mpdf-003` Phase 31:
# that file was at its 300-line cap, and a third interval with its own state did
# not fit under the compile's.
generated: 2026-11-05
---

# Desktop autosave

**What is true right now**, for the write that follows a pause: when it runs, what
may stop it, and what tells the page. The two intervals that compile are
`rules/desktop-compile.md`; the commands are `rules/desktop.md`.

`app/src/watch.rs:AUTOSAVE_DEBOUNCE` is **1500 ms**, a guess inside the one to two
seconds the author asked for (`specs/desktop_app_spec.md` Phase 31's OQ-C), on its
own channel that `Session::edit` nudges beside the typing one: a compile and a write
per pause answer different questions. `arm` rebuilds it with the typing channel and
its closure carries `recompile_with`'s stale-document guard.
`project/src/preview.rs:Preview::autosave` answers `Clean`, `Held`, `Suspended`,
`Wrote` or `Failed`, **in this order**: nothing open; a buffer equal to the
last-saved text (before the disk is read, so a clean buffer over a changed disk is
the watch loop's `Taken`); `suspended`; a divergence already set, which also carries
a refused switch or trash; then the file read and `external_change`, where
`Diverged` sets the divergence and writes nothing. A write error is `Failed`, never
a divergence, and the next pause tries again. **A Save-as outside the project sets
`suspended`**, leaving the buffer dirty against the original; a `save`, a Save-as
inside and a moved pane clear it, and an open rebuilds it false.

**`Session::autosave` honours the hold and `Session::flush` ignores it.** The hold
is an `Arc<AtomicBool>` the page sets around each dialog, cloned into the timer's
closure; releasing it nudges the channel. The flush runs first in `open`,
`set_main` and `set_edited` and on `RunEvent::ExitRequested`, in `Session` and not
the crate, so the browser gets no autosave. `Wrote` and `Failed` reach the page
through `Session::set_on_autosave`; the rest say nothing.

**The page** listens for `autosaved`, placing its string through `sayReceipt`, and
`autosave-failed`, putting its sentence where `fail` does. A `blur` invokes
`autosave` and Open, Save as and Export invoke `autosave_hold(true)` first and
`false` in a `finally`; **every one of those invocations swallows its answer and its
rejection**, because the browser host answers neither command and every other
rejection in the page reaches `fail`. `web/host/host.mjs` is unchanged and sends
neither event, and `rules/web-app.md` records the browser side.
