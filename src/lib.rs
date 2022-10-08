/*!
# Readable password generator

**Gen**erate a **re**adable **pass**word from an ordered list of words extracted from text.
For improved security, numbers and special characters are inserted at random places.

The point is to replace the standard password generation that is very
tedious to input manually, with a still very secure but much easier
to write password. For the rare occasion where you have to input
it manually, like on a smartphone you're not syncing them to.
It also makes for some interesting passwords,
depending on what you choose to use as source.

Written based on a Computerphile video:
[How to Choose a Password](https://youtu.be/3NjQ9b3pgIg).

# Example

```no_run
use genrepass::*;
use std::error::Error;
use std::process::exit;

fn main() {
    // Take care of errors.
    if let Err(e) = run() {
        eprintln!("Error: {}.", e);
        exit(1);
    }
}

// Create a function for easier error management.
fn run() -> Result<(), Box<dyn Error>> {
    // Create a configuration with default values.
    let mut config = PassConfig::new();

    // Load in and parse the text to use for the password generation.
    config.get_words_from_path("/home/alex/Documents/notes")?;

    // Can be done multiple times to add different directories or files.
    config.get_words_from_path("/home/alex/Documents/Journal/2020.md")?;

    // Can also just load it from a String.
    config.get_words_from_str("A string I got from somewhere");

    // Change the configuration by changing the fields.
    config.pass_amount = 5;
    config.capitalise = true;
    config.length = "30-50".to_string();

    // Check that the configuration is valid.
    let validated = config.validate()?;

    // Generate the password/s based on the validated configuration.
    let passwords = validated.generate_passwords();

    // Use the vector however you need.
    // In this case we put each password on a separate line and print them.
    println!("{}", passwords.join("\n"));

    Ok(())
}
```
*/

use deunicode::deunicode;
use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};
use regex::Regex;
use snafu::Snafu;
use std::{fs, fs::metadata, ops::RangeInclusive, path::Path, str::FromStr};

/// Used for configuring the password generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassConfig {
    /// ### Uppercase the first character of every word
    ///
    /// Makes the password much easier to read, but also slightly less secure
    /// due to the predictability of having capitalised words. Still, the
    /// highly improved readability makes it worth it to always have it on.
    ///
    /// **Default: false**
    pub capitalise: bool,

    /// ### Replace the original characters
    ///
    /// Instead of inserting the numbers and special characters (which preserves
    /// the original letters), replace the characters at random positions.
    ///
    /// **Default: false**
    pub replace: bool,

    /// ### Shuffle the words
    ///
    /// Useful if the source text is just a list of words without order anyway
    /// and you want to have a different order with each run.
    ///
    /// **Default: false**
    pub randomise: bool,

    /// ### Amount of passwords to generate
    ///
    /// Useful for providing a list of passwords to choose from.
    ///
    /// **Default: 1**
    pub pass_amount: usize,

    /// ### Amount of times to try generating password before truncating
    ///
    /// If the range is too small or an exact number, it'll be harder
    /// to get a fitting set of words, so the word selection will restart if
    /// the password exceeds the maximum length. But since it would keep
    /// looping if it doesn't find the right length it needs a way to stop,
    /// which in this case is simply truncating the password to the maximum length.
    ///
    /// **Default: 10**
    pub reset_amount: usize,

    /// ### Set the length of the password
    ///
    /// Can either be a range like 24-30, which will generate a password
    /// between that length, or it can be an exact number like 25
    /// for a password of that exact length.
    ///
    /// **Default: 24-30**
    pub length: RangeInclusive<usize>,

    /// ### Amount of numbers to insert
    ///
    /// Can take either a range like 2-4 or an exact amount like 2.
    /// Doesn't take into consideration the amount of numbers already
    /// in the password if 'keep-nums' is activated.
    ///
    /// **Default: 1-2**
    pub number_amount: RangeInclusive<usize>,

    /// ### Amount of special characters to insert
    ///
    /// Can take either a range like 2-4 or an exact amount like 2.
    ///
    /// **Default: 1-2**
    pub special_chars_amount: RangeInclusive<usize>,

    /// ### The special characters to insert
    ///
    /// Non-ASCII characters are not supported and will error.
    ///
    /// **Default: ^!(-_=)$<\[@.#\]>%{~,+}&\***
    pub special_chars: String,

    /// ### Amount of uppercase characters
    ///
    /// Can take either a range like 2-4 or an exact amount like 2. If there are no
    /// uppercase characters, the [`force_upper`](PassConfig#structfield.force_upper)
    /// flag is turned on automatically to capitalise up to the specified amount of alphabetic characters.
    /// But if there's at least one uppercase character there won't be any capitalisation
    /// unless [`force_upper`](PassConfig#structfield.force_upper) is turned on manually.
    ///
    /// **Default: 1-2**
    pub upper_amount: RangeInclusive<usize>,

    /// ### Amount of lowercase characters
    ///
    /// Can take either a range like 2-4 or an exact amount like 2. If there are no
    /// lowercase characters, the [`force_lower`](PassConfig#structfield.force_lower)
    /// flag is turned on automatically to decapitalise up to the specified amount of alphabetic characters.
    /// But if there's at least one lowercase character there won't be any decapitalisation
    /// unless [`force_lower`](PassConfig#structfield.force_lower) is turned on manually.
    ///
    /// **Default: 1-2**
    pub lower_amount: RangeInclusive<usize>,

    /// ### Choose to keep numbers from the source in the password
    ///
    /// It will treat blocks of numbers as words, not counting them towards the amount
    /// of numbers to insert into the password.
    ///
    /// **Default: false**
    pub keep_numbers: bool,

    /// ### Force the specified amount of uppercase characters
    ///
    /// Gets ignored if [`dont_upper`](PassConfig#structfield.dont_upper) is also set.
    ///
    /// **Default: false**
    pub force_upper: bool,

    /// ### Force the specified amount of lowercase characters
    ///
    /// Gets ignored if [`dont_lower`](PassConfig#structfield.dont_lower) is also set.
    ///
    /// **Default: false**
    pub force_lower: bool,

    /// ### Don't uppercase at all to keep original casing
    ///
    /// Ignores [`force_upper`](PassConfig#structfield.force_upper), both manual and automatic.
    ///
    /// **Default: false**
    pub dont_upper: bool,

    /// ### Don't lowercase at all to keep original casing
    ///
    /// Ignores [`force_lower`](PassConfig#structfield.force_lower), both manual and automatic.
    ///
    /// **Default: false**
    pub dont_lower: bool,

    words: Vec<String>,
}

