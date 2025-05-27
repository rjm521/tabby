#!/bin/bash

# Qwen2 聊天模型服务启动脚本
# 用途: 启动Qwen2-1.5B-Instruct聊天服务，方便进行自测

set -e

# 配置参数
MODEL="StarCoder-1B"
CHAT_MODEL="Qwen2-1.5B-Instruct"
DEVICE="cpu"
PORT="8080"
RUST_LOG="debug"
LOG_DIR="./logs"
LOG_FILE="$LOG_DIR/chat_service_$(date +%Y%m%d_%H%M%S).log"
PID_FILE="./chat_service.pid"

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

# 检查端口是否被占用
check_port() {
    if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
        print_message $RED "端口 $PORT 已被占用，请先停止相关服务或更改端口"
        exit 1
    fi
}

# 创建日志目录
create_log_dir() {
    if [ ! -d "$LOG_DIR" ]; then
        mkdir -p "$LOG_DIR"
        print_message $GREEN "创建日志目录: $LOG_DIR"
    fi
}

# 停止现有服务
stop_existing_service() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            print_message $YELLOW "停止现有服务 (PID: $pid)..."
            kill "$pid"
            sleep 2
            if kill -0 "$pid" 2>/dev/null; then
                print_message $RED "强制停止服务..."
                kill -9 "$pid"
            fi
        fi
        rm -f "$PID_FILE"
    fi
}

# 启动服务
start_service() {
    print_message $BLUE "启动Qwen2聊天服务..."
    print_message $BLUE "模型: $MODEL"
    print_message $BLUE "聊天模型: $CHAT_MODEL"
    print_message $BLUE "设备: $DEVICE"
    print_message $BLUE "端口: $PORT"
    print_message $BLUE "日志级别: $RUST_LOG"
    print_message $BLUE "日志文件: $LOG_FILE"

    # 启动服务（使用用户提供的完整启动命令）
    nohup env RUST_LOG=$RUST_LOG ./target/debug/tabby serve --model $MODEL --chat-model $CHAT_MODEL --device $DEVICE --port $PORT > "$LOG_FILE" 2>&1 &

    local pid=$!
    echo $pid > "$PID_FILE"

    print_message $GREEN "服务已启动 (PID: $pid)"
    print_message $GREEN "访问地址: http://localhost:$PORT"
    print_message $GREEN "日志文件: $LOG_FILE"
}

# 检查服务状态
check_service_status() {
    sleep 3
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            print_message $GREEN "✅ 服务运行正常 (PID: $pid)"
            print_message $BLUE "💡 查看日志: tail -f $LOG_FILE"
            print_message $BLUE "💡 停止服务: ./stop_chat_service.sh 或 kill $pid"
            return 0
        else
            print_message $RED "❌ 服务启动失败"
            if [ -f "$LOG_FILE" ]; then
                print_message $RED "错误日志:"
                tail -10 "$LOG_FILE"
            fi
            return 1
        fi
    else
        print_message $RED "❌ PID文件不存在，服务可能启动失败"
        return 1
    fi
}

# 显示帮助信息
show_help() {
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  --model MODEL         指定代码补全模型 (默认: $MODEL)"
    echo "  -m, --chat-model MODEL 指定聊天模型 (默认: $CHAT_MODEL)"
    echo "  -d, --device DEVICE   指定设备 (默认: $DEVICE)"
    echo "  -p, --port PORT       指定端口 (默认: $PORT)"
    echo "  --log-level LEVEL     指定日志级别 (默认: $RUST_LOG)"
    echo "  -h, --help           显示帮助信息"
    echo ""
    echo "示例:"
    echo "  $0                    # 使用默认参数启动"
    echo "  $0 -p 8081            # 使用端口8081启动"
    echo "  $0 -d gpu             # 使用GPU启动"
    echo "  $0 --model CodeLlama-7B --chat-model Qwen2-7B-Instruct  # 使用不同模型"
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --model)
            MODEL="$2"
            shift 2
            ;;
        -m|--chat-model)
            CHAT_MODEL="$2"
            shift 2
            ;;
        -d|--device)
            DEVICE="$2"
            shift 2
            ;;
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        --log-level)
            RUST_LOG="$2"
            shift 2
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
    print_message $BLUE "🚀 启动Qwen2聊天服务自测脚本"

    # 执行启动流程
    check_port
    create_log_dir
    stop_existing_service
    start_service

    # 检查服务状态
    if check_service_status; then
        print_message $GREEN "🎉 服务启动成功！可以开始自测了"
    else
        print_message $RED "💥 服务启动失败，请检查日志"
        exit 1
    fi
}

# 执行主函数
main