# Claude Code 新状态标识适配方案

## 问题分析

Claude Code 更新了工作状态标识，现在主要显示 `(esc to interrupt)` 格式，而现有代码可能对这种新格式的支持不够完善。

## 当前检测逻辑

1. **activity.rs**: 使用 `is_claude_active()` 函数检测活动状态
2. **monitor.rs**: 使用 `check_if_should_skip_llm_call()` 函数进行跳过LLM调用的判断
3. **正则表达式**: `r"[\*✶✢·✻✽][^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)"`

## 适配策略

### 1. 向后兼容性设计
- 保持对旧格式的支持
- 增加对新格式的支持
- 优先使用新格式检测，回退到旧格式

### 2. 新格式特点
- 简化的执行条格式
- 可能只显示 `(esc to interrupt)` 而不是完整的时间信息
- 可能简化了状态指示器

### 3. 实现方案
- 更新正则表达式，使其更宽松
- 增加对新格式的特殊处理
- 优化时间提取逻辑
- 增强状态判断的鲁棒性

## 具体实现

### 1. 正则表达式优化
```rust
// 旧的正则表达式（过于严格）
r"[\*✶✢·✻✽][^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)"

// 新的正则表达式（更宽松）
r"[\*✶✢·✻✽][^)]*\([^)]*(?:esc to interrupt|tokens|Processing|Cogitating|Thinking)[^)]*\)"
```

### 2. 时间提取优化
```rust
// 支持更多格式的时间提取
fn extract_execution_time(text: &str) -> Option<u64> {
    // 1. 尝试提取标准格式 (123s)
    let time_pattern = regex::Regex::new(r"\((\d+)s[^)]*\)").unwrap();
    if let Some(caps) = time_pattern.captures(text) {
        if let Some(time_str) = caps.get(1) {
            return time_str.as_str().parse::<u64>().ok();
        }
    }
    
    // 2. 尝试提取简化格式
    let simple_pattern = regex::Regex::new(r"(\d+)s").unwrap();
    if let Some(caps) = simple_pattern.captures(text) {
        if let Some(time_str) = caps.get(1) {
            return time_str.as_str().parse::<u64>().ok();
        }
    }
    
    None
}
```

### 3. 活动状态检测优化
```rust
pub fn is_claude_active(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    
    for line in lines.iter() {
        let trimmed = line.trim();
        
        // 1. 检查新格式：只有 (esc to interrupt)
        if trimmed.contains("(esc to interrupt)") {
            return true;
        }
        
        // 2. 检查简化格式：没有tokens但有esc to interrupt
        if trimmed.contains("esc to interrupt") && !trimmed.contains("Done") {
            return true;
        }
        
        // 3. 保持原有的检测逻辑（向后兼容）
        if trimmed.contains('(') && trimmed.contains(')') && trimmed.contains("tokens") {
            if trimmed.contains("Done") {
                return false;
            }
            
            let time_pattern = regex::Regex::new(r"\b\d+s\b").unwrap();
            if time_pattern.is_match(trimmed) {
                return true;
            }
        }
    }
    
    // 保持原有的其他检测逻辑
    // ...
}
```

## 风险评估

### 低风险
- 向后兼容性设计
- 渐进式部署
- 现有测试用例仍然有效

### 中等风险
- 新格式可能有未预料的变化
- 需要充分测试各种边界情况

### 高风险
- 如果新格式完全不同，可能需要重新设计

## 测试计划

### 1. 单元测试
- 测试正则表达式匹配
- 测试时间提取功能
- 测试活动状态检测

### 2. 集成测试
- 测试完整的监控流程
- 测试LLM调用逻辑
- 测试错误处理

### 3. 真实环境测试
- 在实际的Claude Code环境中测试
- 验证各种状态切换
- 确认功能正常工作

## 部署计划

### 1. 开发阶段
- 实现代码变更
- 编写测试用例
- 验证功能正确性

### 2. 测试阶段
- 运行完整的测试套件
- 在测试环境中验证
- 修复发现的问题

### 3. 生产部署
- 逐步推出新版本
- 监控系统表现
- 准备回滚方案

## 监控指标

### 1. 功能指标
- 状态检测准确率
- 响应时间
- 错误率

### 2. 性能指标
- CPU使用率
- 内存使用量
- 网络延迟

### 3. 用户体验指标
- 用户反馈
- 系统稳定性
- 功能完整性

## 总结

这个适配方案采用向后兼容的策略，通过优化正则表达式和检测逻辑来支持Claude Code的新状态标识格式。方案风险较低，具有较好的可扩展性和维护性。