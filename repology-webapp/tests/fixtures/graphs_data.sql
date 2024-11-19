-- first point is intentionally long in the past to check whether it's handled properly producing a line
INSERT INTO statistics_history(ts, snapshot) VALUES
	(now() - interval '128 day', '{"num_packages":0,"num_problems":0,"num_maintainers":0,"num_metapackages":0}'),
	(now(), '{"num_packages":10,"num_problems":10,"num_maintainers":10,"num_metapackages":10}');

INSERT INTO repositories_history(ts, snapshot) VALUES
	(now() - interval '128 day', '{"freebsd":{"num_maintainers":0,"num_problems":0,"num_metapackages":0,"num_metapackages_unique":0,"num_metapackages_newest":0,"num_metapackages_outdated":0,"num_metapackages_problematic":0,"num_metapackages_vulnerable":0}}'::jsonb),
	(now(), '{"freebsd":{"num_maintainers":10,"num_problems":10,"num_metapackages":10,"num_metapackages_unique":10,"num_metapackages_newest":10,"num_metapackages_outdated":10,"num_metapackages_problematic":10,"num_metapackages_vulnerable":10}}'::jsonb);

INSERT INTO repositories_history_new(repository_id, ts, num_maintainers, num_problems, num_projects, num_projects_unique, num_projects_newest, num_projects_outdated, num_projects_problematic, num_projects_vulnerable) VALUES
	(1, now() - interval '128 day', 0, 0, 0, 0, 0, 0, 0, 0),
	(1, now(), 0, 0, 0, 0, 0, 0, 0, 0);
