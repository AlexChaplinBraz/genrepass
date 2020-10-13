use copypasta_ext::{prelude::*, x11_fork::ClipboardContext};
use deunicode::deunicode;
use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};
use regex::Regex;
use std::{
    error::Error,
    fmt::Write,
    fs,
    fs::metadata,
    path::{Path, PathBuf},
    process::exit,
    str::FromStr,
};
use structopt::StructOpt;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}.", e);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = Args::from_args();
    args.configure()?;

    let mut passwords = String::new();

    for _ in 0..args.pass_amount {
        generate_password(&args, &mut passwords)?;
    }

    if args.clipboard {
        ClipboardContext::new()
            .and_then(|mut ctx| ctx.set_contents(passwords.into()))
            .map_err(|e| -> Box<dyn Error> { e })?;
    } else {
        print!("{}", passwords);
    }

    Ok(())
}

/// Readable password generator
///
/// Generate a readable password from an ordered list of words extracted from text.
/// For improved security, numbers and special characters are inserted at random places.
///
/// The point is to replace the standard password generation that is very
/// tedious to input manually, with a still very secure but much easier
/// to write password. For the rare occasion where you have to input
/// it manually, like on a smartphone you're not syncing them to.
/// It also makes for some interesting passwords,
/// depending on what you choose to use as source.
///
/// Written based on a Computerphile video:
/// How to Choose a Password (https://youtu.be/3NjQ9b3pgIg).
#[derive(Debug, StructOpt)]
#[structopt(author)]
struct Args {
    /// Uppercase the first character of every word
    ///
    /// Makes the password much easier to read, but also slightly less secure
    /// due to the predictability of having capitalised words. Still, the
    /// highly improved readability makes it worth it to always have it on.
    #[structopt(short = "C", long = "capitalise")]
    capitalise: bool,

    /// Replace the original characters
    ///
    /// Instead of inserting the numbers and special characters (which preserves
    /// the original words), replace the characters at random positions.
    #[structopt(short = "r", long = "replace")]
    replace: bool,

    /// Shuffle the words
    ///
    /// Useful if the source text is just a list of words without order anyway
    /// and you want to have a different order with each run of the program.
    #[structopt(short = "X", long = "randomize")]
    randomize: bool,

    /// Copy the generated password/s to clipboard instead of writing to stdout
    #[structopt(short = "c", long = "clipboard")]
    clipboard: bool,

    /// Amount of passwords to generate
    ///
    /// Each password comes on a new line. Useful for providing a list of
    /// passwords to choose from.
    #[structopt(short = "p", long = "pass-amount", default_value = "1")]
    pass_amount: usize,

    /// Amount of times to try generating password before truncating
    ///
    /// If the range is too small or an exact number, it'll be harder
    /// to get a fitting set of words, so the word selection will restart if
    /// the password exceeds the maximum length. But since it would keep
    /// looping if it doesn't find the right length it needs a way to stop,
    /// which in this case is simply truncating the password to the maximum length.
    #[structopt(short = "R", long = "resets", default_value = "10")]
    max_resets: usize,

    /// Set the length of the password
    ///
    /// Can either be a range like 24-30, which will generate a password
    /// between that length, or it can be an exact number like 25
    /// for a password of that exact length.
    #[structopt(short = "L", long = "length", default_value = "24-30")]
    length: String,

    /// Amount of numbers to insert
    ///
    /// Can take either a range like 2-4 or an exact amount like 2.
    /// Doesn't take into consideration the amount of numbers already
    /// in the password if '--keep-nums' is activated.
    #[structopt(short = "n", long = "num", default_value = "1-2")]
    num: String,

    /// Amount of special characters to insert
    ///
    /// Can take either a range like 2-4 or an exact amount like 2.
    #[structopt(short = "s", long = "special", default_value = "1-2")]
    special: String,

    /// The special characters to insert
    ///
    /// Non-ASCII characters are not supported.
    #[structopt(short = "S", long = "chars", default_value = "^!(-_=)$<[@.#]>%{~,+}&*")]
    special_chars: String,

