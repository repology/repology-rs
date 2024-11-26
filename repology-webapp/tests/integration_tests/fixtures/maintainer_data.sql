INSERT INTO maintainers(id, maintainer, orphaned_at) VALUES
	(1, 'orphaned@example.com', '2024-01-01 00:00:00'),
	(2, 'orphaned-in-future@example.com', '2124-01-01 00:00:00');

INSERT INTO maintainers(id, maintainer, num_packages, num_projects, counts_per_repo, num_projects_per_category) VALUES
	(3, 'active@example.com', 10, 10, '{"freebsd":[10,11,12,13,14,15]}'::jsonb, '{"games":10}'::jsonb),
	(4, 'fallback-mnt-foo@repology', 1, 1, '{"freebsd":[1,1,1,1,1,1]}'::jsonb, '{"games":10}'::jsonb),
	(5, 'no-vuln-column@example.com', 10, 10, '{"freebsd":[10,11,12,13,14]}'::jsonb, '{"games":10}'::jsonb);
