//! 测试工具模块
//! 
//! 提供测试辅助功能，包括模拟对象、测试数据、性能分析等

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use async_trait::async_trait;

// 重新导出核心监控函数以便测试使用
pub use crate::monitor::{
    extract_execution_time,
    is_time_increasing,
    check_if_should_skip_llm_call,
    has_substantial_progress,
    is_just_time_counter,
    PaneStatus,
};

// 为测试暴露私有函数
pub use crate::monitor::{
    has_substantial_content_change,
    extract_core_content,
};

pub use crate::activity::is_claude_active;
pub use crate::config::Config;
pub use crate::args::Args;

/// 测试固件数据结构
#[derive(Debug, Clone)]
pub struct TestFixture {
    pub description: String,
    pub pane_content: String,
    pub expected_status: PaneStatus,
    pub expected_skip_llm: bool,
    pub expected_progress: bool,
}

/// 测试场景步骤
#[derive(Debug, Clone)]
pub struct TestStep {
    pub name: String,
    pub action: String,
    pub expected_output: Option<String>,
    pub expected_status: Option<PaneStatus>,
}

/// 测试场景
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: String,
    pub description: String,
    pub steps: Vec<TestStep>,
}

/// 模拟监控服务trait
#[async_trait]
pub trait MockMonitorService: Send + Sync {
    async fn check_pane_status(&self, pane_id: &str) -> PaneStatus;
    async fn handle_stuck_pane(&self, pane_id: &str) -> Result<(), String>;
}

/// 性能分析器
pub struct PerformanceProfiler {
    measurements: HashMap<String, Duration>,
    start_times: HashMap<String, Instant>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            start_times: HashMap::new(),
        }
    }

    pub fn start_measurement(&mut self, name: &str) {
        self.start_times.insert(name.to_string(), Instant::now());
    }

    pub fn end_measurement(&mut self, name: &str) {
        if let Some(start_time) = self.start_times.remove(name) {
            let duration = start_time.elapsed();
            self.measurements.insert(name.to_string(), duration);
        }
    }

    pub fn get_measurement(&self, name: &str) -> Option<Duration> {
        self.measurements.get(name).copied()
    }

    pub fn get_measurements(&self) -> &HashMap<String, Duration> {
        &self.measurements
    }
}

/// 异步测试辅助器
pub struct AsyncTestHelper {
    timeout: Duration,
}

