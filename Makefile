.PHONY: setup-db

setup-db:
	@echo "Setting up database..."
	@rm -rf db.sqlite3
	@sqlx database setup