INSERT INTO packages(effname, version, versionclass, flags, repo, family, trackname, visiblename, projectname_seed, origversion, rawversion, shadow) VALUES
	('zsh', '3.0', 5, 0, '', '', '', '', '', '', '', false),
	('zsh', '1_0_0', 1, 0, '', '', '', '', '', '', '', false),
	('zsh', '1.0.0', 1, 0, '', '', '', '', '', '', '', false),
	('zsh', '1.0', 1, 0, '', '', '', '', '', '', '', false),
	('zsh', '2024', 2, 1 << 8, '', '', '', '', '', '', '', false),  -- sink
	('zsh', '2025', 3, 0, '', '', '', '', '', '', '', false);  -- ignored
