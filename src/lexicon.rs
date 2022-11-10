use deunicode::deunicode;
use rand::{seq::SliceRandom, thread_rng};
use std::mem::{swap, take};
use unicode_segmentation::UnicodeSegmentation;

/// A list of words used for password generation.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Lexicon {
    /// The way to split the text into words.
    pub split: Split,

    /// Flag for transliterating any Unicode text into ASCII text during word extraction.
    ///
    /// ```
    /// use deunicode::deunicode;
    /// assert_eq!(deunicode("😃"), "smiley");
    /// assert_eq!(deunicode("🥫"), "canned food");
    /// assert_eq!(deunicode("📬"), "mailbox with mail");
    /// assert_eq!(deunicode("🇪🇸"), "ES");
    /// assert_eq!(deunicode("Æneid"), "AEneid");
    /// assert_eq!(deunicode("étude"), "etude");
    /// assert_eq!(deunicode("北亰"), "Bei Jing");
    /// assert_eq!(deunicode("ᔕᓇᓇ"), "shanana");
    /// assert_eq!(deunicode("げんまい茶"), "genmaiCha");
    /// assert_eq!(deunicode("…"), "...");
    /// ```
    ///
    /// # Guarantees and Warnings
    ///
    /// Here are some guarantees you have when enabling `deunicode`:
    ///   * The words returned will be valid ASCII; the decimal representation of
    ///     every `char` in the words will be between 0 and 127, inclusive.
    ///   * Every ASCII character (0x0000 - 0x007F) is mapped to itself.
    ///   * All Unicode characters will translate to a string containing newlines
    ///     (`"\n"`) or ASCII characters in the range 0x0020 - 0x007E. So for example,
    ///     no Unicode character will translate to `\u{01}`. The exception is if the
    ///     ASCII character itself is passed in, in which case it will be mapped to
    ///     itself. (So `'\u{01}'` will be mapped to `"\u{01}"`.)
    ///
    /// There are, however, some things you should keep in mind:
    ///   * As stated, some transliterations do produce `\n` characters.
    ///   * Some Unicode characters transliterate to an empty string on purpose.
    ///   * Some Unicode characters are unknown and transliterate to `"[?]"`.
    ///   * Many Unicode characters transliterate to multi-character strings. For
    ///     example, 北 is transliterated as "Bei ".
    ///   * Han characters are mapped to Mandarin, and will be mostly illegible to Japanese readers.
    pub deunicode: Deunicode,

    /// Flag for randomising all the words at the end of word extraction.
    pub randomise: bool,

    /// All the extracted words.
    words: Vec<String>,
}

impl Lexicon {
    /// Create a new [`Lexicon`] with a specific split mode and everything turned off.
    pub fn new(split: Split) -> Self {
        Self {
            split,
            ..Default::default()
        }
    }

    /// Extract words from a string.
    ///
    /// The `filter` closure is passed directly into [`String::retain()`], which runs on each split word.
    ///
    /// You can choose to use one of the default filters provided by [`CharFilter`],
    /// or you can pass your own closure with custom parsing.
    /// Look at [`CharFilter::closure()`] for examples.
    pub fn extract_words<F>(&mut self, text: &str, mut filter: F)
    where
        F: FnMut(char) -> bool,
    {
        if text.is_empty() {
            return;
        }

        let deunicoded;
        let text = if let Deunicode::BeforeSplitting = self.deunicode {
            deunicoded = deunicode(text);
            &deunicoded
        } else {
            text
        };

        let mut split_words: Vec<String> = match &self.split {
            Split::UnicodeWords => text.unicode_words().map(str::to_string).collect(),
            Split::WordBounds => text.split_word_bounds().map(str::to_string).collect(),
            Split::UnicodeWhitespace => text.split_whitespace().map(str::to_string).collect(),
            Split::AsciiWhitespace => text.split_ascii_whitespace().map(str::to_string).collect(),
            Split::Chars(chars) => text.split(&chars[..]).map(str::to_string).collect(),
        };

        for word in split_words.iter_mut() {
            if word.is_empty() {
                continue;
            }

            if let Deunicode::BeforeFiltering = self.deunicode {
                let mut deunicoded = deunicode(word);
                swap(word, &mut deunicoded);
            }

            word.retain(&mut filter);

            if word.is_empty() {
                continue;
            }

            if let Deunicode::AfterFiltering = self.deunicode {
                let mut deunicoded = deunicode(word);

                if deunicoded.is_empty() {
                    continue;
                } else {
                    self.words.push(take(&mut deunicoded));
                }
            } else {
                self.words.push(take(word));
            }
        }

        if self.randomise {
            self.randomise();
        }
    }

