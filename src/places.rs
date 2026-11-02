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

    pub fn find(&self, name: &str) -> Option<(f64, f64)> {
        self.places
            .iter()
            .find(|(n, _, _)| n == name)
            .map(|(_, lat, lon)| (*lat, *lon))
    }
}

pub fn degrees(text: &str, limit: f64) -> Result<f64, String> {
    let value: f64 = text
        .parse()
        .map_err(|_| format!("`{text}` is not a number of degrees"))?;
    if !(-limit..=limit).contains(&value) {
        return Err(format!("{text} is outside -{limit} to {limit}"));
    }
    Ok(value)
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
