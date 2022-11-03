use deunicode::deunicode;
use rand::{seq::SliceRandom, thread_rng};
use std::mem::take;
use unicode_segmentation::UnicodeSegmentation;

/// A list of words used for password generation.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Lexicon {
    pub split_mode: SplitMode,
    pub deunicode: bool,
    pub randomise: bool,
    words: Vec<String>,
}

impl Lexicon {
    /// Create a new [`Lexicon`] with a specific split mode and everything turned off.
    pub fn new(split_mode: SplitMode) -> Self {
        Self {
            split_mode,
            ..Default::default()
        }
    }

    /// Extract words from a string.
    ///
    /// How the words are parsed is controlled by the `filter` closure,
    /// which is passed to `.retain()` on each word.
    ///
    /// Returns immediately if `text` is empty.
    pub fn extract_words<F>(&mut self, text: &str, mut filter: F)
    where
        F: FnMut(char) -> bool,
    {
        if text.is_empty() {
            return;
        }

        let deunicoded;
        let text = if self.deunicode {
            deunicoded = deunicode(text);
            &deunicoded
        } else {
            text
        };

        let mut split_words: Vec<String> = match self.split_mode {
            SplitMode::UnicodeWords => text.unicode_words().map(str::to_string).collect(),
            SplitMode::WordBounds => text.split_word_bounds().map(str::to_string).collect(),
            SplitMode::UnicodeWhitespace => text.split_whitespace().map(str::to_string).collect(),
            SplitMode::AsciiWhitespace => {
                text.split_ascii_whitespace().map(str::to_string).collect()
            }
        };

        for word in split_words.iter_mut() {
            word.retain(&mut filter);

            if word.is_empty() {
                continue;
            }

            self.words.push(take(word));
        }

        if self.randomise {
            self.randomise();
        }
    }

    /// Shuffle the words.
    pub fn randomise(&mut self) {
        self.words.shuffle(&mut thread_rng());
    }

    /// Get a reference to the vector of words.
    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// Clear the vector of words.
    pub fn clear_words(&mut self) {
        self.words.clear();
    }

    /// Remove a word at index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove_word_at(&mut self, index: usize) {
        self.words.remove(index);
    }
}

/// The way to split the text into words.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum SplitMode {
    #[default]
    UnicodeWords,
    WordBounds,
    UnicodeWhitespace,
    AsciiWhitespace,
}

/// Some reasonable character filtering options.
#[derive(Debug)]
pub enum CharFilter {
    Ascii,
    AsciiWithoutPunctuation,
    AsciiWithoutDigits,
    AsciiWithoutDigitsOrPunctuation,
    Unicode,
    UnicodeWithoutAsciiDigits,
    UnicodeWithoutNumbers,
    UnicodeWithoutAsciiPunctuation,
    UnicodeWithoutAsciiDigitsOrAsciiPunctuation,
    UnicodeWithoutNumbersOrAsciiPunctuation,
}

impl CharFilter {
    /// Returns a closure for use in [`Lexicon::extract_words()`].
    pub fn closure(&self) -> impl FnMut(char) -> bool {
        match self {
            CharFilter::Ascii => {
                |c: char| c.is_ascii() && !c.is_ascii_whitespace() && !c.is_ascii_control()
            }
            CharFilter::AsciiWithoutPunctuation => |c: char| {
                c.is_ascii()
                    && !c.is_ascii_punctuation()
                    && !c.is_ascii_whitespace()
                    && !c.is_ascii_control()
            },
            CharFilter::AsciiWithoutDigits => |c: char| {
                c.is_ascii()
                    && !c.is_ascii_digit()
                    && !c.is_ascii_whitespace()
                    && !c.is_ascii_control()
            },
            CharFilter::AsciiWithoutDigitsOrPunctuation => |c: char| {
                c.is_ascii()
                    && !c.is_ascii_digit()
                    && !c.is_ascii_punctuation()
                    && !c.is_ascii_whitespace()
                    && !c.is_ascii_control()
            },
            CharFilter::Unicode => |c: char| !c.is_whitespace() && !c.is_control(),
            CharFilter::UnicodeWithoutAsciiDigits => {
                |c: char| !c.is_ascii_digit() && !c.is_whitespace() && !c.is_control()
            }
            CharFilter::UnicodeWithoutNumbers => {
                |c: char| !c.is_numeric() && !c.is_whitespace() && !c.is_control()
            }
            CharFilter::UnicodeWithoutAsciiPunctuation => {
                |c: char| !c.is_ascii_punctuation() && !c.is_whitespace() && !c.is_control()
            }
            CharFilter::UnicodeWithoutAsciiDigitsOrAsciiPunctuation => |c: char| {
                !c.is_ascii_digit()
                    && !c.is_ascii_punctuation()
                    && !c.is_whitespace()
                    && !c.is_control()
            },
            CharFilter::UnicodeWithoutNumbersOrAsciiPunctuation => |c: char| {
                !c.is_numeric()
                    && !c.is_ascii_punctuation()
                    && !c.is_whitespace()
                    && !c.is_control()
            },
        }
    }
}
