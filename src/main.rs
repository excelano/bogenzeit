// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

//! The bz command: a Bogenzeit reading for now or for a given instant, the
//! table of one Bogenzeit day, and the time between two instants. Flags are
//! read wherever they appear.

mod places;

use std::process::ExitCode;

use bz::{Date, Form, HalfArc, Instant, Precision, Reading, SolarDay, SolarDuration};
use places::Places;

const HELP: &str = "\
bz: Bogenzeit, time as position on the Sun's arc

Usage:
  bz [PLACE] [FORMAT]
  bz now [PLACE] [FORMAT]
  bz at <UTC> [PLACE] [FORMAT]
  bz day <DATE> [PLACE] [--json]
  bz between <UTC> [<UTC>] [PLACE] [--precision P] [--json]
  bz places
  bz place <NAME> <LAT> <LON>
  bz place <NAME>
  bz place --forget <NAME>

UTC is an RFC 3339 instant such as 2026-11-01T17:08:00Z. DATE is a
Bogenzeit date, YYYY-MM-DD: the civil date of that day's solar noon.
LAT and LON are decimal degrees, north and east positive, or with a
hemisphere letter: N29.67230 W095.47833.

PLACE is --lat LAT --lon LON, or --place NAME, or nothing for the default
place. FORMAT is any of:

  --form F         span (59 ☉), offset (11.77 ks ☉) or combined
                   (59 [11.77 ks] ☉, the default)
  --precision P    low, medium or high: whole spans and one decimal of ks,
                   one and two, or two and three; without it, whole spans
                   and two decimals of ks, as the written forms are
  --json           the reading as data, which neither flag changes

bz between gives the time between two instants in spans and ks, both
rounded down. With one instant it is read from now: in 24 spans, or 23
spans ago. Spans are undefined across a day with no sunrise or sunset;
ks always work.

Bare bz is the reading for now at the default place. Places are kept in
~/.config/bz/config (or $XDG_CONFIG_HOME/bz/config), which bz place writes:

  bz place home 29.76 -95.37       add or replace home; the first place
                                   written becomes the default
  bz place camp N36.45 W104.95     the same with hemisphere letters
  bz place camp                    make camp the default
  bz place --forget camp           remove camp
  bz places                        list the places and the default

The file is one place per line as `name = lat, lon`, with `place = name`
naming the default, and can be edited by hand.

Instants print truncated to the whole second, toward the past, like every
Bogenzeit readout.

Exit codes: 0 success; 2 bad invocation (an unknown flag, a missing argument,
an instant, date or coordinate that does not parse).
";

/// Rust ignores SIGPIPE, so `bz day ... | head` would end in a panic on a
/// closed stdout; the default disposition ends the process quietly instead.
#[cfg(unix)]
fn quiet_on_closed_pipe() {
    extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
    }
    const SIGPIPE: i32 = 13;
    const SIG_DFL: usize = 0;
    // SAFETY: resetting a signal disposition to the default before any
    // thread or I/O exists has no preconditions beyond the C ABI.
    unsafe {
        signal(SIGPIPE, SIG_DFL);
    }
}

#[cfg(not(unix))]
fn quiet_on_closed_pipe() {}

struct Invocation {
    verb: String,
    operands: Vec<String>,
    forget: bool,
    lat: f64,
    lon: f64,
    json: bool,
    form: Form,
    precision: Option<Precision>,
}

/// Where the reading is for: given outright, or named from the config file.
enum Where {
    Coordinates(f64, f64),
    Place(Option<String>),
}

fn main() -> ExitCode {
    quiet_on_closed_pipe();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{HELP}");
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("bz {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    match parse(&args).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("bz: {message}");
            ExitCode::from(2)
        }
    }
}