    /// Amount of uppercase characters
    ///
    /// Can take either a range like 2-4 or an exact amount like 2. If there are no
    /// uppercase characters, the '--force-upper' flag is turned on automatically
    /// to capitalise up to the specified amount of alphabetic characters. But if
    /// there's at least one uppercase character there won't be any capitalisation
    /// unless '--force-upper' is turned on manually.
    #[structopt(short = "u", long = "upper", default_value = "1-2")]
    upper: String,

    /// Amount of lowercase characters
    ///
    /// Can take either a range like 2-4 or an exact amount like 2. If there are no
    /// lowercase characters, the '--force-lower' flag is turned on automatically
    /// to decapitalise up to the specified amount of alphabetic characters. But if
    /// there's at least one lowercase character there won't be any decapitalisation
    /// unless '--force-lower' is turned on manually.
    #[structopt(short = "l", long = "lower", default_value = "1-2")]
    lower: String,

    /// Choose to keep numbers from the source in the password
    ///
    /// It will treat blocks of numbers as words, not counting them towards the amount
    /// of numbers to insert into the password.
    #[structopt(short = "k", long = "keep-nums")]
    keep_numbers: bool,

    /// Force the specified amount of uppercase characters
    ///
    /// Gets ignored if '--dont-upper' is also set.
    #[structopt(short = "F", long = "force-upper")]
    force_upper: bool,

    /// Force the specified amount of lowercase characters
    ///
    /// Gets ignored if '--dont-lower' is also set.
    #[structopt(short = "f", long = "force-lower")]
    force_lower: bool,

    /// Don't uppercase at all to keep original casing
    ///
    /// Ignores '--force-upper', both manual and automatic.
    #[structopt(short = "D", long = "dont-upper")]
    dont_upper: bool,

    /// Don't lowercase at all to keep original casing
    ///
    /// Ignores '--force-lower', both manual and automatic.
    #[structopt(short = "d", long = "dont-lower")]
    dont_lower: bool,

    /// Path to text file or directory with text files to source words from
    ///
    /// In case of a directory, it recursively parses every file inside it while
    /// ignoring non-plaintext files and following links.
    ///
    /// Accepts UTF-8 characters, but translates them to ASCII for use in the password.
    /// So if a word in another language is encountered, it will be transformed into a
    /// kind of phonetic spelling in ASCII, and if an emoji is encountered, it will be
    /// translated into its meaning, for example, :D would become 'grinning'.
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    #[structopt(skip)]
    text: Vec<String>,
}

impl Args {
    fn configure(&mut self) -> Result<(), Box<dyn Error>> {
        self.text = {
            let md = metadata(&self.path)?;
            let mut temp_text = String::new();

            if md.is_file() {
                temp_text = fs::read_to_string(&self.path)?;
            } else if md.is_dir() {
                get_text_from_dir(&self.path, &mut temp_text)?;
            } else {
                Err("wrong path")?;
            }

            if temp_text.is_empty() {
                Err("no text provided")?;
            }

            if !temp_text.is_ascii() {
                temp_text = deunicode(&temp_text);
            }

            let re;
            if self.keep_numbers {
                re = Regex::new(r"\w+").unwrap();
            } else {
                re = Regex::new(r"[^\d\W]+").unwrap();
            }

            let mut filtered_text = Vec::<String>::new();

            for caps in re.captures_iter(&temp_text) {
                if let Some(cap) = caps.get(0) {
                    filtered_text.push(cap.as_str().to_string());
                }
            }

            if filtered_text.len() == 1 {
                Err("can't form a password from a single word")?;
            }

            if self.randomize {
                filtered_text.shuffle(&mut thread_rng());
            }

            filtered_text
        };

        Ok(())
    }
}

#[derive(Debug)]
struct Password {
    password: String,
    reset_count: usize,
    min_len: usize,
    max_len: usize,
    num: usize,
    special: usize,
    total_inserts: usize,
    upper: usize,
    lower: usize,
    force_upper: bool,
    force_lower: bool,
    insertables: Vec<char>,
}

