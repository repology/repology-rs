INSERT INTO metapackages(id, effname, num_repos, num_families) VALUES
	(1, 'zsh', 4, 3),
	(2, 'orphaned-with-reports', 0, 0),
	(3, 'orphaned-without-reports', 0, 0),
	(4, 'xss-attempt', 1, 1),
	(5, 'all-flags', 1, 1),
	(6, 'many-reports', 1, 1);

INSERT INTO reports(effname, need_verignore, need_split, need_merge, need_vuln, comment) VALUES
	('orphaned-with-reports', false, false, false, false, ''),
	('xss-attempt', false, false, false, false, '<a href="malicious">'),
	('all-flags', true, true, true, true, 'Some comment');

INSERT INTO reports(effname, need_verignore, need_split, need_merge, need_vuln, comment)
SELECT 'many-reports', false, false, false, false, 'Spam'
FROM generate_series(0, 200);
