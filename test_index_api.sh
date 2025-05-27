#!/bin/bash

# Tabby 索引 API 测试脚本
# 测试所有新增的 code indexing API 接口

set -e

# 配置
BASE_URL="http://localhost:8080"
AUTH_TOKEN=""  # 如果需要认证，在这里设置token

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# HTTP请求函数
make_request() {
    local method=$1
    local endpoint=$2
    local data=$3
    local content_type=${4:-"application/json"}

    local curl_cmd="curl -s -X $method"

    if [ -n "$AUTH_TOKEN" ]; then
        curl_cmd="$curl_cmd -H 'Authorization: Bearer $AUTH_TOKEN'"
    fi

    curl_cmd="$curl_cmd -H 'Content-Type: $content_type'"

    if [ -n "$data" ]; then
        curl_cmd="$curl_cmd -d '$data'"
    fi

    curl_cmd="$curl_cmd $BASE_URL$endpoint"

    eval $curl_cmd
}

# 测试函数
test_endpoint() {
    local name=$1
    local method=$2
    local endpoint=$3
    local data=$4

    log_info "测试: $name"
    log_info "请求: $method $endpoint"

    local response=$(make_request "$method" "$endpoint" "$data")
    local status=$?

    if [ $status -eq 0 ]; then
        log_success "✅ $name - 请求成功"
        echo "响应: $response" | head -c 200
        echo "..."
        echo
    else
        log_error "❌ $name - 请求失败"
    fi
    echo "----------------------------------------"
}

# 主测试函数
main() {
    log_info "🚀 开始测试 Tabby 索引 API"
    echo "========================================="

    # 1. 基础索引操作测试
    log_info "📋 测试基础索引操作"

    test_endpoint "获取索引信息" "GET" "/v1/index/info"

    test_endpoint "获取文档列表" "GET" "/v1/index/documents/code"

    # 2. 搜索功能测试
    log_info "🔍 测试搜索功能"

    local search_data='{
        "query": "function main",
        "language": "rust",
        "limit": 5,
        "offset": 0
    }'
    test_endpoint "代码内容搜索" "POST" "/v1/index/search" "$search_data"

    test_endpoint "文件路径搜索" "GET" "/v1/index/search/files?q=main.rs&limit=5"

    test_endpoint "语义搜索" "POST" "/v1/index/search/semantic" "$search_data"

    # 3. 索引管理测试
    log_info "🗂️ 测试索引管理"

    test_endpoint "获取索引状态" "GET" "/v1/index/idx_abc123def456/status"

    # 注意：删除操作可能会影响现有索引，在生产环境中谨慎使用
    # test_endpoint "删除索引" "DELETE" "/v1/index/idx_abc123def456"

    # 4. 配置管理测试
    log_info "⚙️ 测试配置管理"

    test_endpoint "获取索引配置" "GET" "/v1/index/config"

    local config_data='{
        "max_file_size": 1048576,
        "include_patterns": ["**/*.rs", "**/*.py"],
        "exclude_patterns": ["target/**", "**/*.pyc"],
        "languages": ["rust", "python"]
    }'
    test_endpoint "验证索引配置" "POST" "/v1/index/config/validate" "$config_data"

    # 5. 智能分析测试
    log_info "🧠 测试智能分析"

    local analyze_data='{
        "content": "fn main() {\n    println!(\"Hello, world!\");\n}",
        "filepath": "src/main.rs",
        "language": "rust"
    }'
    test_endpoint "代码分析" "POST" "/v1/index/analyze" "$analyze_data"

    test_endpoint "获取索引建议" "GET" "/v1/index/suggestions"

    # 6. 批量操作测试
    log_info "📦 测试批量操作"

    local batch_data='{
        "sources": [
            {
                "source": "https://github.com/rust-lang/rust.git",
                "name": "rust-lang",
                "language": "rust",
                "max_file_size": 1024,
                "exclude": ["tests/**"],
                "include": ["src/**"]
            }
        ],
        "concurrency": 2
    }'
    # 注意：批量创建会实际创建索引，在测试环境中使用
    # test_endpoint "批量创建索引" "POST" "/v1/index/batch/create" "$batch_data"

    log_success "🎉 所有API测试完成！"
}

# 检查服务是否运行
check_service() {
    log_info "检查 Tabby 服务状态..."

    local health_response=$(make_request "GET" "/v1/health")
    local status=$?

    if [ $status -eq 0 ]; then
        log_success "✅ Tabby 服务正在运行"
        echo "健康状态: $health_response"
        echo
        return 0
    else
        log_error "❌ Tabby 服务未运行或无法访问"
        log_error "请确保服务在 $BASE_URL 上运行"
        return 1
    fi
}

# 显示帮助信息
show_help() {
    echo "Tabby 索引 API 测试脚本"
    echo
    echo "用法: $0 [选项]"
    echo
    echo "选项:"
    echo "  -h, --help          显示此帮助信息"
    echo "  -u, --url URL       设置 Tabby 服务 URL (默认: http://localhost:8080)"
    echo "  -t, --token TOKEN   设置认证 token"
    echo
    echo "示例:"
    echo "  $0                                    # 使用默认设置测试"
    echo "  $0 -u http://localhost:9090          # 指定服务URL"
    echo "  $0 -t your_auth_token                # 使用认证token"
    echo
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        -t|--token)
            AUTH_TOKEN="$2"
            shift 2
            ;;
        *)
            log_error "未知参数: $1"
            show_help
            exit 1
            ;;
    esac
done

# 执行测试
if check_service; then
    main
else
    exit 1
fi