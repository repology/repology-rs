# Repology

[![CI](https://github.com/repology/repology-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/repology/repology-rs/actions/workflows/ci.yml)

Repology is a service which monitors *a lot* of package repositories
and other sources and aggregates data on software package versions,
reporting new releases and packaging problems.

This repository is a gradual rewrite of Repology components to Rust.

## Components

- **repology-common** - common code for all Repology components, such as 
  data classes and database queries.
- **repology-webapp** - backend of Repology web site.

## Status

To start with, we're rewriting most loaded Replogy website backend
endpoints such as `/api/v1/project/{project}`, used by a lot of new
version reporter tools which generate about 1/4 of Repology traffic,
and `/badges/**` which are now widely used on many sites.

For the record, here's performance difference on `/api/v1/project/`,
measured with [oha](https://github.com/hatoo/oha). Python backend is
served by `uwsgi` with 20 workers, proxied by nginx `uwsgi_pass`, Rust
app is proxied by nginx `proxy_pass`, both also compressed with nginx
`gzip on` `gzip_comp_level 5` as done on prod). Some measurements done
for different projects to see how size of data returned from the database
affects the performance (`00-dirtree` is small project with 1 package,
`firefox` very large with 4k packages and `zsh` is medium with 700
packages).

| Project      | Metric                | Python                  | Rust   | Gain  |
|--------------|-----------------------|-------------------------|--------|-------|
| `zsh`        | avg latency @ 10 rps  | 56 ms                   | 24 ms  | 2.3x  |
| `zsh`        | P99 latency @ 10 rps  | 396 ms                  | 105 ms | 3.8x  |
| `zsh`        | Max RPS               | 76.6                    | 319    | 4.2x  |
| `zsh`        | avg latency @ max rps | 651 ms                  | 156 ms | 4.2x  |
| `zsh`        | P99 latency @ max rps | 1309 ms                 | 314 ms | 4.2x  |
| `zsh`        | Memory use (RSS)      | avg 68 MB (per worker!) | 18 MB  | 75x   |
| `00-dirtree` | Max RPS               | 230                     | 3400   | 14.8x |
| `00-dirtree` | avg latency @ max rps | 217 ms                  | 14 ms  | 15.5x |
| `00-dirtree` | P99 latency @ max rps | 716 ms                  | 85 ms  | 8.4x  |
| `firefox`    | Max RPS               | 12                      | 56.5   | 4.72x |
| `firefox`    | avg latency @ max rps | 4.12 s                  | 883 ms | 4.66x |
| `firefox`    | P99 latency @ max rps | 6.73 s                  | 1.16 s | 5.8x  |

I didn't quite expect to see this much gain on database-heavy requests, but python
seem to add huge overhead regardless of the load type. It's not quite fair however
to compare sync python to async rust, and it would be interesting to see how
[quart](https://github.com/pallets/quart) performs, but I don't have resources for
that. Prior research has shown that it increases latency, but can handle more traffic.

## Author

- [Dmitry Marakasov](https://github.com/AMDmi3) <amdmi3@amdmi3.ru>

## License

- [GPLv3 or later](LICENSE).