impl Password {
    fn new(args: &Args) -> Result<Password, Box<dyn Error>> {
        let mut rng = thread_rng();

        let mut min_len = 0;
        let mut max_len = 0;
        process_range(&args.length, &mut min_len, &mut max_len)?;
        if max_len - min_len > 50 {
            min_len = rng.gen_range(min_len, max_len - 49);
            max_len = min_len + 50;
        }

        let mut min_num = 0;
        let mut max_num = 0;
        process_range(&args.num, &mut min_num, &mut max_num)?;
        let num = rng.gen_range(min_num, max_num + 1);

        let mut min_special = 0;
        let mut max_special = 0;
        process_range(&args.special, &mut min_special, &mut max_special)?;
        let special = rng.gen_range(min_special, max_special + 1);

        let mut min_upper = 0;
        let mut max_upper = 0;
        process_range(&args.upper, &mut min_upper, &mut max_upper)?;
        let upper = rng.gen_range(min_upper, max_upper + 1);

        let mut min_lower = 0;
        let mut max_lower = 0;
        process_range(&args.lower, &mut min_lower, &mut max_lower)?;
        let lower = rng.gen_range(min_lower, max_lower + 1);

        let total_inserts = num + special;
        if total_inserts > max_len {
            Err("special character amount exceeds password length")?;
        }

        if !args.replace {
            if min_len < total_inserts {
                Err("can't have password length be lower than the total amount of insertables")?;
            }

            min_len = min_len - total_inserts;
            max_len = max_len - total_inserts;
        }

        let insertables = {
            let mut chars = Vec::with_capacity(total_inserts);
            let num_range = Uniform::new(0, 10);
            let char_range = Uniform::new(0, args.special_chars.len());

            for _ in 0..num {
                let num = rng.sample(&num_range).to_string().chars().next().unwrap();
                chars.push(num);
            }

            for _ in 0..special {
                let index = rng.sample(&char_range);
                let c = args.special_chars.chars().nth(index);
                match c {
                    Some(c) => chars.push(c.clone()),
                    None => Err("unsuported special character")?,
                }
            }

            chars.shuffle(&mut rng);
            chars
        };

        Ok(Password {
            password: String::with_capacity(max_len),
            reset_count: 0,
            min_len,
            max_len,
            num,
            special,
            total_inserts,
            upper,
            lower,
            force_upper: {
                if args.force_upper {
                    true
                } else {
                    false
                }
            },
            force_lower: {
                if args.force_lower {
                    true
                } else {
                    false
                }
            },
            insertables,
        })
    }
}

fn get_text_from_dir(dir: &Path, mut text: &mut String) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            get_text_from_dir(&path, &mut text)?;
        } else {
            text.push_str(fs::read_to_string(&path).unwrap_or_default().as_str());
        }
    }

    Ok(())
}

fn generate_password(args: &Args, result: &mut String) -> Result<(), Box<dyn Error>> {
    let mut pass = Password::new(&args)?;

    get_pass_string(&args, &mut pass)?;

    if args.replace {
        replace_chars(&mut pass);
    } else {
        insert_chars(&mut pass);
    }

    ensure_case(&args, &mut pass);

    writeln!(result, "{}", pass.password)?;

    Ok(())
}

