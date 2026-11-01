# Bogenzeit — Spec (draft 0.12)

Bogenzeit is a personal way to see and think about time, centered on the local Sun.
It connects to UTC and civil time zones, but the primary readout follows
solar noon, sunrise, and sunset at the observer's location.

Status: draft 0.12. First lives as this file; the **Tagbogen** watch face comes next.

**Bogenzeit** ("arc time", abbreviated **BZ**) is the time standard: time read as position on the Sun's arc, not as a count of hours. **Tagbogen** is the watch face that displays it.

---

## 1. Principles

1. **UTC is the reference.** Every moment is a UTC instant. Nothing personal is stored; every readout is computed.
2. **Solar noon is the anchor.** Sunrise and sunset are the secondary anchors.
3. **Location is an input.** Solar times change with longitude (east–west), latitude (north–south), and season. Location must be tracked, not assumed.
4. **Keep existing names for existing definitions.** Days are days, months are months.
5. **No hours or minutes in the readout.** The second is the base unit. Readouts use kiloseconds (fixed) and spans (proportional).
6. **New terms only for new definitions.** Anything defined differently gets its own name, marked *(coined)* in this spec.
7. **Purpose: seeing and thinking, not logging.** The system describes where you are in the solar day. It does not record activity.

---

## 2. Terms

### 2.1 Existing terms (used as standard)

| Term | Definition used here |
|---|---|
| Second | SI base unit. The true unit underneath everything. |
| Kilosecond (ks) | 1000 s (about 16.7 min). SI prefix, standard meaning. An apparent solar day is about 86.4 ks. |
| UTC | Coordinated Universal Time. The reference for every instant. |
| Civil time | Local time-zone time (IANA zone, including DST). Shown for coordination only. |
| Local apparent solar time | Sundial time at the observer's longitude. Solar noon = 12:00. |
| Solar noon | The Sun crosses the local meridian (highest point of the day). |
| Solar midnight | The Sun crosses the anti-meridian (lowest point). Center of the night. |
| Apparent solar day | Solar midnight to solar midnight, centered on solar noon. Length varies slightly (±~30 s) through the year. |
| Sunrise / sunset | Sun's upper limb on the horizon; Sun's center at −0.833° altitude (includes refraction). |
| Civil / nautical / astronomical twilight | Sun's center at −6° / −12° / −18°. |
| Daylight | Sunrise to sunset. |
| Temporal hours | Daylight (or night) divided into 12 equal parts; their length changes with the season. |
| Equation of time | Apparent solar time minus mean solar time. Ranges about −14 to +16 min over the year. |
| Day (civil date) | The calendar date in the local zone. Still the label for "which day." |
| Week, month, year | Unchanged Gregorian definitions. |
| Equinox, solstice | Unchanged astronomical definitions. |
| Tagbogen (day arc) | German astronomical term: the part of a body's daily path above the horizon. For the Sun: sunrise → sunset. |
| Nachtbogen (night arc) | The part of the daily path below the horizon. For the Sun: sunset → sunrise. |

### 2.2 Coined terms

