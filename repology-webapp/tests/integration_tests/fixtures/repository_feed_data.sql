INSERT INTO repository_events(repository_id, ts, metapackage_id, type, data) VALUES
	-- same time as we're also testing stable ordering by type
	(1, '2124-01-01 00:00:00', 1, 'added', '{}'::jsonb),
	(1, '2124-01-01 00:00:00', 1, 'removed', '{}'::jsonb),
	(1, '2124-01-01 00:00:00', 1, 'uptodate', '{"version": "111"}'::jsonb),
	(1, '2124-01-01 00:00:00', 1, 'outdated', '{"version": "222", "newest_versions": ["333", "444"]}'::jsonb),
	(1, '2124-01-01 00:00:00', 1, 'outdated', '{"version": "555"}'::jsonb),
	(1, '2124-01-01 00:00:00', 1, 'ignored', '{"version": "666"}'::jsonb);
