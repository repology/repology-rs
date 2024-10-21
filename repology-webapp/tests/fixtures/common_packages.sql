INSERT INTO packages(id, effname, version, versionclass, flags, repo, family, trackname, visiblename, projectname_seed, origversion, rawversion, shadow) VALUES
	(1, 'zsh', '1.1', 1, 0, 'freebsd', 'freebsd', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	(2, 'zsh', '0.9', 2, 0, 'ubuntu_12', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	(3, 'zsh', '1.0', 2, 0, 'ubuntu_24', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	(4, 'zsh', '1.2', 3, 0, 'ubuntu_24', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	(5, 'zsh', '1.0', 2, 0, 'freshcode', 'freshcode', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false);

INSERT INTO metapackages(id, effname, num_repos) VALUES
	(1, 'zsh', 4),
	(2, 'orphaned', 0);
