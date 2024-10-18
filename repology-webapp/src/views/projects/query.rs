// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(warnings, unused)]

use anyhow::Result;
use indoc::indoc;
use sqlx::query;

use super::common::ProjectForListing;

#[derive(Default)]
pub struct ProjectsFilter<'a> {
    pub start_project_name: Option<&'a str>,
    pub end_project_name: Option<&'a str>,
    pub project_name_substring: Option<&'a str>,
    pub maintainer: Option<&'a str>,
    pub in_repo: Option<&'a str>,
    pub not_in_repo: Option<&'a str>,
    pub min_repositories: Option<i32>,
    pub max_repositories: Option<i32>,
    pub min_families: Option<i32>,
    pub max_families: Option<i32>,
    pub min_repositories_newest: Option<i32>,
    pub max_repositories_newest: Option<i32>,
    pub min_families_newest: Option<i32>,
    pub max_families_newest: Option<i32>,
    pub category: Option<&'a str>,
    pub require_newest: bool,
    pub require_outdated: bool,
    pub require_problematic: bool,
    pub require_has_related: bool,
    pub require_vulnerable: bool,
    pub limit: i32,
}

#[tracing::instrument(skip_all)]
pub async fn query_listing_projects(
    pool: &sqlx::PgPool,
    filter: &ProjectsFilter<'_>,
) -> Result<Vec<ProjectForListing>> {
    Ok(sqlx::query_as(indoc! {"
        SELECT * FROM (
            SELECT
                effname,
                num_families,
                has_related
            FROM metapackages
            WHERE
                num_repos_nonshadow > 0

                -- pagination
                AND ($1 IS NULL OR effname >= $1)
                AND ($2 IS NULL OR effname <= $2)

                -- substring
                AND ($3 IS NULL OR effname ILIKE ('%%' || $3 || '%%'))

                -- count ranges
                AND ($7 IS NULL OR num_repos >= $7)
                AND ($8 IS NULL OR num_repos <= $8)
                AND ($9 IS NULL OR num_families >= $9)
                AND ($10 IS NULL OR num_families <= $10)
                AND ($11 IS NULL OR num_repos_newest >= $11)
                AND ($12 IS NULL OR num_repos_newest <= $12)
                AND ($13 IS NULL OR num_families_newest >= $13)
                AND ($14 IS NULL OR num_families_newest <= $14)

                -- category
                AND ($15 IS NULL OR
                    effname IN (
                        SELECT
                            effname
                        FROM category_metapackages
                        WHERE category = $15
                    )
                )

                -- has_related
                AND (NOT $19 OR has_related)

                -- not_in_repo
                AND ($6 IS NULL OR
                    NOT EXISTS (
                        SELECT *
                        FROM repo_metapackages
                        WHERE
                            effname = metapackages.effname AND
                            repository_id = (SELECT id FROM repositories WHERE name = $6)
                    )
                )

                -- maintainer, in_repo and flags are checked together
                AND CASE
                    WHEN $4 IS NOT NULL AND $5 IS NOT NULL THEN (
                        -- both maintainer and in_repo conditions
                        EXISTS (
                            SELECT * FROM maintainer_and_repo_metapackages WHERE
                                (
                                    effname = metapackages.effname AND
                                    maintainer_id = (SELECT id FROM maintainers WHERE maintainer = $4) AND
                                    repository_id = (SELECT id FROM repositories WHERE name = $5)
                                )
                                AND (NOT $16 OR newest)
                                AND (NOT $17 OR outdated)
                                AND (NOT $18 OR problematic)
                                AND (NOT $20 OR vulnerable)
                        )
                    )
                    WHEN $4 IS NOT NULL AND $5 IS NULL THEN (
                        -- only maintainer condition
                        EXISTS (
                            SELECT * FROM maintainer_metapackages WHERE
                                (
                                    effname = metapackages.effname AND
                                    maintainer_id = (SELECT id FROM maintainers WHERE maintainer = $4)
                                )
                                AND (NOT $16 OR newest)
                                AND (NOT $17 OR outdated)
                                AND (NOT $18 OR problematic)
                                AND (NOT $20 OR vulnerable)
                        )
                    )
                    WHEN $4 IS NULL AND $5 IS NOT NULL THEN (
                        -- only in_repo condition
                        EXISTS (
                            SELECT * FROM repo_metapackages WHERE
                                (
                                    effname = metapackages.effname AND
                                    repository_id = (SELECT id FROM repositories WHERE name = $5)
                                )
                                AND (NOT $16 OR newest)
                                AND (NOT $17 OR outdated)
                                AND (NOT $18 OR problematic)
                                AND (NOT $20 OR vulnerable)
                        )
                    )
                    ELSE (
                        -- neither maintainer nor in_repo condition
                        (NOT $16 OR EXISTS (SELECT * FROM repo_metapackages WHERE effname = metapackages.effname AND newest)) AND
                        (NOT $17 OR EXISTS (SELECT * FROM repo_metapackages WHERE effname = metapackages.effname AND outdated)) AND
                        (NOT $18 OR EXISTS (SELECT * FROM repo_metapackages WHERE effname = metapackages.effname AND problematic)) AND
                        (NOT $20 OR EXISTS (SELECT * FROM repo_metapackages WHERE effname = metapackages.effname AND vulnerable))
                    )
                END
            ORDER BY
                CASE WHEN $2 IS NULL THEN effname END,
                CASE WHEN $2 IS NOT NULL THEN effname END DESC
            LIMIT $21
        ) AS tmp
        ORDER BY effname
    "})
    .bind(&filter.start_project_name)      // $1
    .bind(&filter.end_project_name)        // $2
    .bind(&filter.project_name_substring)  // $3
    .bind(&filter.maintainer)              // $4
    .bind(&filter.in_repo)                 // $5
    .bind(&filter.not_in_repo)             // $6
    .bind(&filter.min_repositories)        // $7
    .bind(&filter.max_repositories)        // $8
    .bind(&filter.min_families)            // $9
    .bind(&filter.max_families)            // $10
    .bind(&filter.min_repositories_newest) // $11
    .bind(&filter.max_repositories_newest) // $12
    .bind(&filter.min_families_newest)     // $13
    .bind(&filter.max_families_newest)     // $14
    .bind(&filter.category)                // $15
    .bind(&filter.require_newest)          // $16
    .bind(&filter.require_outdated)        // $17
    .bind(&filter.require_problematic)     // $18
    .bind(&filter.require_has_related)     // $19
    .bind(&filter.require_vulnerable)      // $20
    .bind(&filter.limit)                   // $21
    .fetch_all(pool)
    .await?)
}
