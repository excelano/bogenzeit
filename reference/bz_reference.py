#!/usr/bin/env python3
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)
"""
Bogenzeit (BZ) reference implementation — prototype, stdlib only.

Purpose: define the computation precisely and generate golden test vectors
(vectors/bz_vectors.json). Native ports (Rust, Swift, Kotlin) must reproduce
the vectors within 1 second for event times and exactly for displayed values.

Algorithm: NOAA "General Solar Position Calculations" (fractional-year
series for equation of time and declination). Accuracy vs. the real Sun is
about a minute at mid-latitudes; what matters for the ports is that they
all implement THIS algorithm identically.

Usage:
  python3 bz_reference.py 2026-11-01T17:08:00Z 29.76 -95.37
  python3 bz_reference.py --vectors > ../vectors/bz_vectors.json
"""
import json
import math
import sys
from datetime import date, datetime, timedelta, timezone

SUNRISE_ALT = -0.833   # deg; Sun's center, includes refraction (BZ spec §2.1)
TWILIGHTS = {"civil": -6.0, "nautical": -12.0, "astronomical": -18.0}
DAY_S = 86400.0

QUARTERS_DAY = ["Afterrise", "Forenoon", "Afternoon", "Foreset"]
QUARTERS_NIGHT = ["Afterset", "Foremid", "Aftermid", "Forerise"]


def _is_leap(y):
    return y % 4 == 0 and (y % 100 != 0 or y % 400 == 0)


def _solar_terms(d: date, utc_hour: float):
    """Equation of time (minutes) and declination (radians) for a UTC date/hour."""
    days_in_year = 366 if _is_leap(d.year) else 365
    doy = d.timetuple().tm_yday
    g = 2 * math.pi / days_in_year * (doy - 1 + (utc_hour - 12) / 24)
    eot = 229.18 * (0.000075 + 0.001868 * math.cos(g) - 0.032077 * math.sin(g)
                    - 0.014615 * math.cos(2 * g) - 0.040849 * math.sin(2 * g))
    decl = (0.006918 - 0.399912 * math.cos(g) + 0.070257 * math.sin(g)
            - 0.006758 * math.cos(2 * g) + 0.000907 * math.sin(2 * g)
            - 0.002697 * math.cos(3 * g) + 0.00148 * math.sin(3 * g))
    return eot, decl


def solar_noon_utc(d: date, lon: float) -> datetime:
    """Solar noon (☉) for civil date d at longitude lon (deg, east positive)."""
    approx_hour = 12 - lon / 15          # first guess, UTC hours
    eot, _ = _solar_terms(d, approx_hour)
    minutes = 720 - 4 * lon - eot
    return datetime(d.year, d.month, d.day, tzinfo=timezone.utc) + timedelta(minutes=minutes)


def half_arc_s(d: date, lat: float, lon: float, alt_deg: float):
    """Seconds from ☉ to the Sun reaching alt_deg, or None (polar: never reaches it)."""
    noon = solar_noon_utc(d, lon)
    _, decl = _solar_terms(d, noon.hour + noon.minute / 60)
    phi = math.radians(lat)
    x = (math.sin(math.radians(alt_deg)) - math.sin(phi) * math.sin(decl)) / (math.cos(phi) * math.cos(decl))
    if x < -1:
        return "always_above"
    if x > 1:
        return "always_below"
    return math.degrees(math.acos(x)) * 240.0   # 1° of hour angle = 240 s


def day_events(d: date, lat: float, lon: float):
    noon = solar_noon_utc(d, lon)
    h = half_arc_s(d, lat, lon, SUNRISE_ALT)
    ev = {"noon": noon, "half_day_s": h}
    if isinstance(h, float):
        ev["sunrise"] = noon - timedelta(seconds=h)
        ev["sunset"] = noon + timedelta(seconds=h)
    return ev


def solar_midnight(d_before: date, lon: float) -> datetime:
    """○ between the solar days of d_before and d_before+1 (midpoint of the two ☉)."""
    n1 = solar_noon_utc(d_before, lon)
    n2 = solar_noon_utc(d_before + timedelta(days=1), lon)
    return n1 + (n2 - n1) / 2