fn get_pass_string(args: &Args, pass: &mut Password) -> Result<(), Box<dyn Error>> {
    let mut rng = thread_rng();
    let start_index = rng.gen_range(0, args.text.len() - 1);

    let mut text = args.text.clone();
    let mut words = text.iter_mut().skip(start_index).peekable();

    loop {
        if let Some(mut w) = words.next() {
            if args.capitalise {
                capitalise(&mut w, 0);
            }

            pass.password.push_str(w.as_str());

            match words.peek() {
                Some(p) => {
                    let mut allowance = 0;
                    if pass.password.len() < pass.max_len {
                        allowance = pass.max_len - pass.password.len();
                    }

                    if p.len() > allowance {
                        if pass.password.len() >= pass.min_len
                            && pass.password.len() <= pass.max_len
                        {
                            break;
                        } else if pass.reset_count >= args.max_resets {
                            pass.password.truncate(pass.max_len);
                            break;
                        } else {
                            pass.reset_count += 1;
                            pass.password.clear();
                            continue;
                        }
                    } else if pass.password.len() < pass.min_len {
                        continue;
                    } else if p.len() <= allowance && rng.gen_bool(0.8) {
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

    if pass.password.len() == 0 {
        Err("empty password")?;
    }

    Ok(())
}

fn replace_chars(pass: &mut Password) {
    let mut rng = thread_rng();
    let range = Uniform::new(0, pass.password.len());
    let mut new_pass = String::with_capacity(pass.max_len);
    let mut pos = Vec::with_capacity(pass.total_inserts);

    while pos.len() < pass.total_inserts {
        let num = rng.sample(&range);

        if !pos.contains(&num) {
            pos.push(num);
        }
    }

    for (i, c) in pass.password.char_indices() {
        if pos.contains(&i) {
            new_pass.push(pass.insertables.pop().unwrap());
        } else {
            new_pass.push(c);
        }
    }

    pass.password = new_pass;
}

fn insert_chars(pass: &mut Password) {
    let mut rng = thread_rng();

    for _ in 0..pass.total_inserts {
        let index = rng.gen_range(0, pass.password.len());
        let c = pass.insertables.pop().unwrap();

        pass.password.insert(index, c);
    }
}

fn ensure_case(args: &Args, pass: &mut Password) {
    let mut rng = thread_rng();

    let u_amount = pass
        .password
        .matches(|c: char| c.is_ascii_uppercase())
        .count();

    let mut l_indeces: Vec<usize> = pass
        .password
        .char_indices()
        .filter(|(_, c)| c.is_ascii_lowercase())
        .collect::<Vec<(usize, char)>>()
        .into_iter()
        .map(|(i, _)| i)
        .collect();

    if u_amount == 0 {
        pass.force_upper = true;
    } else if u_amount >= pass.upper {
        pass.force_upper = false;
    } else {
        pass.upper -= u_amount;
    }

    if pass.upper > l_indeces.len() {
        pass.upper = l_indeces.len();
    }

    if pass.force_upper && !args.dont_upper {
        for _ in 0..pass.upper {
            let i = l_indeces.remove(rng.gen_range(0, l_indeces.len()));
            capitalise(&mut pass.password.as_mut_str(), i)
        }
    }

    let mut u_indeces: Vec<usize> = pass
        .password
        .char_indices()
        .filter(|(_, c)| c.is_ascii_uppercase())
        .collect::<Vec<(usize, char)>>()
        .into_iter()
        .map(|(i, _)| i)
        .collect();

    if l_indeces.len() == 0 {
        pass.force_lower = true;
    } else if l_indeces.len() >= pass.lower {
        pass.force_lower = false;
    } else {
        pass.lower -= l_indeces.len();
    }

    if pass.lower > u_indeces.len() {
        pass.lower = u_indeces.len();
    }

    if pass.force_lower && !args.dont_lower {
        for _ in 0..pass.lower {
            let i = u_indeces.remove(rng.gen_range(0, u_indeces.len()));
            decapitalise(&mut pass.password.as_mut_str(), i)
        }
    }
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

fn process_range(range: &String, min: &mut usize, max: &mut usize) -> Result<(), Box<dyn Error>> {
    if range.contains("-") {
        let r: Vec<&str> = range.split("-").collect();
        *min = usize::from_str(r[0])?;
        *max = usize::from_str(r[1])?;

        if r.len() > 2 {
            Err("invalid range")?;
        }

        if max < min {
            Err("right side of range can't be smaller than left side")?;
        }
    } else {
        *min = usize::from_str(&range)?;
        *max = min.clone();
    }

    Ok(())
}
