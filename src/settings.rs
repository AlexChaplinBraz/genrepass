use crate::{helpers::get_text_from_dir, password::Password};
use deunicode::deunicode;
use rand::{seq::SliceRandom, thread_rng};
use regex::Regex;
use snafu::{ensure, Snafu};
use std::{
    fs,
    fs::metadata,
    ops::RangeInclusive,
    path::Path,
    sync::{RwLock, RwLockReadGuard},
};

/// Used for configuring the password generator.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PasswordSettings {
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
    pub(crate) special_chars: String,

    /// ### Amount of uppercase characters
    ///
    /// Can take either a range like 2-4 or an exact amount like 2. If there are no
    /// uppercase characters, the [`force_upper`](PasswordSettings#structfield.force_upper)
    /// flag is turned on automatically to capitalise up to the specified amount of alphabetic characters.
    /// But if there's at least one uppercase character there won't be any capitalisation
    /// unless [`force_upper`](PasswordSettings#structfield.force_upper) is turned on manually.
    ///
    /// **Default: 1-2**
    pub upper_amount: RangeInclusive<usize>,

    /// ### Amount of lowercase characters
    ///
    /// Can take either a range like 2-4 or an exact amount like 2. If there are no
    /// lowercase characters, the [`force_lower`](PasswordSettings#structfield.force_lower)
    /// flag is turned on automatically to decapitalise up to the specified amount of alphabetic characters.
    /// But if there's at least one lowercase character there won't be any decapitalisation
    /// unless [`force_lower`](PasswordSettings#structfield.force_lower) is turned on manually.
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
    /// Gets ignored if [`dont_upper`](PasswordSettings#structfield.dont_upper) is also set.
    ///
    /// **Default: false**
    pub force_upper: bool,

    /// ### Force the specified amount of lowercase characters
    ///
    /// Gets ignored if [`dont_lower`](PasswordSettings#structfield.dont_lower) is also set.
    ///
    /// **Default: false**
    pub force_lower: bool,

    /// ### Don't uppercase at all to keep original casing
    ///
    /// Ignores [`force_upper`](PasswordSettings#structfield.force_upper), both manual and automatic.
    ///
    /// **Default: false**
    pub dont_upper: bool,

    /// ### Don't lowercase at all to keep original casing
    ///
    /// Ignores [`force_lower`](PasswordSettings#structfield.force_lower), both manual and automatic.
    ///
    /// **Default: false**
    pub dont_lower: bool,

    pub(crate) words: RwLock<Vec<String>>,
}

impl Default for PasswordSettings {
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
            words: RwLock::new(Vec::new()),
        }
    }
}

impl PasswordSettings {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        PasswordSettings::default()
    }

    /// ### The special characters to insert
    ///
    /// Non-ASCII characters are not supported and will error.
    ///
    /// **Default: ^!(-_=)$<\[@.#\]>%{~,+}&\***
    pub fn set_special_chars(&mut self, chars: &str) -> Result<(), NonAsciiSpecialCharsError> {
        ensure!(chars.is_ascii(), NonAsciiSpecialCharsSnafu);

        self.special_chars = chars.to_owned();
        Ok(())
    }

    pub fn get_special_chars(&self) -> &str {
        &self.special_chars
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
                self.words.write().unwrap().push(cap.as_str().to_owned());
            }
        }

        if self.randomise {
            self.words.write().unwrap().shuffle(&mut thread_rng());
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
                self.words.write().unwrap().push(cap.as_str().to_owned());
            }
        }

        if self.randomise {
            self.words.write().unwrap().shuffle(&mut thread_rng());
        }
    }

    /// Get a reference to the vector of words.
    pub fn get_words(&self) -> RwLockReadGuard<Vec<String>> {
        self.words.read().unwrap()
    }

    /// Clear the vector of words.
    pub fn clear_words(&mut self) {
        self.words.write().unwrap().clear();
    }

    /// Remove a word at index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove_word_at(&mut self, index: usize) {
        self.words.write().unwrap().remove(index);
    }

    /// Generate a vector of passwords.
    pub fn generate(&self) -> Result<Vec<String>, NotEnoughWordsError> {
        ensure!(
            !self.words.read().unwrap().is_empty() && self.words.read().unwrap().len() > 1,
            NotEnoughWordsSnafu
        );

        let mut passwords = Vec::new();

        for _ in 0..self.pass_amount {
            passwords.push(Password::init(self).generate(self));
        }

        Ok(passwords)
    }

    /// Generate a vector of passwords with [`rayon`].
    #[cfg(feature = "rayon")]
    pub fn generate_parallel(&self) -> Result<Vec<String>, NotEnoughWordsError> {
        use rayon::prelude::*;
        use std::sync::mpsc::channel;

        ensure!(
            !self.words.read().unwrap().is_empty() && self.words.read().unwrap().len() > 1,
            NotEnoughWordsSnafu
        );

        let mut password_settings = Vec::new();

        for _ in 0..self.pass_amount {
            password_settings.push(Password::init(self));
        }

        let (sender, receiver) = channel();

        password_settings
            .into_par_iter()
            .for_each_with(sender, |sender, mut password| {
                sender
                    .send(password.generate(self))
                    .expect("receiver should still be alive until all passwords are generated");
            });

        let mut passwords = Vec::new();

        while let Ok(value) = receiver.try_recv() {
            passwords.push(value);
        }

        Ok(passwords)
    }
}

/// When non-ASCII characters are found during [`PasswordSettings::set_special_chars()`].
#[derive(Debug, Snafu)]
#[snafu(display("non-ASCII special characters aren't allowed for insertables"))]
pub struct NonAsciiSpecialCharsError;

/// When [`PasswordSettings`] holds either one or zero words.
///
/// The reason one word isn't allowed is due to the use of [`std::iter::Peekable`].
#[derive(Debug, Snafu)]
#[snafu(display("not enough words for password generation"))]
pub struct NotEnoughWordsError;
