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
only thing that writes `vectors/bz_vectors.json`. A conforming port, in
whatever language, passes those vectors: event times within a second, displayed
values exact. Add a vector by adding a case to the reference and regenerating
the file, then carry the file to each conforming port's repository.
[bogenzeit-js](https://github.com/excelano/bogenzeit-js), for a clock on a web
page, uses its own approximation and does not claim conformance.

Bogentag, the Apple Watch and iPhone apps that show Bogenzeit, is at
https://github.com/excelano/bogentag.

## Install and run

On Debian and Ubuntu, from the Excelano apt repository:

    curl -fsSL https://excelano.com/apt/setup.sh | sudo sh && sudo apt install bz

Anywhere with a Rust toolchain:

    cargo install bz

Then:

    bz now --lat 29.76 --lon -95.37
    bz at 2026-11-01T17:08:00Z --lat 29.76 --lon -95.37
    bz day 2026-11-01 --lat 29.76 --lon -95.37
    bz between 2026-11-01T12:00:00Z 2026-11-01T17:08:00Z --lat 29.76 --lon -95.37

`now` and `at` print the full timestamp, the quarter and eighth, the span length
today and the day's events; `--form span`, `offset` or `combined` picks the
written form of the position, and `--precision low`, `medium` or `high` its
steps. `day` prints the day's table of anchors, horizon events, eighth
boundaries and twilights in time order, with the quarter or eighth each one
begins. `between` gives the time between two instants in spans and ks, or with
one instant the time from now, as `in 24 spans` or `23 spans ago`. `--json`
gives any of them as an object; `bz --help` has the rest.

Coordinates can live in a config file instead, written by `bz place`:

    bz place home 29.76 -95.37
    bz place camp N36.45 W104.95

The first place written is the default, `bz place camp` makes another the
default, and `bz place --forget camp` removes one. Then `bz` alone reads for the
default, `bz --place camp` for another, and `bz places` lists them. The file is
`~/.config/bz/config`, one place per line as `name = lat, lon` with
`place = name` for the default, and can be edited by hand. Coordinates on the
command line still win.

## License

MIT. See [LICENSE](LICENSE).
