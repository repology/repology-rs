INSERT INTO problems(package_id, repo, name, effname, maintainer, "type", data) VALUES
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'homepage_dead', '{"url":"https://example.com/","code":404}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'homepage_permanent_https_redirect', '{"url":"https://example.com/","target":"https://example.com/"}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'homepage_discontinued_google', '{"url":"https://example.com/"}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'homepage_discontinued_codeplex', '{"url":"https://example.com/"}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'homepage_discontinued_gna', '{"url":"https://example.com/"}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'homepage_discontinued_cpan', '{"url":"https://example.com/"}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'download_dead', '{"url":"https://example.com/","code":404}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'download_permanent_https_redirect', '{"url":"https://example.com/","target":"https://example.com/"}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'homepage_sourceforge_missing_trailing_slash', '{"url":"https://example.com/"}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'cpe_unreferenced', '{
		"cpe":{
			"cpe_vendor":"*",
			"cpe_product":"*",
			"cpe_edition":"*",
			"cpe_lang":"*",
			"cpe_sw_edition":"*",
			"cpe_target_sw":"*",
			"cpe_target_hw":"*",
			"cpe_other":"*"
		},
		"suggestions":[
			{
				"cpe_vendor":"suggesion_a",
				"cpe_product":"*",
				"cpe_edition":"*",
				"cpe_lang":"*",
				"cpe_sw_edition":"*",
				"cpe_target_sw":"*",
				"cpe_target_hw":"*",
				"cpe_other":"*"
			},
			{
				"cpe_vendor":"suggestion_b",
				"cpe_product":"*",
				"cpe_edition":"*",
				"cpe_lang":"*",
				"cpe_sw_edition":"*",
				"cpe_target_sw":"*",
				"cpe_target_hw":"*",
				"cpe_other":"*"
			}
		]
	}'::jsonb),
	(1, 'freebsd', 'shells/zsh', 'zsh', 'johndoe@example.com', 'cpe_missing', '{
		"suggestions":[
			{
				"cpe_vendor":"suggesion_a",
				"cpe_product":"*",
				"cpe_edition":"*",
				"cpe_lang":"*",
				"cpe_sw_edition":"*",
				"cpe_target_sw":"*",
				"cpe_target_hw":"*",
				"cpe_other":"*"
			},
			{
				"cpe_vendor":"suggestion_b",
				"cpe_product":"*",
				"cpe_edition":"*",
				"cpe_lang":"*",
				"cpe_sw_edition":"*",
				"cpe_target_sw":"*",
				"cpe_target_hw":"*",
				"cpe_other":"*"
			}
		]
	}'::jsonb);
