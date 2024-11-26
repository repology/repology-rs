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
	now() - interval '2 week',
	now() - interval '1 week',
	'{}'::jsonb,
	0,
	NULL
),
(
	2,
	'stripped',
	'stripped',
	'Stripped',
	'active',
	now() - interval '2 week',
	now() + interval '5 second',
	'{}'::jsonb,
	1,
	NULL
),
(
	3,
	'good',
	'good',
	'Good',
	'active',
	now() - interval '2 week',
	now() + interval '5 second',
	'{"repolinks":[{"url":"https://example.com/goodrepo","desc":"Repository homepage"}]}'::jsonb,
	1,
	'{0,1,7,9,19}'::integer[]
);
