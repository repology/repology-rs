INSERT INTO links(url)
	VALUES ('https://example.com/not-checked');

INSERT INTO links(url, ipv4_last_success, ipv4_last_failure, ipv4_success, ipv4_status_code, ipv4_permanent_redirect_target)
	VALUES ('https://example.com/ipv4-failure', NULL, now(), FALSE, 404, NULL);

INSERT INTO links(url, ipv4_last_success, ipv4_last_failure, ipv4_success, ipv4_status_code, ipv4_permanent_redirect_target)
	VALUES ('https://example.com/ipv4-success', now(), NULL, TRUE, 200, NULL);

INSERT INTO links(url, ipv4_last_success, ipv4_last_failure, ipv4_success, ipv4_status_code, ipv4_permanent_redirect_target)
	VALUES ('https://example.com/ipv4-redirect', now(), NULL, TRUE, 200, 'https://example.com/ipv4-redirect-target');

INSERT INTO links(url, ipv6_last_success, ipv6_last_failure, ipv6_success, ipv6_status_code, ipv6_permanent_redirect_target)
	VALUES ('https://example.com/ipv6-failure', NULL, now(), FALSE, 404, NULL);

INSERT INTO links(url, ipv6_last_success, ipv6_last_failure, ipv6_success, ipv6_status_code, ipv6_permanent_redirect_target)
	VALUES ('https://example.com/ssl-failure', NULL, now(), FALSE, -500, NULL);

INSERT INTO links(url, ipv6_last_success, ipv6_last_failure, ipv6_success, ipv6_status_code, ipv6_permanent_redirect_target)
	VALUES ('https://example.com/ipv6-success', now(), NULL, TRUE, 200, NULL);

INSERT INTO links(url, ipv6_last_success, ipv6_last_failure, ipv6_success, ipv6_status_code, ipv6_permanent_redirect_target)
	VALUES ('https://example.com/ipv6-redirect', now(), NULL, TRUE, 200, 'https://example.com/ipv6-redirect-target');
