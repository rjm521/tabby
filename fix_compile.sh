#!/bin/bash
set -e

echo "Fixing compilation issues..."

# 设置环境变量
export SQLX_OFFLINE=true
export DATABASE_URL="sqlite:tabby.sqlite"

# 清理并重新构建
echo "Cleaning up..."
rm -f tabby.sqlite tabby.sqlite-shm tabby.sqlite-wal

echo "Testing compilation..."
cd ee/tabby-db
cargo check --quiet 2>&1 | head -20

echo "Testing full project compilation..."
cd ../..
make dev-build 2>&1 | tail -30

echo "Done."