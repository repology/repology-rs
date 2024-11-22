INSERT INTO maintainers(id, maintainer, orphaned_at) VALUES
	(1, 'orphaned@example.com', now() - interval '7 day'); 

INSERT INTO maintainers(id, maintainer, num_packages, num_projects, counts_per_repo, num_projects_per_category) VALUES
	(2, 'active@example.com', 10, 10, '{"freebsd":[10,11,12,13,14,15]}'::jsonb, '{"games":10}'::jsonb),
	(3, 'fallback-mnt-foo@repology', 1, 1, '{"freebsd":[1,1,1,1,1,1]}'::jsonb, '{"games":10}'::jsonb);