| Term | Definition |
|---|---|
| **Bogenzeit (BZ)** *(coined)* | This time standard: spans, ks offsets, ☉/○ anchors, and positional notation, built on local apparent solar time. Name chosen over "Local Solar Time" (already means sundial time; LST also = Local Sidereal Time). |
| **Noon offset** *(coined, working name)* | Time from today's local solar noon, in kiloseconds, two decimals (10 s resolution). Written with the ☉ glyph (§3). |
| **Midnight offset** *(coined, working name)* | Time from local solar midnight, in kiloseconds. Written with the ○ glyph (§3). Used at night in place of the noon offset. |
| **Span** *(coined, working name)* | Proportional unit: each half of the Tagbogen is 100 spans, and each half of the Nachtbogen is 100 spans. Span length in seconds = (that half's length in s) / 100, so it changes with season and latitude. |

| **Quarters** *(coined)* | Each arc is split into four quarters at 50 spans. Eight named periods: Afterrise, Forenoon, Afternoon, Foreset (Tagbogen); Afterset, Foremid, Aftermid, Forerise (Nachtbogen). See §3.6. |

**Naming note — "span":** no conflict as a time *unit* (no standard or historical unit of time is called a span). Overlaps to be aware of: "span" is a traditional unit of *length* (~9 in, a hand's width); in software, `Span` is an interval with a start and end (OpenTelemetry, Rust `tracing`) and `TimeSpan` is a duration type in .NET. In code, use a qualified name (e.g. `SolarSpan`) to avoid collisions.

Naming of coined terms is still open.

---

## 3. Notation and readout

### 3.1 Anchor glyphs

| Anchor | Glyph | Code point | Meaning |
|---|---|---|---|
| Solar noon | ☉ | U+2609 SUN | The astronomical symbol for the Sun: the disc with its center. |
| Solar midnight | ○ | U+25CB WHITE CIRCLE | The same disc with the center gone: the Sun on the far side. |

Both are plain text symbols, not emoji.

**Matching rule:** ☉ and ○ must always render at the same size, weight, and baseline position. They come from different Unicode blocks (Miscellaneous Symbols vs. Geometric Shapes), and no font tested draws them identically:

| Font | ☉ width | ○ width | Difference |
|---|---|---|---|
| Noto Sans Symbols 2 | 0.768 em | 0.792 em | ○ 3% larger |
| Noto Sans Math | 0.768 em | 0.734 em | ○ 4% smaller |
| DejaVu Sans | 0.730 em | 0.763 em | ○ 5% larger, sits 0.125 em lower |
| DejaVu Sans Mono | 0.559 em | 0.596 em | ○ 7% larger |

Therefore:
- **Watch face:** draw both anchors as vector shapes from one shared definition (same outer radius, same stroke). ☉ = that circle plus a center dot; ○ = that circle alone. Do not render them from a font.
- **Bundled font (if text rendering is needed):** subset a font and redraw ○ as ☉'s outline with the dot removed.
- **Plain text (this file, notes):** exact matching is not guaranteed; accepted.
- Both anchors should be visibly larger than the digits, so ○ is not read as `o`, `0`, or `°`.

### 3.2 Positional notation (display)

The glyph is a fixed point on a timeline read left to right. **Position replaces sign:**

| Display | Meaning |
|---|---|
| `11.77 ☉` | 11.77 ks *before* solar noon (the number comes first: still to come) |
| `☉` | exactly solar noon |
| `☉ 5.50` | 5.50 ks *after* solar noon |
| `8.00 ○` | 8.00 ks before solar midnight |
| `☉ 100` / `100 ○` | sunset (end of daylight / start of night — the same instant) |

- No `+` or `−` characters in display.
- A thin space (U+2009) separates number and glyph.
- The same grammar is used for spans and for ks offsets. Units distinguish them: spans are bare numbers, offsets carry `ks`.
- The glyph tells day from night, so no separate night marker is needed.
- Left = before assumes left-to-right text. In right-to-left contexts, isolate the readout (U+2066 … U+2069) or the meaning can flip.

### 3.3 Tagbogen (sunrise → sunset)

- **Primary (large): span position**, e.g. `59 ☉`. Sunrise is `100 ☉`, sunset is `☉ 100`.
- **Secondary (small): noon offset**, e.g. `11.77 ks ☉`.
- Context: half-daylight length, e.g. `½ day 19.8 ks` (so a span is 198 s today).
- Civil time shown small, for coordination only.

### 3.4 Nachtbogen (sunset → sunrise)

- **Primary: span position** relative to ○, e.g. `48 ○`. Sunset is `100 ○`, sunrise is `○ 100`.
- **Secondary: midnight offset**, e.g. `11.11 ks ○`.
- At sunset the anchor switches from ☉ to ○; at sunrise it switches back. Each half of the day is read against its own center.
- Twilight phases shown by name (civil / nautical / astronomical) when active.

### 3.5 Polar edge cases

At high latitudes there may be no sunrise or sunset on some dates (polar day or polar night). Solar noon and solar midnight still exist every day, so the noon offset always works; sunrise/sunset fields show "none" and spans are not shown.

### 3.6 Quarters

Each arc splits into four quarters at the 50-span marks. Every quarter touches exactly one horizon event (sunrise or sunset) and one anchor (☉ or ○), and is named for its **nearest horizon or anchor event**, with **Fore-** (before it) or **After-** (after it).

The rule matches positional notation: *Fore-* = number left of the glyph, *After-* = number right of the glyph.

| Arc | Spans | Quarter | Nearest event | Today, Houston (CST, approx.) |
|---|---|---|---|---|
| Tagbogen | `100 ☉` → `50 ☉` | **Afterrise** | after sunrise | 06:34 → 09:20 |
| | `50 ☉` → `☉` | **Forenoon** | before solar noon | 09:20 → 12:05 |
| | `☉` → `☉ 50` | **Afternoon** | after solar noon | 12:05 → 14:50 |
| | `☉ 50` → `☉ 100` | **Foreset** | before sunset | 14:50 → 17:36 |
| Nachtbogen | `100 ○` → `50 ○` | **Afterset** | after sunset | 17:36 → 20:50 |
| | `50 ○` → `○` | **Foremid** | before solar midnight | 20:50 → 00:05 |
| | `○` → `○ 50` | **Aftermid** | after solar midnight | 00:05 → 03:20 |
| | `○ 50` → `○ 100` | **Forerise** | before sunrise | 03:20 → 06:35 |

**Quarter boundaries** have no names of their own; they are written in notation: `50 ☉`, `☉ 50`, `50 ○`, `○ 50`.

**Roots:** *rise* = sunrise, *noon* = solar noon (☉), *set* = sunset, *mid* = solar midnight (○). Full forms Foremidnight / Aftermidnight may be used where space allows.

**Properties**
- Quarter length = 50 spans of its arc, so it changes with season and latitude. Today a Tagbogen quarter is about 9.9 ks; in Houston it ranges from roughly 9 ks (December) to 12–13 ks (June). Nachtbogen quarters do the opposite.
- The Tagbogen and Nachtbogen boundaries meet at sunrise and sunset, so the eight quarters cover the full apparent solar day with no gaps.

**Naming notes**
- All eight are coined Bogenzeit terms and are always capitalized. *Forenoon* and *Afternoon* are also existing English words with looser meanings; the capital letter marks the Bogenzeit definition ("Afternoon" = `☉` → `☉ 50`, not "sometime after lunch").
- *Foreset* is also a geology term (foreset beds); no conflict in time contexts.
- Boundaries: a quarter includes its start and excludes its end (e.g. exactly `50 ☉` is Forenoon).
- Polar day/night: with no sunrise or sunset, quarters are undefined (same fallback as spans, §3.5).

**Readout:** the watch face may show the current quarter name as context, e.g. `Forenoon | 21 ☉`. A single drawn quadrant mark (not a font glyph, per the matching rule in §3.1) can indicate the quarter graphically.

### 3.7 Written forms

Spans and ks do not nest the way hours, minutes, and seconds do: a span is not a whole number of ks, and its length varies. They are two parallel readings of the same moment, so there is no `hh:mm:ss`-style colon form. Instead there are three standard forms:

| Form | Before ☉ | After ☉ | Use |
|---|---|---|---|
| **Span** | `59 ☉` | `☉ 3` | Everyday, glanceable |
| **Offset** | `11.77 ks ☉` | `☉ 0.70 ks` | Fixed length; comparison and durations |
| **Combined** | `59 [11.77 ks] ☉` | `☉ 3 [0.70 ks]` | Notes, logs, full readout |

Rules:
- The glyph appears once; all numbers sit on the side the positional rule (§3.2) puts them.
- Combined form: span first, then the offset in square brackets. Brackets mean "the same moment, as a fixed offset".
- Spans are bare numbers; offsets always carry `ks`.
- Same forms at night with ○: `40 [8.00 ks] ○`.

**Precision:** ks with three decimals is exact to the second (`11.766 ks ☉` = 11,766 s before solar noon). This is the true counterpart of `hh:mm:ss`: two decimals for display, three for exact.

**Separators:**
- `[ ]` — parallel reading (offset alongside span).
- `|` — between fields of a full timestamp or readout: `2026-11-01 | 29.76, −95.37 | 59 [11.77 ks] ☉`; `Forenoon | 21 ☉`.
- Not used: `·` (confusable with `.`), `~` (reads as "approximately"), `/` (reads as a ratio), `,` (decimal mark in German).
- Markdown note: in Markdown files, a bracketed reading directly followed by `(` would form a link; keep a space or other text after `]`.

---

## 4. Display vs. data

Positional notation is **display only**. Anything stored, exchanged, or computed uses signed values:

| Quantity | Data form | Example |
|---|---|---|
| Instant | UTC, RFC 3339 or Unix seconds | `2026-11-01T14:49:00Z` |
| Noon / midnight offset | signed integer seconds + anchor | `{anchor: "noon", offset_s: -11766}` |
| Span position | signed float + anchor | `{anchor: "noon", span: -59.3}` |

Negative = before the anchor, positive = after. Display renders negative values with the number left of the glyph and positive values to the right.

---

## 5. Dates, resolution, timestamps, and location

### 5.1 Date and day switchover

Days, weeks, months, and years are unchanged Gregorian units. Only the **switchover** moves:

- The date changes at **solar midnight (○)**, not civil midnight.
- The Bogenzeit date of an apparent solar day = the Gregorian (civil) date on which that day's ☉ falls.
- A day includes its starting ○ and excludes its ending ○. So Aftermid and Forerise belong to the new date; Afterset and Foremid belong to the old one.
- Example: Houston tonight, ○ ≈ 00:05 CST. From 00:00 to 00:05 civil, the civil date is 2 Nov but the Bogenzeit date is still 1 Nov.

### 5.2 Display resolution and rounding

| Readout | Resolution | Typical size |
|---|---|---|
| Span | whole number | ~200 s (varies with season and arc) |
| Offset | ks, two decimals | 10 s |
| Second | whole number | 1 s (data and precise work) |

**Rounding rule: round down the signed value** (floor toward the past). The display always shows the last boundary that has been crossed, never the next one, as a clock does with hours and minutes.

- After an anchor: `☉ 3.7` spans → shows `☉ 3`.
- Before an anchor: −59.3 spans (59.3 before ☉) → floor is −60 → shows `60 ☉`. It changes to `59 ☉` exactly when 59 spans remain.
- So `☉` appears only from exact solar noon, and `☉ 100` (sunset) only at exact sunset. No event is shown early.
- The same rule applies to ks offsets (floor of signed seconds ÷ 10). A readout that cannot refresh every 10 s, such as a watch-face complication, may show one decimal, floored the same way.

**No intermediate unit.** Quarters (~10 ks) → spans (~200 s) → seconds is a usable ladder, comparable to hours → minutes → seconds. ks is a parallel fixed scale, not a level in that ladder.

### 5.3 Full Bogenzeit timestamp

A position alone (`59 ☉`) is ambiguous, since spans depend on date and place. A full timestamp is:

**Gregorian date (per §5.1) + coordinates + position**

- Display: `2026-11-01 | 29.76, −95.37 | 59 [11.77 ks] ☉` (see §3.7)
- Data: `{date: "2026-11-01", lat: 29.76, lon: -95.37, anchor: "noon", span: -59.3}` (or `offset_s` instead of `span`)
- Coordinates in decimal degrees. Two decimals (~1 km) are enough for display, since 0.01° of longitude ≈ 2.4 s of solar time; data should keep full GPS precision.
- The UTC instant remains canonical. A Bogenzeit timestamp converts to exactly one UTC instant, and back.

### 5.4 Elevation and terrain

Ignored. Sunrise and sunset are computed for a sea-level, flat horizon (Sun's center at −0.833°).

- **Solar noon and solar midnight are unaffected.** They are meridian crossings and do not depend on the horizon. ☉ and ○ stay exact anywhere.
- Only sunrise, sunset, and therefore span lengths and quarter boundaries shift. Observer elevation lowers the apparent horizon (for example, at ~2,000 m sunrise is several minutes earlier); surrounding terrain raises it. Both effects are symmetric around ☉, so spans remain symmetric.
- Ignoring them keeps Bogenzeit reproducible from date + coordinates alone.

### 5.5 Location changes

- **Normative:** Bogenzeit is defined by the exact coordinates at each instant. It shifts continuously as you move.
- **Implementations** may approximate by recomputing only when needed. Recommended: recompute when moved more than **10 km**, and at least once per solar day. At Houston's latitude, 10 km of east–west movement shifts ☉ by about 25 s, which is below one ks display step (100 s).
- A timestamp (§5.3) records the coordinates actually used for that reading.
- Coordination between people in different places is out of scope for now.

### 5.6 Span length reference

A span's length in seconds = (half of its arc) / 100. Computed with the §6 method (NOAA approximation; accurate to about a second):

| Place | Latitude | Day span | Night span |
|---|---|---|---|
| Equator | 0° | 218 s (3.6 min), nearly constant | 214 s (3.6 min) |
| Houston | 29.76° N | 184–253 s (3.1–4.2 min) | 179–248 s (3.0–4.1 min) |
| Philmont | 36.45° N | 174–264 s (2.9–4.4 min) | 168–258 s (2.8–4.3 min) |
| Berlin | 52.52° N | 138–303 s (2.3–5.1 min) | 129–294 s (2.1–4.9 min) |
| Reykjavík | 64.15° N | 74–381 s (1.2–6.4 min) | 51–358 s (0.9–6.0 min) |
| Tromsø | 69.65° N | 0–432 s | 0–432 s (polar day/night) |

**Properties**
- **Day span + night span = 432 s, always.** Half the daylight plus half the night is always 12 h (43,200 s), so 100 day spans + 100 night spans = 43,200 s. When day spans shrink, night spans grow by exactly the same amount.
- **Hard bounds: 0 < span < 432 s** (7.2 min, 0.432 ks). A span is always smaller than 1 ks. It reaches the bounds only at polar day or polar night (§3.5).
- **Readout rhythm:** the ks offset steps every 10 s; at mid-latitudes a span lasts ~150–300 s, so the secondary ks readout ticks fifteen to thirty times per span.
- At Houston's latitude spans stay within ~3–4 min year-round, close to the scale of familiar clock minutes.

---

## 6. Computation

**Inputs**
- UTC instant
- Latitude and longitude (from device location, or a saved place)
- IANA time zone (for civil display only)

**Method**
- Solar position: NOAA solar calculator algorithm (equation of time + declination). Good to about a minute at mid-latitudes. A higher-precision method (e.g. SPA) can replace it later.
- Solar noon (UTC) = 12:00 − 4 min × longitude° − equation of time.
- Sunrise/sunset: hour angle at −0.833° altitude. Twilights at −6°, −12°, −18°.
- Noon offset (s) = instant − solar noon of the current apparent solar day; display ÷ 1000, two decimals.
- Midnight offset (s) = instant − nearest solar midnight (during the night).
- Day span = 100 × (instant − solar noon) / (solar noon − sunrise) before noon, or / (sunset − solar noon) after.
- Night span = 100 × (instant − solar midnight) / (solar midnight − sunset) before midnight, or / (sunrise − solar midnight) after.

**Rules**
- The apparent solar day containing an instant runs from the preceding to the following solar midnight.
- Location changes take effect immediately. The readout follows the Sun where you are.
- DST and time-zone changes affect only the civil display, never the solar readout.

---

## 7. Worked example — Houston, Sunday 1 Nov 2026

Location: 29.76° N, 95.37° W. DST ended at 02:00 this morning; now CST (UTC−6).

| Event | Civil (CST) | Offset | Span |
|---|---|---|---|
| Sunrise | 06:34 | 19.83 ks ☉ | 100 ☉ |
| 08:49 (example) | 08:49 | 11.77 ks ☉ | 59 ☉ |
| Solar noon | 12:05 | ☉ | ☉ |
| Sunset | 17:36 | ☉ 19.83 ks | ☉ 100 = 100 ○ |
| 21:00 (example) | 21:00 | 11.11 ks ○ | 48 ○ |
| Solar midnight | ~00:05 (2 Nov) | ○ | ○ |

Half-daylight ≈ 19,845 s, so one day span ≈ 198 s. Half-night ≈ 23.3 ks, so one night span ≈ 233 s. (Night values approximate.)

Why noon is at 12:05 CST:
- Houston is about 5.4° west of the CST meridian (90° W), so the Sun is about 21.5 min later.
- The equation of time is near its yearly maximum (about +16.4 min), so the Sun is about 16 min earlier.
- Net: about +5 min.

Yesterday under CDT, solar noon was about 13:05 civil. The civil clock jumped an hour; the ☉ readout did not.

---

## 8. Open decisions

1. **Final names** for "noon offset", "midnight offset" and "span".
2. **Precision:** NOAA (~1 min) is enough for a face; is SPA-level precision wanted for anything?
3. **Location source on the watch:** live GPS, last known fix, or saved places (home, camp). Recalculation policy is set in §5.5.
4. **Polar case for spans:** with no sunrise/sunset, spans are undefined; fall back to ks only.

**Decided**

- ~~Daylight secondary readout~~ — decided: spans primary, ks noon offset secondary.
- ~~Night secondary readout~~ — decided: ks from solar midnight.
- ~~Anchor glyphs~~ — decided: ☉ noon, ○ midnight.
- ~~Names~~ — decided: Bogenzeit (standard), Tagbogen (watch face).
- ~~Quarters~~ — decided: four per arc, named Fore-/After- + nearest event (§3.6).
- ~~Names for quarter boundaries~~ — decided: none. The 50-span points are written in notation only (`50 ☉`, `☉ 50`, `50 ○`, `○ 50`). Named transitions are limited to sunrise, solar noon, sunset, solar midnight.
- ~~Date switchover~~ — decided: at solar midnight (§5.1).
- ~~Rounding~~ — decided: whole spans, floor of signed value (§5.2).
- ~~Offset resolution~~ — decided: two decimals of ks (10 s) for display; one is allowed where a readout cannot refresh that often (§5.2).
- ~~Full timestamp~~ — decided: Gregorian date + coordinates + position (§5.3).
- ~~Elevation/terrain~~ — decided: ignored (§5.4).
- ~~Written forms~~ — decided: span, offset, and combined `59 [11.8 ks] ☉`; `|` between fields (§3.7).
- ~~Abbreviation~~ — decided: BZ, following the German convention where Z = Zeit (cf. MEZ). Not BZT ("Bogenzeit Time" is redundant, and a zone-style ...T code implies a fixed UTC offset, which BZ does not have). Use `bz` as the code/package prefix.
- ~~Sign notation~~ — decided: positional (left = before, right = after) for display; signed values for data.

---

## 9. Next: Tagbogen watch face

Not yet specified. Inputs carried forward: §3 notation and readout, §4 data forms, §5 dates/resolution/location, §6 computation (must run on-device and offline), §8 decisions.
