# Claude Code 新格式适配需求分析文档

## 问题概述

Claude Code 更新了工作状态标识，现在只有"(esc to interrupt)"这些内容了，需要适配这种新格式。

## 1. 当前状态检测逻辑分析

### 1.1 核心检测文件

#### `src/activity.rs` - 活动状态检测
**主要功能**：
- 检测Claude Code是否处于活动状态
- 使用正则表达式匹配标准执行条格式
- 时间格式检测（数字+s）
- tokens计数检测

**关键代码片段**：
```rust
// 检查Claude Code的标准读秒格式：*(状态)… (时间 · 数量 tokens · esc to interrupt)
// 例如：* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)
// 或：✶ Perusing… (28s · ⚒ 414 tokens · esc to interrupt)
if trimmed.contains('(') && trimmed.contains(')') && trimmed.contains("tokens") {
    // 特殊处理：如果是Done状态，不认为是活动状态
    if trimmed.contains("Done") {
        return false; // Done状态不是活动状态
    }
    
    // 检查是否有时间格式（数字+s）在括号内
    let time_pattern = regex::Regex::new(r"\b\d+s\b").unwrap();
    if time_pattern.is_match(trimmed) {
        // 这是标准的Claude Code读秒状态，肯定是活动状态
        return true;
    }
}
```

#### `src/monitor.rs` - 监控逻辑
**主要功能**：
- 执行条格式匹配
- 时间提取和递增检测
- 跳过LLM调用判断

**关键代码片段**：
```rust
// 执行条格式匹配 - 支持多种Unicode符号
let execution_bar_pattern = regex::Regex::new(r"[\*✶✢·✻✽][^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)").unwrap();

// 时间提取
pub fn extract_execution_time(text: &str) -> Option<u64> {
    // 匹配格式：(数字s) - 更宽松的模式，能从复杂格式中提取
    let time_pattern = regex::Regex::new(r"\((\d+)s[^)]*\)").unwrap();
    // ...
}
```

### 1.2 当前正则表达式模式

#### 执行条格式匹配
```rust
r"[\*✶✢·✻✽][^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)"
```

#### 时间提取模式
```rust
r"\((\d+)s[^)]*\)"  // 提取时间
r"\b\d+s\b"         // 检测时间格式
```

#### 内容变化检测
```rust
r"(\d+)s"           // 时间数字变化
r"(\d+)\s*tokens?"  // token计数变化
```

### 1.3 支持的状态关键词

#### 活动状态关键词
```rust
"Cogitating", "Herding", "Meandering", "Reticulating", "Thinking", "Philosophising",
"Processing", "Compiling", "Building", "Executing",
"Reading", "Writing", "Generating", "Creating", "Analyzing",
"Calling", "Searching", "Browsing", "Loading", "Saving"
```

#### 状态指示符号
```rust
"*", "✶", "✢", "·", "✻", "✽"  // 执行条前缀
"↑", "↓", "⚒"                  // token指示符
```

## 2. 新格式特点分析

### 2.1 当前观察到的格式

根据实际捕获的数据，Claude Code的执行条格式为：
```
* Combobulating… (92s · ⚒ 47 tokens · esc to interrupt)
✶ Combobulating… (131s · ⚒ 47 tokens · esc to interrupt)
✻ Perusing… (14s · ⚒ 258 tokens · esc to interrupt)
* Philosophising… (500s · ↓ 7 tokens · esc to interrupt)
```

### 2.2 新格式的核心特征

1. **固定结尾**：所有执行条都以 `(esc to interrupt)` 结尾
2. **时间格式**：`(数字s)` 格式，如 `(92s)`, `(131s)`
3. **token计数**：包含 `tokens` 关键词和数量
4. **状态符号**：使用各种Unicode符号作为前缀
5. **分隔符**：使用 `·` 作为分隔符

### 2.3 与旧格式的对比

| 特征 | 旧格式 | 新格式 |
|------|--------|--------|
| 结尾标识 | `(esc to interrupt)` | `(esc to interrupt)` |
| 时间格式 | `(数字s)` | `(数字s)` |
| token计数 | `数字 tokens` | `数字 tokens` |
| 状态词汇 | 英文状态词 | 英文状态词 |
| 分隔符 | `·` | `·` |
| 前缀符号 | `*`, `✶`, `✢`, `·`, `✻`, `✽` | `*`, `✶`, `✢`, `·`, `✻`, `✽` |

**结论**：从代码分析来看，当前系统已经正确支持新格式，问题可能在于：

1. **正则表达式过于严格**：当前正则表达式要求 `tokens` 和 `esc to interrupt` 之间必须有其他内容
2. **边界情况处理**：可能存在某些边界情况未被覆盖
3. **格式变化**：Claude Code可能在某些情况下使用了略微不同的格式

## 3. 需要适配的检测逻辑

### 3.1 正则表达式适配

#### 当前问题
```rust
r"[\*✶✢·✻✽][^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)"
```

这个正则表达式的问题：
- 要求 `tokens` 和 `esc to interrupt` 之间必须有内容 `[^)]*`
- 如果格式变为 `(数字s · esc to interrupt)` 则会匹配失败

#### 适配方案
```rust
// 更宽松的匹配模式
r"[\*✶✢·✻✽][^)]*\([^)]*\d+s[^)]*esc to interrupt\)"

// 或者更精确的匹配
r"[\*✶✢·✻✽][^)]*\([^)]*\d+s[^)]*(?:tokens[^)]*)?esc to interrupt\)"
```

### 3.2 时间提取逻辑适配

