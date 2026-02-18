# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2019-02-15

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
