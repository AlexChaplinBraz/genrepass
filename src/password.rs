use crate::{
    helpers::{capitalise, decapitalise},
    settings::PasswordSettings,
};
use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};
use std::mem::take;

pub(crate) struct Password {
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
    pub(crate) fn generate(&mut self, config: &PasswordSettings) -> String {
        self.get_pass_string(config);

        if config.replace {
            self.replace_chars();
        } else {
            self.insert_chars();
        }

        self.ensure_case(config);

        take(&mut self.password)
    }

    pub(crate) fn new(config: &PasswordSettings) -> Password {
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

    fn get_pass_string(&mut self, config: &PasswordSettings) {
        let mut rng = thread_rng();
        let start_index = rng.gen_range(0..config.words.read().unwrap().len());

        let text = config.words.read().unwrap();
        let mut words = text.iter().skip(start_index).peekable();

        loop {
            if let Some(w) = words.next() {
                if config.capitalise {
                    let w = w[0..1].to_ascii_uppercase() + &w[1..];
                    self.password.push_str(w.as_str());
                } else {
                    self.password.push_str(w.as_str());
                }

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
                        words = text.iter().skip(0).peekable();
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

    fn ensure_case(&mut self, config: &PasswordSettings) {
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
