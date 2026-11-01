// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

//! The floor rule (Bogenzeit spec §5.2) and the day switchover (§5.1),
//! checked by stepping across each boundary: the display changes at the
//! boundary and never before it.

use bz::{Instant, Quarter, Reading, SolarDay};

const HOUSTON: (f64, f64) = (29.76, -95.37);

fn houston(t: Instant) -> Reading {
    Reading::at(t, HOUSTON.0, HOUSTON.1)
}

fn houston_day() -> SolarDay {
    houston(Instant::parse_rfc3339("2026-11-01T17:08:00Z").unwrap()).day
}

#[test]
fn noon_glyph_appears_only_from_exact_solar_noon() {
    let noon = houston_day().noon;
    let before = houston(noon - 0.001);
    assert_eq!(before.span_form().unwrap(), "1 ☉");
    assert_eq!(before.offset_form(), "0.01 ks ☉");
    assert_eq!(before.combined_form().unwrap(), "1 [0.01 ks] ☉");
    assert_eq!(before.quarter(), Some(Quarter::Forenoon));
    let at = houston(noon);
    assert_eq!(at.span_form().unwrap(), "☉");
    assert_eq!(at.offset_form(), "☉");
    assert_eq!(at.combined_form().unwrap(), "☉");
    assert_eq!(at.quarter(), Some(Quarter::Afternoon));
    let after = houston(noon + 150.0);
    assert_eq!(after.span_form().unwrap(), "☉");
    assert_eq!(after.combined_form().unwrap(), "☉ 0 [0.15 ks]");
}

#[test]
fn sunset_shows_only_at_exact_sunset_and_switches_anchor() {
    let sunset = houston_day().sunset().unwrap();
    let before = houston(sunset - 0.001);
    assert_eq!(before.span_form().unwrap(), "☉ 99");
    assert_eq!(before.quarter(), Some(Quarter::Foreset));
    let at = houston(sunset);
    assert_eq!(at.span_form().unwrap(), "100 ○");
    assert_eq!(at.quarter(), Some(Quarter::Afterset));
    assert_eq!(at.day.date, before.day.date);
}

#[test]
fn sunrise_switches_anchor_back() {
    let sunrise = houston_day().sunrise().unwrap();
    let before = houston(sunrise - 0.001);
    assert_eq!(before.span_form().unwrap(), "○ 99");
    assert_eq!(before.quarter(), Some(Quarter::Forerise));
    let at = houston(sunrise);
    assert_eq!(at.span_form().unwrap(), "100 ☉");
    assert_eq!(at.quarter(), Some(Quarter::Afterrise));
}

#[test]
fn quarter_boundary_belongs_to_the_quarter_it_starts() {
    let day = houston_day();
    let fifty = day.noon - day.half_day_s().unwrap() / 2.0;
    let just_after = houston(fifty + 0.001);
    assert_eq!(just_after.span_form().unwrap(), "50 ☉");
    assert_eq!(just_after.quarter(), Some(Quarter::Forenoon));
    let just_before = houston(fifty - 0.001);
    assert_eq!(just_before.span_form().unwrap(), "51 ☉");
    assert_eq!(just_before.quarter(), Some(Quarter::Afterrise));
}

#[test]
fn kilosecond_readout_steps_every_ten_seconds() {
    let noon = houston_day().noon;
    assert_eq!(houston(noon + 9.999).offset_form(), "☉");
    assert_eq!(houston(noon + 10.0).offset_form(), "☉ 0.01 ks");
    assert_eq!(houston(noon - 10.0).offset_form(), "0.01 ks ☉");
    assert_eq!(houston(noon - 10.001).offset_form(), "0.02 ks ☉");
    assert_eq!(houston(noon + 1234.5).offset_form(), "☉ 1.23 ks");
}

#[test]
fn date_changes_at_solar_midnight() {
    let day = houston_day();
    let last = houston(day.midnight_end - 0.001);
    assert_eq!(last.day.date.to_string(), "2026-11-01");
    assert_eq!(last.span_form().unwrap(), "1 ○");
    let first = houston(day.midnight_end);
    assert_eq!(first.day.date.to_string(), "2026-11-02");
    assert_eq!(first.span_form().unwrap(), "○");
    assert_eq!(first.quarter(), Some(Quarter::Aftermid));
    assert_eq!(first.day.midnight_start, day.midnight_end);
}
