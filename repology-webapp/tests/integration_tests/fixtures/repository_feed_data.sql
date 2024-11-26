INSERT INTO repository_events(repository_id, ts, metapackage_id, type, data) VALUES
	(1, now() + interval '5 second', 1, 'added', '{}'::jsonb),
	(1, now() + interval '5 second', 1, 'removed', '{}'::jsonb),
	(1, now() + interval '5 second', 1, 'uptodate', '{"version": "111"}'::jsonb),
	(1, now() + interval '5 second', 1, 'outdated', '{"version": "222", "newest_versions": ["333", "444"]}'::jsonb),
	(1, now() + interval '5 second', 1, 'outdated', '{"version": "555"}'::jsonb),
	(1, now() + interval '5 second', 1, 'ignored', '{"version": "666"}'::jsonb);
