# Claude Watch 测试套件文档

## 概述

本文档描述了claude-watch项目的完整测试套件，包括单元测试、集成测试、性能测试等。测试套件旨在确保项目的核心功能正常工作，并提供高质量的代码覆盖率。

## 测试结构

### 1. 核心测试文件

#### `tests/comprehensive_tests.rs`
这是主要的测试文件，包含了所有核心功能的测试：

- **活动检测测试** (`test_activity_detection_comprehensive`)
- **时间提取测试** (`test_time_extraction_comprehensive`)
- **跳过LLM调用测试** (`test_skip_llm_call_comprehensive`)
- **进展检测测试** (`test_progress_detection_comprehensive`)
- **内容变化检测测试** (`test_content_change_detection`)
- **真实场景测试** (`test_real_world_scenarios`)
- **边界情况测试** (`test_edge_cases`)
- **性能敏感操作测试** (`test_performance_sensitive_operations`)

#### `tests/activity_detection.rs`
专门测试活动检测功能：
- 标准Claude Code格式识别
- 工具调用检测
- 多行内容检测
- 边界情况处理

#### `tests/time_detection.rs`
专门测试时间相关功能：
- 时间提取
- 时间递增检测
- 活动与时间关系

#### `tests/integration.rs`
集成测试，验证完整的工作流程：
- 完整监控场景
- 状态转换
- 决策矩阵

### 2. 测试工具模块

#### `src/testing.rs`
提供测试辅助功能：

**核心数据结构**：
- `TestFixture`: 测试固件数据
- `TestScenario`: 测试场景
- `TestStep`: 测试步骤
- `PerformanceProfiler`: 性能分析器
- `AsyncTestHelper`: 异步测试辅助器
- `TestValidator`: 测试验证器

**模拟对象**：
- `MockMonitorService`: 模拟监控服务trait
- `MockMonitorServiceImpl`: 模拟监控服务实现

**测试数据生成**：
- `TestDataGenerator`: 生成随机测试数据
- `TestFixtures`: 提供预定义测试固件
- `TestScenarios`: 提供预定义测试场景

**断言辅助**：
- `assert_activity!`: 活动检测断言宏
- `assert_skip_llm!`: 跳过LLM调用断言宏
- `assert_progress!`: 进展检测断言宏
- `assert_time!`: 时间提取断言宏

## 运行测试

### 运行所有测试
```bash
cargo test
```

### 运行特定测试文件
```bash
cargo test --test comprehensive_tests
cargo test --test activity_detection
cargo test --test integration
```

### 运行特定测试
```bash
cargo test test_activity_detection_comprehensive
cargo test test_real_world_scenarios
```

### 运行性能测试
```bash
cargo test --test performance_tests
```

### 仅运行编译检查
```bash
cargo test --no-run
```

## 测试覆盖的功能

### 1. 活动检测 (`is_claude_active`)
- 标准Claude Code执行条格式识别
- 工具调用状态检测
- 简单时间格式识别
- 进度指示器识别
- 多行内容处理
- 边界情况处理

### 2. 时间提取 (`extract_execution_time`)
- 从复杂格式中提取时间值
- 处理各种时间格式
- 错误格式处理

### 3. 跳过LLM调用 (`check_if_should_skip_llm_call`)
- 标准执行条格式判断
- 中断状态识别
- 命令提示符识别
- 错误状态处理

### 4. 进展检测 (`has_substantial_progress`)
- 工具调用进展
- 思考状态进展
- 完成状态进展
- 错误状态进展
- 长文本进展

### 5. 内容变化检测 (`has_substantial_content_change`)
- 时间变化过滤
- Token变化过滤
- 实质性内容变化检测
- 系统信息变化过滤

### 6. 核心内容提取 (`extract_core_content`)
- 时间标准化
- Token标准化
- 系统信息移除
- 状态指示器标准化

### 7. 时间递增检测 (`is_time_increasing`)
- Pane级别时间追踪
- 递增逻辑验证
- 并发访问处理

### 8. 纯时间计数器检测 (`is_just_time_counter`)
- 纯时间格式识别
- 带最小内容的时间识别
- 实质性内容时间识别

## 测试场景

### 真实世界场景

1. **正常工作场景**
   ```
   * Herding… (169s · ↑ 8.7k tokens · esc to interrupt)
   
   The user is asking me to continue working on the task.
   I'm processing multiple files and analyzing the code structure.
   ```

2. **工具调用场景**
   ```
   Tool use: Reading file
   * Cogitating… (234s · ↓ 11.2k tokens · esc to interrupt)
   
   Reading the contents of src/main.rs...
   Analyzing the code structure...
   ```

3. **任务完成场景**
   ```
   ✅ Task completed successfully
   
   All files have been processed.
   The task is finished.
   Ready for new instructions.
   ```

