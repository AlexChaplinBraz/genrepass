# genrepass's changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.4] - 2022-10-01
<!--BEGIN=1.1.4-->
Last 1.x release before refactor. Updated all dependencies.

### Added

- GitHub Actions workflow for releasing new versions.

### Fixed

- Documentation links.
- Typos.
<!--END=1.1.4-->
## [1.1.3] - 2020-10-19

Note: two of these last three versions changed the API,
but I didn't want to bump the major version for something so minor.
Since it was all in the same day there were no users yet,
so, I've simply yanked the previous versions because semantic versioning isn't being followed.

### Changed

- Renamed the fields with quantities to have an "_amount" suffix.

## [1.1.2] - 2020-10-19

### Fixed

- Reachable unreachable. A special case where the insertables amount was higher than the password length in insert mode.
- Spelling.

## [1.1.1] - 2020-10-19

### Fixed

- Spelling.

## [1.1.0] - 2020-10-19

### Changed

- Converted from a binary crate into a library crate.
- Moved out the CLI into its own crate [genrepass-cli](https://github.com/AlexChaplinBraz/genrepass-cli).

## [1.0.1] - 2020-10-13

### Changed

- Refactored `Password::new()`.
- From `clipboard-ext` to `copypasta-ext`, adding support for Wayland clipboard
  [[PR1]](https://github.com/AlexChaplinBraz/genrepass/pull/1).

## [1.0.0] - 2020-09-23

Ported my [`genrepass.sh`](https://github.com/AlexChaplinBraz/shell-scripts/tree/master/genrepass) script to Rust.

[Unreleased]: https://github.com/AlexChaplinBraz/genrepass/compare/1.1.4...HEAD
[1.1.4]: https://github.com/AlexChaplinBraz/genrepass/compare/ccf3e03...1.1.4
[1.1.3]: https://github.com/AlexChaplinBraz/genrepass/compare/31f67db...ccf3e03
[1.1.2]: https://github.com/AlexChaplinBraz/genrepass/compare/dfc17bd...31f67db
[1.1.1]: https://github.com/AlexChaplinBraz/genrepass/compare/3d8fd4e...dfc17bd
[1.1.0]: https://github.com/AlexChaplinBraz/genrepass/compare/bdbd989...3d8fd4e
[1.0.1]: https://github.com/AlexChaplinBraz/genrepass/compare/8908ce4...bdbd989
[1.0.0]: https://github.com/AlexChaplinBraz/genrepass/tree/8908ce4
