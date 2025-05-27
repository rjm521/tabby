#!/bin/bash

# Qwen2 聊天模型服务停止脚本
# 用途: 停止Qwen2聊天服务

set -e

# 配置参数
PID_FILE="./chat_service.pid"
PORT="8080"

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

# 停止服务
stop_service() {
    local stopped=false

    # 通过PID文件停止
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            print_message $YELLOW "正在停止服务 (PID: $pid)..."
            kill "$pid"

            # 等待服务停止
            local count=0
            while kill -0 "$pid" 2>/dev/null && [ $count -lt 10 ]; do
                sleep 1
                count=$((count + 1))
                print_message $BLUE "等待服务停止... ($count/10)"
            done

            if kill -0 "$pid" 2>/dev/null; then
                print_message $RED "服务未能正常停止，强制终止..."
                kill -9 "$pid"
                sleep 1
            fi

            if ! kill -0 "$pid" 2>/dev/null; then
                print_message $GREEN "✅ 服务已停止 (PID: $pid)"
                stopped=true
            fi
        else
            print_message $YELLOW "PID文件存在但进程不存在，清理PID文件"
        fi
        rm -f "$PID_FILE"
    fi

    # 通过端口查找并停止服务
    if ! $stopped; then
        local port_pid=$(lsof -ti :$PORT 2>/dev/null || true)
        if [ -n "$port_pid" ]; then
            print_message $YELLOW "发现端口 $PORT 上的服务 (PID: $port_pid)，正在停止..."
            kill "$port_pid" 2>/dev/null || true
            sleep 2

            # 检查是否还在运行
            if kill -0 "$port_pid" 2>/dev/null; then
                print_message $RED "强制停止端口 $PORT 上的服务..."
                kill -9 "$port_pid" 2>/dev/null || true
            fi

            if ! kill -0 "$port_pid" 2>/dev/null; then
                print_message $GREEN "✅ 端口 $PORT 上的服务已停止"
                stopped=true
            fi
        fi
    fi

    if ! $stopped; then
        print_message $BLUE "ℹ️  没有发现正在运行的聊天服务"
    fi
}

# 清理临时文件
cleanup() {
    print_message $BLUE "清理临时文件..."

    # 清理PID文件
    if [ -f "$PID_FILE" ]; then
        rm -f "$PID_FILE"
        print_message $GREEN "清理PID文件: $PID_FILE"
    fi

    # 可以在这里添加其他清理逻辑
    print_message $GREEN "✅ 清理完成"
}

# 显示帮助信息
show_help() {
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -p, --port PORT       指定要停止的服务端口 (默认: $PORT)"
    echo "  -c, --cleanup         停止服务后进行清理"
    echo "  -h, --help           显示帮助信息"
    echo ""
    echo "示例:"
    echo "  $0                    # 停止默认端口的服务"
    echo "  $0 -p 8081            # 停止端口8081的服务"
    echo "  $0 -c                 # 停止服务并清理"
}

# 解析命令行参数
CLEANUP=false
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -c|--cleanup)
            CLEANUP=true
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
    print_message $BLUE "🛑 停止Qwen2聊天服务"

    stop_service

    if $CLEANUP; then
        cleanup
    fi

    print_message $GREEN "🎉 停止操作完成"
}

# 执行主函数
main