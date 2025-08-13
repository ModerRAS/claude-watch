pub mod activity;
pub mod config;
pub mod logger;
pub mod monitor;
pub mod llm;
pub mod tmux;
pub mod args;

// 重新导出主要的公共接口
pub use activity::is_claude_active;
pub use monitor::{has_substantial_progress, is_just_time_counter, check_if_should_skip_llm_call, extract_execution_time, is_time_increasing};

// 导出日志宏
#[macro_use]
extern crate log;

// 定义日志宏
#[macro_export]
macro_rules! log_content_change {
    ($pane:expr, $details:expr) => {
        crate::logger::monitor_logger().log_content_change($pane, $details);
    };
}

#[macro_export]
macro_rules! log_stuck_detection {
    ($pane:expr, $stuck_sec:expr) => {
        crate::logger::monitor_logger().log_stuck_detection($pane, $stuck_sec);
    };
}

#[macro_export]
macro_rules! log_llm_judgment {
    ($status:expr) => {
        crate::logger::monitor_logger().log_llm_judgment($status, None);
    };
    ($status:expr, $confidence:expr) => {
        crate::logger::monitor_logger().log_llm_judgment($status, Some($confidence));
    };
}

#[macro_export]
macro_rules! log_activation_attempt {
    ($method:expr, $success:expr) => {
        crate::logger::monitor_logger().log_activation_attempt($method, $success);
    };
}

#[macro_export]
macro_rules! log_completion_monitoring {
    ($pane:expr, $check_count:expr) => {
        crate::logger::monitor_logger().log_completion_monitoring($pane, $check_count);
    };
}

#[macro_export]
macro_rules! log_error {
    ($context:expr, $error:expr) => {
        crate::logger::monitor_logger().log_error($context, $error);
    };
}

#[macro_export]
macro_rules! log_warning {
    ($context:expr, $warning:expr) => {
        crate::logger::monitor_logger().log_warning($context, $warning);
    };
}
pub use llm::{ask_llm_for_activation, ask_llm_final_status, TaskStatus};
pub use config::{Config, LlmConfig, OpenAiConfig, OpenRouterConfig, OllamaConfig};