impl Default for PassConfig {
    /// A set of recommended settings for generating a password.
    fn default() -> Self {
        Self {
            capitalise: false,
            replace: false,
            randomise: false,
            pass_amount: 1,
            reset_amount: 10,
            length: 24..=30,
            number_amount: 1..=2,
            special_chars_amount: 1..=2,
            special_chars: String::from("^!(-_=)$<[@.#]>%{~,+}&*"),
            upper_amount: 1..=2,
            lower_amount: 1..=2,
            keep_numbers: false,
            force_upper: false,
            force_lower: false,
            dont_upper: false,
            dont_lower: false,
            words: Vec::new(),
        }
    }
}

impl PassConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        PassConfig::default()
    }

    /// Extract words from file or directory with text files.
    ///
    /// In case of a directory, it recursively parses every file inside it while
    /// following links and ignoring non-plaintext files.
    ///
    /// In case no words were extracted nothing is added and no error is given.
    ///
    /// Accepts UTF-8 characters, but translates them to ASCII for use in the password.
    /// So if a word in another language is encountered, it will be transformed into a
    /// kind of phonetic spelling in ASCII, and if an emoji is encountered, it will be
    /// translated into its meaning, for example, :D would become 'grinning'.
    ///
    /// # Errors:
    ///
    /// This method will return an IO error in the following situations,
    /// but is not limited to just these cases:
    ///
    /// - `path` does not exist.
    /// - The user lacks permissions to perform metadata call on path.
    /// - The process lacks permissions to view the contents.
    pub fn get_words_from_path(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let md = metadata(&path)?;
        let mut text = String::new();

        if md.is_file() {
            text = fs::read_to_string(&path)?;
        } else if md.is_dir() {
            get_text_from_dir(&path, &mut text)?;
        } else {
            unreachable!("Unexpected metadata error");
        }

        if text.is_empty() {
            return Ok(());
        }

        if !text.is_ascii() {
            text = deunicode(&text);
        }

        let re = if self.keep_numbers {
            Regex::new(r"\w+").unwrap()
        } else {
            Regex::new(r"[^\d\W]+").unwrap()
        };

        for caps in re.captures_iter(&text) {
            if let Some(cap) = caps.get(0) {
                self.words.push(cap.as_str().to_owned());
            }
        }

        if self.randomise {
            self.words.shuffle(&mut thread_rng());
        }

        Ok(())
    }

    /// Extract words from a string.
    ///
    /// In case no words were extracted nothing is added and no error is given.
    ///
    /// Accepts UTF-8 characters, but translates them to ASCII for use in the password.
    /// So if a word in another language is encountered, it will be transformed into a
    /// kind of phonetic spelling in ASCII, and if an emoji is encountered, it will be
    /// translated into its meaning, for example, :D would become 'grinning'.
    pub fn get_words_from_str(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let converted;
        let ascii = match text {
            ascii if ascii.is_ascii() => ascii,
            utf8 => {
                converted = deunicode(utf8);
                &converted
            }
        };

        let re = if self.keep_numbers {
            Regex::new(r"\w+").unwrap()
        } else {
            Regex::new(r"[^\d\W]+").unwrap()
        };

        for caps in re.captures_iter(ascii) {
            if let Some(cap) = caps.get(0) {
                self.words.push(cap.as_str().to_owned());
            }
        }

        if self.randomise {
            self.words.shuffle(&mut thread_rng());
        }
    }

    /// Get a reference to the vector of words.
    pub fn get_words(&self) -> &[String] {
        &self.words
    }

    /// Check configuration for errors and get a validated configuration.
    ///
    /// # Errors:
    ///
    /// Each error has a message, especially [`InvalidRange`](ValidationError#variant.InvalidRange),
    /// which specifies the field the error came from.
    /// Read [`ValidationError`] for more information.
    pub fn validate(&self) -> Result<ValidatedConfig, ValidationError> {
        // TODO: Figure out a different way to validate values.
        /* let (_, _) = match process_range(&self.length) {
            Ok(a) => a,
            Err(e) => {
                return Err(ValidationError::InvalidRange {
                    field: "length".to_string(),
                    message: e,
                })
            }
        };

        let (_, _) = match process_range(&self.number_amount) {
            Ok(a) => a,
            Err(e) => {
                return Err(ValidationError::InvalidRange {
                    field: "number".to_string(),
                    message: e,
                })
            }
        };

        let (_, _) = match process_range(&self.special_chars_amount) {
            Ok(a) => a,
            Err(e) => {
                return Err(ValidationError::InvalidRange {
                    field: "special chars".to_string(),
                    message: e,
                })
            }
        };

        let (_, _) = match process_range(&self.upper_amount) {
            Ok(a) => a,
            Err(e) => {
                return Err(ValidationError::InvalidRange {
                    field: "uppercase".to_string(),
                    message: e,
                })
            }
        };

        let (_, _) = match process_range(&self.lower_amount) {
            Ok(a) => a,
            Err(e) => {
                return Err(ValidationError::InvalidRange {
                    field: "lowercase".to_string(),
                    message: e,
                })
            }
        }; */

        if !self.special_chars.is_ascii() {
            return Err(ValidationError::NonAsciiSpecialChars);
        }

        if self.words.is_empty() || self.words.len() == 1 {
            return Err(ValidationError::NoWords);
        }

        Ok(ValidatedConfig {
            capitalise: self.capitalise,
            replace: self.replace,
            randomise: self.randomise,
            pass_amount: self.pass_amount,
            reset_amount: self.reset_amount,
            length: self.length.clone(),
            number_amount: self.number_amount.clone(),
            special_chars_amount: self.special_chars_amount.clone(),
            special_chars: self.special_chars.clone(),
            upper_amount: self.upper_amount.clone(),
            lower_amount: self.lower_amount.clone(),
            keep_numbers: self.keep_numbers,
            force_upper: self.force_upper,
            force_lower: self.force_lower,
            dont_upper: self.dont_upper,
            dont_lower: self.dont_lower,
            words: self.words.clone(),
        })
    }
}