4. **错误状态场景**
   ```
   Error: compilation failed
   
   src/main.rs:12:5: error: expected identifier
   
   Compilation terminated.
   ```

5. **中断状态场景**
   ```
   Interrupted by user
   >
   ```

### 边界情况

1. **空输入**
2. **纯空格**
3. **格式错误**
4. **特殊字符**
5. **非常长的输入**
6. **Unicode字符**

## 性能测试

### 性能指标
- **活动检测**: 1000次调用 < 100ms
- **时间提取**: 1000次调用 < 100ms
- **进展检测**: 1000次调用 < 100ms

### 性能测试方法
```rust
let mut profiler = PerformanceProfiler::new();

profiler.start_measurement("activity_detection");
for _ in 0..1000 {
    let _ = is_claude_active("* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)");
}
profiler.end_measurement("activity_detection");

let duration = profiler.get_measurement("activity_detection").unwrap();
assert!(duration < Duration::from_millis(100));
```

## 测试数据

### 预定义测试固件
```rust
let fixtures = TestFixtures::new();
let monitor_fixtures = fixtures.get_monitor_fixtures();

for fixture in monitor_fixtures {
    // 每个固件包含：
    // - description: 描述信息
    // - pane_content: pane内容
    // - expected_status: 期望状态
    // - expected_skip_llm: 期望跳过LLM调用
    // - expected_progress: 期望进展
}
```

### 动态测试数据生成
```rust
// 生成随机终端输出
let random_output = TestDataGenerator::generate_random_terminal_output();

// 生成时间序列
let time_series = TestDataGenerator::generate_time_series_output(100, 5);

// 生成混合内容
let mixed_content = TestDataGenerator::generate_mixed_content();
```

## 异步测试

### 异步测试辅助器
```rust
let helper = AsyncTestHelper::new();

// 测试超时机制
let result = helper.run_with_timeout(async {
    // 异步操作
    Ok::<(), String>(())
}).await;

// 测试重试机制
let result = helper.retry_async(
    || async { /* 异步操作 */ },
    5,  // 最大重试次数
    Duration::from_millis(10)  // 重试间隔
).await;
```

## 测试验证

### 验证框架
```rust
let mut validator = TestValidator::new();

// 验证数值范围
validator.validate_number_range("execution_time", time, 0, 10000);

// 验证字符串格式
validator.validate_string_matches_regex("pane_format", "%6", r"^%\d+$");

// 验证非空字符串
validator.validate_string_not_empty("skip_reason", "active_execution_bar");

// 验证布尔条件
validator.validate_true("activity_detected", activity);

assert!(validator.is_valid());
```

## 测试最佳实践

### 1. 测试命名
- 使用描述性的测试名称
- 遵循 `test_功能_场景` 的命名模式
- 例如: `test_activity_detection_standard_format`

### 2. 测试组织
- 按功能模块组织测试
- 使用 `#[test]` 和 `#[tokio::test]` 标记
- 相关测试放在一起

### 3. 测试数据
- 使用预定义测试固件
- 避免硬编码测试数据
- 使用数据生成器创建动态测试数据

### 4. 断言
- 使用具体的错误消息
- 测试期望值和实际值
- 使用宏简化重复的断言

### 5. 异步测试
- 正确处理异步操作
- 使用适当的超时和重试机制
- 避免闭包捕获问题

## 常见问题

### 1. 编译错误
- 确保所有依赖都已添加到 `Cargo.toml`
- 检查模块导入是否正确
- 确保函数签名匹配

### 2. 运行时错误
- 检查全局状态重置
- 确保测试数据的有效性
- 处理异步操作的竞态条件

### 3. 性能问题
- 避免在测试中进行大量计算
- 使用适当的性能阈值
- 考虑测试环境的差异性

## 扩展测试

### 添加新测试
1. 在相应的测试文件中添加测试函数
2. 使用现有的测试工具和辅助函数
3. 确保测试覆盖新的功能点
4. 运行测试确保通过

### 添加新的测试工具
1. 在 `src/testing.rs` 中添加新的工具
2. 提供清晰的文档和使用示例
3. 确保工具的稳定性和可靠性
4. 更新测试文档

## 总结

claude-watch的测试套件提供了全面的功能覆盖，包括：

- ✅ **完整的功能测试**: 覆盖所有核心功能
- ✅ **集成测试**: 验证完整工作流程
- ✅ **性能测试**: 确保性能要求
- ✅ **边界情况测试**: 处理异常情况
- ✅ **异步测试**: 支持异步操作
- ✅ **测试工具**: 提供丰富的测试辅助功能

测试套件设计遵循以下原则：

- **可维护性**: 清晰的结构和文档
- **可扩展性**: 易于添加新测试
- **可靠性**: 稳定的测试结果
- **实用性**: 真实场景的测试覆盖

通过运行 `cargo test` 可以执行完整的测试套件，确保代码质量和功能正确性。