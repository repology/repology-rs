INSERT INTO statistics_history(ts, snapshot) VALUES
	-- point is intentionally long in the past to check whether it's handled properly producing a line
	(now() - interval '128 day', '{"num_packages": 0, "num_problems": 0, "num_maintainers": 0, "num_metapackages": 0}'),
	(now(), '{"num_packages": 10, "num_problems": 10, "num_maintainers": 10, "num_metapackages": 10}');