fn parse(args: &[String]) -> Result<Invocation, String> {
    let mut positional = Vec::new();
    let mut lat = None;
    let mut lon = None;
    let mut place = None;
    let mut forget = false;
    let mut json = false;
    let mut form = Form::Combined;
    let mut precision = None;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let (flag, inline_value) = match arg.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f, Some(v.to_string())),
            _ => (arg.as_str(), None),
        };
        let mut value = |name: &str| -> Result<String, String> {
            if let Some(v) = inline_value.clone() {
                return Ok(v);
            }
            i += 1;
            args.get(i)
                .cloned()
                .ok_or_else(|| format!("{name} needs a value; see bz --help"))
        };
        match flag {
            "--" => {
                positional.extend(args[i + 1..].iter().cloned());
                break;
            }
            "--lat" => lat = Some(value("--lat")?),
            "--lon" => lon = Some(value("--lon")?),
            "--place" => place = Some(value("--place")?),
            "--forget" => forget = true,
            "--json" => json = true,
            "--form" => {
                let v = value("--form")?;
                form = Form::parse(&v)
                    .ok_or_else(|| format!("--form is span, offset or combined, not `{v}`"))?;
            }
            "--precision" => {
                let v = value("--precision")?;
                precision = Some(
                    Precision::parse(&v)
                        .ok_or_else(|| format!("--precision is low, medium or high, not `{v}`"))?,
                );
            }
            f if f.starts_with('-') && f.len() > 1 && f.parse::<f64>().is_err() => {
                return Err(format!("unknown flag {f}; see bz --help"));
            }
            _ => positional.push(arg.clone()),
        }
        i += 1;
    }
    let mut positional = positional.into_iter();
    // Bare `bz`, flags or not, is the reading for now.
    let verb = positional.next().unwrap_or_else(|| "now".to_string());
    let operands: Vec<String> = positional.collect();
    let limit = match verb.as_str() {
        "place" => 3,
        "between" => 2,
        _ => 1,
    };
    if operands.len() > limit {
        return Err(format!(
            "unexpected argument `{}`; see bz --help",
            operands[limit]
        ));
    }
    let location = match (lat, lon, place) {
        (Some(lat), Some(lon), None) => Where::Coordinates(
            places::degrees(&lat, 90.0).map_err(|e| format!("--lat {e}"))?,
            places::degrees(&lon, 180.0).map_err(|e| format!("--lon {e}"))?,
        ),
        (None, None, place) => Where::Place(place),
        (Some(_), Some(_), Some(_)) => {
            return Err("--place or --lat and --lon, not both; see bz --help".into())
        }
        _ => return Err("--lat and --lon go together; see bz --help".into()),
    };
    let (lat, lon) = if verb == "places" || verb == "place" {
        (0.0, 0.0)
    } else {
        locate(location)?
    };
    Ok(Invocation {
        verb,
        operands,
        forget,
        lat,
        lon,
        json,
        form,
        precision,
    })
}

/// The coordinates to use: the ones given, else the named or default place
/// from the config file.
fn locate(location: Where) -> Result<(f64, f64), String> {
    match location {
        Where::Coordinates(lat, lon) => Ok((lat, lon)),
        Where::Place(name) => {
            let places = Places::load()?;
            let path = Places::path()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            match name.or(places.default.clone()) {
                Some(name) => places
                    .find(&name)
                    .ok_or_else(|| format!("no place named {name} in {path}; `bz places` lists them")),
                None => Err(format!(
                    "no place: give --lat and --lon, or put `place = home` and `home = LAT, LON` in {path}"
                )),
            }
        }
    }
}

fn run(inv: Invocation) -> Result<(), String> {
    let operand = |what: &str| {
        inv.operands
            .first()
            .cloned()
            .ok_or_else(|| format!("bz {} needs {what}; see bz --help", inv.verb))
    };
    match inv.verb.as_str() {
        "now" => {
            if !inv.operands.is_empty() {
                return Err("bz now takes no operand; see bz --help".into());
            }
            print_reading(&Reading::at(Instant::now(), inv.lat, inv.lon), &inv);
        }
        "at" => {
            let t = Instant::parse_rfc3339(&operand("an instant")?)?;
            print_reading(&Reading::at(t, inv.lat, inv.lon), &inv);
        }
        "between" => {
            let first = Instant::parse_rfc3339(&operand("an instant")?)?;
            let (from, to, from_now) = match inv.operands.get(1) {
                Some(second) => (first, Instant::parse_rfc3339(second)?, false),
                None => (Instant::now(), first, true),
            };
            print_between(from, to, from_now, &inv);
        }
        "day" => {
            let date = Date::parse(&operand("a date")?)?;
            print_day(
                &SolarDay::compute(date, inv.lat, inv.lon),
                inv.lat,
                inv.lon,
                inv.json,
            );
        }
        "places" => {
            if !inv.operands.is_empty() {
                return Err("bz places takes no operand; see bz --help".into());
            }
            print_places(&Places::load()?, inv.json);
        }
        "place" => {
            let mut places = Places::load()?;
            match (inv.operands.as_slice(), inv.forget) {
                ([name], true) => places.forget(name)?,
                ([name], false) => places.make_default(name)?,
                ([name, lat, lon], false) => {
                    if name.contains(['=', ',', '#']) || name.trim() != name {
                        return Err(format!("`{name}` is not a place name; use letters, digits, - and _"));
                    }
                    let lat = places::degrees(lat, 90.0).map_err(|e| format!("latitude {e}"))?;
                    let lon = places::degrees(lon, 180.0).map_err(|e| format!("longitude {e}"))?;
                    places.set(name, lat, lon);
                }
                _ => {
                    return Err("bz place <NAME> <LAT> <LON>, bz place <NAME>, or bz place --forget <NAME>; see bz --help".into())
                }
            }
            let path = places.save()?;
            println!("{}", path.display());
            print_places(&places, false);
        }
        other => return Err(format!("unknown verb `{other}`; see bz --help")),
    }
    Ok(())
}

