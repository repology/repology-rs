COVERAGE_FLAGS=-Cinstrument-coverage -Zcoverage-options=condition

snapshots:
	env INSTA_UPDATE=unseen cargo test snapshot_tests

snapshots-review:
	cargo insta review

coverage:
	cargo llvm-cov --open

coverage-unit:
	cargo llvm-cov --open -- --skip integration_tests --skip snapshot_tests

coverage-integration:
	cargo llvm-cov --open -- integration_tests

coverage-snapshot:
	cargo llvm-cov --open -- snapshot_tests
