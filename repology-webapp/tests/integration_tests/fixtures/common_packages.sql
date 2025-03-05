INSERT INTO packages(id, effname, version, versionclass, flags, repo, family, trackname, visiblename, projectname_seed, origversion, rawversion, shadow) VALUES
	(1, 'zsh', '1.1', 1, 0, 'freebsd', 'freebsd', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	(2, 'zsh', '0.9', 2, 0, 'ubuntu_12', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	(3, 'zsh', '1.0', 2, 0, 'ubuntu_24', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	(4, 'zsh', '1.2', 3, 0, 'ubuntu_24', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	(5, 'zsh', '1.0', 2, 0, 'freshcode', 'freshcode', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false);

INSERT INTO metapackages(id, effname, num_repos, num_families) VALUES
	(1, 'zsh', 4, 3),
	(2, 'orphaned', 0, 0),
	(3, 'zsh-old', 0, 0);

INSERT INTO project_redirects(project_id, repository_id, is_actual, trackname) VALUES
	(1, 1, true, 'shells/zsh'),
	(3, 1, false, 'shells/zsh');
