.PHONY: db-up db-migrate db-seed db-status db-reset db-down
.PHONY: test-api test-escrow test-oracle test-zk test-all
.PHONY: settlement-flow devnet-smoke deploy-local deploy-devnet devnet-oracle devnet-setup local-smoke demo-settle

LOCAL_VALIDATOR_URL ?= http://127.0.0.1:8899
LOCAL_PROGRAM_ID := $(shell solana-keygen pubkey target/deploy/escrow-keypair.json 2>/dev/null)
DEVNET_RPC_URL ?= https://api.devnet.solana.com
PYTH_PRICE_FEED ?= 7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE
# Smoke deposit in lamports (default 10000 = 0.00001 SOL). Demo: SMOKE_DEPOSIT_LAMPORTS=500000000
SMOKE_DEPOSIT_LAMPORTS ?= 10000

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

demo-settle:
	npm install --prefix scripts
	npm install --prefix oracle-connector
	npm install --prefix zk-prover
	npm run demo-settle --prefix scripts

deploy-local:
	@test -f target/deploy/escrow-keypair.json || (echo "Missing target/deploy/escrow-keypair.json — run anchor build first." && exit 1)
	export CARGO_TARGET_DIR=$$PWD/target && \
		rm -f target/sbpf-solana-solana/release/escrow.so && \
		PATH="$$HOME/.cargo/bin:$$PATH" cargo-build-sbf --manifest-path programs/escrow/Cargo.toml --arch v0
	cp target/sbpf-solana-solana/release/escrow.so target/deploy/escrow.so
	solana program deploy target/deploy/escrow.so \
		--program-id target/deploy/escrow-keypair.json \
		--url $(LOCAL_VALIDATOR_URL)

deploy-devnet:
	@test -f target/deploy/escrow-keypair.json || (echo "Missing target/deploy/escrow-keypair.json — run anchor build first." && exit 1)
	@echo "WARNING: devnet deploy costs ~1+ SOL. Run once; avoid redeploy unless the program changed."
	export CARGO_TARGET_DIR=$$PWD/target && \
		rm -f target/sbpf-solana-solana/release/escrow.so && \
		PATH="$$HOME/.cargo/bin:$$PATH" cargo-build-sbf --manifest-path programs/escrow/Cargo.toml --arch v0
	@test -f target/sbpf-solana-solana/release/escrow.so || (echo "cargo-build-sbf did not produce escrow.so — check Solana/Anchor toolchain." && exit 1)
	cp target/sbpf-solana-solana/release/escrow.so target/deploy/escrow.so
	@echo "Built $$(wc -c < target/deploy/escrow.so | tr -d ' ') byte escrow.so"
	solana program deploy target/deploy/escrow.so \
		--program-id target/deploy/escrow-keypair.json \
		--url $(DEVNET_RPC_URL) \
		--max-len $$(wc -c < target/deploy/escrow.so | tr -d ' ')
	@PROGRAM_ID=$$(solana-keygen pubkey target/deploy/escrow-keypair.json); \
		echo "Deployed escrow $$PROGRAM_ID on devnet"

devnet-oracle:
	npm install --prefix scripts
	SOLANA_RPC_URL="$(DEVNET_RPC_URL)" npm run bootstrap-devnet-oracle --prefix scripts

devnet-setup:
	@PROGRAM_ID=$$(solana-keygen pubkey target/deploy/escrow-keypair.json 2>/dev/null); \
	if [ -z "$$PROGRAM_ID" ]; then \
		echo "Missing target/deploy/escrow-keypair.json — run anchor build first."; exit 1; \
	fi; \
	if solana program show "$$PROGRAM_ID" --url $(DEVNET_RPC_URL) >/dev/null 2>&1; then \
		echo "Program $$PROGRAM_ID already on devnet — skipping deploy (saves SOL)"; \
	else \
		echo "First-time devnet deploy (~1+ SOL)..."; \
		$(MAKE) deploy-devnet DEVNET_RPC_URL="$(DEVNET_RPC_URL)"; \
	fi
	$(MAKE) devnet-oracle DEVNET_RPC_URL="$(DEVNET_RPC_URL)"

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
	SOLANA_RPC_URL="$$RPC" \
		SMOKE_DEPOSIT_LAMPORTS="$(SMOKE_DEPOSIT_LAMPORTS)" \
		npm run devnet-smoke --prefix scripts

local-smoke:
	bash scripts/local-smoke.sh
