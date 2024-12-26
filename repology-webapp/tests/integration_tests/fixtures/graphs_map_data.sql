INSERT INTO repositories(id, name, sortname, "desc", state, first_seen, last_seen, metadata, num_metapackages, num_metapackages_newest) VALUES
	(1, 'ubuntu_12', 'ubuntu_12', 'Ubuntu 12', 'active', now(), now(), '{"color": "aabbcc"}'::json, 10000, 10000),
	(2, 'ubuntu_20', 'ubuntu_20', 'Ubuntu 20', 'active', now(), now(), '{"color": "bbccdd"}'::json, 20000, 20000);
