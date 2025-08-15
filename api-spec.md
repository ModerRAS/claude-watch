# Claude Watch 测试API和接口规范

## API 概述

本文档定义了claude-watch项目的测试接口规范，包括内部API、测试Mock接口、性能测试接口等。

## 版本信息

- **API版本**: v1.0.0
- **文档版本**: 1.0.0
- **最后更新**: 2025-08-14

## 目录结构

```
src/
├── lib.rs              # 公共API导出
├── activity.rs         # 活动检测API
├── monitor.rs          # 监控逻辑API
├── config.rs           # 配置管理API
├── llm.rs              # LLM调用API
├── tmux.rs             # tmux交互API
└── testing/            # 测试相关API
    ├── mod.rs
    ├── mocks.rs        # Mock实现
    ├── test_data.rs    # 测试数据
    └── fixtures.rs     # 测试固件
```

## 公共API规范

### 活动检测接口

```rust
// src/activity.rs

/// 检测Claude Code是否处于活动状态
/// 
/// # 参数
/// * `text` - 要检测的文本内容
/// 
/// # 返回值
/// * `bool` - true表示活动状态，false表示非活动状态
/// 
/// # 示例
/// ```rust
/// use claude_watch::is_claude_active;
/// 
/// let result = is_claude_active("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)");
/// assert!(result);
/// ```
pub fn is_claude_active(text: &str) -> bool {
    // 实现细节...
}
```

#### 测试用例规范

```rust
// tests/activity_detection.rs

#[cfg(test)]
mod activity_detection_tests {
    use super::*;
    
    /// 测试标准Claude Code执行条格式
    #[test]
    fn test_activity_detection_standard_format() {
        let test_cases = vec![
            // 输入文本, 期望结果
            ("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)", true),
            ("* Cogitating… (169s · ↓ 8.7k tokens · esc to interrupt)", true),
            ("* Processing… (56s · ↑ 2.1k tokens · esc to interrupt)", true),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(is_claude_active(input), expected, 
                       "Failed for input: {}", input);
        }
    }
    
    /// 测试工具调用状态检测
    #[test]
    fn test_activity_detection_tool_calls() {
        let test_cases = vec![
            ("Tool use: Reading file", true),
            ("Calling tool: function_name", true),
            ("Function call: api_request", true),
            ("Reading file: src/main.rs", true),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(is_claude_active(input), expected, 
                       "Failed for tool call: {}", input);
        }
    }
    
    /// 测试边界情况
    #[test]
    fn test_activity_detection_edge_cases() {
        let test_cases = vec![
            ("", false),
            ("   ", false),
            ("* Herding… (not-a-number s · tokens)", false),
            ("(343s) but no tokens", true),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(is_claude_active(input), expected, 
                       "Failed for edge case: '{}'", input);
        }
    }
}
```

### 监控逻辑接口

```rust
// src/monitor.rs

/// 提取Claude Code执行条中的时间值
/// 
/// # 参数
/// * `text` - 包含时间信息的文本
/// 
/// # 返回值
/// * `Option<u64>` - 提取的时间值（秒），如果无法提取则返回None
/// 
/// # 示例
/// ```rust
/// use claude_watch::extract_execution_time;
/// 
/// let time = extract_execution_time("* Herding… (343s · ↑ 14.2k tokens)");
/// assert_eq!(time, Some(343));
/// ```
pub fn extract_execution_time(text: &str) -> Option<u64> {
    // 实现细节...
}

/// 检查时间是否在递增（表明Claude Code在工作）
/// 
/// # 参数
/// * `current_text` - 当前文本内容
/// * `pane` - tmux窗格标识符
/// 
/// # 返回值
/// * `bool` - true表示时间在递增，false表示时间未递增
/// 
/// # 安全性
/// 此函数使用unsafe代码来管理全局状态，调用时需要确保线程安全
pub fn is_time_increasing(current_text: &str, pane: &str) -> bool {
    // 实现细节...
}

/// 检查是否有实质性的进展，而不只是时间计数器
/// 
/// # 参数
/// * `text` - 要检查的文本内容
/// 
/// # 返回值
/// * `bool` - true表示有实质性进展，false表示没有实质性进展
pub fn has_substantial_progress(text: &str) -> bool {
    // 实现细节...
}

