// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

//! A Bogenzeit reading: where an instant sits on the Sun's arc at a place.
//! Data forms are signed (Bogenzeit spec §4); display forms put the number
//! on the side of the glyph that the positional rule gives it (§3.2) after
//! flooring toward the past (§5.2).

use crate::calendar::{Date, Instant};
use crate::solar::{
    half_arc, solar_midnight, solar_noon, HalfArc, ASTRONOMICAL_TWILIGHT_ALT_DEG,
    CIVIL_TWILIGHT_ALT_DEG, HORIZON_ALT_DEG, NAUTICAL_TWILIGHT_ALT_DEG,
};

/// The space between a number and its glyph in a display form. The vectors
/// are written with a plain space.
const SEP: &str = " ";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Noon,
    Midnight,
}

impl Anchor {
    pub fn glyph(self) -> char {
        match self {
            Anchor::Noon => '☉',
            Anchor::Midnight => '○',
        }
    }

    /// The horizon glyph written for the 100-span position before this
    /// anchor (§3.2): △ after sunrise, ▽ after sunset.
    pub fn horizon_glyph(self) -> char {
        match self {
            Anchor::Noon => '△',
            Anchor::Midnight => '▽',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Anchor::Noon => "noon",
            Anchor::Midnight => "midnight",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quarter {
    Afterrise,
    Forenoon,
    Afternoon,
    Foreset,
    Afterset,
    Foremid,
    Aftermid,
    Forerise,
}

impl Quarter {
    pub fn name(self) -> &'static str {
        match self {
            Quarter::Afterrise => "Afterrise",
            Quarter::Forenoon => "Forenoon",
            Quarter::Afternoon => "Afternoon",
            Quarter::Foreset => "Foreset",
            Quarter::Afterset => "Afterset",
            Quarter::Foremid => "Foremid",
            Quarter::Aftermid => "Aftermid",
            Quarter::Forerise => "Forerise",
        }
    }

    /// The quarter holding a signed span on an arc: start inclusive, end
    /// exclusive, so exactly the anchor is Afternoon or Aftermid (§3.6).
    pub fn of(anchor: Anchor, span: f64) -> Quarter {
        let index = ((span + 100.0) / 50.0).floor().clamp(0.0, 3.0) as usize;
        let quarters = match anchor {
            Anchor::Noon => [
                Quarter::Afterrise,
                Quarter::Forenoon,
                Quarter::Afternoon,
                Quarter::Foreset,
            ],
            Anchor::Midnight => [
                Quarter::Afterset,
                Quarter::Foremid,
                Quarter::Aftermid,
                Quarter::Forerise,
            ],
        };
        quarters[index]
    }
}

/// A day without a sunrise or sunset (§3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Polar {
    /// The Sun never sets: polar day.
    AlwaysAbove,
    /// The Sun never rises: polar night.
    AlwaysBelow,
}

impl Polar {
    pub fn name(self) -> &'static str {
        match self {
            Polar::AlwaysAbove => "always_above",
            Polar::AlwaysBelow => "always_below",
        }
    }
}

/// How dark the Nachtbogen is at an instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NightPhase {
    CivilTwilight,
    NauticalTwilight,
    AstronomicalTwilight,
    Night,
}

impl NightPhase {
    pub fn name(self) -> &'static str {
        match self {
            NightPhase::CivilTwilight => "civil twilight",
            NightPhase::NauticalTwilight => "nautical twilight",
            NightPhase::AstronomicalTwilight => "astronomical twilight",
            NightPhase::Night => "night",
        }
    }
}

/// One Bogenzeit day: the apparent solar day whose ☉ falls on `date`, from
/// its starting ○ (included) to its ending ○ (excluded), per §5.1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolarDay {
    pub date: Date,
    pub noon: Instant,
    pub midnight_start: Instant,
    pub midnight_end: Instant,
    pub horizon: HalfArc,
    pub civil: HalfArc,
    pub nautical: HalfArc,
    pub astronomical: HalfArc,
}

impl SolarDay {
    pub fn compute(date: Date, lat: f64, lon: f64) -> SolarDay {
        SolarDay {
            date,
            noon: solar_noon(date, lon),
            midnight_start: solar_midnight(date.plus_days(-1), lon),
            midnight_end: solar_midnight(date, lon),
            horizon: half_arc(date, lat, lon, HORIZON_ALT_DEG),
            civil: half_arc(date, lat, lon, CIVIL_TWILIGHT_ALT_DEG),
            nautical: half_arc(date, lat, lon, NAUTICAL_TWILIGHT_ALT_DEG),
            astronomical: half_arc(date, lat, lon, ASTRONOMICAL_TWILIGHT_ALT_DEG),
        }
    }