    /// Read texts from paths and extract the words.
    ///
    /// The way this method is configured:
    /// * Symbolic links aren't followed
    /// * Directories and files returning any kind of IO error are silently skipped
    /// * Hidden directories and files (meaning they start with `.`) are ignored,
    ///   except if you pass the path to the hidden directory or file directly
    /// * Some common extensions are ignored by default because they can't be parsed to UTF-8 anyway
    /// * Extensions are compared ignoring ASCII case, with just the text after the last `.`
    /// * Passing a path to a file ignores all filtering
    /// * All the files that pass the filtering are checked for if they are valid UTF-8
    ///   by reading a few bytes at the start of the file
    ///
    /// See [`Lexicon::extract_words()`] for how the words are extracted.
    #[cfg(feature = "from_path")]
    pub fn extract_words_from_path<F>(
        &mut self,
        paths: &[impl AsRef<std::path::Path>],
        depth: usize,
        extensions: Option<&[&str]>,
        filter: F,
    ) where
        F: FnMut(char) -> bool,
    {
        use simdutf8::compat::from_utf8;
        use std::{
            fs::{read_to_string, File},
            io::Read,
        };
        use walkdir::{DirEntry, WalkDir};

        // A list of extensions that could appear in something like ~/Documents
        // but that are not able to be read as UTF-8 anyway,
        // some even giving false positives like PDF and MP3.
        let ignored_extensions = [
            "pdf", "epub", "mobi", "azw3", "doc", "docx", "mp3", "mp4", "avi", "ogg", "jpg",
            "jpeg", "png", "gif",
        ];

        let filter_entry = |e: &DirEntry| {
            if e.depth() == 0 && e.file_type().is_file() {
                true
            } else if e.depth() != 0
                && e.file_name()
                    .to_str()
                    .map(|s| s.starts_with("."))
                    .unwrap_or_default()
            {
                false
            } else if e.file_type().is_file() {
                match e.file_name().to_str() {
                    Some(s) => match s.rsplit_once('.') {
                        Some((_, ext)) => {
                            if ignored_extensions
                                .iter()
                                .any(|allowed_ext| allowed_ext.eq_ignore_ascii_case(ext))
                            {
                                false
                            } else {
                                match extensions {
                                    Some(extensions) => extensions
                                        .iter()
                                        .any(|allowed_ext| allowed_ext.eq_ignore_ascii_case(ext)),
                                    None => true,
                                }
                            }
                        }
                        None => !extensions.is_some(),
                    },
                    None => false,
                }
            } else {
                true
            }
        };

        let mut texts = String::new();
        let mut buf = [0; 64];

        for path in paths {
            for entry in WalkDir::new(&path)
                .max_depth(depth)
                .into_iter()
                .filter_entry(|e| filter_entry(e))
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    if let Ok(mut file) = File::open(entry.path()) {
                        if let Ok(_) = file.read(&mut buf) {
                            match from_utf8(&buf) {
                                Ok(_) => {
                                    if let Ok(text) = read_to_string(entry.path()) {
                                        texts.push('\n');
                                        texts.push_str(&text);
                                    }
                                }
                                Err(e) => {
                                    if e.valid_up_to() >= 56 {
                                        if let Ok(text) = read_to_string(entry.path()) {
                                            texts.push('\n');
                                            texts.push_str(&text);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    buf = [0; 64];
                }
            }
        }

        self.extract_words(&texts, filter);
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
pub enum Split {
    /// Splits the text into words based on on
    /// [UAX#29 word boundaries](http://www.unicode.org/reports/tr29/#Word_Boundaries).
    ///
    /// Here, "words" are just those substrings which, after splitting on
    /// UAX#29 word boundaries, contain any alphanumeric characters. That is, the
    /// substring must contain at least one character with the
    /// [Alphabetic](http://unicode.org/reports/tr44/#Alphabetic)
    /// property, or with
    /// [General_Category=Number](http://unicode.org/reports/tr44/#General_Category_Values).
    ///
    /// # Examples
    ///
    /// If emoji are present, they are only acknowledged for their word boundaries
    /// and ignored as they're not alphanumeric characters.
    ///
    /// ```
    /// # use genrepass::{Lexicon, Split};
    /// let text = "The ⚡quick⚡ (\"brown\") 🐒 can't❌jump 32.3 feet, right?";
    /// let expected = &["The", "quick", "brown", "can't", "jump", "32.3", "feet", "right"];
    ///
    /// let mut lexicon = Lexicon::new(Split::UnicodeWords);
    /// lexicon.extract_words(text, |_| true);
    ///
    /// assert_eq!(lexicon.words(), expected);
    /// ```
    ///
    /// Enabling deunicoding produces subpar results.
    /// Look at [`Split::WordBounds`] for that.
    ///
    /// ```
    /// # use genrepass::{Deunicode, Lexicon, Split};
    /// let text = "The ⚡quick⚡ (\"brown\") 🐒 can't❌jump 32.3 feet, right?";
    /// let expected = &["The", "zap", "quickzap", "brown", "monkey", "can'tx", "jump", "32.3", "feet", "right"];
    ///
    /// let mut lexicon = Lexicon::new(Split::UnicodeWords);
    /// lexicon.deunicode = Deunicode::BeforeSplitting;
    /// lexicon.extract_words(text, |_| true);
    ///
    /// assert_eq!(lexicon.words(), expected);
    /// ```
    #[default]
    UnicodeWords,

    /// Splits the text based on
    /// [UAX#29 word boundaries](http://www.unicode.org/reports/tr29/#Word_Boundaries)
    /// such that the concatenation of the words is just the original text.
    ///
    /// # Examples
    ///
    /// ```
    /// # use genrepass::{Lexicon, Split};
    /// let text = "The ⚡quick⚡ (\"brown\")    🐒 can't❌jump too high.";
    /// let expected = &[
    ///     "The", " ", "⚡", "quick", "⚡", " ", "(", "\"", "brown", "\"", ")", "    ", "🐒", " ",
    ///     "can't", "❌", "jump", " ", "too", " ", "high", ".",
    /// ];
    ///
    /// let mut lexicon = Lexicon::new(Split::WordBounds);
    /// lexicon.extract_words(text, |_| true);
    ///
    /// assert_eq!(lexicon.words(), expected);
    ///
    /// // Then if we concatenate the split words, we get back the initial string.
    /// assert_eq!(text, lexicon.words().join(""));
    ///
    /// ```
    ///
    /// This is more useful than [`Split::UnicodeWords`] when you want to preserve the emoji as their own words.
    ///
    /// ```
    /// # use genrepass::{Lexicon, Split};
    /// let text = "The ⚡quick⚡ (\"brown\")    🐒 can't❌jump too high.";
    /// let expected = &["The", "⚡", "quick", "⚡", "brown", "🐒", "can't", "❌", "jump", "too", "high", "."];
    ///
    /// let mut lexicon = Lexicon::new(Split::WordBounds);
    /// lexicon.extract_words(text, |c| c != '(' && c != ')' && c != '"' && !c.is_whitespace());
    ///
    /// assert_eq!(lexicon.words(), expected);
    /// ```
    ///
    /// This is also the best way to deunicode words after splitting them,
    /// so that the translated emoji become their own words.
    ///
    /// ```
    /// # use genrepass::{Deunicode, Lexicon, Split};
    /// let text = "The ⚡quick⚡ (\"brown\")    🐒 can't❌jump too high.";
    /// let expected = &["The", "zap", "quick", "zap", "brown", "monkey", "can't", "x", "jump", "too", "high", "."];
    ///
    /// let mut lexicon = Lexicon::new(Split::WordBounds);
    /// lexicon.deunicode = Deunicode::BeforeFiltering;
    /// lexicon.extract_words(text, |c| c != '(' && c != ')' && c != '"' && !c.is_whitespace());
    ///
    /// assert_eq!(lexicon.words(), expected);
    /// ```
    WordBounds,

    /// Splits the text into words separated by any amount of Unicode whitespace.
    ///
    /// 'Whitespace' is defined according to the terms of the Unicode Derived
    /// Core Property `White_Space`. If you only want to split on ASCII whitespace
    /// instead, use [`Split::AsciiWhitespace`].
    ///
    /// # Example
    ///
    /// ```
    /// # use genrepass::{Lexicon, Split};
    /// let text = "The ⚡quick⚡  \u{2009}  (\"brown\")    🐒 can't❌jump 32.3\u{3000}feet, right?";
    /// let expected = &["The", "⚡quick⚡", "(\"brown\")", "🐒", "can't❌jump", "32.3", "feet,", "right?"];
    ///
    /// let mut lexicon = Lexicon::new(Split::UnicodeWhitespace);
    /// lexicon.extract_words(text, |_| true);
    ///
    /// assert_eq!(lexicon.words(), expected);
    /// ```
    UnicodeWhitespace,

    /// Splits the text into words separated by any amount of ASCII whitespace.
    ///
    /// Supposed to be faster than [`Split::UnicodeWhitespace`].
    ///
    /// # Example
    ///
    /// ```
    /// # use genrepass::{Lexicon, Split};
    /// let text = "The ⚡quick⚡  \u{2009}  (\"brown\")    🐒\tcan't❌jump\n\t32.3\u{3000}feet, right?";
    /// let expected = &["The", "⚡quick⚡", "\u{2009}", "(\"brown\")", "🐒", "can't❌jump", "32.3\u{3000}feet,", "right?"];
    ///
    /// let mut lexicon = Lexicon::new(Split::AsciiWhitespace);
    /// lexicon.extract_words(text, |_| true);
    ///
    /// assert_eq!(lexicon.words(), expected);
    /// ```
    AsciiWhitespace,

    /// Splits the text on any occurrence of the characters.
    ///
    /// # Example
    ///
    /// ```
    /// # use genrepass::{Lexicon, Split};
    /// let text = "The ⚡quick⚡  \u{2009}  (\"brown\")    🐒\tcan't❌jump\n\t32.3\u{3000}feet, right?";
    /// let expected = &["The", "⚡quick⚡", "(\"brown\")", "🐒", "can't", "jump", "32.3", "feet,", "right?"];
    ///
    /// let mut lexicon = Lexicon::new(Split::Chars(vec![' ', '\t', '\n', '\u{2009}', '\u{3000}', '❌']));
    /// lexicon.extract_words(text, |_| true);
    ///
    /// assert_eq!(lexicon.words(), expected);
    /// ```
    Chars(Vec<char>),
}

/// When the deunicoding happens.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Deunicode {
    /// No deunicoding takes place. The default when creating a [`Lexicon`].
    #[default]
    Deactivated,

    /// Deunicode the entire text before splitting.
    BeforeSplitting,

    /// Deunicode each split word before filtering characters.
    BeforeFiltering,

    /// Deunicode each split word after it had been filtered.
    AfterFiltering,
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
    ///
    /// This closure is designed to be passed to [`String::retain()`].
    /// It runs on each `char` and only keeps the `char`s that returned `true`.
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
