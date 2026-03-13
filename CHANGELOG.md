# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-03-13

### Fixed

- Relative paths to cargo manifests when there is a rust workspace in the current directory (#5)

### Added

- Rust crates to compile std & co for the current toolchain. (#5)
  This is required for rust-analyzer to parse std symbols such as "String".
  Perhaps in the future, cargo vendor will perform this task for us but,
  for now, it needs to be manual.

## [1.0.1] - 2026-02-18

### Fixed

- When the CLI parameter to configure pip only on site-level now install no-index option only for the site and not for the user (#3)

## [1.0.0] - 2026-02-15

### Added

- Add a settings to set the name of the archive and its extracted folder.
- Add CLI options to skip steps.

### Changed

- Keep the installation going if one of the steps fails.
- Support running twice the packaging without cleaning the temporary folder.

### Fixed

- Setting "rust: binstall" set to "false" no longer errors.
- Rust & Pip configuration: the paths now point to an absolute path (and not a relative one).
- Installing of rust tools: the search location was wrong.

## [0.1.0] - 2026-02-07

Initial minimal version
