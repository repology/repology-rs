INSERT INTO repositories(id, name, sortname, "desc", state, first_seen, last_seen, metadata) VALUES
	(1, 'freebsd', 'freebsd', 'FreeBSD', 'active', now(), now(), '{"singular": "FreeBSD port", "type": "repository"}'::json);

INSERT INTO packages(effname, version, versionclass, flags, repo, family, trackname, visiblename, projectname_seed, origversion, rawversion, shadow) VALUES
	('zsh', '1.1', 1, 0, 'freebsd', 'freebsd', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '0.9', 2, 0, 'ubuntu_12', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '1.0', 2, 0, 'ubuntu_24', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '1.2', 3, 0, 'ubuntu_24', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '1.0', 2, 0, 'freshcode', 'freshcode', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false);

INSERT INTO metapackages(effname, num_repos) VALUES
	('zsh', 4),
	('orphaned', 0);
