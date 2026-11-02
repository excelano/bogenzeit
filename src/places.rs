// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

//! Named places from the config file, so `bz now` needs no coordinates on
//! the command line. The file is `$XDG_CONFIG_HOME/bz/config`, or
//! `~/.config/bz/config`, one place per line as `name = lat, lon`, and
//! `place = name` for the one used when none is asked for.

use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Places {
    pub default: Option<String>,
    pub places: Vec<(String, f64, f64)>,
}

impl Places {
    pub fn path() -> Option<PathBuf> {
        if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").filter(|d| !d.is_empty()) {
            return Some(PathBuf::from(dir).join("bz").join("config"));
        }
        std::env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("bz")
                .join("config")
        })
    }

    /// The config file's places; an absent file is an empty list.
    pub fn load() -> Result<Places, String> {
        let Some(path) = Self::path() else {
            return Ok(Places {
                default: None,
                places: Vec::new(),
            });
        };
        match fs::read_to_string(&path) {
            Ok(text) => Self::parse(&text).map_err(|e| format!("{}: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Places {
                default: None,
                places: Vec::new(),
            }),
            Err(e) => Err(format!("{}: {e}", path.display())),
        }
    }

    pub fn parse(text: &str) -> Result<Places, String> {
        let mut default = None;
        let mut places = Vec::new();
        for (number, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once('=').ok_or_else(|| {
                format!(
                    "line {}: expected `name = lat, lon` or `place = name`",
                    number + 1
                )
            })?;
            let (key, value) = (key.trim(), value.trim());
            if key == "place" {
                default = Some(value.to_string());
                continue;
            }
            let (lat, lon) = value
                .split_once(',')
                .ok_or_else(|| format!("line {}: `{key}` needs `lat, lon`", number + 1))?;
            let lat = degrees(lat.trim(), 90.0)
                .map_err(|e| format!("line {}: {key}: {e}", number + 1))?;
            let lon = degrees(lon.trim(), 180.0)
                .map_err(|e| format!("line {}: {key}: {e}", number + 1))?;
            places.push((key.to_string(), lat, lon));
        }
        if let Some(name) = &default {
            if !places.iter().any(|(n, _, _)| n == name) {
                return Err(format!("place = {name}, but no line defines {name}"));
            }
        }
        Ok(Places { default, places })
    }

    /// Adds or replaces a place; the first place written becomes the default.
    pub fn set(&mut self, name: &str, lat: f64, lon: f64) {
        if let Some(entry) = self.places.iter_mut().find(|(n, _, _)| n == name) {
            entry.1 = lat;
            entry.2 = lon;
        } else {
            self.places.push((name.to_string(), lat, lon));
        }
        if self.default.is_none() {
            self.default = Some(name.to_string());
        }
    }

    pub fn make_default(&mut self, name: &str) -> Result<(), String> {
        if self.find(name).is_none() {
            return Err(format!("no place named {name}; `bz places` lists them"));
        }
        self.default = Some(name.to_string());
        Ok(())
    }

    pub fn forget(&mut self, name: &str) -> Result<(), String> {
        let before = self.places.len();
        self.places.retain(|(n, _, _)| n != name);
        if self.places.len() == before {
            return Err(format!("no place named {name}; `bz places` lists them"));
        }
        if self.default.as_deref() == Some(name) {
            self.default = self.places.first().map(|(n, _, _)| n.clone());
        }
        Ok(())
    }

    /// The file as `bz place` writes it: the default first, then every
    /// place. Comments in a hand-written file do not survive a rewrite.
    pub fn render(&self) -> String {
        let mut out = String::new();
        if let Some(name) = &self.default {
            out.push_str(&format!("place = {name}\n"));
        }
        for (name, lat, lon) in &self.places {
            out.push_str(&format!("{name} = {lat}, {lon}\n"));
        }
        out
    }

    /// Writes the file, making its folder, through a temporary file and a
    /// rename so a crash leaves the old file or the new, never half of one.
    pub fn save(&self) -> Result<PathBuf, String> {
        let path = Self::path().ok_or("no home directory to keep the config in")?;
        let dir = path.parent().ok_or("the config path has no folder")?;
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let temp = dir.join(".config.bz-tmp");
        fs::write(&temp, self.render()).map_err(|e| format!("{}: {e}", temp.display()))?;
        fs::rename(&temp, &path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(path)
    }

    pub fn find(&self, name: &str) -> Option<(f64, f64)> {
        self.places
            .iter()
            .find(|(n, _, _)| n == name)
            .map(|(_, lat, lon)| (*lat, *lon))
    }
}

