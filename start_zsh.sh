#!/bin/bash

# 快速启动 zsh 和 Oh My Zsh 的脚本
# 使用方法: ./start_zsh.sh 或 source start_zsh.sh

echo "🚀 启动 zsh 和 Oh My Zsh..."

# 检查 zsh 是否安装
if ! command -v zsh &> /dev/null; then
    echo "❌ zsh 未安装，请先安装 zsh"
    exit 1
fi

# 检查 Oh My Zsh 是否安装
if [ ! -d "$HOME/.oh-my-zsh" ]; then
    echo "❌ Oh My Zsh 未安装，请先安装 Oh My Zsh"
    exit 1
fi

# 启动 zsh
echo "✅ 启动 zsh..."
exec zsh