/// 检查是否应该跳过LLM调用，避免误判为卡住
/// 
/// # 参数
/// * `text` - 要检查的文本内容
/// 
/// # 返回值
/// * `bool` - true表示应该跳过LLM调用，false表示需要调用LLM
pub fn check_if_should_skip_llm_call(text: &str) -> bool {
    // 实现细节...
}
```

#### 监控逻辑测试接口

```rust
// tests/monitor_logic.rs

#[cfg(test)]
mod monitor_tests {
    use super::*;
    
    /// 测试时间提取功能
    #[test]
    fn test_time_extraction() {
        let test_cases = vec![
            ("* Herding… (343s · ↑ 14.2k tokens)", Some(343)),
            ("56s · ↓ 2.3k tokens", Some(56)),
            ("No time here", None),
            ("* Herding… (not-a-number s · tokens)", None),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(extract_execution_time(input), expected, 
                       "Failed for input: {}", input);
        }
    }
    
    /// 测试实质性进展检测
    #[test]
    fn test_substantial_progress_detection() {
        let test_cases = vec![
            // 有实质性进展的情况
            ("Tool use: Reading file", true),
            ("* Herding… (344s · ↑ 14.3k tokens)", true),
            ("Cogitating...", true),
            
            // 没有实质性进展的情况
            ("Interrupted by user\n>", false),
            ("Same content repeated", false),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(has_substantial_progress(input), expected, 
                       "Failed for progress detection: {}", input);
        }
    }
    
    /// 测试LLM调用跳过逻辑
    #[test]
    fn test_llm_call_skip_logic() {
        let test_cases = vec![
            // 应该跳过LLM调用的情况
            ("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)", true),
            ("Cogitating...", true),
            ("Interrupted by user\n>", false), // 明确中断状态，不跳过
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(check_if_should_skip_llm_call(input), expected, 
                       "Failed for skip logic: {}", input);
        }
    }
}
```

### 配置管理接口

```rust
// src/config.rs

/// 应用程序配置结构
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// tmux相关配置
    pub tmux: TmuxConfig,
    /// 监控相关配置
    pub monitoring: MonitoringConfig,
    /// LLM相关配置
    pub llm: LlmConfig,
}

/// 创建默认配置
/// 
/// # 返回值
/// * `Config` - 默认配置实例
pub fn default_config() -> Config {
    Config {
        tmux: TmuxConfig::default(),
        monitoring: MonitoringConfig::default(),
        llm: LlmConfig::default(),
    }
}

/// 从文件加载配置
/// 
/// # 参数
/// * `path` - 配置文件路径
/// 
/// # 返回值
/// * `Result<Config, String>` - 成功返回配置，失败返回错误信息
pub fn load_config(path: &str) -> Result<Config, String> {
    // 实现细节...
}

/// 验证配置有效性
/// 
/// # 参数
/// * `config` - 要验证的配置
/// 
/// # 返回值
/// * `Result<(), String>` - 成功返回Ok，失败返回错误信息
pub fn validate_config(config: &Config) -> Result<(), String> {
    // 实现细节...
}
```

#### 配置测试接口

```rust
// tests/config_tests.rs

#[cfg(test)]
mod config_tests {
    use super::*;
    
    /// 测试默认配置
    #[test]
    fn test_default_config() {
        let config = default_config();
        
        // 验证默认值
        assert_eq!(config.monitoring.interval, 2);
        assert_eq!(config.monitoring.stuck_sec, 30);
        assert_eq!(config.monitoring.max_retry, 3);
    }
    
    /// 测试配置文件加载
    #[test]
    fn test_config_loading() {
        let config_content = r#"
        tmux:
          pane: "%0"
        monitoring:
          interval: 5
          stuck_sec: 60
          max_retry: 5
        llm:
          backend: "openai"
        "#;
        
        let config = load_from_str(config_content).unwrap();
        assert_eq!(config.monitoring.interval, 5);
        assert_eq!(config.monitoring.stuck_sec, 60);
    }
    
    /// 测试配置验证
    #[test]
    fn test_config_validation() {
        let mut config = default_config();
        
        // 有效配置
        assert!(validate_config(&config).is_ok());
        
        // 无效配置 - interval为0
        config.monitoring.interval = 0;
        assert!(validate_config(&config).is_err());
        
        // 无效配置 - stuck_sec为0
        config.monitoring.interval = 2;
        config.monitoring.stuck_sec = 0;
        assert!(validate_config(&config).is_err());
    }
}
```

## Mock接口规范

### Tmux Mock接口

```rust
// src/testing/mocks.rs

