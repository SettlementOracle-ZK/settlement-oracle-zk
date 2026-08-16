.PHONY: db-up db-migrate db-seed db-status db-reset db-down

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
