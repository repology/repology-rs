INSERT INTO metapackages(id, effname, num_repos, num_families) VALUES
	(1, 'zsh', 4, 3),
	(2, 'orphaned-with-history', 0, 0),
	(3, 'orphaned-without-history', 0, 0);

INSERT INTO metapackages_events(effname, ts, type, data) VALUES
	('orphaned-with-history', '2024-01-01 00:00:00', 'history_end', '{"last_repos":["freebsd"]}'),

	('zsh', '2024-01-01 00:00:00', 'history_start', '{"all_repos":["freebsd","ubuntu_10"],"newest_repos":["freebsd"],"newest_versions":["1.0"]}'),
	('zsh', '2024-01-01 00:00:01', 'version_update', '{"repos":["freebsd"],"branch":"newest","versions":["1.1"]}'),
	('zsh', '2024-01-01 00:00:02', 'repos_update', '{"repos_added":["ubuntu_24"],"repos_removed":["ubuntu_10"]}'),
	('zsh', '2024-01-01 00:00:03', 'catch_up', '{"repos":["ubuntu_24"],"branch":"newest"}'),
	('zsh', '2024-01-01 00:00:04', 'history_end', '{"last_repos":["freebsd","ubuntu_24","ubuntu_10"]}');