/// Decimal degrees, signed or with a hemisphere letter before or after:
/// `-95.47833`, `W095.47833` and `95.47833W` are the same longitude.
pub fn degrees(text: &str, limit: f64) -> Result<f64, String> {
    let trimmed = text.trim();
    let first = trimmed.chars().next().map(|c| c.to_ascii_uppercase());
    let last = trimmed.chars().last().map(|c| c.to_ascii_uppercase());
    let (number, sign) = match (first, last) {
        (Some(c @ ('N' | 'S' | 'E' | 'W')), _) => (trimmed[1..].trim(), hemisphere(c)),
        (_, Some(c @ ('N' | 'S' | 'E' | 'W'))) => {
            (trimmed[..trimmed.len() - 1].trim(), hemisphere(c))
        }
        _ => (trimmed, 1.0),
    };
    let value: f64 = number
        .parse()
        .map_err(|_| format!("`{text}` is not a number of degrees"))?;
    if sign < 0.0 && value < 0.0 {
        return Err(format!("`{text}` has both a minus sign and a hemisphere"));
    }
    let value = value * sign;
    if !(-limit..=limit).contains(&value) {
        return Err(format!("{text} is outside -{limit} to {limit}"));
    }
    Ok(value)
}

fn hemisphere(letter: char) -> f64 {
    if letter == 'S' || letter == 'W' {
        -1.0
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_places_and_a_default() {
        let p = Places::parse(
            "# where I am\nplace = home\nhome = 29.76, -95.37\ncamp = 36.45, -104.95  # Philmont\n",
        )
        .unwrap();
        assert_eq!(p.default.as_deref(), Some("home"));
        assert_eq!(p.find("camp"), Some((36.45, -104.95)));
        assert_eq!(p.places.len(), 2);
    }

    #[test]
    fn sets_defaults_and_forgets_and_renders() {
        let mut p = Places::parse("").unwrap();
        p.set("home", 29.76, -95.37);
        p.set("camp", 36.45, -104.95);
        assert_eq!(p.default.as_deref(), Some("home"));
        p.set("home", 29.8, -95.4);
        assert_eq!(p.find("home"), Some((29.8, -95.4)));
        p.make_default("camp").unwrap();
        assert!(p.make_default("nowhere").is_err());
        assert_eq!(
            p.render(),
            "place = camp\nhome = 29.8, -95.4\ncamp = 36.45, -104.95\n"
        );
        p.forget("camp").unwrap();
        assert_eq!(p.default.as_deref(), Some("home"));
        assert!(p.forget("camp").is_err());
        let again = Places::parse(&p.render()).unwrap();
        assert_eq!(again.places.len(), 1);
    }

    #[test]
    fn reads_hemisphere_letters() {
        assert_eq!(degrees("N29.67230", 90.0).unwrap(), 29.6723);
        assert_eq!(degrees("W095.47833", 180.0).unwrap(), -95.47833);
        assert_eq!(degrees("95.47833W", 180.0).unwrap(), -95.47833);
        assert_eq!(degrees("s 33.87", 90.0).unwrap(), -33.87);
        assert_eq!(degrees("-95.47833", 180.0).unwrap(), -95.47833);
        assert!(degrees("W-95", 180.0).is_err());
        assert!(degrees("N", 90.0).is_err());
    }

    #[test]
    fn refuses_a_default_without_a_line_and_bad_degrees() {
        assert!(Places::parse("place = home\n")
            .unwrap_err()
            .contains("no line defines home"));
        assert!(Places::parse("home = 95, 0\n")
            .unwrap_err()
            .contains("outside"));
        assert!(Places::parse("home 29.76\n")
            .unwrap_err()
            .contains("line 1"));
        assert!(Places::parse("").unwrap().places.is_empty());
    }
}