/// Immutable configuration given after validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedConfig {
    capitalise: bool,
    replace: bool,
    randomise: bool,
    pass_amount: usize,
    reset_amount: usize,
    length: RangeInclusive<usize>,
    number_amount: RangeInclusive<usize>,
    special_chars_amount: RangeInclusive<usize>,
    special_chars: String,
    upper_amount: RangeInclusive<usize>,
    lower_amount: RangeInclusive<usize>,
    keep_numbers: bool,
    force_upper: bool,
    force_lower: bool,
    dont_upper: bool,
    dont_lower: bool,
    words: Vec<String>,
}

impl ValidatedConfig {
    /// Generate a vector of passwords.
    pub fn generate_passwords(&self) -> Vec<String> {
        let mut passwords = Vec::new();

        for _ in 0..self.pass_amount {
            passwords.push(Password::generate(self));
        }

        passwords
    }
}

/// The possible errors when checking the configuration.
#[derive(Debug, Snafu)]
pub enum ValidationError {
    /// For when the range processor doesn't receive either a "20-30" or a "25" style string.
    ///
    /// The range processor does some clean-up beforehand to remove trailing and repeating dashes.
    /// So `---20-----30--` becomes `20-30`, and gives no error or custom message in this case.
    #[snafu(display("Invalid {} range: {}", field,  message.message))]
    InvalidRange { field: String, message: RangeError },