def bz_date(t: datetime, lon: float) -> date:
    """Bogenzeit date (BZ spec §5.1): the civil date of the ☉ of the solar day containing t.
    A solar day includes its starting ○ and excludes its ending ○."""
    guess = (t + timedelta(hours=lon / 15)).date()
    for d in (guess - timedelta(days=1), guess, guess + timedelta(days=1)):
        start = solar_midnight(d - timedelta(days=1), lon)
        end = solar_midnight(d, lon)
        if start <= t < end:
            return d
    raise RuntimeError("no solar day found")


def floor_div(x: float, step: float) -> float:
    return math.floor(x / step) * step


def bogenzeit(t: datetime, lat: float, lon: float) -> dict:
    d = bz_date(t, lon)
    today = day_events(d, lat, lon)
    noon = today["noon"]
    mid_before = solar_midnight(d - timedelta(days=1), lon)
    mid_after = solar_midnight(d, lon)

    out = {"bz_date": d.isoformat(), "solar_noon_utc": noon.isoformat(),
           "solar_midnight_start_utc": mid_before.isoformat(),
           "solar_midnight_end_utc": mid_after.isoformat()}

    h = today["half_day_s"]
    if not isinstance(h, float):
        out["polar"] = h
        out["anchor"] = "noon" if abs((t - noon).total_seconds()) <= abs((t - mid_after).total_seconds()) and abs((t - noon).total_seconds()) <= abs((t - mid_before).total_seconds()) else "midnight"
        anchor_t = noon if out["anchor"] == "noon" else (mid_before if t < noon else mid_after)
        off = (t - anchor_t).total_seconds()
        out["offset_s"] = round(off, 3)
        out["display_offset_ks"] = round(floor_div(off, 10) / 1000, 2)
        out["span"] = None
        out["display_span"] = None
        out["quarter"] = None
        return out

    sunrise, sunset = today["sunrise"], today["sunset"]
    out["sunrise_utc"] = sunrise.isoformat()
    out["sunset_utc"] = sunset.isoformat()
    out["tagbogen_s"] = round(2 * h, 3)

    if sunrise <= t < sunset:                       # Tagbogen
        anchor = "noon"
        off = (t - noon).total_seconds()
        span = 100 * off / h
        q = QUARTERS_DAY[min(3, int((span + 100) // 50))]
    else:                                           # Nachtbogen
        anchor = "midnight"
        if t >= sunset:                             # evening side: ○ ahead
            mid = mid_after
            next_d = d + timedelta(days=1)
            nxt = day_events(next_d, lat, lon)
            half_night = (mid - sunset).total_seconds() if t < mid else (nxt["sunrise"] - mid).total_seconds()
        else:                                       # morning side: ○ behind
            mid = mid_before
            prev = day_events(d - timedelta(days=1), lat, lon)
            half_night = (sunrise - mid).total_seconds() if t >= mid else (mid - prev["sunset"]).total_seconds()
        off = (t - mid).total_seconds()
        span = 100 * off / half_night
        q = QUARTERS_NIGHT[min(3, int((span + 100) // 50))]
        out["half_night_s"] = round(half_night, 3)

    out["anchor"] = anchor
    out["offset_s"] = round(off, 3)
    out["span"] = round(span, 4)
    out["display_span"] = int(math.floor(span))              # floor of signed value (BZ spec §5.2)
    out["display_offset_ks"] = round(floor_div(off, 10) / 1000, 2)
    out["quarter"] = q
    out["written"] = written_form(out["display_span"], out["display_offset_ks"], anchor)
    return out


def written_form(span: int, ks: float, anchor: str) -> str:
    """Combined written form (BZ spec §3.7): number left of glyph = before, right = after."""
    g = "☉" if anchor == "noon" else "○"
    if span < 0:
        return f"{-span} [{-ks:.2f} ks] {g}"
    if span == 0 and ks == 0:
        return g
    return f"{g} {span} [{ks:.2f} ks]"


CASES = [
    ("Houston, 1 Nov 2026, 11:08 CST", "2026-11-01T17:08:00Z", 29.76, -95.37),
    ("Houston, 1 Nov 2026, solar noon", "2026-11-01T18:05:06Z", 29.76, -95.37),
    ("Houston, 1 Nov 2026, 21:00 CST", "2026-11-02T03:00:00Z", 29.76, -95.37),
    ("Houston, 2 Nov 2026, 00:02 CST (civil date 2 Nov, BZ date 1 Nov)", "2026-11-02T06:02:00Z", 29.76, -95.37),
    ("Houston, 2 Nov 2026, 03:00 CST", "2026-11-02T09:00:00Z", 29.76, -95.37),
    ("Philmont, 21 Jun 2027, 09:00 MDT", "2027-06-21T15:00:00Z", 36.45, -104.95),
    ("Equator, Greenwich, 20 Mar 2027, 06:30 UTC", "2027-03-20T06:30:00Z", 0.0, 0.0),
    ("Berlin, 21 Dec 2026, 14:00 CET", "2026-12-21T13:00:00Z", 52.52, 13.40),
    ("Tromsø, 21 Jun 2027, 12:00 UTC (polar day)", "2027-06-21T12:00:00Z", 69.65, 18.96),
    ("Houston, 1 Nov 2026, last whole second before sunset", "2026-11-01T23:35:38Z", 29.76, -95.37),
    ("Houston, 1 Nov 2026, first whole second after sunset", "2026-11-01T23:35:39Z", 29.76, -95.37),
    ("Houston, 1 Nov 2026, last whole second before sunrise", "2026-11-01T12:34:33Z", 29.76, -95.37),
    ("Houston, 1 Nov 2026, first whole second after sunrise", "2026-11-01T12:34:34Z", 29.76, -95.37),
    ("Equator, Greenwich, 20 Mar 2027, last whole second before ○ (BZ date 19 Mar)", "2027-03-20T00:08:18Z", 0.0, 0.0),
    ("Equator, Greenwich, 20 Mar 2027, first whole second after ○ (BZ date 20 Mar)", "2027-03-20T00:08:19Z", 0.0, 0.0),
    ("Sydney, 21 Dec 2026, 15:00 AEDT (southern summer)", "2026-12-21T04:00:00Z", -33.87, 151.21),
    ("Ushuaia, 21 Jun 2027, 10:00 ART (southern winter)", "2027-06-21T13:00:00Z", -54.80, -68.30),
    ("Suva, 1 Jan 2027, 00:30 FJT (UTC date 31 Dec)", "2026-12-31T12:30:00Z", -18.14, 178.44),
    ("Apia, 1 Jan 2027, 00:30 WSST (UTC+13, longitude west)", "2026-12-31T11:30:00Z", -13.83, -171.75),
    ("Date line, 180°E, 1 Jan 2027, 00:30 UTC", "2027-01-01T00:30:00Z", 0.0, 180.0),
    ("Date line, 180°W, 1 Jan 2027, 00:30 UTC (same instant, BZ date 31 Dec)", "2027-01-01T00:30:00Z", 0.0, -180.0),
    ("Berlin, 29 Feb 2028, 12:00 CET (leap day)", "2028-02-29T11:00:00Z", 52.52, 13.40),
    ("Tromsø, 21 Dec 2026, 12:00 CET (polar night, noon anchor)", "2026-12-21T11:00:00Z", 69.65, 18.96),
    ("Tromsø, 21 Dec 2026, 01:00 CET (polar night, midnight anchor)", "2026-12-21T00:00:00Z", 69.65, 18.96),
    ("Longyearbyen, 20 Feb 2027, 12:00 CET (first day the Sun clears the horizon)", "2027-02-20T11:00:00Z", 78.22, 15.63),
]


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "--vectors":
        vecs = []
        for name, ts, lat, lon in CASES:
            t = datetime.fromisoformat(ts.replace("Z", "+00:00"))
            vecs.append({"name": name, "utc": ts, "lat": lat, "lon": lon, "expected": bogenzeit(t, lat, lon)})
        print(json.dumps({"algorithm": "NOAA general solar position (fractional year)",
                          "tolerance": {"event_times_s": 1, "display_values": "exact"},
                          "vectors": vecs}, indent=2, ensure_ascii=False))
        return
    ts, lat, lon = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
    t = datetime.fromisoformat(ts.replace("Z", "+00:00"))
    print(json.dumps(bogenzeit(t, lat, lon), indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
