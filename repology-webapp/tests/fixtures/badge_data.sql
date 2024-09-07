INSERT INTO repositories(id, name, sortname, "desc", state, first_seen, last_seen, metadata) VALUES
	(1, 'freebsd', 'freebsd', 'FreeBSD', 'active', now(), now(), '{"singular": "FreeBSD port", "type": "repository"}'::json),
	(2, 'ubuntu', 'ubuntu', 'Ubuntu', 'active', now(), now(), '{"singular": "Ubuntu package", "type": "repository"}'::json),
	(3, 'freshcode', 'freshcode', 'freshcode.club', 'active', now(), now(), '{"singular": "freshcode.club entry", "type": "site"}'::json);

INSERT INTO packages(effname, version, versionclass, flags, repo, family, trackname, visiblename, projectname_seed, origversion, rawversion, shadow) VALUES
	('zsh', '1.1', 1, 0, 'freebsd', 'freebsd', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '1.0', 2, 0, 'ubuntu', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '1.2', 3, 0, 'ubuntu', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false);
