# Repology

[![CI](https://github.com/repology/repology-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/repology/repology-rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/repology/repology-rs/graph/badge.svg?token=sViAafpNLI)](https://app.codecov.io/gh/repology/repology-rs)

Repology is a service which monitors *a lot* of package repositories
and other sources and aggregates data on software package versions,
reporting new releases and packaging problems.

This repository is a gradual rewrite of Repology components to Rust.

## Components

- **repology-common** - common code for all Repology components, such as 
  data classes and database queries.
- **repology-webapp** - backend of Repology web site.

## Requirements

This code requires rust nightly.

## Status

To start with, we're rewriting most loaded Replogy website backend
endpoints such as `/api/v1/project/{project}`, used by a lot of new
version reporter tools which generate about 1/4 of Repology traffic,
and `/badges/**` which are now widely used on many sites.

## Author

- [Dmitry Marakasov](https://github.com/AMDmi3) <amdmi3@amdmi3.ru>

## License

- [GPLv3 or later](LICENSE).