    pub fn polar(&self) -> Option<Polar> {
        match self.horizon {
            HalfArc::Seconds(_) => None,
            HalfArc::AlwaysAbove => Some(Polar::AlwaysAbove),
            HalfArc::AlwaysBelow => Some(Polar::AlwaysBelow),
        }
    }

    /// Solar noon to sunset, which is also sunrise to solar noon.
    pub fn half_day_s(&self) -> Option<f64> {
        self.horizon.seconds()
    }

    pub fn tagbogen_s(&self) -> Option<f64> {
        self.half_day_s().map(|h| 2.0 * h)
    }

    /// Seconds in one day span today.
    pub fn day_span_s(&self) -> Option<f64> {
        self.half_day_s().map(|h| h / 100.0)
    }

    pub fn sunrise(&self) -> Option<Instant> {
        self.before_noon(self.horizon)
    }

    pub fn sunset(&self) -> Option<Instant> {
        self.after_noon(self.horizon)
    }

    /// The morning instant at which the Sun reaches the altitude of `arc`.
    pub fn before_noon(&self, arc: HalfArc) -> Option<Instant> {
        arc.seconds().map(|s| self.noon - s)
    }

    /// The evening instant at which the Sun reaches the altitude of `arc`.
    pub fn after_noon(&self, arc: HalfArc) -> Option<Instant> {
        arc.seconds().map(|s| self.noon + s)
    }

    /// Starting ○ to sunrise.
    pub fn half_night_morning_s(&self) -> Option<f64> {
        self.sunrise().map(|rise| rise - self.midnight_start)
    }

    /// Sunset to ending ○.
    pub fn half_night_evening_s(&self) -> Option<f64> {
        self.sunset().map(|set| self.midnight_end - set)
    }

    /// The night phase at an instant of this day, or None in daylight.
    pub fn night_phase_at(&self, t: Instant) -> Option<NightPhase> {
        let from_noon = (t - self.noon).abs();
        let within = |arc: HalfArc| match arc {
            HalfArc::Seconds(s) => from_noon < s,
            HalfArc::AlwaysAbove => true,
            HalfArc::AlwaysBelow => false,
        };
        if within(self.horizon) {
            None
        } else if within(self.civil) {
            Some(NightPhase::CivilTwilight)
        } else if within(self.nautical) {
            Some(NightPhase::NauticalTwilight)
        } else if within(self.astronomical) {
            Some(NightPhase::AstronomicalTwilight)
        } else {
            Some(NightPhase::Night)
        }
    }
}

/// The Bogenzeit date of an instant (§5.1): the label of the solar day that
/// contains it, where a solar day runs from one ○ to the next.
pub fn bz_date(t: Instant, lon: f64) -> Date {
    let guess = (t + lon / 15.0 * 3600.0).utc_date();
    for offset in [-1, 0, 1] {
        let d = guess.plus_days(offset);
        let start = solar_midnight(d.plus_days(-1), lon);
        let end = solar_midnight(d, lon);
        if start <= t && t < end {
            return d;
        }
    }
    unreachable!("every instant falls in one of three neighbouring solar days")
}

/// Where the instant sits on its arc, or why it has no arc position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Position {
    OnArc {
        /// Signed spans from the anchor: negative before, positive after.
        span: f64,
        /// Seconds in one span of this half-arc.
        span_s: f64,
        quarter: Quarter,
    },
    Polar(Polar),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    pub instant: Instant,
    pub lat: f64,
    pub lon: f64,
    pub day: SolarDay,
    pub anchor: Anchor,
    /// Signed seconds from the anchor: negative before, positive after.
    pub offset_s: f64,
    pub position: Position,
    pub night_phase: Option<NightPhase>,
}