#### 当前逻辑
```rust
r"\((\d+)s[^)]*\)"  // 当前时间提取
```

#### 可能的问题
- 如果格式变为 `(数字s)` 而没有其他内容，可能匹配失败

#### 适配方案
```rust
r"\((\d+)s(?:[^)]*)?\)"  // 更宽松的时间提取
```

### 3.3 活动检测逻辑适配

#### 当前逻辑
```rust
if trimmed.contains('(') && trimmed.contains(')') && trimmed.contains("tokens") {
    // 必须包含 tokens 关键词
}
```

#### 适配方案
如果新格式不再包含 `tokens` 关键词，需要修改检测逻辑：
```rust
if trimmed.contains('(') && trimmed.contains(')') && trimmed.contains("esc to interrupt") {
    // 检查 esc to interrupt 而不是 tokens
}
```

## 4. 影响范围分析

### 4.1 直接影响的文件

1. **`src/activity.rs`** - 核心活动检测逻辑
2. **`src/monitor.rs`** - 监控和正则表达式匹配
3. **`src/testing.rs`** - 测试数据生成
4. **所有测试文件** - 需要更新测试用例

### 4.2 间接影响的文件

1. **文档文件** - 需要更新格式说明
2. **示例文件** - 需要更新示例数据
3. **基准测试** - 需要更新测试数据

### 4.3 功能影响

1. **活动状态检测** - 核心功能，必须正确工作
2. **时间提取** - 影响递增检测
3. **跳过LLM调用** - 影响性能和准确性
4. **卡住检测** - 影响系统核心价值

## 5. 实现方案

### 5.1 适配策略

#### 策略1：向后兼容适配
- 保持现有功能不变
- 增加对新格式的支持
- 优先使用新格式检测，回退到旧格式

#### 策略2：完全替换
- 分析实际格式变化
- 直接替换所有相关正则表达式
- 更新所有测试用例

#### 策略3：渐进式适配
- 先添加新格式支持
- 逐步替换旧格式
- 保持测试覆盖

### 5.2 具体实现步骤

#### 步骤1：格式分析
```rust
// 添加格式分析函数
pub fn analyze_claude_format(text: &str) -> FormatType {
    if text.contains("tokens") && text.contains("esc to interrupt") {
        FormatType::FullFormat
    } else if text.contains("esc to interrupt") {
        FormatType::SimpleFormat
    } else {
        FormatType::Unknown
    }
}
```

#### 步骤2：正则表达式更新
```rust
// 在 activity.rs 中更新
pub fn is_claude_active(text: &str) -> bool {
    // 新格式检测
    let new_format_pattern = regex::Regex::new(r"[\*✶✢·✻✽][^)]*\([^)]*\d+s[^)]*esc to interrupt\)").unwrap();
    
    // 旧格式检测（向后兼容）
    let old_format_pattern = regex::Regex::new(r"[\*✶✢·✻✽][^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)").unwrap();
    
    new_format_pattern.is_match(text) || old_format_pattern.is_match(text)
}
```

#### 步骤3：时间提取优化
```rust
// 在 monitor.rs 中更新
pub fn extract_execution_time(text: &str) -> Option<u64> {
    // 更宽松的时间提取模式
    let time_pattern = regex::Regex::new(r"\((\d+)s(?:[^)]*)?\)").unwrap();
    // ...
}
```

#### 步骤4：测试用例更新
```rust
// 添加新格式测试用例
#[test]
fn test_new_format_detection() {
    assert!(is_claude_active("* Processing… (56s · esc to interrupt)"));
    assert!(is_claude_active("✶ Thinking… (123s · esc to interrupt)"));
}
```

### 5.3 测试要求

#### 单元测试
- 新格式检测功能
- 时间提取功能
- 向后兼容性
- 边界情况处理

#### 集成测试
- 完整监控流程
- 状态转换
- 错误处理

#### 性能测试
- 正则表达式性能
- 内存使用
- 响应时间

## 6. 风险评估

### 6.1 技术风险

1. **正则表达式错误**：可能导致误检或漏检
2. **向后兼容性**：可能影响现有用户
3. **性能影响**：更复杂的正则表达式可能影响性能

### 6.2 业务风险

1. **功能失效**：核心检测功能可能失效
2. **用户体验**：可能导致误报或漏报
3. **维护成本**：增加代码复杂度

### 6.3 缓解措施

1. **充分测试**：确保所有情况都被覆盖
2. **渐进式部署**：逐步推出新功能
3. **监控告警**：添加运行时监控

## 7. 推荐方案

### 7.1 方案选择

推荐**策略1：向后兼容适配**，原因：
1. 风险最小
2. 用户无感知
3. 可以逐步验证新格式
4. 便于回滚

### 7.2 实施优先级

1. **高优先级**：核心检测逻辑适配
2. **中优先级**：测试用例更新
3. **低优先级**：文档和示例更新

### 7.3 时间估算

- 分析和设计：2-4小时
- 核心功能实现：4-6小时
- 测试用例更新：2-3小时
- 文档更新：1-2小时
- **总计**：9-15小时

## 8. 结论

基于代码分析，当前系统已经对Claude Code的新格式有较好的支持，但可能存在一些边界情况需要处理。建议采用向后兼容的适配策略，确保系统稳定性的同时支持新格式。

关键是要准确识别新格式的具体变化，然后有针对性地进行适配，而不是盲目修改所有相关代码。

---

**文档创建时间**：2025-08-21  
**分析人员**：Claude Code Requirements Analyst  
**下一步**：根据实际格式变化情况，制定具体的实现计划