use brunch::{Bench, Benches};
use genrepass::{CharFilter, Lexicon, PasswordSettings, Split};
use std::time::Duration;

fn main() {
    let mut ps_license = PasswordSettings::default();
    let mut ps_src = PasswordSettings::default();
    let mut ps_examples = PasswordSettings::default();

    println!("Extracting words from path (original):");

    let mut benches = Benches::default();

    benches.push(
        Bench::new("load from path: LICENSE")
            .with_samples(200)
            .run(|| {
                ps_license.clear_words();
                ps_license.get_words_from_path("LICENSE").unwrap();
            }),
    );
    benches.push(
        Bench::new("load from path: src/")
            .with_samples(200)
            .run(|| {
                ps_src.clear_words();
                ps_src.get_words_from_path("src").unwrap();
            }),
    );
    benches.push(
        Bench::new("load from path: examples/")
            .with_samples(200)
            .with_timeout(Duration::from_secs(300))
            .run(|| {
                ps_examples.clear_words();
                ps_examples
                    .get_words_from_path("examples/egui-app/src")
                    .unwrap();
                ps_examples
                    .get_words_from_path("examples/egui-app/Cargo.toml")
                    .unwrap();
            }),
    );

    benches.finish();

    let license_word_len = ps_license.words().len();
    let src_word_len = ps_src.words().len();
    let examples_word_len = ps_examples.words().len();

    println!(
        "\
Words extracted from:
       LICENSE: {license_word_len}
          src/: {src_word_len}
     examples/: {examples_word_len}
"
    );

    println!("Extracting words from path (Lexicon):");

    let mut benches = Benches::default();

    let mut lexicon_license = Lexicon::new("LICENSE", Split::UnicodeWords);
    let mut lexicon_src = Lexicon::new("src/", Split::UnicodeWords);
    let mut lexicon_examples = Lexicon::new("examples/", Split::UnicodeWords);

    benches.push(
        Bench::new("load from path: LICENSE")
            .with_samples(200)
            .run(|| {
                lexicon_license.clear_words();
                lexicon_license.extract_words_from_path(&["LICENSE"], 0, None, |_| true);
            }),
    );
    benches.push(
        Bench::new("load from path: src/")
            .with_samples(200)
            .run(|| {
                lexicon_src.clear_words();
                lexicon_src.extract_words_from_path(
                    &["src"],
                    1,
                    None,
                    CharFilter::AsciiWithoutDigitsOrPunctuation.closure(),
                );
            }),
    );
    benches.push(
        Bench::new("load from path: examples/")
            .with_samples(200)
            .with_timeout(Duration::from_secs(300))
            .run(|| {
                lexicon_examples.clear_words();
                lexicon_examples.extract_words_from_path(
                    &["examples"],
                    3,
                    Some(&["rs", "toml"]),
                    CharFilter::AsciiWithoutDigitsOrPunctuation.closure(),
                );
            }),
    );

    benches.finish();

    let license_word_len = lexicon_license.words().len();
    let src_word_len = lexicon_src.words().len();
    let examples_word_len = lexicon_examples.words().len();

    println!(
        "\
Words extracted from:
       LICENSE: {license_word_len}
          src/: {src_word_len}
     examples/: {examples_word_len}
"
    );

    if true {
        return;
    }

    println!("Single-threaded generation:");

    let mut benches = Benches::default();

    ps_license.pass_amount = 1;
    ps_src.pass_amount = 1;
    ps_examples.pass_amount = 1;
    benches.push(
        Bench::new(format!("1 from LICENSE ({license_word_len} words)"))
            .run(|| ps_license.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("1 from src/ ({src_word_len} words)"))
            .run(|| ps_src.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("1 from examples/ ({examples_word_len} words)"))
            .run(|| ps_examples.generate().unwrap()),
    );

    ps_license.pass_amount = 10;
    ps_src.pass_amount = 10;
    ps_examples.pass_amount = 10;
    benches.push(
        Bench::new(format!("10 from LICENSE ({license_word_len} words)"))
            .run(|| ps_license.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("10 from src/ ({src_word_len} words)"))
            .run(|| ps_src.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("10 from examples/ ({examples_word_len} words)"))
            .run(|| ps_examples.generate().unwrap()),
    );

    ps_license.pass_amount = 100;
    ps_src.pass_amount = 100;
    ps_examples.pass_amount = 100;
    benches.push(
        Bench::new(format!("100 from LICENSE ({license_word_len} words)"))
            .run(|| ps_license.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("100 from src/ ({src_word_len} words)"))
            .run(|| ps_src.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("100 from examples/ ({examples_word_len} words)"))
            .run(|| ps_examples.generate().unwrap()),
    );

    ps_license.pass_amount = 1000;
    ps_src.pass_amount = 1000;
    ps_examples.pass_amount = 1000;
    benches.push(
        Bench::new(format!("1000 from LICENSE ({license_word_len} words)"))
            .run(|| ps_license.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("1000 from src/ ({src_word_len} words)"))
            .run(|| ps_src.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("1000 from examples/ ({examples_word_len} words)"))
            .run(|| ps_examples.generate().unwrap()),
    );

    ps_license.pass_amount = 10000;
    ps_src.pass_amount = 10000;
    ps_examples.pass_amount = 10000;
    benches.push(
        Bench::new(format!("10000 from LICENSE ({license_word_len} words)"))
            .with_timeout(Duration::from_secs(60))
            .run(|| ps_license.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("10000 from src/ ({src_word_len} words)"))
            .with_timeout(Duration::from_secs(60))
            .run(|| ps_src.generate().unwrap()),
    );
    benches.push(
        Bench::new(format!("10000 from examples/ ({examples_word_len} words)"))
            .with_timeout(Duration::from_secs(60))
            .run(|| ps_examples.generate().unwrap()),
    );

    benches.finish();

    #[cfg(feature = "rayon")]
    {
        println!("Multi-threaded generation:");

        let mut benches = Benches::default();

        ps_license.pass_amount = 1;
        ps_src.pass_amount = 1;
        ps_examples.pass_amount = 1;
        benches.push(
            Bench::new(format!("1 from LICENSE ({license_word_len} words)"))
                .run(|| ps_license.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("1 from src/ ({src_word_len} words)"))
                .run(|| ps_src.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("1 from examples/ ({examples_word_len} words)"))
                .run(|| ps_examples.generate_parallel().unwrap()),
        );

        ps_license.pass_amount = 10;
        ps_src.pass_amount = 10;
        ps_examples.pass_amount = 10;
        benches.push(
            Bench::new(format!("10 from LICENSE ({license_word_len} words)"))
                .run(|| ps_license.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("10 from src/ ({src_word_len} words)"))
                .run(|| ps_src.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("10 from examples/ ({examples_word_len} words)"))
                .run(|| ps_examples.generate_parallel().unwrap()),
        );

        ps_license.pass_amount = 100;
        ps_src.pass_amount = 100;
        ps_examples.pass_amount = 100;
        benches.push(
            Bench::new(format!("100 from LICENSE ({license_word_len} words)"))
                .run(|| ps_license.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("100 from src/ ({src_word_len} words)"))
                .run(|| ps_src.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("100 from examples/ ({examples_word_len} words)"))
                .run(|| ps_examples.generate_parallel().unwrap()),
        );

        ps_license.pass_amount = 1000;
        ps_src.pass_amount = 1000;
        ps_examples.pass_amount = 1000;
        benches.push(
            Bench::new(format!("1000 from LICENSE ({license_word_len} words)"))
                .run(|| ps_license.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("1000 from src/ ({src_word_len} words)"))
                .run(|| ps_src.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("1000 from examples/ ({examples_word_len} words)"))
                .run(|| ps_examples.generate_parallel().unwrap()),
        );

        ps_license.pass_amount = 10000;
        ps_src.pass_amount = 10000;
        ps_examples.pass_amount = 10000;
        benches.push(
            Bench::new(format!("10000 from LICENSE ({license_word_len} words)"))
                .with_timeout(Duration::from_secs(60))
                .run(|| ps_license.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("10000 from src/ ({src_word_len} words)"))
                .with_timeout(Duration::from_secs(60))
                .run(|| ps_src.generate_parallel().unwrap()),
        );
        benches.push(
            Bench::new(format!("10000 from examples/ ({examples_word_len} words)"))
                .with_timeout(Duration::from_secs(60))
                .run(|| ps_examples.generate_parallel().unwrap()),
        );

        benches.finish();
    }
}
