pub mod activity;
pub mod config;
pub mod monitor;
pub mod llm;
pub mod tmux;
pub mod args;

// 重新导出主要的公共接口
pub use activity::is_claude_active;
pub use monitor::{has_substantial_progress, is_just_time_counter, check_if_should_skip_llm_call, extract_execution_time, is_time_increasing};
pub use llm::{ask_llm_for_activation, ask_llm_final_status, TaskStatus};