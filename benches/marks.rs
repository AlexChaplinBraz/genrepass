use brunch::{benches, Bench};
use genrepass::PasswordSettings;
use std::time::Duration;

fn main() {
    let mut ps = PasswordSettings::default();
    ps.pass_amount = 100;
    ps.get_words_from_path("LICENSE").unwrap();

    benches!(inline:
        Bench::new("single-threaded generation of 100 from LICENSE")
            .with_timeout(Duration::from_secs(30))
            .run(|| ps.generate().unwrap()),
        Bench::new("multi-threaded generation of 100 from LICENSE")
            .with_timeout(Duration::from_secs(30))
            .run(|| ps.generate_parallel().unwrap()),
        Bench::new("single-threaded generation of 100 from src/")
            .with_timeout(Duration::from_secs(180))
            .run(|| {
                ps.get_words_from_path("src").unwrap();
                ps.generate().unwrap();
            }),
        Bench::new("multi-threaded generation of 100 from src/")
            .with_timeout(Duration::from_secs(180))
            .run(|| ps.generate_parallel().unwrap()),
    );
}
