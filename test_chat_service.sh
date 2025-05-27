#!/bin/bash

# Qwen2 聊天模型服务测试脚本
# 用途: 测试聊天服务是否正常工作

set -e

# 配置参数
PORT="8080"
HOST="localhost"

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

    local url="http://$HOST:$PORT"

    # 检查端口是否开放
    if ! nc -z "$HOST" "$PORT" 2>/dev/null; then
        print_message $RED "❌ 端口 $PORT 不可访问"
        return 1
    fi

    print_message $GREEN "✅ 端口 $PORT 可访问"

    # 检查HTTP响应
    local http_code=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")

    if [ "$http_code" == "000" ]; then
        print_message $RED "❌ HTTP连接失败"
        return 1
    elif [ "$http_code" == "200" ]; then
        print_message $GREEN "✅ HTTP连接正常 (状态码: $http_code)"
    else
        print_message $YELLOW "⚠️  HTTP连接异常 (状态码: $http_code)"
    fi

    return 0
}

# 测试聊天API
test_chat_api() {
    print_message $BLUE "💬 测试聊天API..."

    local url="http://$HOST:$PORT/v1/chat/completions"
    local test_message="你好，请简单介绍一下你自己。"

    # 构建请求数据
    local request_data=$(cat <<EOF
{
    "model": "Qwen2-1.5B-Instruct",
    "messages": [
        {
            "role": "user",
            "content": "$test_message"
        }
    ],
    "max_tokens": 100,
    "temperature": 0.7
}
EOF
)

    print_message $BLUE "📤 发送测试消息: $test_message"

    # 发送请求并检查响应
    local response=$(curl -s -X POST "$url" \
        -H "Content-Type: application/json" \
        -d "$request_data" 2>/dev/null || echo "")

    if [ -z "$response" ]; then
        print_message $RED "❌ 聊天API无响应"
        return 1
    fi

    # 检查响应是否包含错误
    if echo "$response" | grep -q '"error"'; then
        print_message $RED "❌ 聊天API返回错误:"
        echo "$response" | python3 -m json.tool 2>/dev/null || echo "$response"
        return 1
    fi

    # 检查响应是否包含正常的聊天回复
    if echo "$response" | grep -q '"choices"'; then
        print_message $GREEN "✅ 聊天API响应正常"
        print_message $BLUE "📥 AI回复内容:"
        echo "$response" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if 'choices' in data and len(data['choices']) > 0:
        content = data['choices'][0]['message']['content']
        print('    ' + content.replace('\n', '\n    '))
    else:
        print('    (无法解析回复内容)')
except:
    print('    (响应格式异常)')
" 2>/dev/null || echo "    (无法解析响应)"
        return 0
    else
        print_message $RED "❌ 聊天API响应格式异常"
        echo "$response"
        return 1
    fi
}

# 性能测试
test_performance() {
    print_message $BLUE "⚡ 性能测试..."

    local url="http://$HOST:$PORT/v1/chat/completions"
    local test_message="请计算1+1等于几？"

    local request_data=$(cat <<EOF
{
    "model": "Qwen2-1.5B-Instruct",
    "messages": [
        {
            "role": "user",
            "content": "$test_message"
        }
    ],
    "max_tokens": 50,
    "temperature": 0.1
}
EOF
)

    print_message $BLUE "📊 测试响应时间..."

    local start_time=$(date +%s.%N)
    local response=$(curl -s -X POST "$url" \
        -H "Content-Type: application/json" \
        -d "$request_data" 2>/dev/null || echo "")
    local end_time=$(date +%s.%N)

    if [ -n "$response" ] && echo "$response" | grep -q '"choices"'; then
        local duration=$(echo "$end_time - $start_time" | bc 2>/dev/null || echo "N/A")
        print_message $GREEN "✅ 响应时间: ${duration}秒"

        if [ "$duration" != "N/A" ]; then
            local duration_ms=$(echo "$duration * 1000" | bc 2>/dev/null || echo "N/A")
            if [ "$duration_ms" != "N/A" ]; then
                print_message $BLUE "📈 响应时间: ${duration_ms}毫秒"
            fi
        fi
    else
        print_message $RED "❌ 性能测试失败"
        return 1
    fi

    return 0
}

# 显示帮助信息
show_help() {
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -p, --port PORT       指定服务端口 (默认: $PORT)"
    echo "  -H, --host HOST       指定服务主机 (默认: $HOST)"
    echo "  --connectivity-only   仅测试连通性"
    echo "  --api-only           仅测试聊天API"
    echo "  --performance-only   仅测试性能"
    echo "  -h, --help           显示帮助信息"
    echo ""
    echo "示例:"
    echo "  $0                    # 完整测试"
    echo "  $0 -p 8081            # 测试端口8081的服务"
    echo "  $0 --connectivity-only # 仅测试连通性"
}

# 解析命令行参数
CONNECTIVITY_ONLY=false
API_ONLY=false
PERFORMANCE_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -H|--host)
            HOST="$2"
            shift 2
            ;;
        --connectivity-only)
            CONNECTIVITY_ONLY=true
            shift
            ;;
        --api-only)
            API_ONLY=true
            shift
            ;;
        --performance-only)
            PERFORMANCE_ONLY=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            print_message $RED "未知参数: $1"
            show_help
            exit 1
            ;;
    esac
done

# 主函数
main() {
    print_message $BLUE "🧪 开始测试Qwen2聊天服务"
    print_message $BLUE "🌐 目标服务: http://$HOST:$PORT"
    echo

    local all_tests_passed=true

    # 连通性测试
    if ! test_connectivity; then
        all_tests_passed=false
        if $CONNECTIVITY_ONLY; then
            exit 1
        fi
    fi

    if $CONNECTIVITY_ONLY; then
        exit 0
    fi

    echo

    # API测试
    if ! $PERFORMANCE_ONLY; then
        if ! test_chat_api; then
            all_tests_passed=false
            if $API_ONLY; then
                exit 1
            fi
        fi
    fi

    if $API_ONLY; then
        exit 0
    fi

    echo

    # 性能测试
    if ! $API_ONLY; then
        if ! test_performance; then
            all_tests_passed=false
        fi
    fi

    echo

    # 总结
    if $all_tests_passed; then
        print_message $GREEN "🎉 所有测试通过！聊天服务工作正常"
    else
        print_message $RED "💥 部分测试失败，请检查服务状态"
        exit 1
    fi
}

# 执行主函数
main