use regex::Regex;
use snafu::{ensure, Snafu};
use std::{fs, ops::RangeInclusive, path::Path, str::FromStr};

/// Get a positive inclusive range (..=) from a string in the format of "20-50" or "24".
///
/// This function does some clean-up beforehand to remove trailing and repeating dashes.
/// So `---20-----30--` becomes `20-30`, and gives no error.
///
/// TODO: Adjust it accordingly when making the example GUI.
pub fn range_inc_from_str(range: &str) -> Result<RangeInclusive<usize>, ParseRangeError> {
    let min;
    let max;

    let range = range.trim_start_matches('-').trim_end_matches('-');
    let re = Regex::new(r"-+").unwrap();
    let range = re.replace_all(range, "-");

    ensure!(range.matches('-').count() <= 1, MoreThanTwoSidesSnafu);

    ensure!(
        range.chars().all(|c| c.is_numeric() || c == '-'),
        ContainsNonintegerOrDashSnafu
    );

    if range.contains('-') {
        let r: Vec<&str> = range.split('-').collect();
        min = usize::from_str(r[0]).unwrap();
        max = usize::from_str(r[1]).unwrap();

        ensure!(min <= max, RightSideIsSmallerSnafu);

        Ok(RangeInclusive::new(min, max))
    } else {
        min = usize::from_str(&range).unwrap();
        max = min;

        Ok(RangeInclusive::new(min, max))
    }
}

/// The errors that parsing a range from a string can return.
#[derive(Debug, Snafu)]
pub enum ParseRangeError {
    /// When the string contains more than two numbers separated by a dash like "20-30-40".
    #[snafu(display("more than two sides"))]
    MoreThanTwoSides,
    /// When the string contains something other than integers and dashes like "25.5-40".
    #[snafu(display("contains something other than integers and a - (dash)"))]
    ContainsNonintegerOrDash,
    /// When the right side of the range is smaller than the left side like "35-25".
    #[snafu(display("right side of range can't be smaller than left side"))]
    RightSideIsSmaller,
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