fn print_reading(r: &Reading, inv: &Invocation) {
    if inv.json {
        println!("{}", reading_json(r));
        return;
    }
    let day = &r.day;
    println!("{}", r.timestamp_as(inv.form, inv.precision));
    match (r.quarter(), r.eighth(), r.polar()) {
        (Some(quarter), Some(eighth), _) => {
            println!("{:<10}{}", "quarter", quarter.name());
            if eighth.name() != quarter.name() {
                println!("{:<10}{}", "eighth", eighth.name());
            }
        }
        (_, _, Some(polar)) => println!("{}", polar.label()),
        _ => {}
    }
    if let Some(phase) = r.night_phase {
        println!("{:<10}{}", "phase", phase.name());
    }
    if let Some(span_s) = r.span_s() {
        println!("{:<10}{:.0} s", "span", span_s);
    }
    println!("{:<10}{}", "○ start", day.midnight_start.rfc3339());
    println!("{:<10}{}", "△", opt(day.sunrise()));
    println!("{:<10}{}", "☉", day.noon.rfc3339());
    println!("{:<10}{}", "▽", opt(day.sunset()));
    println!("{:<10}{}", "○ end", day.midnight_end.rfc3339());
}

fn print_between(from: Instant, to: Instant, from_now: bool, inv: &Invocation) {
    let d = SolarDuration::between(from, to, inv.lat, inv.lon);
    if inv.json {
        let fields = vec![
            ("from", quoted(&from.rfc3339_millis())),
            ("to", quoted(&to.rfc3339_millis())),
            ("lat", inv.lat.to_string()),
            ("lon", inv.lon.to_string()),
            ("seconds", format!("{:.3}", d.seconds)),
            (
                "spans",
                d.spans.map_or("null".into(), |s| format!("{s:.4}")),
            ),
        ];
        println!("{}", object(&fields));
        return;
    }
    let (spans, ks) = if from_now {
        (
            d.spans_relative(inv.precision),
            d.ks_relative(inv.precision),
        )
    } else {
        (d.spans_text(inv.precision), d.ks_text(inv.precision))
    };
    println!(
        "{} to {} | {}, {}",
        from.rfc3339(),
        to.rfc3339(),
        inv.lat,
        inv.lon
    );
    println!(
        "{:<10}{}",
        "spans",
        spans.unwrap_or_else(|| "none: a day between has no sunrise or sunset".into())
    );
    println!("{:<10}{}", "ks", ks);
}

