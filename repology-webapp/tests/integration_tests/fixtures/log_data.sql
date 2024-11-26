INSERT INTO runs(id, type, repository_id, status, start_ts, finish_ts, num_lines, num_warnings, num_errors) VALUES
	(1, 'fetch', 1, 'running', '2024-01-01 00:00:00', NULL, NULL, NULL, NULL),
	(2, 'fetch', 1, 'successful', '2024-01-01 00:00:00', '2124-01-01 00:00:00', 1, 2, 3);

INSERT INTO log_lines(run_id, lineno, timestamp, severity, message) VALUES
	(1, 1, '2024-01-01 00:00:01', 'notice', 'Hello, world!'),
	(1, 2, '2024-01-01 00:00:02', 'warning', 'Hello, world!'),
	(1, 3, '2024-01-01 00:00:03', 'error', 'Hello, world!'),
	(2, 1, '2024-01-01 00:00:01', 'notice', 'Hello, world!'),
	(2, 2, '2024-01-01 00:00:02', 'warning', 'Hello, world!'),
	(2, 3, '2024-01-01 00:00:03', 'error', 'Hello, world!');
