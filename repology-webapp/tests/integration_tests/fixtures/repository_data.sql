INSERT INTO repositories(
	id,
	name,
	sortname,
	"desc",
	state,
	first_seen,
	last_seen,
	metadata,
	num_metapackages,
	used_package_link_types
) VALUES
(
	1,
	'orphaned',
	'orphaned',
	'Orphaned',
	'legacy',
	'2023-01-01 00:00:00',
	'2024-01-01 00:00:00',
	'{}'::jsonb,
	0,
	NULL
),
(
	2,
	'orphaned_in_future',
	'orphaned_in_future',
	'Orphaned in future',
	'legacy',
	'2023-01-01 00:00:00',
	'2124-01-01 00:00:00',
	'{}'::jsonb,
	0,
	NULL
),
(
	3,
	'stripped',
	'stripped',
	'Stripped',
	'active',
	'2023-01-01 00:00:00',
	'2124-01-01 00:00:00',
	'{}'::jsonb,
	1,
	NULL
),
(
	4,
	'good',
	'good',
	'Good',
	'active',
	'2023-01-01 00:00:00',
	'2124-01-01 00:00:00',
	'{"repolinks":[{"url":"https://example.com/goodrepo","desc":"Repository homepage"}]}'::jsonb,
	1,
	'{0,1,7,9,19}'::integer[]
),
(
	5,
	'empty',
	'empty',
	'Empty',
	'active',
	'2023-01-01 00:00:00',
	'2124-01-01 00:00:00',
	'{}'::jsonb,
	0,
	NULL
);
