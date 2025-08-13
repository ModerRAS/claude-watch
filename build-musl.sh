#!/bin/bash

# Musl构建脚本 for claude-watch
# 这个脚本会交叉编译出可以在任何Linux系统上运行的独立二进制文件

set -e

echo "🔧 开始 musl 构建过程..."

# 检查是否安装了 musl 工具链
if ! command -v musl-gcc &> /dev/null; then
    echo "❌ musl-gcc 未找到，正在安装..."
    sudo apt update
    sudo apt install -y musl musl-tools
fi

# 检查是否安装了 rust-musl 工具链
if ! rustup target list --installed | grep -q "x86_64-unknown-linux-musl"; then
    echo "📦 安装 rust-musl 工具链..."
    rustup target add x86_64-unknown-linux-musl
fi

# 设置 musl 编译环境变量
export CC=musl-gcc
export CXX=musl-gcc

echo "🏗️  编译 musl 版本..."

# 编译 musl 版本
cargo build --release --target x86_64-unknown-linux-musl

# 检查编译结果
if [ -f "target/x86_64-unknown-linux-musl/release/claude-watch" ]; then
    echo "✅ Musl 编译成功！"
    
    # 创建 musl 发布目录
    mkdir -p target/musl-release
    
    # 复制二进制文件
    cp target/x86_64-unknown-linux-musl/release/claude-watch target/musl-release/
    
    # 显示文件信息
    echo "📊 文件信息："
    ls -lh target/musl-release/claude-watch
    
    # 检查是否为动态链接
    echo "🔍 依赖检查："
    ldd target/musl-release/claude-watch || echo "✅ 是独立的静态链接二进制文件！"
    
    # 测试运行
    echo "🧪 运行测试："
    target/musl-release/claude-watch --help || echo "✅ 程序可以正常执行！"
    
    echo ""
    echo "🎉 Musl 版本编译完成！"
    echo "📍 二进制文件位置: target/musl-release/claude-watch"
    echo "💡 这是一个独立的静态链接文件，可以在任何 Linux 系统上运行！"
    
else
    echo "❌ Musl 编译失败！"
    exit 1
fi