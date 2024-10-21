CREATE TEMP TABLE tmp(
	effname TEXT NOT NULL,
	versionclass INTEGER NOT NULL,
	repo TEXT NOT NULL
) ON COMMIT DROP;

INSERT INTO tmp(effname, versionclass, repo) VALUES
	-- for pagination, substring tests
	('pkg_barbar_',        1, 'ubuntu_12'),
	('pkg_foofoo_',        1, 'ubuntu_12'),

	-- for inrepo/notinrepo tests
	('pkg_12e',            1, 'ubuntu_12'),
	('pkg_24e',            1, 'ubuntu_24'),
	('pkg_1224e',          1, 'ubuntu_12'),
	('pkg_1224e',          1, 'ubuntu_24'),
	('pkg_1224_newest_12', 1, 'ubuntu_12'),
	('pkg_1224_newest_12', 2, 'ubuntu_24'),
	('pkg_1224_newest_24', 2, 'ubuntu_12'),
	('pkg_1224_newest_24', 1, 'ubuntu_24');

INSERT INTO packages(effname, version, versionclass, flags, repo, family, trackname, visiblename, projectname_seed, origversion, rawversion, shadow)
SELECT
	effname,
	'',
	versionclass,
	0,
	repo,
	'',
	'',
	'',
	'',
	'',
	'',
	false
FROM tmp;

INSERT INTO metapackages(
	effname,
    num_repos,
    num_repos_nonshadow,
    num_families,
    num_repos_newest,
    num_families_newest,
    has_cves
)
SELECT
	effname,
	count(DISTINCT repo),
	count(DISTINCT repo) FILTER (WHERE NOT shadow),
	count(DISTINCT family),
	count(DISTINCT repo) FILTER (WHERE versionclass = 1 OR versionclass = 5),
	count(DISTINCT family) FILTER (WHERE versionclass = 1 OR versionclass = 5),
	false
FROM packages
GROUP BY effname;

INSERT INTO repo_metapackages(
    repository_id,
    effname,

    newest,
    outdated,
    problematic,
    "unique",

    vulnerable
)
SELECT
    (SELECT id FROM repositories WHERE name = repo) AS repository_id,
    effname,

    count(*) FILTER (WHERE versionclass = 1 OR versionclass = 4 OR versionclass = 5) > 0,
    count(*) FILTER (WHERE versionclass = 2) > 0,
    count(*) FILTER (WHERE versionclass = 3 OR versionclass = 7 OR versionclass = 8) > 0,

    max(num_families) = 1,

    count(*) FILTER (WHERE (flags & (1 << 16))::boolean) > 0
FROM packages
INNER JOIN metapackages USING(effname)
WHERE num_repos_nonshadow > 0
GROUP BY effname, repo;
