INSERT INTO links(url)
	VALUES ('https://example.com/not-checked');

INSERT INTO links(url, ipv4_last_success, ipv4_last_failure, ipv4_status_code, ipv4_permanent_redirect_target)
	VALUES ('https://example.com/ipv4-failure', NULL, '2024-01-01 00:00:00', 404, NULL);

INSERT INTO links(url, ipv4_last_success, ipv4_last_failure, ipv4_status_code, ipv4_permanent_redirect_target)
	VALUES ('https://example.com/ipv4-success', '2024-01-01 00:00:00', NULL, 200, NULL);

INSERT INTO links(url, ipv4_last_success, ipv4_last_failure, ipv4_status_code, ipv4_permanent_redirect_target)
	VALUES ('https://example.com/ipv4-redirect', '2024-01-01 00:00:00', NULL, 200, 'https://example.com/ipv4-redirect-target');

INSERT INTO links(url, ipv6_last_success, ipv6_last_failure, ipv6_status_code, ipv6_permanent_redirect_target)
	VALUES ('https://example.com/ipv6-failure', NULL, '2024-01-01 00:00:00', 404, NULL);

INSERT INTO links(url, ipv6_last_success, ipv6_last_failure, ipv6_status_code, ipv6_permanent_redirect_target)
	VALUES ('https://example.com/ssl-failure', NULL, '2024-01-01 00:00:00', -500, NULL);

INSERT INTO links(url, ipv6_last_success, ipv6_last_failure, ipv6_status_code, ipv6_permanent_redirect_target)
	VALUES ('https://example.com/ipv6-success', '2024-01-01 00:00:00', NULL, 200, NULL);

INSERT INTO links(url, ipv6_last_success, ipv6_last_failure, ipv6_status_code, ipv6_permanent_redirect_target)
	VALUES ('https://example.com/ipv6-redirect', '2024-01-01 00:00:00', NULL, 200, 'https://example.com/ipv6-redirect-target');

UPDATE links SET first_extracted = '2024-01-01 00:00:00';
