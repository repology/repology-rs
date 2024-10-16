INSERT INTO runs(id, type, repository_id, status, start_ts, finish_ts, num_lines, num_warnings, num_errors) VALUES
	(1, 'fetch', 1, 'running', now() - interval '1h', NULL, NULL, NULL, NULL),
	(2, 'fetch', 1, 'successful', now() - interval '1h', now(), 1, 2, 3);

INSERT INTO log_lines(run_id, lineno, timestamp, severity, message) VALUES
	(1, 1, now(), 'notice', 'Hello, world!'),
	(1, 2, now(), 'warning', 'Hello, world!'),
	(1, 3, now(), 'error', 'Hello, world!'),
	(2, 1, now(), 'notice', 'Hello, world!'),
	(2, 2, now(), 'warning', 'Hello, world!'),
	(2, 3, now(), 'error', 'Hello, world!');
