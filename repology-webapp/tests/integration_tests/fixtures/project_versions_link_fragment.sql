INSERT INTO links(id, url) VALUES
	(1, 'https://example.com/linkfragtest');

INSERT INTO packages(id, effname, version, versionclass, flags, repo, family, trackname, visiblename, projectname_seed, origversion, rawversion, shadow, links) VALUES
	(1, 'linkfragtest', '1.0', 1, 0, 'freebsd', 'freebsd', 'linkfragtest', 'linkfragtest', 'linkfragtest', '1.0', '1.0', false, '[[5, 1, "section"]]');

INSERT INTO metapackages(id, effname, num_repos, num_families) VALUES
	(1, 'linkfragtest', 1, 1);
