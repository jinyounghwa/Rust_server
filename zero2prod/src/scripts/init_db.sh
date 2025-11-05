#!/usr/bin/env bash

set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "error: psql is not installed."
    echo >&2 "Install PostgreSQL client or use: brew install postgresql"
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "error: sqlx is not installed."
    echo >&2 "Install with: cargo install --version='0.6' sqlx-cli --no-default-features --features postgres"
    exit 1
fi

DB_USER="${DB_USER:=postgres}"
DB_PASSWORD="${DB_PASSWORD:=password}"
DB_HOST="${DB_HOST:=localhost}"
DB_PORT="${DB_PORT:=5432}"
DB_NAME="${DB_NAME:=newsletter}"

# Docker 컨테이너가 이미 실행 중인지 확인
if ! docker ps --filter "name=zero2prod-db" --format '{{.Names}}' | grep -q "zero2prod-db"; then
    echo "Starting Docker PostgreSQL container..."
    docker run -d \
        --name zero2prod-db \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}:5432" \
        postgres:latest \
        postgres -N 1000
else
    echo "PostgreSQL container is already running."
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    echo "Postgres is still unavailable - sleeping"
    sleep 1
done

echo >&2 "Postgres is up and running on port ${DB_PORT} - running migrations now!"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL

sqlx database create
sqlx migrate run

echo >&2 "Postgres has been migrated, ready to go!"