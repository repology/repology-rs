-- first point is intentionally long in the past to check whether it's handled properly producing a line
INSERT INTO statistics_history(ts, snapshot) VALUES
	(now() - interval '128 day', '{"num_packages":1,"num_problems":1,"num_maintainers":1,"num_metapackages":1}'),
	-- emulate time difference between application and database
	(now() + interval '5 second', '{"num_packages":10,"num_problems":10,"num_maintainers":10,"num_metapackages":10}');

INSERT INTO repositories_history(ts, snapshot) VALUES
	(now() - interval '128 day', '{"freebsd":{"num_maintainers":1,"num_problems":1,"num_metapackages":1,"num_metapackages_unique":1,"num_metapackages_newest":1,"num_metapackages_outdated":1,"num_metapackages_problematic":1,"num_metapackages_vulnerable":1}}'::jsonb),
	(now() + interval '5 second', '{"freebsd":{"num_maintainers":10,"num_problems":10,"num_metapackages":10,"num_metapackages_unique":10,"num_metapackages_newest":10,"num_metapackages_outdated":10,"num_metapackages_problematic":10,"num_metapackages_vulnerable":10}}'::jsonb);

INSERT INTO repositories_history_new(repository_id, ts, num_maintainers, num_problems, num_projects, num_projects_unique, num_projects_newest, num_projects_outdated, num_projects_problematic, num_projects_vulnerable) VALUES
	(1, now() - interval '128 day', 1, 1, 1, 1, 1, 1, 1, 1),
	(1, now() + interval '5 second', 10, 10, 10, 10, 10, 10, 10, 10);
