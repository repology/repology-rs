COVERAGE_PATH!=mktemp -d /tmp/.repology-coverage-XXXXXXXX

cov:
	env RUSTFLAGS=-Cinstrument-coverage LLVM_PROFILE_FILE=${COVERAGE_PATH}/profile_%10m_%p.profraw cargo test
	grcov ${COVERAGE_PATH}/ --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing --keep-only 'repology-webapp/*' --keep-only 'repology-common/*' -o ${COVERAGE_PATH}
	xdg-open "file://${COVERAGE_PATH}/html/index.html"
