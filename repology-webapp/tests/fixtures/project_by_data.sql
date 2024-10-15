INSERT INTO metapackages(id, effname, num_repos) VALUES
	(1, 'zsh', 4),
	(2, 'iperf2', 0),
	(3, 'iperf3', 0);

INSERT INTO project_names(project_id, repository_id, name_type, name) VALUES
	(1, 1, 'srcname', 'shells/zsh'),
	(1, 1, 'binname', 'zsh'),
	(2, 3, 'srcname', 'iperf'),
	(2, 3, 'binname', 'iperf'),
	(3, 3, 'srcname', 'iperf'),
	(3, 3, 'binname', 'iperf');
