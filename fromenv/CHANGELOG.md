# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/ollyswanson/fromenv/releases/tag/fromenv-v0.1.0) - 2025-08-10

### Added

- Adjust error message based on error count
- Add "path" to error reporting
- Improve diagnostics from unrecognized fields
- requirements

### Fixed

- Update pub struct error message

### Other

- Specify version of fromenv-derive
- Add missing items to manifest
- Use cargo-rdme to interpolate rustdocs
- Add licenses
- Remove `FromEnvError` from public API
- Document usage + motivation
- Include Send + Sync bounds on BoxError
- Rename `ParserResult` to `ParseResult`
- Test error messages + rename ui tests
- Add more trybuild tests
- Scaffold trybuild tests
- Global rename to `FromEnv`
