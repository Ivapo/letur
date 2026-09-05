---
title: A Book the Panel Lists
author: Iva Po
bibliography: refs.bib
---

# The book

This master names two sections and nothing that is missing, so it compiles.
The panel's marked-missing row is a property of the listing rather than of this
file, and `listing` is handed the name it does not find.

It cites `refs.bib` [@panel], so the bibliography is one of the files the
compile reads through the pane's own closure rather than one nobody names.

A sentence naming [another file](other.md) inside it carries an *inline link*
and **not an include**, and the difference is what the pane draws: an include
marker is the whole of its own line, and this one is not.

```rust
fn main() {}
```

[](sections/text.md)

[](parts/ch1/deep.md)
