CREATE TABLE update_status (
	name text NOT NULL PRIMARY KEY,
	last_update_attempt_time timestamptz NULL,
	current_full_update_offset bigint NULL,
	last_full_update_time timestamptz NULL,
	last_update_time timestamptz NULL
);

CREATE TABLE cves (
	cve_id text NOT NULL PRIMARY KEY,
	published timestamptz NOT NULL,
	last_modified timestamptz NOT NULL,
	matches jsonb,
	cpe_pairs text[]
);

CREATE TABLE cve_updates (
	cpe_vendor text NOT NULL,
	cpe_product text NOT NULL
);

CREATE TABLE cpes (
	cpe_vendor text NOT NULL,
	cpe_product text NOT NULL,
	cpe_version text NOT NULL,
	cpe_update text NOT NULL,
	cpe_edition text NOT NULL,
	cpe_lang text NOT NULL,
	cpe_sw_edition text NOT NULL,
	cpe_target_sw text NOT NULL,
	cpe_target_hw text NOT NULL,
	cpe_other text NOT NULL
);

CREATE UNIQUE INDEX cpes_pkey ON cpes(cpe_vendor, cpe_product, cpe_version, cpe_update, cpe_edition, cpe_lang, cpe_sw_edition, cpe_target_sw, cpe_target_hw, cpe_other);
