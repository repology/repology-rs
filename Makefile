COVERAGE_FLAGS=-Cinstrument-coverage -Zcoverage-options=condition

snapshots:
	env INSTA_UPDATE=unseen cargo test

snapshots-review:
	cargo instal revirew

coverage:
	cargo llvm-cov --features=coverage --open

coverage-unit:
	cargo llvm-cov --features=coverage --open -- --skip integration_tests --skip snapshot_tests

coverage-integration:
	cargo llvm-cov --features=coverage --open -- integration_tests

coverage-snapshot:
	cargo llvm-cov --features=coverage --open -- snapshot_tests