impl Reading {
    pub fn at(t: Instant, lat: f64, lon: f64) -> Reading {
        let day = SolarDay::compute(bz_date(t, lon), lat, lon);
        let night_phase = day.night_phase_at(t);
        let (anchor, offset_s, position) = match (day.sunrise(), day.sunset(), day.half_day_s()) {
            (Some(rise), Some(set), Some(half_day)) => {
                if rise <= t && t < set {
                    let span = arc_span(t - day.noon, half_day);
                    let position = Position::OnArc {
                        span,
                        span_s: half_day / 100.0,
                        quarter: Quarter::of(Anchor::Noon, span),
                    };
                    (Anchor::Noon, t - day.noon, position)
                } else {
                    let (midnight, half_night) = if t >= set {
                        (day.midnight_end, day.midnight_end - set)
                    } else {
                        (day.midnight_start, rise - day.midnight_start)
                    };
                    let span = arc_span(t - midnight, half_night);
                    let position = Position::OnArc {
                        span,
                        span_s: half_night / 100.0,
                        quarter: Quarter::of(Anchor::Midnight, span),
                    };
                    (Anchor::Midnight, t - midnight, position)
                }
            }
            _ => {
                let polar = day
                    .polar()
                    .expect("a day without a horizon crossing is polar");
                let from_noon = (t - day.noon).abs();
                let nearer_noon = from_noon <= (t - day.midnight_end).abs()
                    && from_noon <= (t - day.midnight_start).abs();
                let (anchor, anchor_t) = if nearer_noon {
                    (Anchor::Noon, day.noon)
                } else if t < day.noon {
                    (Anchor::Midnight, day.midnight_start)
                } else {
                    (Anchor::Midnight, day.midnight_end)
                };
                (anchor, t - anchor_t, Position::Polar(polar))
            }
        };
        Reading {
            instant: t,
            lat,
            lon,
            day,
            anchor,
            offset_s,
            position,
            night_phase,
        }
    }

    pub fn span(&self) -> Option<f64> {
        match self.position {
            Position::OnArc { span, .. } => Some(span),
            Position::Polar(_) => None,
        }
    }

    pub fn span_s(&self) -> Option<f64> {
        match self.position {
            Position::OnArc { span_s, .. } => Some(span_s),
            Position::Polar(_) => None,
        }
    }

    pub fn quarter(&self) -> Option<Quarter> {
        match self.position {
            Position::OnArc { quarter, .. } => Some(quarter),
            Position::Polar(_) => None,
        }
    }

    pub fn polar(&self) -> Option<Polar> {
        self.day.polar()
    }

    /// Whole spans, floored toward the past (§5.2).
    pub fn display_span(&self) -> Option<i64> {
        self.span().map(|s| s.floor() as i64)
    }

    /// The offset in tens of seconds, floored toward the past: the ks
    /// readout's two-decimal digit count (§5.2).
    pub fn display_offset_steps(&self) -> i64 {
        (self.offset_s / 10.0).floor() as i64
    }

    pub fn display_offset_ks(&self) -> f64 {
        self.display_offset_steps() as f64 / 100.0
    }

    /// `59 ☉`, `☉`, `☉ 3`, and `△` or `▽` for the first span after sunrise
    /// or sunset (§3.2); None on a polar day.
    pub fn span_form(&self) -> Option<String> {
        let g = self.anchor.glyph();
        self.display_span().map(|span| match span {
            -100 => self.anchor.horizon_glyph().to_string(),
            s if s < 0 => format!("{}{SEP}{g}", -s),
            0 => g.to_string(),
            s => format!("{g}{SEP}{s}"),
        })
    }

    /// `11.77 ks ☉`, `☉`, `☉ 0.70 ks`.
    pub fn offset_form(&self) -> String {
        let g = self.anchor.glyph();
        match self.display_offset_steps() {
            s if s < 0 => format!("{}{SEP}{g}", ks(-s)),
            0 => g.to_string(),
            s => format!("{g}{SEP}{}", ks(s)),
        }
    }

    /// `59 [11.77 ks] ☉`, `☉`, `☉ 3 [0.70 ks]` (§3.7); None on a polar day.
    pub fn combined_form(&self) -> Option<String> {
        let g = self.anchor.glyph();
        let steps = self.display_offset_steps();
        self.display_span().map(|span| {
            if span < 0 {
                format!("{}{SEP}[{}]{SEP}{g}", -span, ks(-steps))
            } else if span == 0 && steps == 0 {
                g.to_string()
            } else {
                format!("{g}{SEP}{span}{SEP}[{}]", ks(steps))
            }
        })
    }

    /// The full timestamp (§5.3): date, coordinates and position. A polar day
    /// carries the offset form in place of the combined form.
    pub fn timestamp(&self) -> String {
        let position = self.combined_form().unwrap_or_else(|| self.offset_form());
        format!(
            "{} | {}, {} | {}",
            self.day.date, self.lat, self.lon, position
        )
    }
}

/// Spans from the anchor, held within the arc: the bounds tests put the
/// instant inside the arc, and the clamp stops a last-bit rounding of the
/// subtraction from showing 101 at exact sunrise.
fn arc_span(offset_s: f64, half_arc_s: f64) -> f64 {
    (100.0 * offset_s / half_arc_s).clamp(-100.0, 100.0)
}

/// Hundredths of a kilosecond as `11.77 ks`.
fn ks(steps: i64) -> String {
    format!("{}.{:02} ks", steps / 100, steps % 100)
}