/// Mock tmux客户端，用于测试
pub struct MockTmuxClient {
    /// 预设的响应内容
    pub responses: Vec<String>,
    /// 记录发送的命令
    pub sent_commands: Vec<String>,
    /// 当前响应索引
    pub response_index: usize,
}

impl MockTmuxClient {
    /// 创建新的Mock客户端
    pub fn new() -> Self {
        Self {
            responses: Vec::new(),
            sent_commands: Vec::new(),
            response_index: 0,
        }
    }
    
    /// 设置响应内容
    /// 
    /// # 参数
    /// * `responses` - 响应内容列表
    pub fn set_responses(&mut self, responses: Vec<String>) {
        self.responses = responses;
        self.response_index = 0;
    }
    
    /// 添加单个响应
    /// 
    /// # 参数
    /// * `response` - 响应内容
    pub fn add_response(&mut self, response: &str) {
        self.responses.push(response.to_string());
    }
    
    /// 获取发送的命令
    /// 
    /// # 返回值
    /// * `&[String]` - 发送的命令列表
    pub fn get_sent_commands(&self) -> &[String] {
        &self.sent_commands
    }
    
    /// 清空发送的命令记录
    pub fn clear_commands(&mut self) {
        self.sent_commands.clear();
    }
}

/// 为MockTmuxClient实现tmux::TmuxClient trait
impl tmux::TmuxClient for MockTmuxClient {
    fn capture(&mut self, _pane: &str) -> String {
        if self.response_index < self.responses.len() {
            let response = self.responses[self.response_index].clone();
            self.response_index += 1;
            response
        } else {
            self.responses.last().cloned().unwrap_or_default()
        }
    }
    
    fn send_keys(&mut self, keys: &str, _pane: &str) {
        self.sent_commands.push(keys.to_string());
    }
}
```

### LLM Mock接口

```rust
// src/testing/mocks.rs

/// Mock LLM服务，用于测试
pub struct MockLlmService {
    /// 预设的响应状态
    pub responses: Vec<TaskStatus>,
    /// 是否应该失败
    pub should_fail: bool,
    /// 失败时的错误信息
    pub error_message: String,
    /// 当前响应索引
    pub response_index: usize,
}

impl MockLlmService {
    /// 创建新的Mock服务
    pub fn new() -> Self {
        Self {
            responses: vec![TaskStatus::Done],
            should_fail: false,
            error_message: String::new(),
            response_index: 0,
        }
    }
    
    /// 设置响应状态
    /// 
    /// # 参数
    /// * `responses` - 响应状态列表
    pub fn set_responses(&mut self, responses: Vec<TaskStatus>) {
        self.responses = responses;
        self.response_index = 0;
    }
    
    /// 设置失败状态
    /// 
    /// # 参数
    /// * `error_message` - 错误信息
    pub fn set_failure(&mut self, error_message: &str) {
        self.should_fail = true;
        self.error_message = error_message.to_string();
    }
    
    /// 设置成功状态
    pub fn set_success(&mut self) {
        self.should_fail = false;
        self.error_message.clear();
    }
}

/// 为MockLlmService实现llm::LlmService trait
impl llm::LlmService for MockLlmService {
    async fn ask_final_status(&self, _text: &str) -> Result<TaskStatus, String> {
        if self.should_fail {
            return Err(self.error_message.clone());
        }
        
        if self.response_index < self.responses.len() {
            Ok(self.responses[self.response_index])
        } else {
            Ok(self.responses.last().cloned().unwrap_or(TaskStatus::Done))
        }
    }
    
    async fn ask_for_activation(&self, _prompt: &str) -> Result<String, String> {
        if self.should_fail {
            return Err(self.error_message.clone());
        }
        
        Ok("请继续处理任务".to_string())
    }
}
```

## 测试固件接口

### 测试环境设置

```rust
// src/testing/fixtures.rs

/// 测试环境配置
pub struct TestEnvironment {
    /// tmux会话名称
    pub tmux_session: String,
    /// 测试窗格名称
    pub test_pane: String,
    /// 配置文件路径
    pub config_file: String,
    /// Mock tmux客户端
    pub mock_tmux: MockTmuxClient,
    /// Mock LLM服务
    pub mock_llm: MockLlmService,
}

