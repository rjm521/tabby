#!/bin/bash
set -e

echo "Testing database migration fix..."

# 删除现有的数据库文件
rm -f tabby.sqlite tabby.sqlite-shm tabby.sqlite-wal

echo "Deleted existing database files"

# 检查编译
cd ee/tabby-db
echo "Running cargo check in tabby-db..."
cargo check 2>&1 | head -20

echo "Done."