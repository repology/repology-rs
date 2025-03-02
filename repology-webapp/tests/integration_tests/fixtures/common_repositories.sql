INSERT INTO repositories(id, name, sortname, "desc", state, first_seen, last_seen, metadata) VALUES
	(1, 'freebsd', 'freebsd', 'FreeBSD', 'active', now() + interval '5 seconds', now() + interval '5 seconds', '{"singular": "FreeBSD port", "type": "repository"}'::json),
	(2, 'ubuntu_12', 'ubuntu_12', 'Ubuntu 12', 'active', now() + interval '5 seconds', now() + interval '5 seconds', '{"singular": "Ubuntu 12 package", "type": "repository", "valid_till": "1900-01-01"}'::json),
	(3, 'ubuntu_24', 'ubuntu_24', 'Ubuntu 24', 'active', now() + interval '5 seconds', now() + interval '5 seconds', '{"singular": "Ubuntu 24 package", "type": "repository"}'::json),
	(4, 'freshcode', 'freshcode', 'freshcode.club', 'active', now() + interval '5 seconds', now() + interval '5 seconds', '{"singular": "freshcode.club entry", "type": "site"}'::json),
	-- legacy repositories may have meta not filled
	(5, 'ubuntu_10', 'ubuntu_10', 'Ubuntu 10', 'legacy', now() + interval '5 seconds', now() + interval '5 seconds', '{}'::json);

UPDATE repositories
SET
	num_packages = 12345;
