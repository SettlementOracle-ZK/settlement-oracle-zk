.PHONY: db-up db-migrate db-seed db-status db-reset db-down
.PHONY: test-api test-escrow test-oracle test-zk test-all
.PHONY: settlement-flow devnet-smoke deploy-local local-smoke

LOCAL_VALIDATOR_URL ?= http://127.0.0.1:8899
LOCAL_PROGRAM_ID := $(shell solana-keygen pubkey target/deploy/escrow-keypair.json 2>/dev/null)
DEVNET_RPC_URL ?= https://api.devnet.solana.com
PYTH_PRICE_FEED ?= 7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE

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

deploy-local:
	@test -f target/deploy/escrow-keypair.json || (echo "Missing target/deploy/escrow-keypair.json — run anchor build first." && exit 1)
	export CARGO_TARGET_DIR=$$PWD/target && \
		PATH="$$HOME/.cargo/bin:$$PATH" cargo-build-sbf --manifest-path programs/escrow/Cargo.toml --arch v0
	cp target/sbpf-solana-solana/release/escrow.so target/deploy/escrow.so
	solana program deploy target/deploy/escrow.so \
		--program-id target/deploy/escrow-keypair.json \
		--url $(LOCAL_VALIDATOR_URL)

devnet-smoke:
	npm install --prefix scripts
	@RPC=""; \
	if [ -n "$$SOLANA_RPC_URL" ]; then \
		RPC="$$SOLANA_RPC_URL"; \
		echo "Using SOLANA_RPC_URL=$$RPC"; \
	elif solana cluster-version --url $(LOCAL_VALIDATOR_URL) >/dev/null 2>&1; then \
		RPC="$(LOCAL_VALIDATOR_URL)"; \
		echo "Local validator detected — using $$RPC"; \
	else \
		RPC="$(DEVNET_RPC_URL)"; \
		echo "No local validator — using $$RPC"; \
	fi; \
	PROGRAM_ID="$(LOCAL_PROGRAM_ID)"; \
	if [ "$$RPC" = "$(LOCAL_VALIDATOR_URL)" ] && [ -n "$$PROGRAM_ID" ] && \
		! solana program show "$$PROGRAM_ID" --url "$$RPC" >/dev/null 2>&1; then \
		echo "Escrow program not on local validator — running make deploy-local"; \
		$(MAKE) deploy-local LOCAL_VALIDATOR_URL="$$RPC"; \
	fi; \
	PYTH_PRICE_FEED="$(PYTH_PRICE_FEED)" SOLANA_RPC_URL="$$RPC" \
		npm run devnet-smoke --prefix scripts

local-smoke:
	bash scripts/local-smoke.sh
