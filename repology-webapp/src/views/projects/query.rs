// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use indoc::{formatdoc, indoc};

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

struct QueryAndJoiner {
    query: String,
}

impl QueryAndJoiner {
    fn new() -> Self {
        Self { query: "(".into() }
    }

    fn push(&mut self, operand: &str) {
        if self.query.len() > 1 {
            self.query += " AND ";
        }
        self.query += "(";
        self.query += operand;
        self.query += ")";
    }

    fn finish(mut self) -> String {
        if self.query.len() == 1 {
            self.query += "TRUE";
        }
        self.query += ")";
        self.query
    }
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn query_listing_projects(
    pool: &sqlx::PgPool,
    filter: &ProjectsFilter<'_>,
) -> Result<Vec<ProjectForListing>> {
    let mut query_conditions = QueryAndJoiner::new();

    // pagination
    if filter.start_project_name.is_some() {
        query_conditions.push("effname >= $1");
    }
    if filter.end_project_name.is_some() {
        query_conditions.push("effname <= $2");
    }

    // substring
    if filter.project_name_substring.is_some() {
        query_conditions.push("effname ILIKE ('%%' || $3 || '%%')");
    }

    // count ranges
    if filter.min_repositories.is_some() {
        query_conditions.push("num_repos >= $7");
    }
    if filter.max_repositories.is_some() {
        query_conditions.push("num_repos <= $8");
    }
    if filter.min_families.is_some() {
        query_conditions.push("num_families >= $9");
    }
    if filter.max_families.is_some() {
        query_conditions.push("num_families <= $10");
    }
    if filter.min_repositories_newest.is_some() {
        query_conditions.push("num_repos_newest >= $11");
    }
    if filter.max_repositories_newest.is_some() {
        query_conditions.push("num_repos_newest <= $12");
    }
    if filter.min_families_newest.is_some() {
        query_conditions.push("num_families_newest >= $13");
    }
    if filter.max_families_newest.is_some() {
        query_conditions.push("num_families_newest <= $14");
    }

    // category
    if filter.category.is_some() {
        query_conditions
            .push("effname IN (SELECT effname FROM category_metapackages WHERE category = $15)");
    }

    // has_related
    if filter.require_has_related {
        query_conditions.push("has_related");
    }

    // not_in_repo
    if filter.not_in_repo.is_some() {
        query_conditions.push(indoc! {"
            NOT EXISTS (
                SELECT *
                FROM repo_metapackages
                WHERE
                    effname = metapackages.effname AND
                    repository_id = (SELECT id FROM repositories WHERE name = $6)
            )
        "});
    }

    // maintainer, in_repo and flags are checked together
    {
        let mut binding_condition = QueryAndJoiner::new();
        if filter.require_newest {
            binding_condition.push("newest");
        }
        if filter.require_outdated {
            binding_condition.push("outdated");
        }
        if filter.require_problematic {
            binding_condition.push("problematic");
        }
        if filter.require_vulnerable {
            binding_condition.push("vulnerable");
        }

        match (filter.maintainer.is_some(), filter.in_repo.is_some()) {
            (true, true) => {
                binding_condition.push(indoc! {"
                    effname = metapackages.effname AND
                    maintainer_id = (SELECT id FROM maintainers WHERE maintainer = $4) AND
                    repository_id = (SELECT id FROM repositories WHERE name = $5)
                "});
                query_conditions.push(&format!(
                    "EXISTS(SELECT * FROM maintainer_and_repo_metapackages WHERE {})",
                    binding_condition.finish()
                ));
            }
            (true, false) => {
                binding_condition.push(indoc! {"
                    effname = metapackages.effname AND
                    maintainer_id = (SELECT id FROM maintainers WHERE maintainer = $4)
                "});
                query_conditions.push(&format!(
                    "EXISTS(SELECT * FROM maintainer_metapackages WHERE {})",
                    binding_condition.finish()
                ));
            }
            (false, true) => {
                binding_condition.push(indoc! {"
                    effname = metapackages.effname AND
                    repository_id = (SELECT id FROM repositories WHERE name = $5)
                "});
                query_conditions.push(&format!(
                    "EXISTS(SELECT * FROM repo_metapackages WHERE {})",
                    binding_condition.finish()
                ));
            }
            (false, false) => {
                if filter.require_newest {
                    query_conditions.push("EXISTS (SELECT * FROM repo_metapackages WHERE effname = metapackages.effname AND newest)");
                }
                if filter.require_outdated {
                    query_conditions.push("EXISTS (SELECT * FROM repo_metapackages WHERE effname = metapackages.effname AND outdated)");
                }
                if filter.require_problematic {
                    query_conditions.push("EXISTS (SELECT * FROM repo_metapackages WHERE effname = metapackages.effname AND problematic)");
                }
                if filter.require_vulnerable {
                    query_conditions.push("EXISTS (SELECT * FROM repo_metapackages WHERE effname = metapackages.effname AND vulnerable)");
                }
            }
        }
    }

    Ok(sqlx::query_as(&formatdoc! {"
        SELECT * FROM (
            SELECT
                effname,
                num_families,
                has_related
            FROM metapackages
            WHERE
                num_repos_nonshadow > 0 AND {}
            ORDER BY
                CASE WHEN $2 IS NULL THEN effname END,
                CASE WHEN $2 IS NOT NULL THEN effname END DESC
            LIMIT $21
        ) AS tmp
        ORDER BY effname
    ", query_conditions.finish()})
    .bind(&filter.start_project_name) // $1
    .bind(&filter.end_project_name) // $2
    .bind(&filter.project_name_substring) // $3
    .bind(&filter.maintainer) // $4
    .bind(&filter.in_repo) // $5
    .bind(&filter.not_in_repo) // $6
    .bind(&filter.min_repositories) // $7
    .bind(&filter.max_repositories) // $8
    .bind(&filter.min_families) // $9
    .bind(&filter.max_families) // $10
    .bind(&filter.min_repositories_newest) // $11
    .bind(&filter.max_repositories_newest) // $12
    .bind(&filter.min_families_newest) // $13
    .bind(&filter.max_families_newest) // $14
    .bind(&filter.category) // $15
    .bind(&filter.require_newest) // $16
    .bind(&filter.require_outdated) // $17
    .bind(&filter.require_problematic) // $18
    .bind(&filter.require_has_related) // $19
    .bind(&filter.require_vulnerable) // $20
    .bind(&filter.limit) // $21
    .fetch_all(pool)
    .await?)
}
