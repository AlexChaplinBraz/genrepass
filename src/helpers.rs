use crate::settings::RangeError;
use regex::Regex;
use std::{fs, ops::RangeInclusive, path::Path, str::FromStr};

/// Get a positive inclusive range from a string in the format of "20-50".
///
/// Trims off any extra dashes at the start and end and between them.
///
/// TODO: Adjust it accordingly when making the example GUI.
pub fn range_inc_from_str(range: &str) -> Result<RangeInclusive<usize>, RangeError> {
    let min;
    let max;

    let range = range.trim_start_matches('-').trim_end_matches('-');
    let re = Regex::new(r"-+").unwrap();
    let range = re.replace_all(range, "-");

    if range.matches('-').count() > 1 {
        return Err(RangeError {
            message: "more than two sides",
        });
    }

    if !range.chars().all(|c| c.is_numeric() || c == '-') {
        return Err(RangeError {
            message: "contains something other than integers and a - (dash)",
        });
    }

    if range.contains('-') {
        let r: Vec<&str> = range.split('-').collect();
        min = usize::from_str(r[0]).unwrap();
        max = usize::from_str(r[1]).unwrap();

        if max < min {
            return Err(RangeError {
                message: "right side of range can't be smaller than left side",
            });
        }

        Ok(RangeInclusive::new(min, max))
    } else {
        min = usize::from_str(&range).unwrap();
        max = min;

        Ok(RangeInclusive::new(min, max))
    }
}

pub(crate) fn get_text_from_dir(
    dir: impl AsRef<Path>,
    text: &mut String,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            get_text_from_dir(&path, text)?;
        } else {
            text.push_str(fs::read_to_string(&path).unwrap_or_default().as_str());
        }
    }

    Ok(())
}

pub(crate) fn capitalise(s: &mut str, i: usize) {
    if let Some(c) = s.get_mut(i..i + 1) {
        c.make_ascii_uppercase();
    }
}

pub(crate) fn decapitalise(s: &mut str, i: usize) {
    if let Some(c) = s.get_mut(i..i + 1) {
        c.make_ascii_lowercase();
    }
}
