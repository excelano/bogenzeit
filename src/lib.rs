// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

//! Bogenzeit (BZ): time read as position on the Sun's arc. The crate is
//! `bogenzeit`, imported as `bz`, and depends on nothing.
//!
//! `docs/bogenzeit-spec.md` in the repository is normative. `reading` turns
//! a UTC instant and coordinates into a [`Reading`]; `solar` holds the NOAA
//! series every port shares; `calendar` is the little of a calendar the
//! engine needs.

pub mod calendar;
pub mod reading;
pub mod solar;

pub use calendar::{Date, Instant};
pub use reading::{
    bz_date, Anchor, Eighth, Form, NightPhase, Polar, Position, Precision, Quarter, Reading,
    SolarDay, SolarDuration,
};
pub use solar::HalfArc;
