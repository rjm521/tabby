#!/bin/bash

# 测试用例生成API测试脚本
# 用途: 测试测试用例生成API的功能和性能

set -e

# 配置参数
PORT="8081"
HOST="localhost"
API_ENDPOINT="http://$HOST:$PORT/v1/test/generate"
TEST_TOKEN="test_token" # 实际使用时需要替换为有效的JWT token

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}[$(date '+%Y-%m-%d %H:%M:%S')] $message${NC}"
}

# 测试服务连通性
test_connectivity() {
    print_message $BLUE "🔍 测试服务连通性..."

    # 使用curl检查端口是否可访问
    local http_code=$(curl -s -o /dev/null -w "%{http_code}" "$API_ENDPOINT" -X POST -H "Content-Type: application/json" -d '{}' 2>/dev/null || echo "000")

    if [ "$http_code" == "000" ]; then
        print_message $RED "❌ 端口 $PORT 不可访问"
        return 1
    else
        print_message $GREEN "✅ 端口 $PORT 可访问 (状态码: $http_code)"
    fi

    return 0
}

# 测试API功能
test_api_functionality() {
    print_message $BLUE "🧪 测试API功能..."

    # 测试用例1: 基本功能测试
    local test_case1=$(cat <<EOF
{
    "api_desc": "GET /api/users/{id} - 获取用户信息接口\n参数:\n- id: 用户ID (必填, 整数)\n返回:\n- 200: 成功返回用户信息\n- 404: 用户不存在\n- 500: 服务器错误"
}
EOF
)

    print_message $BLUE "📤 发送测试用例1: 基本功能测试"
    local response1=$(curl -s -X POST "$API_ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TEST_TOKEN" \
        -d "$test_case1" 2>/dev/null || echo "")

    if [ -z "$response1" ]; then
        print_message $RED "❌ API无响应"
        return 1
    fi

    # 检查响应是否包含错误
    if echo "$response1" | grep -q '"error"'; then
        print_message $RED "❌ API返回错误:"
        echo "$response1" | python3 -m json.tool 2>/dev/null || echo "$response1"
        return 1
    fi

    # 检查响应是否包含测试用例
    if echo "$response1" | grep -q '"test_case"'; then
        print_message $GREEN "✅ API响应正常"
        print_message $BLUE "📥 生成的测试用例:"
        echo "$response1" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if 'test_case' in data:
        print('    ' + data['test_case'].replace('\n', '\n    '))
    else:
        print('    (无法解析测试用例)')
except:
    print('    (响应格式异常)')
" 2>/dev/null || echo "    (无法解析响应)"
    else
        print_message $RED "❌ API响应格式异常"
        echo "$response1"
        return 1
    fi

    # 测试用例2: 边界条件测试
    local test_case2=$(cat <<EOF
{
    "api_desc": "POST /api/orders - 创建订单接口\n参数:\n- user_id: 用户ID (必填, 整数)\n- items: 商品列表 (必填, 数组)\n  - item_id: 商品ID (必填, 整数)\n  - quantity: 数量 (必填, 整数, 最小值1)\n返回:\n- 201: 成功创建订单\n- 400: 参数错误\n- 404: 用户或商品不存在\n- 500: 服务器错误"
}
EOF
)

    print_message $BLUE "📤 发送测试用例2: 边界条件测试"
    local response2=$(curl -s -X POST "$API_ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TEST_TOKEN" \
        -d "$test_case2" 2>/dev/null || echo "")

    if [ -z "$response2" ]; then
        print_message $RED "❌ API无响应"
        return 1
    fi

    # 检查响应是否包含错误
    if echo "$response2" | grep -q '"error"'; then
        print_message $RED "❌ API返回错误:"
        echo "$response2" | python3 -m json.tool 2>/dev/null || echo "$response2"
        return 1
    fi

    # 检查响应是否包含测试用例
    if echo "$response2" | grep -q '"test_case"'; then
        print_message $GREEN "✅ API响应正常"
        print_message $BLUE "📥 生成的测试用例:"
        echo "$response2" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if 'test_case' in data:
        print('    ' + data['test_case'].replace('\n', '\n    '))
    else:
        print('    (无法解析测试用例)')
except:
    print('    (响应格式异常)')
" 2>/dev/null || echo "    (无法解析响应)"
    else
        print_message $RED "❌ API响应格式异常"
        echo "$response2"
        return 1
    fi

    return 0
}

