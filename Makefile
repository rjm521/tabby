# Tabby Makefile

# 变量定义
CARGO = cargo
RUSTC = rustc
RUSTFMT = rustfmt
CLIPPY = clippy
TEST = test
BUILD = build
CLEAN = clean
CHECK = check
INSTALL = install
UPDATE = update

# 默认目标
.PHONY: all
all: build

# 构建所有目标
.PHONY: build
build:
	$(CARGO) $(BUILD) --release

# 构建特定目标
.PHONY: build-tabby
build-tabby:
	$(CARGO) $(BUILD) --release -p tabby

.PHONY: build-index-cli
build-index-cli:
	$(CARGO) $(BUILD) --release -p tabby-index-cli

# 开发构建（不优化）
.PHONY: dev-build
dev-build:
	$(CARGO) $(BUILD)

# 运行测试
.PHONY: test
test:
	$(CARGO) $(TEST)

# 运行特定包的测试
.PHONY: test-tabby
test-tabby:
	$(CARGO) $(TEST) -p tabby

.PHONY: test-index-cli
test-index-cli:
	$(CARGO) $(TEST) -p tabby-index-cli

# 代码格式检查
.PHONY: fmt
fmt:
	$(CARGO) fmt --all

# 代码格式检查（不修改文件）
.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

# 代码检查
.PHONY: clippy
clippy:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# 清理构建文件
.PHONY: clean
clean:
	$(CARGO) $(CLEAN)

# 更新依赖
.PHONY: update
update:
	$(CARGO) $(UPDATE)

# 安装依赖
.PHONY: install
install:
	$(CARGO) $(INSTALL)

# 运行 tabby 服务器
.PHONY: run
run:
	$(CARGO) run --release --bin tabby serve

# 运行 tabby-index-cli
.PHONY: run-index-cli
run-index-cli:
	$(CARGO) run --release -p tabby-index-cli

# 构建并运行所有测试
.PHONY: ci
ci: fmt-check clippy test

# 帮助信息
.PHONY: help
help:
	@echo "Tabby Makefile 帮助信息:"
	@echo "  make build          - 构建所有目标（发布模式）"
	@echo "  make build-tabby    - 构建 tabby 主程序"
	@echo "  make build-index-cli - 构建 tabby-index-cli"
	@echo "  make dev-build      - 开发模式构建（不优化）"
	@echo "  make test           - 运行所有测试"
	@echo "  make test-tabby     - 运行 tabby 测试"
	@echo "  make test-index-cli - 运行 tabby-index-cli 测试"
	@echo "  make fmt            - 格式化所有代码"
	@echo "  make fmt-check      - 检查代码格式"
	@echo "  make clippy         - 运行 clippy 检查"
	@echo "  make clean          - 清理构建文件"
	@echo "  make update         - 更新依赖"
	@echo "  make install        - 安装依赖"
	@echo "  make run            - 运行 tabby 服务器"
	@echo "  make run-index-cli  - 运行 tabby-index-cli"
	@echo "  make ci             - 运行 CI 检查（格式、clippy、测试）"

fix:
	cargo machete --fix || true
	cargo +nightly fmt
	cargo clippy --fix --allow-dirty --allow-staged

fix-ui:
	pnpm lint:fix

update-ui:
	pnpm build
	rm -rf ee/tabby-webserver/ui && cp -R ee/tabby-ui/out ee/tabby-webserver/ui
	rm -rf ee/tabby-webserver/email_templates && cp -R ee/tabby-email/out ee/tabby-webserver/email_templates

update-db-schema:
	sqlite3 ee/tabby-db/schema.sqlite ".schema --indent" > ee/tabby-db/schema/schema.sql
	sqlite3 ee/tabby-db/schema.sqlite -init  ee/tabby-db/schema/sqlite-schema-visualize.sql "" > schema.dot
	dot -Tsvg schema.dot > ee/tabby-db/schema/schema.svg
	rm schema.dot

dev:
	tmuxinator start -p .tmuxinator/tabby.yml
		
bump-version:
	cargo ws version --force "*" --no-individual-tags --allow-branch "main"

bump-release-version:
	cargo ws version --allow-branch "r*" --no-individual-tags --force "*"

update-openapi-doc:
	curl http://localhost:8080/api-docs/openapi.json | jq '                                                       \
	delpaths([                                                                                                    \
		  ["paths", "/v1beta/chat/completions"],                                                                  \
		  ["paths", "/v1beta/search"],                                                                            \
		  ["paths", "/v1beta/server_setting"],                                                                    \
		  ["components", "schemas", "CompletionRequest", "properties", "prompt"],                                 \
		  ["components", "schemas", "CompletionRequest", "properties", "debug_options"],                          \
		  ["components", "schemas", "CompletionResponse", "properties", "debug_data"],                            \
		  ["components", "schemas", "DebugData"],                                                                 \
		  ["components", "schemas", "DebugOptions"],                                                              \
		  ["components", "schemas", "ServerSetting"]                                                              \
	  ])' | jq '.servers[0] |= { url: "https://playground.app.tabbyml.com", description: "Playground server" }'   \
			    > website/static/openapi.json

update-graphql-schema:
	cargo run --package tabby-schema --example update-schema --features=schema-language
