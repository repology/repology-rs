COVERAGE_FLAGS=-Cinstrument-coverage -Zcoverage-options=condition

cov: llvm-cov

grcov:
	rm -rf target/coverage-profile
	mkdir -p target/coverage-output-grcov
	env RUSTFLAGS="${COVERAGE_FLAGS}" LLVM_PROFILE_FILE=$$(pwd)/target/coverage-profile/profile_%10m_%p.profraw cargo test --features=coverage
	grcov target/coverage-profile --binary-path ./target/debug/ -s . -t html --ignore-not-existing --branch --keep-only 'repology-webapp/*' --keep-only 'repology-common/*' --keep-only 'repology-vulnupdater/*' -o target/coverage-output-grcov
	xdg-open "file://$$(pwd)/target/coverage-output-grcov/html/index.html"

llvm-cov:
	rm -rf target/coverage-profile
	mkdir -p target/coverage-output-llvm-cov
	env RUSTFLAGS="${COVERAGE_FLAGS}" LLVM_PROFILE_FILE=$$(pwd)/target/coverage-profile/profile_%10m_%p.profraw cargo test --features=coverage
	llvm-profdata merge \
		--sparse \
		target/coverage-profile/*.profraw \
		-o target/coverage-profile/merged.profdata
	llvm-cov show \
		--use-color \
		--ignore-filename-regex=/.cargo/registry \
		--ignore-filename-regex=rustc/ \
		--ignore-filename-regex=libversion/ \
		--ignore-filename-regex=repology-webapp-test-utils/ \
		-Xdemangler=rustfilt \
		$$(env RUSTFLAGS="${COVERAGE_FLAGS}" cargo test --tests --no-run --message-format=json | jq -r 'select(.profile.test == true) | .filenames[]' | grep -v dSYM | sed -e 's|.*|--object &|') \
		--instr-profile=target/coverage-profile/merged.profdata \
		--show-line-counts-or-regions \
		--show-instantiations \
		--format=html \
		-o target/coverage-output-llvm-cov
	xdg-open "file://$$(pwd)/target/coverage-output-llvm-cov/index.html"
