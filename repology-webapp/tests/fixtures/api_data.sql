INSERT INTO packages(
	effname,
	repo,
	family,
	subrepo,
	srcname,
	binname,
	visiblename,
	trackname,
	projectname_seed,
	version,
	origversion,
	rawversion,
	versionclass,
	comment,
	maintainers,
	licenses,
	category,
	flags,
	shadow
) VALUES
(
	'full',
	'repo',
	'family',
	'subrepo',
	'srcname',
	'binname',
	'visiblename',
	'trackname',
	'projectname_seed',
	'1.0',
	'1.0',
	'1.0_1',
	1,
	'Summary',
	'{"foo@example.com", "bar@example.com"}',
	'{"GPLv2", "GPLv3"}',
	'games',
	0,
	false
);
  
INSERT INTO packages(effname, repo, family, visiblename, trackname, projectname_seed, version, origversion, rawversion, versionclass, flags, shadow) VALUES
	('minimal', 'repo', 'family', 'visiblename', 'trackname', 'projectname_seed', '1.0', '1.0', '1.0_1', 1, 0, false),
	('vulnerable', 'repo', 'family', 'visiblename', 'trackname', 'projectname_seed', '1.0', '1.0', '1.0_1', 1, 1 << 16, false);
