// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

//! The bz command: a Bogenzeit reading for now or for a given instant, and
//! the table of one Bogenzeit day. Flags are read wherever they appear.

use std::process::ExitCode;

use bz::{Date, HalfArc, Instant, Reading, SolarDay};

const HELP: &str = "\
bz: Bogenzeit, time as position on the Sun's arc

Usage:
  bz now --lat LAT --lon LON [--json]
  bz at <UTC> --lat LAT --lon LON [--json]
  bz day <DATE> --lat LAT --lon LON [--json]

UTC is an RFC 3339 instant such as 2026-11-01T17:08:00Z. DATE is a
Bogenzeit date, YYYY-MM-DD: the civil date of that day's solar noon.
LAT and LON are decimal degrees, north and east positive.

Instants print truncated to the whole second, toward the past, like every
Bogenzeit readout.

Exit codes: 0 success; 2 bad invocation (an unknown flag, a missing argument,
an instant, date or coordinate that does not parse).
";

struct Invocation {
    verb: String,
    operand: Option<String>,
    lat: f64,
    lon: f64,
    json: bool,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{HELP}");
        return ExitCode::from(if args.is_empty() { 2 } else { 0 });
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
    let mut json = false;
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
            "--json" => json = true,
            f if f.starts_with('-') && f.len() > 1 => {
                return Err(format!("unknown flag {f}; see bz --help"));
            }
            _ => positional.push(arg.clone()),
        }
        i += 1;
    }
    let mut positional = positional.into_iter();
    let verb = positional
        .next()
        .ok_or("a verb is needed: now, at or day; see bz --help")?;
    let operand = positional.next();
    if let Some(extra) = positional.next() {
        return Err(format!("unexpected argument `{extra}`; see bz --help"));
    }
    let lat = coordinate("--lat", lat, 90.0)?;
    let lon = coordinate("--lon", lon, 180.0)?;
    Ok(Invocation {
        verb,
        operand,
        lat,
        lon,
        json,
    })
}

fn coordinate(name: &str, value: Option<String>, limit: f64) -> Result<f64, String> {
    let text = value.ok_or_else(|| format!("{name} is required; see bz --help"))?;
    let degrees: f64 = text
        .parse()
        .map_err(|_| format!("{name} `{text}` is not a number of degrees"))?;
    if !(-limit..=limit).contains(&degrees) {
        return Err(format!("{name} {text} is outside -{limit} to {limit}"));
    }
    Ok(degrees)
}

fn run(inv: Invocation) -> Result<(), String> {
    let operand = |what: &str| {
        inv.operand
            .clone()
            .ok_or_else(|| format!("bz {} needs {what}; see bz --help", inv.verb))
    };
    match inv.verb.as_str() {
        "now" => {
            if inv.operand.is_some() {
                return Err("bz now takes no operand; see bz --help".into());
            }
            print_reading(&Reading::at(Instant::now(), inv.lat, inv.lon), inv.json);
        }
        "at" => {
            let t = Instant::parse_rfc3339(&operand("an instant")?)?;
            print_reading(&Reading::at(t, inv.lat, inv.lon), inv.json);
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
        other => return Err(format!("unknown verb `{other}`; see bz --help")),
    }
    Ok(())
}

fn print_reading(r: &Reading, json: bool) {
    if json {
        println!("{}", reading_json(r));
        return;
    }
    let day = &r.day;
    println!("{}", r.timestamp());
    match r.quarter() {
        Some(q) => println!("{:<10}{}", "quarter", q.name()),
        None => println!("{:<10}{}", "polar", r.polar().map_or("", |p| p.name())),
    }
    if let Some(phase) = r.night_phase {
        println!("{:<10}{}", "phase", phase.name());
    }
    if let Some(span_s) = r.span_s() {
        println!("{:<10}{:.0} s", "span", span_s);
    }
    println!("{:<10}{}", "○ start", day.midnight_start.rfc3339());
    println!("{:<10}{}", "sunrise", opt(day.sunrise()));
    println!("{:<10}{}", "☉", day.noon.rfc3339());
    println!("{:<10}{}", "sunset", opt(day.sunset()));
    println!("{:<10}{}", "○ end", day.midnight_end.rfc3339());
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
        ("○ start", Some(day.midnight_start), "Aftermid"),
        ("○ 50", morning_half_night, "Forerise"),
        ("astronomical dawn", day.before_noon(day.astronomical), ""),
        ("nautical dawn", day.before_noon(day.nautical), ""),
        ("civil dawn", day.before_noon(day.civil), ""),
        ("sunrise", day.sunrise(), "Afterrise"),
        ("50 ☉", half_day.map(|h| day.noon - h), "Forenoon"),
        ("☉", Some(day.noon), "Afternoon"),
        ("☉ 50", half_day.map(|h| day.noon + h), "Foreset"),
        ("sunset", day.sunset(), "Afterset"),
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
        println!("{:<20}{}", "polar", polar.name());
    }
    for (label, at, begins) in timed {
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

fn opt(t: Option<Instant>) -> String {
    t.map_or_else(|| "none".to_string(), Instant::rfc3339)
}

/// The reading as JSON in the shape of a vector's `expected` object, plus
/// `span_s` and `night_phase`.
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
        "written",
        r.combined_form().map_or("null".into(), |w| quoted(&w)),
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
