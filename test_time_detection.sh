#!/bin/bash

# 测试时间变化检测功能的脚本
# 模拟Claude Code卡住和正常工作的场景

echo "🧪 测试Claude Watch时间变化检测功能"
echo "=================================="

# 创建测试用的tmux session和pane
echo "📋 创建测试环境..."
tmux new-session -d -s claude-test
tmux send-keys -t claude-test "echo 'Claude Code Test Session'" Enter

# 等待一下
sleep 2

# 获取pane ID
PANE_ID=$(tmux list-panes -t claude-test -F "#{pane_id}")
echo "🔍 测试Pane ID: $PANE_ID"

# 场景1: 模拟Claude Code正常工作（时间递增）
echo ""
echo "🔄 场景1: 模拟Claude Code正常工作"
echo "----------------------------------"

# 模拟时间递增的输出
for i in {100..105}; do
    echo "* Herding… (${i}s · ↑ 8.7k tokens · esc to interrupt)"
    tmux send-keys -t claude-test "echo \"* Herding… (${i}s · ↑ 8.7k tokens · esc to interrupt)\"" Enter
    sleep 1
    
    # 运行claude-watch检测
    echo "🔍 运行检测 (时间: ${i}s)..."
    timeout 10s ./target/release/claude-watch --pane "$PANE_ID" --stuck-sec 3 --interval 1 &
    WATCH_PID=$!
    sleep 3
    kill $WATCH_PID 2>/dev/null || true
done

# 场景2: 模拟Claude Code卡住（时间不变化）
echo ""
echo "⏸️ 场景2: 模拟Claude Code卡住"
echo "----------------------------------"

# 模拟时间不变化的输出
for i in {1..5}; do
    echo "* Herding… (100s · ↑ 8.7k tokens · esc to interrupt) - 卡住中"
    tmux send-keys -t claude-test "echo \"* Herding… (100s · ↑ 8.7k tokens · esc to interrupt) - 卡住中\"" Enter
    sleep 1
    
    # 运行claude-watch检测
    echo "🔍 运行检测 (时间不变: 100s)..."
    timeout 15s ./target/release/claude-watch --pane "$PANE_ID" --stuck-sec 3 --interval 1 &
    WATCH_PID=$!
    sleep 5
    kill $WATCH_PID 2>/dev/null || true
done

# 场景3: 模拟复杂的真实场景
echo ""
echo "🌐 场景3: 模拟复杂的真实场景"
echo "----------------------------------"

echo "开始任务..."
tmux send-keys -t claude-test "echo '开始处理复杂任务...'" Enter
sleep 2

# 工作阶段（时间递增）
for i in {150..155}; do
    echo "Tool use: Reading file (${i}s · ↓ 5.2k tokens · esc to interrupt)"
    tmux send-keys -t claude-test "echo \"Tool use: Reading file (${i}s · ↓ 5.2k tokens · esc to interrupt)\"" Enter
    sleep 1
done

# 短暂停顿（时间不变）
echo "深度思考中..."
for i in {1..3}; do
    echo "* Cogitating… (155s · ↓ 6.1k tokens · esc to interrupt)"
    tmux send-keys -t claude-test "echo \"* Cogitating… (155s · ↓ 6.1k tokens · esc to interrupt)\"" Enter
    sleep 1
done

# 继续工作（时间递增）
for i in {156..160}; do
    echo "* Generating code (${i}s · ↑ 9.3k tokens · esc to interrupt)"
    tmux send-keys -t claude-test "echo \"* Generating code (${i}s · ↑ 9.3k tokens · esc to interrupt)\"" Enter
    sleep 1
done

# 卡住状态（时间不变）
echo "可能卡住了..."
for i in {1..4}; do
    echo "* Generating code (160s · ↑ 9.3k tokens · esc to interrupt)"
    tmux send-keys -t claude-test "echo \"* Generating code (160s · ↑ 9.3k tokens · esc to interrupt)\"" Enter
    sleep 1
done

# 运行claude-watch观察复杂场景
echo ""
echo "🔍 观察复杂场景..."
timeout 30s ./target/release/claude-watch --pane "$PANE_ID" --stuck-sec 5 --interval 2 &
WATCH_PID=$!
sleep 20
kill $WATCH_PID 2>/dev/null || true

# 清理测试环境
echo ""
echo "🧹 清理测试环境..."
tmux kill-session -t claude-test

echo ""
echo "✅ 测试完成！"
echo ""
echo "📊 测试结果分析："
echo "1. 场景1 (时间递增): 应该识别为活动状态，不会触发卡住检测"
echo "2. 场景2 (时间不变): 应该触发卡住检测和恢复机制"
echo "3. 场景3 (复杂场景): 应该正确处理工作、暂停、卡住的各种状态"
echo ""
echo "💡 如果看到很多'检测到时间在递增'的消息，说明时间变化检测功能正常工作"
echo "💡 如果看到'调用LLM判断状态'的消息，说明系统正确识别了卡住状态"