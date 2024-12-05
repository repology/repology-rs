INSERT INTO metapackages(id, effname, num_repos, num_families) VALUES
	(1, 'orphaned-with-cves', 0, 0),
	(2, 'orphaned-without-cves', 0, 0),
	(3, 'tworanges', 1, 1),
	(4, 'manyranges', 1, 1);

INSERT INTO cves(cve_id, published, last_modified, matches, cpe_pairs) VALUES
	(
		'CVE-1-1',
		'2024-01-01 00:00:00',
		'2024-02-01 00:00:00',
		'[["foo","foo","*","*","*","*","*","*",null,null,true,true]]'::jsonb,
		'{"foo:foo"}'::text[]
	),
	(
		'CVE-2-2',
		'2024-01-01 00:00:00',
		'2024-02-01 00:00:00',
		'[
			["bar","bar","*","*","*","*","*","*",null,null,true,true],
			["bar","bar","*","*","*","*","*","*","1.1",null,true,true],
			["bar","bar","*","*","*","*","*","*","1.2",null,false,true],
			["bar","bar","*","*","*","*","*","*","1.3","1.4",true,false],
			["bar","bar","*","*","*","*","*","*","1.5","1.6",false,true],
			["bar","bar","*","*","*","*","*","*",null,"1.7",true,true],
			["bar","bar","*","*","*","*","*","*",null,"1.8",true,false]
		]'::jsonb,
		'{"bar:bar"}'::text[]
	),
	(
		'CVE-3-3',
		'2024-01-01 00:00:00',
		'2024-02-01 00:00:00',
		'[
			["baz","baz","*","*","*","*","*","*","1.3","1.4",true,false],
			["baz","baz","*","*","*","*","*","*","1.5","1.6",false,true]
		]'::jsonb,
		'{"baz:baz"}'::text[]
	);

INSERT INTO manual_cpes(effname, cpe_vendor, cpe_product, cpe_edition, cpe_lang, cpe_sw_edition, cpe_target_sw, cpe_target_hw, cpe_other, added_ts) VALUES
	('orphaned-with-cves', 'foo', 'foo', '*', '*', '*', '*', '*', '*', '2024-01-01 00:00:00'),
	('manyranges', 'bar', 'bar', '*', '*', '*', '*', '*', '*', '2024-01-01 00:00:00'),
	('tworanges', 'baz', 'baz', '*', '*', '*', '*', '*', '*', '2024-01-01 00:00:00');
