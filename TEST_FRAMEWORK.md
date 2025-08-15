# Claude Watch 测试框架

本项目包含了完整的测试框架，涵盖单元测试、集成测试、性能测试和测试数据捕获工具。

## 测试结构

```
tests/
├── activity_detection.rs      # 活动检测测试（已存在）
├── progress_detection.rs     # 进度检测测试（已存在）
├── time_detection.rs         # 时间检测测试（已存在）
├── integration_tests.rs      # 集成测试
├── config_tests.rs          # 配置模块测试
├── monitor_tests.rs         # 监控模块测试
├── llm_tests.rs             # LLM模块测试
├── tmux_tests.rs            # Tmux模块测试
├── args_tests.rs            # 参数解析测试
├── performance_tests.rs     # 性能测试
└── capture_test_data.rs     # 测试数据捕获工具

benches/
└── performance_bench.rs     # Criterion性能基准测试

src/testing/
├── mod.rs                   # 测试模块入口
├── mocks.rs                 # Mock服务框架
├── fixtures.rs              # 测试数据固定值
└── test_utils.rs            # 测试工具函数
```

## 运行测试

### 运行所有测试
```bash
cargo test
```

### 运行特定测试
```bash
# 运行配置测试
cargo test config_tests

# 运行监控测试
cargo test monitor_tests

# 运行集成测试
cargo test integration_tests

# 运行性能测试
cargo test performance_tests
```

### 运行性能基准测试
```bash
# 安装Criterion（如果尚未安装）
cargo install cargo-criterion

# 运行基准测试
cargo criterion
```

## 测试框架特性

### 1. Mock服务框架
- `MockLlmService`: 模拟LLM调用
- `MockTmuxService`: 模拟Tmux操作
- `MockMonitorService`: 模拟监控功能
- `MockConfigService`: 模拟配置管理
- `MockArgsService`: 模拟参数解析

### 2. 测试数据固定值
- 预定义的测试数据集合
- 覆盖各种Claude Code状态
- 包含边界条件和错误情况

### 3. 测试工具函数
- `TestValidator`: 数据验证工具
- `PerformanceProfiler`: 性能分析工具
- `AsyncTestHelper`: 异步测试助手
- `TestConfigManager`: 测试配置管理器

### 4. 测试数据捕获
- `TestDataCapture`: 捕获和存储测试数据
- `TestDataCollector`: 便捷的数据收集API
- `RealTimeMonitor`: 实时监控数据捕获
- 支持导出为JSON、CSV和测试代码

## 测试覆盖范围

### 核心功能测试
- ✅ 活动状态检测
- ✅ 时间提取和跟踪
- ✅ 跳过LLM调用逻辑
- ✅ 进度检测
- ✅ 内容变化检测

### 配置管理测试
- ✅ 配置文件解析
- ✅ 命令行参数解析
- ✅ 默认配置生成
- ✅ 配置验证

### LLM集成测试
- ✅ OpenAI API调用
- ✅ Ollama集成
- ✅ OpenRouter支持
- ✅ 启发式检查
- ✅ 智能激活

### Tmux操作测试
- ✅ 窗格内容捕获
- ✅ 按键发送
- ✅ 命令格式验证
- ✅ 错误处理

### 监控流程测试
- ✅ 完整监控周期
- ✅ 卡住检测和恢复
- ✅ 重试机制
- ✅ 完成状态监控

### 性能测试
- ✅ 函数级性能基准
- ✅ 内存使用测试
- ✅ 并发操作测试
- ✅ 边界情况性能

### 集成测试
- ✅ 端到端工作流
- ✅ 模块间协作
- ✅ 错误恢复
- ✅ 真实场景模拟

## 使用Mock服务进行测试

```rust
use claude_watch::testing::*;

#[tokio::test]
async fn test_with_mock_llm() {
    let mut mock_llm = MockLlmService::new();
    
    // 设置Mock期望
    mock_llm.expect_ask_llm_final_status()
        .returning(|_, _| Ok(llm::TaskStatus::Done));
    
    // 使用Mock服务
    let result = mock_llm.ask_llm_final_status("test context", "test pane").await;
    assert_eq!(result, Ok(llm::TaskStatus::Done));
}
```

## 捕获真实测试数据

```rust
use claude_watch::testing::*;

#[tokio::test]
async fn test_data_capture() {
    let mut collector = TestDataCollector::new();
    let config = config::Config::default();
    
    // 开始数据收集
    collector.start_collection(config).unwrap();
    
    // 捕获当前状态
    collector.capture_state(
        "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)",
        "active_processing",
        "Claude正在处理任务"
    ).unwrap();
    
    // 结束收集并保存
    collector.end_collection().unwrap();
    collector.save_collection("test_data.json").unwrap();
}
```

## 性能测试示例

```rust
#[test]
fn test_performance() {
    let mut profiler = PerformanceProfiler::new();
    
    profiler.start_measurement("activity_detection");
    for _ in 0..1000 {
        let _ = activity::is_claude_active("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)");
    }
    profiler.end_measurement("activity_detection");
    
    let time = profiler.get_measurement("activity_detection").unwrap();
    println!("Activity detection: {}ms for 1000 operations", time);
}
```

## 测试最佳实践

1. **使用Mock服务**: 避免外部依赖，确保测试稳定性
2. **测试边界条件**: 包含空字符串、无效输入、极端值
3. **性能基准**: 为关键函数建立性能基准
4. **数据驱动**: 使用预定义测试数据确保一致性
5. **异步测试**: 正确处理异步操作和超时
6. **错误处理**: 测试错误情况和恢复机制
7. **集成测试**: 验证模块间协作
8. **真实数据**: 使用捕获的真实数据测试

## 添加新测试

1. **单元测试**: 在对应模块的测试文件中添加
2. **集成测试**: 在`integration_tests.rs`中添加
3. **性能测试**: 在`performance_tests.rs`或`benches/`中添加
4. **Mock服务**: 在`mocks.rs`中定义新的Mock服务
5. **测试数据**: 在`fixtures.rs`中添加预定义数据

## 故障排除

### 常见问题

1. **编译错误**: 确保所有依赖已正确添加到`Cargo.toml`
2. **测试失败**: 检查Mock服务是否正确配置
3. **超时错误**: 调整异步测试的超时时间
4. **权限错误**: 确保有文件写入权限
5. **依赖冲突**: 检查版本兼容性

### 调试技巧

1. 使用`cargo test -- --nocapture`查看详细输出
2. 使用`cargo test --test <test_name>`运行特定测试
3. 使用`cargo test --lib`仅运行库测试
4. 使用`RUST_BACKTRACE=1`获取详细错误信息

## 贡献指南

1. 新增测试时请确保覆盖率和质量
2. 使用现有的Mock服务和工具函数
3. 遵循现有的测试命名和组织规范
4. 为复杂的测试添加适当的文档和注释
5. 确保测试在所有支持的平台上运行

这个测试框架为claude-watch项目提供了全面的测试覆盖，确保代码质量和功能正确性。