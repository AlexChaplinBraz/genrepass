use brunch::{Bench, Benches};
use genrepass::PasswordSettings;
use std::time::Duration;

fn main() {
    let mut ps_license = PasswordSettings::default();
    let mut ps_src = PasswordSettings::default();
    let mut ps_examples = PasswordSettings::default();

    println!("Parsing words from path:");

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
                ps_examples.get_words_from_path("examples").unwrap();
            }),
    );

    benches.finish();

    let license_word_len = ps_license.get_words().len();
    let src_word_len = ps_src.get_words().len();
    let examples_word_len = ps_examples.get_words().len();

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