fn print_day(day: &SolarDay, lat: f64, lon: f64, json: bool) {
    if json {
        println!("{}", day_json(day, lat, lon));
        return;
    }
    let half = |h: Option<f64>| h.map(|h| h / 2.0);
    let morning_half_night = half(day.half_night_morning_s()).map(|h| day.midnight_start + h);
    let evening_half_night = half(day.half_night_evening_s())
        .map(|h| day.sunset().expect("a day with a half night has a sunset") + h);
    let half_day = day.half_day_s().map(|h| h / 2.0);
    let rows = [
        ("○ start", Some(day.midnight_start), "Night | Aftermid"),
        ("○ 50", morning_half_night, "Forerise"),
        ("astronomical dawn", day.before_noon(day.astronomical), ""),
        ("nautical dawn", day.before_noon(day.nautical), ""),
        ("civil dawn", day.before_noon(day.civil), ""),
        ("△", day.sunrise(), "Morning | Afterrise"),
        ("50 ☉", half_day.map(|h| day.noon - h), "Forenoon"),
        ("☉", Some(day.noon), "Afternoon"),
        ("☉ 50", half_day.map(|h| day.noon + h), "Foreset"),
        ("▽", day.sunset(), "Evening | Afterset"),
        ("civil dusk", day.after_noon(day.civil), ""),
        ("nautical dusk", day.after_noon(day.nautical), ""),
        ("astronomical dusk", day.after_noon(day.astronomical), ""),
        ("50 ○", evening_half_night, "Foremid"),
        ("○ end", Some(day.midnight_end), "next day"),
    ];
    let mut timed: Vec<(&str, Instant, &str)> = rows
        .iter()
        .filter_map(|(label, at, begins)| at.map(|at| (*label, at, *begins)))
        .collect();
    timed.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("instants are finite"));
    let missing: Vec<&str> = rows
        .iter()
        .filter(|(_, at, _)| at.is_none())
        .map(|(label, _, _)| *label)
        .collect();

    println!("{} | {}, {}", day.date, lat, lon);
    if let Some(polar) = day.polar() {
        println!("{}", polar.label());
    }
    for (label, at, begins) in timed {
        let begins = match (day.polar(), begins) {
            (Some(_), "next day") | (None, _) => begins,
            (Some(_), _) => "",
        };
        let line = format!("{label:<20}{:<22}{begins}", at.rfc3339());
        println!("{}", line.trim_end());
    }
    if !missing.is_empty() {
        println!("{:<20}{}", "none today", missing.join(", "));
    }
    if let Some(tagbogen) = day.tagbogen_s() {
        println!(
            "{:<20}{:.0} s, span {:.0} s",
            "Tagbogen",
            tagbogen,
            day.day_span_s().unwrap_or(0.0)
        );
    }
    if let Some(morning) = day.half_night_morning_s() {
        println!(
            "{:<20}{:.0} s, span {:.0} s",
            "half night, morning",
            morning,
            morning / 100.0
        );
    }
    if let Some(evening) = day.half_night_evening_s() {
        println!(
            "{:<20}{:.0} s, span {:.0} s",
            "half night, evening",
            evening,
            evening / 100.0
        );
    }
}

/// The config file's places; the default marked. An empty file is an answer.
fn print_places(places: &Places, json: bool) {
    if json {
        let entries: Vec<String> = places
            .places
            .iter()
            .map(|(name, lat, lon)| {
                format!(
                    "{{\"name\": {}, \"lat\": {lat}, \"lon\": {lon}, \"default\": {}}}",
                    quoted(name),
                    places.default.as_deref() == Some(name)
                )
            })
            .collect();
        println!("[{}]", entries.join(", "));
        return;
    }
    if places.places.is_empty() {
        println!(
            "no places in {}; add `home = LAT, LON` and `place = home`",
            Places::path()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        );
        return;
    }
    let width = places
        .places
        .iter()
        .map(|(name, _, _)| name.chars().count() + 2)
        .max()
        .unwrap_or(0)
        .max(12);
    for (name, lat, lon) in &places.places {
        let mark = if places.default.as_deref() == Some(name) {
            "  default"
        } else {
            ""
        };
        println!("{name:<width$}{lat}, {lon}{mark}");
    }
}

fn opt(t: Option<Instant>) -> String {
    t.map_or_else(|| "none".to_string(), Instant::rfc3339)
}