    /// For when the Config holds either one or zero words.
    /// The reason one word isn't allowed is due to the use of [`std::iter::Peekable`].
    #[snafu(display("No words for password generation"))]
    NoWords,

    /// For when non-ASCII characters are found in [`special_chars`](PassConfig#structfield.special_chars).
    #[snafu(display("Non-ASCII special characters aren't allowed for insertables"))]
    NonAsciiSpecialChars,
}

/// Holds the message for the type of error that occurred while parsing a range.
#[derive(Debug, Snafu)]
pub struct RangeError {
    message: String,
}

struct Password {
    password: String,
    reset_count: usize,
    min_len: usize,
    max_len: usize,
    total_inserts: usize,
    upper: usize,
    lower: usize,
    force_upper: bool,
    force_lower: bool,
    insertables: Vec<char>,
}

impl Password {
    fn generate(config: &ValidatedConfig) -> String {
        let mut pass = Password::init(config);

        pass.get_pass_string(config);

        if config.replace {
            pass.replace_chars();
        } else {
            pass.insert_chars();
        }

        pass.ensure_case(config);

        pass.password
    }

    fn init(config: &ValidatedConfig) -> Password {
        let mut rng = thread_rng();

        let mut min_len = *config.length.start();
        let mut max_len = *config.length.end();
        if max_len - min_len > 50 {
            min_len = rng.gen_range(min_len..=max_len - 50);
            max_len = min_len + 50;
        }

        let num = rng.gen_range(config.number_amount.clone());
        let special = rng.gen_range(config.special_chars_amount.clone());
        let upper = rng.gen_range(config.upper_amount.clone());
        let lower = rng.gen_range(config.lower_amount.clone());

        let mut total_inserts = num + special;
        if total_inserts > max_len {
            total_inserts = max_len;
        }

        if !config.replace {
            if min_len < total_inserts {
                total_inserts = min_len;
            }

            min_len -= total_inserts;
            max_len -= total_inserts;
        }

        let insertables = {
            let mut chars = Vec::with_capacity(total_inserts);
            let num_range = Uniform::new(0, 10);
            let char_range = Uniform::new(0, config.special_chars.len());

            for _ in 0..num {
                let num = rng.sample(&num_range).to_string().chars().next().unwrap();
                chars.push(num);
            }

            for _ in 0..special {
                let index = rng.sample(&char_range);
                let c = config.special_chars.chars().nth(index);
                if let Some(c) = c {
                    chars.push(c)
                }
            }

            chars.shuffle(&mut rng);
            chars
        };

        Password {
            password: String::with_capacity(max_len),
            reset_count: 0,
            min_len,
            max_len,
            total_inserts,
            upper,
            lower,
            force_upper: config.force_upper,
            force_lower: config.force_lower,
            insertables,
        }
    }

    fn get_pass_string(&mut self, config: &ValidatedConfig) {
        let mut rng = thread_rng();
        let start_index = rng.gen_range(0..config.words.len());

        let mut text = config.words.clone();
        let mut words = text.iter_mut().skip(start_index).peekable();

        loop {
            if let Some(w) = words.next() {
                if config.capitalise {
                    capitalise(w, 0);
                }

                self.password.push_str(w.as_str());

                match words.peek() {
                    Some(p) => {
                        let mut allowance = 0;
                        if self.password.len() < self.max_len {
                            allowance = self.max_len - self.password.len();
                        }

                        if p.len() > allowance {
                            if self.password.len() >= self.min_len
                                && self.password.len() <= self.max_len
                            {
                                break;
                            } else if self.reset_count >= config.reset_amount {
                                self.password.truncate(self.max_len);
                                break;
                            } else {
                                self.reset_count += 1;
                                self.password.clear();
                                continue;
                            }
                        } else if self.password.len() < self.min_len
                            || p.len() <= allowance && rng.gen_bool(0.8)
                        {
                            continue;
                        } else {
                            break;
                        }
                    }
                    None => {
                        words = text.iter_mut().skip(0).peekable();
                    }
                }
            }
        }
    }

