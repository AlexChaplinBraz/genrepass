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
    let passwords = validated.generate();

    // Use the vector however you need.
    // In this case we put each password on a separate line and print them.
    println!("{}", passwords.join("\n"));

    Ok(())
}
