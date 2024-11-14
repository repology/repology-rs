UPDATE repositories
SET
	num_metapackages = 10,
	num_metapackages_comparable = 5,
	num_metapackages_newest = 1,
	num_metapackages_outdated = 2,
	num_metapackages_vulnerable = 3,
	num_metapackages_problematic = 4,
	num_maintainers = 7
WHERE
	name = 'freebsd';
