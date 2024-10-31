INSERT INTO maintainer_repo_metapackages_events(repository_id, maintainer_id, ts, metapackage_id, type, data) VALUES
	(1, 1, now(), 1, 'added', '{}'::jsonb),
	(1, 1, now(), 1, 'removed', '{}'::jsonb),
	(1, 1, now(), 1, 'uptodate', '{"version": "111"}'::jsonb),
	(1, 1, now(), 1, 'outdated', '{"version": "222", "newest_versions": ["333", "444"]}'::jsonb),
	(1, 1, now(), 1, 'outdated', '{"version": "555"}'::jsonb),
	(1, 1, now(), 1, 'ignored', '{"version": "666"}'::jsonb);
