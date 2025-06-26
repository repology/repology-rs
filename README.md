# Repology

[![CI](https://github.com/repology/repology-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/repology/repology-rs/actions/workflows/ci.yml)

Repology is a service which monitors *a lot* of package repositories
and other sources and aggregates data on software package versions,
reporting new releases and packaging problems.

This repository is a gradual rewrite of Repology components to Rust.

## Components

- **repology-common** - common code for all Repology components, such as 
  data classes and database queries.
- **repology-webapp** - backend of Repology web site (active development;
  most of repology.org traffic is already served from this implementation).
- **repology-vulnupdater** - tool to sync vulnerability information from
  [National Vulnerability Database](https://nvd.nist.gov/) (complete).

## Requirements

This code requires at least Rust 1.90 nightly.

## Author

- [Dmitry Marakasov](https://github.com/AMDmi3) <amdmi3@amdmi3.ru>

## License

- [GPLv3 or later](LICENSE).
