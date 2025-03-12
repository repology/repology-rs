INSERT INTO metapackages(effname, has_related) VALUES
	('upupup', true),
	('downdowndown', false);

INSERT INTO project_turnover(effname, delta, ts, family) VALUES
	('upupup', 2, now() - interval '23.5 hours', 'foobar'),
	('downdowndown', -2, now() - interval '23.5 hours', 'foobar');
