# Bogenzeit

Bogenzeit (BZ) reads time as position on the Sun's arc instead of as a count of
hours. Solar noon (☉) and solar midnight (○) are the anchors; the readout is in
spans, a hundred per half-arc so that they stretch with the season, and in
kiloseconds, which do not. `18 [3.43 ks] ☉` is eighteen spans and 3.43 ks before
solar noon. The standard is defined in `docs/bogenzeit-spec.md`, and that file
is normative: an implementation that disagrees with it is wrong, and a change to
a definition is a change to the spec first.

## What is here

`src/` is the reference-grade engine and the `bz` command in Rust, with no
dependencies. It computes a reading from a UTC instant and coordinates, offline,
using the NOAA solar position series that every Bogenzeit port implements step
for step.

`reference/bz_reference.py` is the prototype that defines the algorithm, and the
only thing that writes `vectors/bz_vectors.json`. Every port, in whatever
language, passes those vectors: event times within a second, displayed values
exact. Add a vector by adding a case to the reference and regenerating the file,
then carry the file to each port's repository.

Tagbogen, the Apple Watch and iPhone apps that show Bogenzeit, is at
https://github.com/excelano/tagbogen.

## Install and run

    cargo install bz

    bz now --lat 29.76 --lon -95.37
    bz at 2026-11-01T17:08:00Z --lat 29.76 --lon -95.37
    bz day 2026-11-01 --lat 29.76 --lon -95.37

`now` and `at` print the full timestamp, the quarter, the span length today and
the day's events; `day` prints the day's table of anchors, quarter boundaries,
sunrise, sunset and twilights in time order. `--json` gives either as an object;
`bz --help` has the rest.

Coordinates can live in a config file instead, written by `bz place`:

    bz place home 29.76 -95.37
    bz place camp N36.45 W104.95

The first place written is the default, `bz place camp` makes another the
default, and `bz place --forget camp` removes one. Then `bz` alone reads for the
default, `bz --place camp` for another, and `bz places` lists them. The file is
`~/.config/bz/config`, one place per line as `name = lat, lon` with
`place = name` for the default, and can be edited by hand. Coordinates on the
command line still win.
