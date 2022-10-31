use deunicode::deunicode;
use rand::{seq::SliceRandom, thread_rng};
use std::mem::take;
use unicode_segmentation::UnicodeSegmentation;

/// A list of words used for password generation.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Lexicon {
    pub split_mode: SplitMode,
    pub randomise: bool,
    pub deunicode: bool,
    pub remove_ascii_digits: bool,
    pub remove_ascii_punctuation: bool,
    pub remove_non_ascii_alphanumeric: bool,
    pub remove_non_ascii: bool,
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
    /// The behaviour of this method is controlled by the fields on [`Lexicon`].
    ///
    /// Returns immediately if `text` is empty.
    pub fn extract_words(&mut self, text: &str) {
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
            word.retain(|c| {
                !(self.remove_ascii_digits && c.is_ascii_digit()
                    || self.remove_ascii_punctuation && c.is_ascii_punctuation()
                    || self.remove_non_ascii_alphanumeric && !c.is_ascii_alphanumeric()
                    || self.remove_non_ascii && !c.is_ascii())
            });

            if word.is_empty() {
                continue;
            }

            self.words.push(take(word));
        }

        if self.randomise {
            self.randomise();
        }
    }

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