impl TestEnvironment {
    /// 创建测试环境
    pub async fn setup() -> Result<Self, String> {
        let tmux_session = format!("claude-test-{}", uuid::Uuid::new_v4());
        let test_pane = "%0".to_string();
        let config_file = format!("/tmp/claude-test-{}.yaml", uuid::Uuid::new_v4());
        
        // 创建测试配置文件
        let config_content = r#"
        tmux:
          pane: "%0"
        monitoring:
          interval: 1
          stuck_sec: 2
          max_retry: 2
        llm:
          backend: "mock"
        "#;
        
        std::fs::write(&config_file, config_content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        Ok(Self {
            tmux_session,
            test_pane,
            config_file,
            mock_tmux: MockTmuxClient::new(),
            mock_llm: MockLlmService::new(),
        })
    }
    
    /// 清理测试环境
    pub async fn teardown(self) -> Result<(), String> {
        // 删除配置文件
        if std::path::Path::new(&self.config_file).exists() {
            std::fs::remove_file(&self.config_file)
                .map_err(|e| format!("Failed to remove config file: {}", e))?;
        }
        
        Ok(())
    }
    
    /// 获取测试配置
    pub fn get_test_config(&self) -> Config {
        let mut config = default_config();
        config.tmux.pane = self.test_pane.clone();
        config.monitoring.interval = 1;
        config.monitoring.stuck_sec = 2;
        config.monitoring.max_retry = 2;
        config
    }
}
```

### 测试场景定义

```rust
// src/testing/fixtures.rs

/// 测试场景定义
pub struct TestScenario {
    /// 场景名称
    pub name: String,
    /// 场景描述
    pub description: String,
    /// 场景步骤
    pub steps: Vec<TestStep>,
    /// 期望结果
    pub expected_result: TestExpectedResult,
}

/// 测试步骤
pub struct TestStep {
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: String,
    /// 模拟的tmux响应
    pub tmux_response: String,
    /// 等待时间
    pub wait_duration: Duration,
}

/// 期望结果
pub struct TestExpectedResult {
    /// 期望的活动状态
    pub is_active: bool,
    /// 期望是否跳过LLM调用
    pub should_skip_llm: bool,
    /// 期望发送的命令
    pub expected_commands: Vec<String>,
}

/// 预定义测试场景
pub fn get_predefined_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario {
            name: "normal_working".to_string(),
            description: "Claude Code正常工作状态".to_string(),
            steps: vec![
                TestStep {
                    name: "initial_capture".to_string(),
                    description: "初始捕获".to_string(),
                    tmux_response: "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)".to_string(),
                    wait_duration: Duration::from_secs(1),
                },
                TestStep {
                    name: "time_progress".to_string(),
                    description: "时间进展".to_string(),
                    tmux_response: "* Herding… (344s · ↑ 14.3k tokens · esc to interrupt)".to_string(),
                    wait_duration: Duration::from_secs(1),
                },
            ],
            expected_result: TestExpectedResult {
                is_active: true,
                should_skip_llm: true,
                expected_commands: vec![],
            },
        },
        TestScenario {
            name: "stuck_state".to_string(),
            description: "Claude Code卡住状态".to_string(),
            steps: vec![
                TestStep {
                    name: "interrupted".to_string(),
                    description: "被中断".to_string(),
                    tmux_response: "Interrupted by user\n>".to_string(),
                    wait_duration: Duration::from_secs(5),
                },
                TestStep {
                    name: "still_stuck".to_string(),
                    description: "仍然卡住".to_string(),
                    tmux_response: "Interrupted by user\n>".to_string(),
                    wait_duration: Duration::from_secs(5),
                },
            ],
            expected_result: TestExpectedResult {
                is_active: false,
                should_skip_llm: false,
                expected_commands: vec!["Retry".to_string()],
            },
        },
    ]
}
```

## 性能测试接口

### 基准测试接口

```rust
// benches/performance_bench.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

/// 活动检测性能基准测试
fn bench_activity_detection(c: &mut Criterion) {
    let test_cases = vec![
        ("standard_format", "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)"),
        ("tool_call", "Tool use: Reading file"),
        ("empty", ""),
        ("long_text", "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)\nSome additional content\nMore lines"),
    ];
    
    let mut group = c.benchmark_group("activity_detection");
    
    for (name, input) in test_cases {
        group.bench_with_input(BenchmarkId::new("is_claude_active", name), input, |b, input| {
            b.iter(|| is_claude_active(input));
        });
    }
    
    group.finish();
}