    fn replace_chars(&mut self) {
        let mut rng = thread_rng();
        let range = Uniform::new(0, self.password.len());
        let mut new_pass = String::with_capacity(self.max_len);
        let mut pos = Vec::with_capacity(self.total_inserts);

        while pos.len() < self.total_inserts {
            let num = rng.sample(&range);

            if !pos.contains(&num) {
                pos.push(num);
            }
        }

        for (i, c) in self.password.char_indices() {
            if pos.contains(&i) {
                new_pass.push(self.insertables.pop().unwrap());
            } else {
                new_pass.push(c);
            }
        }

        self.password = new_pass;
    }

    fn insert_chars(&mut self) {
        let mut rng = thread_rng();

        if self.password.is_empty() {
            self.password.push(self.insertables.pop().unwrap());
            self.total_inserts -= 1;
        }

        for _ in 0..self.total_inserts {
            let index = rng.gen_range(0..self.password.len());
            let c = self.insertables.pop().unwrap();

            self.password.insert(index, c);
        }
    }

    fn ensure_case(&mut self, config: &ValidatedConfig) {
        let mut rng = thread_rng();

        let u_amount = self
            .password
            .matches(|c: char| c.is_ascii_uppercase())
            .count();

        let mut l_indices: Vec<usize> = self
            .password
            .char_indices()
            .filter(|(_, c)| c.is_ascii_lowercase())
            .collect::<Vec<(usize, char)>>()
            .into_iter()
            .map(|(i, _)| i)
            .collect();

        if u_amount == 0 {
            self.force_upper = true;
        } else if u_amount >= self.upper {
            self.force_upper = false;
        } else {
            self.upper -= u_amount;
        }

        if self.upper > l_indices.len() {
            self.upper = l_indices.len();
        }

        if self.force_upper && !config.dont_upper {
            for _ in 0..self.upper {
                let i = l_indices.remove(rng.gen_range(0..l_indices.len()));
                capitalise(self.password.as_mut_str(), i)
            }
        }

        let mut u_indices: Vec<usize> = self
            .password
            .char_indices()
            .filter(|(_, c)| c.is_ascii_uppercase())
            .collect::<Vec<(usize, char)>>()
            .into_iter()
            .map(|(i, _)| i)
            .collect();

        if l_indices.is_empty() {
            self.force_lower = true;
        } else if l_indices.len() >= self.lower {
            self.force_lower = false;
        } else {
            self.lower -= l_indices.len();
        }

        if self.lower > u_indices.len() {
            self.lower = u_indices.len();
        }

        if self.force_lower && !config.dont_lower {
            for _ in 0..self.lower {
                let i = u_indices.remove(rng.gen_range(0..u_indices.len()));
                decapitalise(self.password.as_mut_str(), i)
            }
        }
    }
}

fn get_text_from_dir(dir: impl AsRef<Path>, text: &mut String) -> Result<(), std::io::Error> {
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

fn capitalise(s: &mut str, i: usize) {
    if let Some(c) = s.get_mut(i..i + 1) {
        c.make_ascii_uppercase();
    }
}

fn decapitalise(s: &mut str, i: usize) {
    if let Some(c) = s.get_mut(i..i + 1) {
        c.make_ascii_lowercase();
    }
}

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
            message: "more than two sides".to_string(),
        });
    }

    if !range.chars().all(|c| c.is_numeric() || c == '-') {
        return Err(RangeError {
            message: "contains something other than integers and a - (dash)".to_string(),
        });
    }

    if range.contains('-') {
        let r: Vec<&str> = range.split('-').collect();
        min = usize::from_str(r[0]).unwrap();
        max = usize::from_str(r[1]).unwrap();

        if max < min {
            return Err(RangeError {
                message: "right side of range can't be smaller than left side".to_string(),
            });
        }

        Ok(RangeInclusive::new(min, max))
    } else {
        min = usize::from_str(&range).unwrap();
        max = min;

        Ok(RangeInclusive::new(min, max))
    }
}
