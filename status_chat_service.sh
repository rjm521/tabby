#!/bin/bash

# Qwen2 聊天模型服务状态查看脚本
# 用途: 查看Qwen2聊天服务运行状态

set -e

# 配置参数
PID_FILE="./chat_service.pid"
PORT="8080"
LOG_DIR="./logs"

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
    echo -e "${color}$message${NC}"
}

# 检查服务状态
check_service_status() {
    print_message $BLUE "=== Qwen2 聊天服务状态检查 ==="
    echo

    local service_running=false
    local pid=""

    # 检查PID文件
    if [ -f "$PID_FILE" ]; then
        pid=$(cat "$PID_FILE")
        print_message $BLUE "📁 PID文件: $PID_FILE"
        print_message $BLUE "🆔 记录的PID: $pid"

        if kill -0 "$pid" 2>/dev/null; then
            print_message $GREEN "✅ 服务正在运行 (PID: $pid)"
            service_running=true
        else
            print_message $RED "❌ PID文件存在但进程不存在"
        fi
    else
        print_message $YELLOW "⚠️  PID文件不存在: $PID_FILE"
    fi

    echo

    # 检查端口占用
    local port_pid=$(lsof -ti :$PORT 2>/dev/null || true)
    if [ -n "$port_pid" ]; then
        print_message $GREEN "🌐 端口 $PORT 正在被使用 (PID: $port_pid)"
        if [ "$port_pid" == "$pid" ]; then
            print_message $GREEN "✅ 端口PID与记录PID一致"
        else
            print_message $YELLOW "⚠️  端口PID与记录PID不一致"
        fi
        service_running=true
    else
        print_message $RED "❌ 端口 $PORT 未被占用"
    fi

    echo

    # 显示进程信息
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        print_message $BLUE "📊 进程详细信息:"
        ps -p "$pid" -o pid,ppid,cmd,etime,pcpu,pmem 2>/dev/null || print_message $RED "无法获取进程信息"
        echo
    fi

    # 显示网络连接
    print_message $BLUE "🌐 网络连接信息:"
    netstat -tlnp 2>/dev/null | grep ":$PORT " || print_message $YELLOW "端口 $PORT 未监听"
    echo

    # 检查最新日志
    if [ -d "$LOG_DIR" ]; then
        local latest_log=$(ls -t "$LOG_DIR"/chat_service_*.log 2>/dev/null | head -1)
        if [ -n "$latest_log" ]; then
            print_message $BLUE "📋 最新日志文件: $latest_log"
            print_message $BLUE "📝 最近10行日志:"
            echo "----------------------------------------"
            tail -10 "$latest_log" 2>/dev/null || print_message $RED "无法读取日志文件"
            echo "----------------------------------------"
        else
            print_message $YELLOW "⚠️  没有找到日志文件"
        fi
    else
        print_message $YELLOW "⚠️  日志目录不存在: $LOG_DIR"
    fi

    echo

    # 总结状态
    if $service_running; then
        print_message $GREEN "🎉 总体状态: 服务正在运行"
        print_message $BLUE "💡 访问地址: http://localhost:$PORT"
        print_message $BLUE "💡 查看日志: tail -f $latest_log"
        print_message $BLUE "💡 停止服务: ./stop_chat_service.sh"
    else
        print_message $RED "💥 总体状态: 服务未运行"
        print_message $BLUE "💡 启动服务: ./start_chat_service.sh"
    fi
}

# 显示帮助信息
show_help() {
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -p, --port PORT       指定要检查的端口 (默认: $PORT)"
    echo "  -l, --logs            只显示日志"
    echo "  -h, --help           显示帮助信息"
    echo ""
    echo "示例:"
    echo "  $0                    # 检查默认端口的服务状态"
    echo "  $0 -p 8081            # 检查端口8081的服务状态"
    echo "  $0 -l                 # 只显示最新日志"
}

# 只显示日志
show_logs_only() {
    if [ -d "$LOG_DIR" ]; then
        local latest_log=$(ls -t "$LOG_DIR"/chat_service_*.log 2>/dev/null | head -1)
        if [ -n "$latest_log" ]; then
            print_message $BLUE "📋 最新日志文件: $latest_log"
            echo "========================================"
            tail -50 "$latest_log" 2>/dev/null || print_message $RED "无法读取日志文件"
            echo "========================================"
            print_message $BLUE "💡 实时查看: tail -f $latest_log"
        else
            print_message $YELLOW "⚠️  没有找到日志文件"
        fi
    else
        print_message $YELLOW "⚠️  日志目录不存在: $LOG_DIR"
    fi
}

# 解析命令行参数
LOGS_ONLY=false
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -l|--logs)
            LOGS_ONLY=true
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
    if $LOGS_ONLY; then
        show_logs_only
    else
        check_service_status
    fi
}

# 执行主函数
main