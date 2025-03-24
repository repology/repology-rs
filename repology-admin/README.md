# Repology admin interface

This is the admin interface for Repology. It's only really useful for
full-fledged Repology instance which processes vulnerability data from
NVD (through [repology-vulnupdater](../repology-vulnupdater) service)
and/or receives user reports.

## Features

- User reports moderation.
  - `New reports` page lists unprocessed reports sent by Repology visitors
    through `Report` project tab and allow marking reports as Accepted,
    Rejected, or deleting them, with the possibility to add or update a
	reply text.
  - `All reports` page is the same, but is not limited to unprocessed
    reports, useful to change the verdict or return a report to unprocessed
	state.
- CPE bindings maintenance.
  - `CPEs` page lists bindings between Repology projects and CPE identifiers
    which are used to link vulnerability reports (CVEs) to the projects. The
	page features active search which updates the list as part of project
	name, CPE product or vendor is typed in the search box.
  - For each entry it lists Repology project, CPE identifier, and existence
    indicators for the project and CVEs or CVE dictionary entries for a given
	CPE. The former are useful to spot entries which need update after Repology
	or NVD changes (e.g. project or CPE vendor/product renames).
  - The page allows to edit or remove each binding.
- Adding new CPE bindings from the recently modified CVEs.
  - `CVEs` page lists CPEs from recently modified CVE entries which are not
    yet linked to any Repology project, and suggests projects to link to based
	on fuzzy matching.

## Running

Running `repology-admin` requires access Repology PostgreSQL database
(TODO: link to database instructions). The required arguments are `--dsn`
which specifies database connections details, and `--listen` which specifies
on which address/port to serve the web interface. You can open that in browser
right after starting the application.

Other arguments are optional and include listen address for Prometheus
metrics export, log directory for storing logs, and a hostname for
Repology instance for constructing links to Repology projects.

## Development

This app currently uses MASH stack
([Maud](https://maud.lambda.xyz/)/[Axum](https://github.com/tokio-rs/axum)/[SQLx](https://github.com/launchbadge/sqlx)/[htmx](https://htmx.org/)),
can probably be used as an example for it.

Notable architectural choices:
- The interface is split into pages, and each page may be served in two
  different ways: plain HTTP request to e.g. /cpes replies with the whole
  page with content filled, but changing pages in the application issue
  HTMX requests which are detected by the server and only inner page HTML
  is served. So changing pages do not involve whole page reloading.
- The code is organized into _components_ (a struct with multiple database
  interaction and render methods) which consolidates logic related to a
  single entity kind and may be reused in multiple places either by being
  embedded into other templates or be served as a HTML fragment, and
  _handlers_ which correspond to HTTP endpoints and methods.

It will be used as a playground for frontend related experiments.