/// The reading as JSON in the shape of a vector's `expected` object, plus
/// `span_s`, `quarter` and `night_phase`.
fn reading_json(r: &Reading) -> String {
    let day = &r.day;
    let mut fields: Vec<(&str, String)> = vec![
        ("bz_date", quoted(&day.date.to_string())),
        ("solar_noon_utc", quoted(&day.noon.rfc3339_millis())),
        (
            "solar_midnight_start_utc",
            quoted(&day.midnight_start.rfc3339_millis()),
        ),
        (
            "solar_midnight_end_utc",
            quoted(&day.midnight_end.rfc3339_millis()),
        ),
    ];
    if let (Some(rise), Some(set), Some(tagbogen)) = (day.sunrise(), day.sunset(), day.tagbogen_s())
    {
        fields.push(("sunrise_utc", quoted(&rise.rfc3339_millis())));
        fields.push(("sunset_utc", quoted(&set.rfc3339_millis())));
        fields.push(("tagbogen_s", format!("{tagbogen:.3}")));
    }
    if let (Some(span_s), bz::Anchor::Midnight) = (r.span_s(), r.anchor) {
        fields.push(("half_night_s", format!("{:.3}", span_s * 100.0)));
    }
    if let Some(polar) = r.polar() {
        fields.push(("polar", quoted(polar.name())));
    }
    fields.push(("anchor", quoted(r.anchor.name())));
    fields.push(("offset_s", format!("{:.3}", r.offset_s)));
    fields.push((
        "span",
        r.span().map_or("null".into(), |s| format!("{s:.4}")),
    ));
    fields.push((
        "span_s",
        r.span_s().map_or("null".into(), |s| format!("{s:.3}")),
    ));
    fields.push((
        "display_span",
        r.display_span().map_or("null".into(), |s| s.to_string()),
    ));
    fields.push(("display_offset_ks", format!("{:.2}", r.display_offset_ks())));
    fields.push((
        "quarter",
        r.quarter().map_or("null".into(), |q| quoted(q.name())),
    ));
    fields.push((
        "eighth",
        r.eighth().map_or("null".into(), |q| quoted(q.name())),
    ));
    fields.push((
        "written",
        r.combined_form().map_or("null".into(), |w| quoted(&w)),
    ));
    fields.push((
        "written_span",
        r.span_form().map_or("null".into(), |w| quoted(&w)),
    ));
    fields.push((
        "night_phase",
        r.night_phase.map_or("null".into(), |p| quoted(p.name())),
    ));
    object(&fields)
}

fn day_json(day: &SolarDay, lat: f64, lon: f64) -> String {
    let instant = |t: Option<Instant>| t.map_or("null".into(), |t| quoted(&t.rfc3339_millis()));
    let seconds = |s: Option<f64>| s.map_or("null".into(), |s| format!("{s:.3}"));
    let arc = |a: HalfArc| match a {
        HalfArc::Seconds(s) => format!("{s:.3}"),
        other => quoted(if other == HalfArc::AlwaysAbove {
            "always_above"
        } else {
            "always_below"
        }),
    };
    let fields = vec![
        ("bz_date", quoted(&day.date.to_string())),
        ("lat", lat.to_string()),
        ("lon", lon.to_string()),
        (
            "polar",
            day.polar().map_or("null".into(), |p| quoted(p.name())),
        ),
        (
            "solar_midnight_start_utc",
            instant(Some(day.midnight_start)),
        ),
        (
            "astronomical_dawn_utc",
            instant(day.before_noon(day.astronomical)),
        ),
        ("nautical_dawn_utc", instant(day.before_noon(day.nautical))),
        ("civil_dawn_utc", instant(day.before_noon(day.civil))),
        ("sunrise_utc", instant(day.sunrise())),
        ("solar_noon_utc", instant(Some(day.noon))),
        ("sunset_utc", instant(day.sunset())),
        ("civil_dusk_utc", instant(day.after_noon(day.civil))),
        ("nautical_dusk_utc", instant(day.after_noon(day.nautical))),
        (
            "astronomical_dusk_utc",
            instant(day.after_noon(day.astronomical)),
        ),
        ("solar_midnight_end_utc", instant(Some(day.midnight_end))),
        ("half_day_s", arc(day.horizon)),
        ("tagbogen_s", seconds(day.tagbogen_s())),
        ("day_span_s", seconds(day.day_span_s())),
        ("half_night_morning_s", seconds(day.half_night_morning_s())),
        ("half_night_evening_s", seconds(day.half_night_evening_s())),
    ];
    object(&fields)
}

fn object(fields: &[(&str, String)]) -> String {
    let body: Vec<String> = fields
        .iter()
        .map(|(k, v)| format!("  \"{k}\": {v}"))
        .collect();
    format!("{{\n{}\n}}", body.join(",\n"))
}

fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}