impl AsyncTestHelper {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }

    pub async fn run_with_timeout<F, T>(&self, future: F) -> Result<T, String>
    where
        F: std::future::Future<Output = Result<T, String>>,
    {
        tokio::time::timeout(self.timeout, future)
            .await
            .map_err(|_| "Test timed out".to_string())?
    }

    pub async fn retry_async<F, T, E>(
        &self,
        mut operation: impl FnMut() -> F,
        max_attempts: usize,
        delay: Duration,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match operation().await {
                Ok(result) => return Ok(result),
                Err(_e) if attempt < max_attempts => {
                    sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// 测试验证器
pub struct TestValidator {
    errors: Vec<String>,
}

impl TestValidator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn validate_number_range(&mut self, name: &str, value: u64, min: u64, max: u64) {
        if value < min || value > max {
            self.errors.push(format!("{}: {} is out of range [{}, {}]", name, value, min, max));
        }
    }

    pub fn validate_string_matches_regex(&mut self, name: &str, value: &str, pattern: &str) {
        let regex = regex::Regex::new(pattern).unwrap_or_else(|_| {
            panic!("Invalid regex pattern: {}", pattern);
        });
        
        if !regex.is_match(value) {
            self.errors.push(format!("{}: '{}' does not match pattern '{}'", name, value, pattern));
        }
    }

    pub fn validate_string_not_empty(&mut self, name: &str, value: &str) {
        if value.is_empty() {
            self.errors.push(format!("{}: string is empty", name));
        }
    }

    pub fn validate_true(&mut self, name: &str, condition: bool) {
        if !condition {
            self.errors.push(format!("{}: condition is false", name));
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }
}

/// 测试固件集合
pub struct TestFixtures {
    fixtures: Vec<TestFixture>,
}

impl TestFixtures {
    pub fn new() -> Self {
        let fixtures = vec![
            TestFixture {
                description: "标准读秒格式".to_string(),
                pane_content: "* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)".to_string(),
                expected_status: PaneStatus::Active,
                expected_skip_llm: true,
                expected_progress: true,
            },
            TestFixture {
                description: "工具调用".to_string(),
                pane_content: "Tool use: Reading file".to_string(),
                expected_status: PaneStatus::Active,
                expected_skip_llm: false,
                expected_progress: true,
            },
            TestFixture {
                description: "任务完成".to_string(),
                pane_content: "✅ Task completed successfully".to_string(),
                expected_status: PaneStatus::Idle,
                expected_skip_llm: false,
                expected_progress: true,
            },
            TestFixture {
                description: "错误状态".to_string(),
                pane_content: "Error: compilation failed".to_string(),
                expected_status: PaneStatus::Idle,
                expected_skip_llm: false,
                expected_progress: true,
            },
            TestFixture {
                description: "中断状态".to_string(),
                pane_content: "Interrupted by user".to_string(),
                expected_status: PaneStatus::Idle,
                expected_skip_llm: false,
                expected_progress: true,
            },
            TestFixture {
                description: "纯时间计数器".to_string(),
                pane_content: "104s".to_string(),
                expected_status: PaneStatus::Idle,
                expected_skip_llm: true,
                expected_progress: false,
            },
        ];

        Self { fixtures }
    }

    pub fn get_monitor_fixtures(&self) -> &[TestFixture] {
        &self.fixtures
    }

    pub fn get_fixture_by_description(&self, description: &str) -> Option<&TestFixture> {
        self.fixtures.iter().find(|f| f.description == description)
    }
}

/// 测试场景集合
pub struct TestScenarios {
    scenarios: Vec<TestScenario>,
}

impl TestScenarios {
    pub fn new() -> Self {
        let scenarios = vec![
            TestScenario {
                name: "full_monitoring_cycle".to_string(),
                description: "完整的监控周期测试".to_string(),
                steps: vec![
                    TestStep {
                        name: "开始监控".to_string(),
                        action: "initialize_monitoring".to_string(),
                        expected_output: Some("Monitoring started".to_string()),
                        expected_status: Some(PaneStatus::Idle),
                    },
                    TestStep {
                        name: "检测活动".to_string(),
                        action: "detect_activity".to_string(),
                        expected_output: Some("Activity detected".to_string()),
                        expected_status: Some(PaneStatus::Active),
                    },
                    TestStep {
                        name: "处理卡住".to_string(),
                        action: "handle_stuck".to_string(),
                        expected_output: Some("Stuck handled".to_string()),
                        expected_status: Some(PaneStatus::Active),
                    },
                    TestStep {
                        name: "任务完成".to_string(),
                        action: "task_completed".to_string(),
                        expected_output: Some("Task completed".to_string()),
                        expected_status: Some(PaneStatus::Idle),
                    },
                ],
            },
        ];

        Self { scenarios }
    }

    pub fn get_scenario(&self, name: &str) -> Option<&TestScenario> {
        self.scenarios.iter().find(|s| s.name == name)
    }

    pub fn get_all_scenarios(&self) -> &[TestScenario] {
        &self.scenarios
    }
}

/// 测试数据生成器
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn generate_random_terminal_output() -> String {
        let templates = vec![
            "* Herding… (123s · ↑ 8.7k tokens · esc to interrupt)",
            "* Cogitating… (456s · ↓ 12.3k tokens · esc to interrupt)",
            "Tool use: Reading file (789s · 5.6k tokens)",
            "Error: compilation failed",
            "✅ Task completed",
            "Processing... (234s)",
        ];
        
        // 简单的循环选择
        let index = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize % templates.len();
        
        templates[index].to_string()
    }

    pub fn generate_time_series_output(start_time: u32, steps: usize) -> Vec<String> {
        (0..steps)
            .map(|i| {
                format!(
                    "* Herding… ({}s · ↑ {} tokens · esc to interrupt)",
                    start_time + i as u32,
                    8.7 + (i as f32) * 0.1
                )
            })
            .collect()
    }

    pub fn generate_mixed_content() -> String {
        let mut content = String::new();
        let lines = vec![
            "Starting task...",
            "Tool use: Reading configuration",
            "* Cogitating… (100s · ↑ 5.2k tokens · esc to interrupt)",
            "Processing data...",
            "Error: Network timeout",
            "Retrying connection...",
            "* Processing… (150s · ↓ 3.1k tokens · esc to interrupt)",
            "✅ Task completed",
        ];
        
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                content.push('\n');
            }
            content.push_str(line);
        }
        
        content
    }
}

