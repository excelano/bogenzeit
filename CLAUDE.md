# CLAUDE.md

Bogenzeit is a time standard. `docs/bogenzeit-spec.md` is normative and no definition in it changes
without David. This crate, `bz`, is its reference-grade engine and command; `reference/bz_reference.py`
defines the algorithm and is the only writer of `vectors/bz_vectors.json`, which every port must pass.
The Swift engine for the Apple apps is excelano/bogenzeit-swift, which carries its own copy of the
vectors; a regenerated file goes there too. HANDOFF.md is the brief the work started from.

## Commands

    cargo build
    cargo test                                  # unit tests, the vectors, the boundary steps
    cargo clippy --all-targets -- -D warnings   # must be silent
    cargo fmt --check
    cargo run -- at 2026-11-01T17:08:00Z --lat 29.76 --lon -95.37
    python3 reference/bz_reference.py --vectors > vectors/bz_vectors.json   # after adding a case

The build directory is the shared one in `~/.cargo/config.toml`, not `target/`. Releases: the apps in
excelano/shipping, run from this directory: `build-release`, then `ship-crates` (crates.io) and
`apt-ship` (the deb).

## Rules

A port reproduces the reference step for step, including the declination being taken at solar noon's
UTC hour and minute with the seconds dropped; a port that is "more accurate" fails the vectors.
Displayed spans and kiloseconds are the floor of the signed value, so an anchor or event shows only
once it has been reached. Position replaces sign: number left of the glyph is before, right is after,
no `+` or `−`. The Bogenzeit date changes at solar midnight. Quarter boundaries are start inclusive.
The engine depends on nothing and runs offline. Don't commit, push or publish unless David asks.
