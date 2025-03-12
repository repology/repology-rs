INSERT INTO global_version_events(effname, ts, type, spread, data) VALUES
	('zsh', now() - interval '23 hours', 'newest_update', 6, '{"versions": ["1.2.3"]}'::jsonb);