/// 时间提取性能基准测试
fn bench_time_extraction(c: &mut Criterion) {
    let test_cases = vec![
        ("standard_format", "* Herding… (343s · ↑ 14.2k tokens)"),
        ("simple_format", "56s · ↓ 2.3k tokens"),
        ("no_time", "No time here"),
        ("invalid_format", "* Herding… (not-a-number s · tokens)"),
    ];
    
    let mut group = c.benchmark_group("time_extraction");
    
    for (name, input) in test_cases {
        group.bench_with_input(BenchmarkId::new("extract_execution_time", name), input, |b, input| {
            b.iter(|| extract_execution_time(input));
        });
    }
    
    group.finish();
}

/// 监控逻辑性能基准测试
fn bench_monitor_logic(c: &mut Criterion) {
    let test_cases = vec![
        ("has_progress", "Tool use: Reading file"),
        ("no_progress", "Interrupted by user\n>"),
        ("time_only", "* Herding… (343s · ↑ 14.2k tokens)"),
    ];
    
    let mut group = c.benchmark_group("monitor_logic");
    
    for (name, input) in test_cases {
        group.bench_with_input(BenchmarkId::new("has_substantial_progress", name), input, |b, input| {
            b.iter(|| has_substantial_progress(input));
        });
        
        group.bench_with_input(BenchmarkId::new("check_if_should_skip_llm_call", name), input, |b, input| {
            b.iter(|| check_if_should_skip_llm_call(input));
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_activity_detection, bench_time_extraction, bench_monitor_logic);
criterion_main!(benches);
```

### 内存使用测试接口

```rust
// tests/memory_usage.rs

#[cfg(test)]
mod memory_tests {
    use super::*;
    use std::alloc::{GlobalAlloc, System, Layout};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// 内存使用跟踪器
    pub struct TrackingAllocator {
        allocated: AtomicUsize,
    }

    impl TrackingAllocator {
        pub const fn new() -> Self {
            Self {
                allocated: AtomicUsize::new(0),
            }
        }

        pub fn get_allocated(&self) -> usize {
            self.allocated.load(Ordering::Relaxed)
        }
    }

    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ptr = System.alloc(layout);
            if !ptr.is_null() {
                self.allocated.fetch_add(layout.size(), Ordering::Relaxed);
            }
            ptr
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout);
            self.allocated.fetch_sub(layout.size(), Ordering::Relaxed);
        }
    }

    #[global_allocator]
    static ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

    /// 测试活动检测的内存使用
    #[test]
    fn test_activity_detection_memory_usage() {
        let initial_memory = ALLOCATOR.get_allocated();
        
        // 执行大量活动检测操作
        for _ in 0..1000 {
            is_claude_active("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)");
            is_claude_active("Tool use: Reading file");
            is_claude_active("");
        }
        
        let final_memory = ALLOCATOR.get_allocated();
        let memory_increase = final_memory.saturating_sub(initial_memory);
        
        // 内存增长应该在合理范围内
        assert!(memory_increase < 1024 * 1024, "Memory usage too high: {} bytes", memory_increase);
    }

    /// 测试长时间运行的内存使用
    #[test]
    fn test_long_running_memory_usage() {
        let initial_memory = ALLOCATOR.get_allocated();
        
        // 模拟长时间运行
        for i in 0..10000 {
            let text = format!("* Herding… ({}s · ↑ 14.2k tokens)", i % 1000);
            is_claude_active(&text);
            extract_execution_time(&text);
            
            // 每1000次检查一次内存
            if i % 1000 == 0 {
                let current_memory = ALLOCATOR.get_allocated();
                let memory_increase = current_memory.saturating_sub(initial_memory);
                assert!(memory_increase < 10 * 1024 * 1024, "Memory leak detected at iteration {}: {} bytes", i, memory_increase);
            }
        }
        
        let final_memory = ALLOCATOR.get_allocated();
        let memory_increase = final_memory.saturating_sub(initial_memory);
        println!("Total memory increase: {} bytes", memory_increase);
    }
}
```

## 集成测试接口

### 端到端测试接口

```rust
// tests/integration.rs

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::testing::{TestEnvironment, get_predefined_scenarios};

    /// 端到端测试：完整监控流程
    #[tokio::test]
    async fn test_full_monitoring_flow() {
        let mut env = TestEnvironment::setup().await.unwrap();
        
        // 设置测试场景
        env.mock_tmux.set_responses(vec![
            "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)".to_string(),
            "* Herding… (344s · ↑ 14.3k tokens · esc to interrupt)".to_string(),
        ]);
        
        env.mock_llm.set_responses(vec![TaskStatus::Done]);
        
        // 执行监控逻辑
        let config = env.get_test_config();
        let mut last_active = std::time::Instant::now();
        let mut retry_count = 0;
        
        // 模拟监控循环
        for _ in 0..3 {
            let text = env.mock_tmux.capture(&config.tmux.pane);
            
            if is_claude_active(&text) {
                last_active = std::time::Instant::now();
                retry_count = 0;
            }
            
            // 模拟等待
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        // 验证结果
        assert_eq!(retry_count, 0, "Should not have retried");
        assert!(env.mock_tmux.get_sent_commands().is_empty(), "Should not have sent commands");
        
        env.teardown().await.unwrap();
    }

    /// 测试卡住检测和恢复流程
    #[tokio::test]
    async fn test_stuck_detection_and_recovery() {
        let mut env = TestEnvironment::setup().await.unwrap();
        
        // 设置卡住场景
        env.mock_tmux.set_responses(vec![
            "Interrupted by user\n>".to_string(),
            "Interrupted by user\n>".to_string(),
            "Retry\n* Herding… (1s · ↑ 100 tokens)".to_string(),
        ]);
        
        env.mock_llm.set_responses(vec![TaskStatus::Stuck]);
        
        // 执行监控逻辑
        let config = env.get_test_config();
        let mut last_active = std::time::Instant::now() - std::time::Duration::from_secs(10);
        let mut retry_count = 0;
        
        // 模拟卡住检测
        let text = env.mock_tmux.capture(&config.tmux.pane);
        
        if !is_claude_active(&text) && last_active.elapsed() >= std::time::Duration::from_secs(config.monitoring.stuck_sec) {
            // 检查是否跳过LLM调用
            let should_skip = check_if_should_skip_llm_call(&text);
            assert!(!should_skip, "Should not skip LLM call for interrupted state");
            
            // 模拟LLM调用和恢复
            if retry_count < config.monitoring.max_retry {
                env.mock_tmux.send_keys("Retry", &config.tmux.pane);
                retry_count += 1;
            }
        }
        
        // 验证恢复命令被发送
        assert_eq!(env.mock_tmux.get_sent_commands(), &["Retry"]);
        
        env.teardown().await.unwrap();
    }

    /// 测试预定义场景
    #[tokio::test]
    async fn test_predefined_scenarios() {
        let scenarios = get_predefined_scenarios();
        
        for scenario in scenarios {
            println!("Testing scenario: {}", scenario.name);
            
            let mut env = TestEnvironment::setup().await.unwrap();
            
            // 设置场景响应
            let responses: Vec<String> = scenario.steps.iter()
                .map(|step| step.tmux_response.clone())
                .collect();
            env.mock_tmux.set_responses(responses);
            
            // 执行场景步骤
            for step in &scenario.steps {
                let text = env.mock_tmux.capture(&env.test_pane);
                
                // 验证活动状态
                let is_active = is_claude_active(&text);
                let should_skip_llm = check_if_should_skip_llm_call(&text);
                
                // 这里可以根据场景期望结果进行断言
                println!("Step {}: active={}, skip_llm={}", step.name, is_active, should_skip_llm);
                
                tokio::time::sleep(step.wait_duration).await;
            }
            
            // 验证最终结果
            let sent_commands = env.mock_tmux.get_sent_commands();
            assert_eq!(sent_commands, &scenario.expected_result.expected_commands);
            
            env.teardown().await.unwrap();
        }
    }
}
```

## 总结

本文档定义了claude-watch项目的完整测试API规范，包括：

1. **核心功能API**: 活动检测、监控逻辑、配置管理等公共接口
2. **测试Mock接口**: tmux和LLM服务的Mock实现
3. **测试固件接口**: 测试环境设置和预定义场景
4. **性能测试接口**: 基准测试和内存使用测试
5. **集成测试接口**: 端到端测试和场景测试

这些接口为claude-watch项目提供了全面的测试支持，确保代码质量和系统稳定性。