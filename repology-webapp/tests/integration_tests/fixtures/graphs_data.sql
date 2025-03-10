-- first point is intentionally long in the past to check whether it's handled properly producing a line
INSERT INTO statistics_history(ts, num_packages, num_problems, num_maintainers, num_projects) VALUES
	(now() - interval '128 day', 1, 1, 1, 1),
	-- emulate time difference between application and database
	(now() + interval '5 second', 10, 10, 10, 10);

INSERT INTO repositories_history(repository_id, ts, num_maintainers, num_problems, num_projects, num_projects_unique, num_projects_newest, num_projects_outdated, num_projects_problematic, num_projects_vulnerable) VALUES
	(1, now() - interval '128 day', 1, 1, 1, 1, 1, 1, 1, 1),
	(1, now() + interval '5 second', 10, 10, 10, 10, 10, 10, 10, 10);
