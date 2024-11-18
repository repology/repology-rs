INSERT INTO packages(id, effname, version, versionclass, flags, repo, family, trackname, visiblename, projectname_seed, origversion, rawversion, shadow) VALUES
	(1, 'zsh', '1.0', 1, 0, 'freebsd', 'freebsd', '', '', '', '', '', false),
	(2, 'gcc', '1.0', 1, 0, 'freebsd', 'freebsd', '', '', '', '', '', false),
	(3, 'binutils', '1.0', 1, 0, 'freebsd', 'freebsd', '', '', '', '', '', false);

INSERT INTO url_relations(metapackage_id, urlhash, weight) VALUES
	(3, 123123123, 1.0),
	(4, 123123123, 1.0);

INSERT INTO metapackages(id, effname, num_repos, num_families) VALUES
	(1, 'zsh', 1, 1),
	(2, 'orphaned', 0, 0),
	(3, 'gcc', 1, 1),
	(4, 'binutils', 1, 1);
