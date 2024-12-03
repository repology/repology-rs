INSERT INTO metapackages(id, effname, num_repos, num_families) VALUES
	(1, 'vulnerable', 1, 1),
	(2, 'orphaned-with-cves', 0, 0),
	(3, 'orphaned-without-cves', 0, 0);

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
		'[["bar","bar","*","*","*","*","*","*","1.0.0","2.0.0",true,false],["bar","bar","*","*","*","*","*","*","1.0.0","2.0.0",false,true]]'::jsonb,
		'{"bar:bar"}'::text[]
	);

INSERT INTO manual_cpes(effname, cpe_vendor, cpe_product, cpe_edition, cpe_lang, cpe_sw_edition, cpe_target_sw, cpe_target_hw, cpe_other) VALUES
	('orphaned-with-cves', 'foo', 'foo', '*', '*', '*', '*', '*', '*'),
	('vulnerable', 'bar', 'bar', '*', '*', '*', '*', '*', '*');
