INSERT INTO repositories(id, name, sortname, "desc", state, first_seen, last_seen, metadata) VALUES
	(1, 'freebsd', 'freebsd', 'FreeBSD', 'active', now(), now(), '{"singular": "FreeBSD port", "type": "repository"}'::json),
	(2, 'ubuntu_12', 'ubuntu_12', 'Ubuntu 12', 'active', now(), now(), '{"singular": "Ubuntu 12 package", "type": "repository", "valid_till": "1900-01-01"}'::json),
	(3, 'ubuntu_24', 'ubuntu_24', 'Ubuntu 24', 'active', now(), now(), '{"singular": "Ubuntu 24 package", "type": "repository"}'::json),
	(4, 'freshcode', 'freshcode', 'freshcode.club', 'active', now(), now(), '{"singular": "freshcode.club entry", "type": "site"}'::json),
	-- legacy repositories may have meta not filled
	(5, 'ubuntu_10', 'ubuntu_10', 'Ubuntu 10', 'legacy', now(), now(), '{}'::json);

INSERT INTO packages(effname, version, versionclass, flags, repo, family, trackname, visiblename, projectname_seed, origversion, rawversion, shadow) VALUES
	('zsh', '1.1', 1, 0, 'freebsd', 'freebsd', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '0.9', 2, 0, 'ubuntu_12', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '1.0', 2, 0, 'ubuntu_24', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '1.2', 3, 0, 'ubuntu_24', 'ubuntu', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false),
	('zsh', '1.0', 2, 0, 'freshcode', 'freshcode', 'zsh', 'zsh', 'zsh', '1.0', '1.0', false);
