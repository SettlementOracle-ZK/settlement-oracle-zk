.PHONY: db-up db-migrate db-seed db-status db-reset db-down
.PHONY: test-api test-escrow test-oracle test-zk test-all
.PHONY: settlement-flow

db-up:
	docker compose up -d postgres

db-migrate:
	cargo run --manifest-path api/Cargo.toml --bin migrate -- up

db-seed:
	cargo run --manifest-path api/Cargo.toml --bin migrate -- seed

db-status:
	cargo run --manifest-path api/Cargo.toml --bin migrate -- status

db-reset:
	cargo run --manifest-path api/Cargo.toml --bin migrate -- reset --yes

db-down:
	docker compose down

test-escrow:
	PATH="$$HOME/.cargo/bin:$$PATH" cargo test -p escrow

test-api:
	DATABASE_URL=postgres://settlement:settlement@127.0.0.1:5433/settlement \
		cargo test --manifest-path api/Cargo.toml

test-oracle:
	cd oracle-connector && npm test

test-zk:
	cd zk-prover && npm test

test-all: test-escrow test-oracle test-zk test-api

settlement-flow:
	npm install --prefix scripts
	npm install --prefix oracle-connector
	npm install --prefix zk-prover
	npm run settlement-flow --prefix scripts

devnet-smoke:
	npm install --prefix scripts
	npm run devnet-smoke --prefix scripts
