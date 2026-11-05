// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use bz::{Anchor, Eighth, Form, Instant, Position, Precision, Quarter, Reading, SolarDuration};

const HOUSTON: (f64, f64) = (29.76, -95.37);

fn houston(t: &str) -> Reading {
    Reading::at(Instant::parse_rfc3339(t).unwrap(), HOUSTON.0, HOUSTON.1)
}

fn at_span(span: f64, offset_s: f64) -> Reading {
    Reading {
        offset_s,
        position: Position::OnArc {
            span,
            span_s: 198.0,
            eighth: Eighth::of(Anchor::Noon, span),
        },
        ..houston("2026-11-01T17:08:00Z")
    }
}

#[test]
fn precision_floors_toward_the_past_at_each_step() {
    let r = at_span(-59.31, -11_766.4);
    assert_eq!(r.form(Form::Span, None).unwrap(), "60 ☉");
    assert_eq!(r.form(Form::Span, Some(Precision::Low)).unwrap(), "60 ☉");
    assert_eq!(
        r.form(Form::Span, Some(Precision::Medium)).unwrap(),
        "59.4 ☉"
    );
    assert_eq!(
        r.form(Form::Span, Some(Precision::High)).unwrap(),
        "59.31 ☉"
    );
    assert_eq!(r.form(Form::Offset, None).unwrap(), "11.77 ks ☉");
    assert_eq!(
        r.form(Form::Offset, Some(Precision::Low)).unwrap(),
        "11.8 ks ☉"
    );
    assert_eq!(
        r.form(Form::Offset, Some(Precision::High)).unwrap(),
        "11.767 ks ☉"
    );
    assert_eq!(
        r.form(Form::Combined, Some(Precision::Medium)).unwrap(),
        "59.4 [11.77 ks] ☉"
    );
}

#[test]
fn after_the_anchor_the_number_follows_the_glyph() {
    let r = at_span(3.7, 700.0);
    assert_eq!(
        r.form(Form::Span, Some(Precision::Medium)).unwrap(),
        "☉ 3.7"
    );
    assert_eq!(
        r.form(Form::Combined, Some(Precision::High)).unwrap(),
        "☉ 3.70 [0.700 ks]"
    );
}

#[test]
fn horizon_glyph_holds_for_the_first_step_at_any_precision() {
    let r = at_span(-99.995, -19_833.0);
    assert_eq!(r.form(Form::Span, None).unwrap(), "△");
    assert_eq!(r.form(Form::Span, Some(Precision::High)).unwrap(), "△");
    assert_eq!(r.form(Form::Combined, None).unwrap(), "100 [19.84 ks] ☉");
    let later = at_span(-99.98, -19_830.0);
    assert_eq!(
        later.form(Form::Span, Some(Precision::High)).unwrap(),
        "99.98 ☉"
    );
    assert_eq!(later.form(Form::Span, None).unwrap(), "△");
}

#[test]
fn quarters_start_at_their_event() {
    let cases = [
        ("2026-11-01T12:34:34Z", Quarter::Morning),
        ("2026-11-01T18:05:05Z", Quarter::Morning),
        ("2026-11-01T18:05:06Z", Quarter::Afternoon),
        ("2026-11-01T23:35:39Z", Quarter::Evening),
        ("2026-11-02T06:05:07Z", Quarter::Night),
    ];
    for (t, quarter) in cases {
        assert_eq!(houston(t).quarter(), Some(quarter), "{t}");
    }
}

#[test]
fn a_polar_day_has_no_quarter() {
    let r = Reading::at(
        Instant::parse_rfc3339("2026-12-21T12:00:00Z").unwrap(),
        78.22,
        15.65,
    );
    assert_eq!(r.quarter(), None);
    assert_eq!(r.form(Form::Span, None), None);
    assert!(r.timestamp_as(Form::Span, None).ends_with("☉ 3.88 ks"));
}

fn duration(spans: f64, seconds: f64) -> SolarDuration {
    SolarDuration {
        seconds,
        spans: Some(spans),
    }
}

#[test]
fn a_countdown_never_shows_zero() {
    assert_eq!(
        duration(23.5, 4_660.0).spans_relative(None).unwrap(),
        "in 24 spans"
    );
    assert_eq!(
        duration(23.0, 4_560.0).spans_relative(None).unwrap(),
        "in 23 spans"
    );
    assert_eq!(
        duration(0.2, 40.0).spans_relative(None).unwrap(),
        "in 1 span"
    );
    assert_eq!(duration(0.2, 40.0).ks_relative(None), "in 0.04 ks");
}

#[test]
fn elapsed_time_counts_whole_spans_passed() {
    assert_eq!(
        duration(-23.5, -4_660.0).spans_relative(None).unwrap(),
        "23 spans ago"
    );
    assert_eq!(
        duration(-1.2, -240.0).spans_relative(None).unwrap(),
        "1 span ago"
    );
    assert_eq!(
        duration(-1.2, -240.0).ks_relative(Some(Precision::Low)),
        "0.2 ks ago"
    );
}

#[test]
fn a_duration_is_written_with_its_unit_and_no_glyph() {
    let d = duration(23.59, 4_937.0);
    assert_eq!(d.spans_text(None).unwrap(), "23 spans");
    assert_eq!(d.spans_text(Some(Precision::High)).unwrap(), "23.59 spans");
    assert_eq!(d.ks_text(None), "4.93 ks");
    assert_eq!(d.ks_text(Some(Precision::Low)), "4.9 ks");
    assert_eq!(duration(1.4, 280.0).spans_text(None).unwrap(), "1 span");
    let polar = SolarDuration {
        seconds: 86_400.0,
        spans: None,
    };
    assert_eq!(polar.spans_text(None), None);
    assert_eq!(polar.ks_text(None), "86.40 ks");
}
