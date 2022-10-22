#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

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
use genrepass::PasswordSettings;
use std::{error::Error, process::exit};

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
    let mut settings = PasswordSettings::new();

    // Load in and parse the text to use for the password generation.
    settings.get_words_from_path("/home/alex/Documents/notes")?;

    // Can be done multiple times to add different directories or files.
    settings.get_words_from_path("/home/alex/Documents/Journal/2020.md")?;

    // Can also just load it from a String.
    settings.get_words_from_str("A string I got from somewhere");

    // Change the configuration by changing the fields.
    settings.pass_amount = 5;
    settings.capitalise = true;
    settings.length = 30..=50;

    // Generate the password/s.
    let passwords = settings.generate()?;

    // Use the vector however you need.
    // In this case we put each password on a separate line and print them.
    println!("{}", passwords.join("\n"));

    Ok(())
}
```

# Features

- `serde` — Enables serialisation and deserialisation
- `rayon` — Enables parallelisation with [`PasswordSettings::generate_parallel()`]
*/

mod helpers;
mod password;
mod settings;
pub use crate::{
    helpers::{range_inc_from_str, ParseRangeError},
    settings::{NonAsciiSpecialCharsError, NotEnoughWordsError, PasswordSettings},
};