/// 模拟监控服务实现
pub struct MockMonitorServiceImpl {
    pub responses: HashMap<String, PaneStatus>,
}

impl MockMonitorServiceImpl {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    pub fn set_response(&mut self, pane_id: &str, status: PaneStatus) {
        self.responses.insert(pane_id.to_string(), status);
    }
}

#[async_trait]
impl MockMonitorService for MockMonitorServiceImpl {
    async fn check_pane_status(&self, pane_id: &str) -> PaneStatus {
        self.responses.get(pane_id).copied().unwrap_or(PaneStatus::Idle)
    }

    async fn handle_stuck_pane(&self, _pane_id: &str) -> Result<(), String> {
        // 模拟处理卡住的操作
        Ok(())
    }
}

/// 测试断言辅助函数
pub mod assertions {
    use super::*;

    pub fn assert_activity_detection(text: &str, expected: bool) {
        let result = is_claude_active(text);
        assert_eq!(result, expected, "活动检测失败: '{}'", text);
    }

    pub fn assert_skip_llm_call(text: &str, expected: bool) {
        let result = check_if_should_skip_llm_call(text);
        assert_eq!(result, expected, "跳过LLM调用检测失败: '{}'", text);
    }

    pub fn assert_progress_detection(text: &str, expected: bool) {
        let result = has_substantial_progress(text);
        assert_eq!(result, expected, "进展检测失败: '{}'", text);
    }

    pub fn assert_time_extraction(text: &str, expected: Option<u64>) {
        let result = extract_execution_time(text);
        assert_eq!(result, expected, "时间提取失败: '{}'", text);
    }

    pub fn assert_content_change(current: &str, previous: &str, expected: bool) {
        let result = has_substantial_content_change(current, previous);
        assert_eq!(result, expected, "内容变化检测失败: current='{}', previous='{}'", current, previous);
    }
}

/// 测试环境设置
pub mod setup {
    use super::*;

    pub fn create_test_config() -> Config {
        Config::default()
    }

    pub fn create_test_args() -> Args {
        use clap::Parser;
        Args::parse_from(&["claude-watch", "--pane", "%6"])
    }

    pub fn reset_global_state() {
        // 重置全局状态以确保测试独立性
        crate::monitor::reset_time_tracker();
    }
}

/// 测试宏
#[macro_export]
macro_rules! assert_activity {
    ($text:expr, $expected:expr) => {
        assert_eq!($crate::testing::is_claude_active($text), $expected, 
                   "活动检测失败: '{}'", $text);
    };
}

#[macro_export]
macro_rules! assert_skip_llm {
    ($text:expr, $expected:expr) => {
        assert_eq!($crate::testing::check_if_should_skip_llm_call($text), $expected, 
                   "跳过LLM调用检测失败: '{}'", $text);
    };
}

#[macro_export]
macro_rules! assert_progress {
    ($text:expr, $expected:expr) => {
        assert_eq!($crate::testing::has_substantial_progress($text), $expected, 
                   "进展检测失败: '{}'", $text);
    };
}

#[macro_export]
macro_rules! assert_time {
    ($text:expr, $expected:expr) => {
        assert_eq!($crate::testing::extract_execution_time($text), $expected, 
                   "时间提取失败: '{}'", $text);
    };
}

#[macro_export]
macro_rules! test_fixture {
    ($fixture:expr) => {
        let result = $crate::testing::is_claude_active(&$fixture.pane_content);
        assert_eq!(result, $fixture.expected_status == $crate::testing::PaneStatus::Active,
                   "固件测试失败: {} - 预期状态: {:?}, 实际: {}", 
                   $fixture.description, $fixture.expected_status, result);
        
        let skip_result = $crate::testing::check_if_should_skip_llm_call(&$fixture.pane_content);
        assert_eq!(skip_result, $fixture.expected_skip_llm,
                   "固件测试失败: {} - 预期跳过LLM: {}, 实际: {}", 
                   $fixture.description, $fixture.expected_skip_llm, skip_result);
        
        let progress_result = $crate::testing::has_substantial_progress(&$fixture.pane_content);
        assert_eq!(progress_result, $fixture.expected_progress,
                   "固件测试失败: {} - 预期进展: {}, 实际: {}", 
                   $fixture.description, $fixture.expected_progress, progress_result);
    };
}