# 性能测试
test_performance() {
    print_message $BLUE "⚡ 性能测试..."

    local test_case=$(cat <<EOF
{
    "api_desc": "GET /api/products - 获取商品列表接口\n参数:\n- page: 页码 (选填, 整数, 默认1)\n- size: 每页数量 (选填, 整数, 默认20)\n- category: 商品分类 (选填, 字符串)\n返回:\n- 200: 成功返回商品列表\n- 400: 参数错误\n- 500: 服务器错误"
}
EOF
)

    # 测试响应时间
    local start_time=$(date +%s%N)
    local response=$(curl -s -X POST "$API_ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TEST_TOKEN" \
        -d "$test_case" 2>/dev/null || echo "")
    local end_time=$(date +%s%N)
    local duration=$((($end_time - $start_time) / 1000000)) # 转换为毫秒

    if [ -z "$response" ]; then
        print_message $RED "❌ 性能测试失败: API无响应"
        return 1
    fi

    print_message $GREEN "✅ 性能测试完成"
    print_message $BLUE "⏱️  响应时间: ${duration}ms"

    # 测试并发性能
    print_message $BLUE "🔄 并发测试 (5个并发请求)..."
    local start_time=$(date +%s%N)
    for i in {1..5}; do
        curl -s -X POST "$API_ENDPOINT" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $TEST_TOKEN" \
            -d "$test_case" > /dev/null 2>&1 &
    done
    wait
    local end_time=$(date +%s%N)
    local duration=$((($end_time - $start_time) / 1000000)) # 转换为毫秒

    print_message $GREEN "✅ 并发测试完成"
    print_message $BLUE "⏱️  总响应时间: ${duration}ms"
    print_message $BLUE "⏱️  平均响应时间: $(($duration / 5))ms"

    return 0
}

# 错误处理测试
test_error_handling() {
    print_message $BLUE "🚨 错误处理测试..."

    # 测试1: 缺少认证token
    print_message $BLUE "📤 测试1: 缺少认证token"
    local response1=$(curl -s -X POST "$API_ENDPOINT" \
        -H "Content-Type: application/json" \
        -d "{}" 2>/dev/null || echo "")

    if [ -z "$response1" ]; then
        print_message $RED "❌ 测试1失败: API无响应"
    elif echo "$response1" | grep -q "401"; then
        print_message $GREEN "✅ 测试1通过: 正确返回401未认证错误"
    else
        print_message $RED "❌ 测试1失败: 未返回预期的401错误"
        echo "$response1"
    fi

    # 测试2: 无效的请求体
    print_message $BLUE "📤 测试2: 无效的请求体"
    local response2=$(curl -s -X POST "$API_ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TEST_TOKEN" \
        -d "invalid json" 2>/dev/null || echo "")

    if [ -z "$response2" ]; then
        print_message $RED "❌ 测试2失败: API无响应"
    elif echo "$response2" | grep -q "400"; then
        print_message $GREEN "✅ 测试2通过: 正确返回400错误请求错误"
    else
        print_message $RED "❌ 测试2失败: 未返回预期的400错误"
        echo "$response2"
    fi

    # 测试3: 空的API描述
    print_message $BLUE "📤 测试3: 空的API描述"
    local response3=$(curl -s -X POST "$API_ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TEST_TOKEN" \
        -d '{"api_desc": ""}' 2>/dev/null || echo "")

    if [ -z "$response3" ]; then
        print_message $RED "❌ 测试3失败: API无响应"
    elif echo "$response3" | grep -q "400"; then
        print_message $GREEN "✅ 测试3通过: 正确返回400错误请求错误"
    else
        print_message $RED "❌ 测试3失败: 未返回预期的400错误"
        echo "$response3"
    fi

    return 0
}

# 主函数
main() {
    print_message $BLUE "🚀 开始测试用例生成API测试"

    # 执行测试
    if ! test_connectivity; then
        print_message $RED "❌ 连通性测试失败"
        exit 1
    fi

    if ! test_api_functionality; then
        print_message $RED "❌ 功能测试失败"
        exit 1
    fi

    if ! test_performance; then
        print_message $RED "❌ 性能测试失败"
        exit 1
    fi

    if ! test_error_handling; then
        print_message $RED "❌ 错误处理测试失败"
        exit 1
    fi

    print_message $GREEN "🎉 所有测试完成"
}

# 执行主函数
main