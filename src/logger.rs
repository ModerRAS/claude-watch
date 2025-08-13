use std::io::{self, Write};
use chrono::Local;
use log::{Level, LevelFilter, SetLoggerError};

/// 自定义日志器，支持颜色输出和结构化格式
pub struct ClaudeLogger {
    level: LevelFilter,
    use_colors: bool,
}

impl ClaudeLogger {
    pub fn new(level: LevelFilter, use_colors: bool) -> Self {
        Self {
            level,
            use_colors,
        }
    }
}

impl log::Log for ClaudeLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let level = record.level();
            let (level_str, color_code) = match level {
                Level::Error => ("ERROR", "\x1b[31m"), // 红色
                Level::Warn => ("WARN ", "\x1b[33m"),  // 黄色
                Level::Info => ("INFO ", "\x1b[32m"), // 绿色
                Level::Debug => ("DEBUG", "\x1b[36m"), // 蓝色
                Level::Trace => ("TRACE", "\x1b[35m"), // 紫色
            };

            // 构建结构化日志格式
            let formatted = if self.use_colors {
                format!(
                    "{} [{}] {} | {}{}: {}{}\x1b[0m",
                    timestamp,
                    record.target(),
                    level_str,
                    color_code,
                    level_str,
                    color_code,
                    record.args()
                )
            } else {
                format!(
                    "{} [{}] {} | {}: {}",
                    timestamp,
                    record.target(),
                    level_str,
                    level_str,
                    record.args()
                )
            };

            // 确保原子性写入
            let mut stdout = io::stdout();
            let _ = stdout.write_all(formatted.as_bytes());
            let _ = stdout.write_all(b"\n");
            let _ = stdout.flush();
        }
    }
}

/// 初始化日志系统
pub fn init_logger(level: LevelFilter, use_colors: bool) -> Result<(), SetLoggerError> {
    let logger = ClaudeLogger::new(level, use_colors);
    log::set_logger(&logger)
}

/// 记录监控相关事件（带上下文信息）
pub struct MonitorLogger {
    level: LevelFilter,
    use_colors: bool,
}

impl MonitorLogger {
    pub fn new(level: LevelFilter, use_colors: bool) -> Self {
        Self {
            level,
            use_colors,
        }
    }

    /// 记录内容变化事件
    pub fn log_content_change(&self, pane: &str, details: &str) {
        if self.level >= Level::Info {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let msg = format!(
                "{} [monitor] 🔄 INFO  | 检测到内容变化 | pane: {} | {}",
                timestamp, pane, details
            );
            self.colored_print(&msg);
        }
    }

    /// 记录卡住检测事件
    pub fn log_stuck_detection(&self, pane: &str, stuck_sec: u64) {
        if self.level >= Level::Info {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let msg = format!(
                "{} [monitor] ⏸️  INFO  | 卡住检测触发 | pane: {} | 停止工作超过 {} 秒",
                timestamp, pane, stuck_sec
            );
            self.colored_print(&msg);
        }
    }

    /// 记录LLM状态判断结果
    pub fn log_llm_judgment(&self, status: &str, confidence: Option<f64>) {
        if self.level >= Level::Info {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let confidence_str = confidence.map(|c| format!("(置信度: {:.1}%)", c)).unwrap_or_default();
            let msg = format!(
                "{} [llm] 🤖 INFO  | LLM状态判断 | 状态: {} {}",
                timestamp, status, confidence_str
            );
            self.colored_print(&msg);
        }
    }

    /// 记录激活尝试
    pub fn log_activation_attempt(&self, method: &str, success: bool) {
        if self.level >= Level::Info {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let status = if success { "✅ 成功" } else { "❌ 失败" };
            let msg = format!(
                "{} [activation] 🚀 INFO  | 激活尝试 | 方法: {} | 结果: {}",
                timestamp, method, status
            );
            self.colored_print(&msg);
        }
    }

    /// 记录完成状态监控
    pub fn log_completion_monitoring(&self, pane: &str, check_count: usize) {
        if self.level >= Level::Info {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let msg = format!(
                "{} [completion] 💤 INFO  | 完成状态监控 | pane: {} | 检查次数: {}",
                timestamp, pane, check_count
            );
            self.colored_print(&msg);
        }
    }

    /// 记录错误事件
    pub fn log_error(&self, context: &str, error: &str) {
        if self.level >= Level::Error {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let msg = format!(
                "{} [error] ❌ ERROR | {} | 错误: {}",
                timestamp, context, error
            );
            self.colored_print(&msg);
        }
    }

    /// 记录警告事件
    pub fn log_warning(&self, context: &str, warning: &str) {
        if self.level >= Level::Warn {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let msg = format!(
                "{} [warning] ⚠️  WARN | {} | 警告: {}",
                timestamp, context, warning
            );
            self.colored_print(&msg);
        }
    }

    fn colored_print(&self, msg: &str) {
        if self.use_colors {
            println!("{}", msg);
        } else {
            // 去除颜色代码
            let clean_msg = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap().replace_all(msg, "");
            println!("{}", clean_msg);
        }
    }
}

impl log::Log for ClaudeLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let level = record.level();
            let (level_str, color_code) = match level {
                Level::Error => ("ERROR", "\x1b[31m"), // 红色
                Level::Warn => ("WARN ", "\x1b[33m"),  // 黄色
                Level::Info => ("INFO ", "\x1b[32m"), // 绿色
                Level::Debug => ("DEBUG", "\x1b[36m"), // 蓝色
                Level::Trace => ("TRACE", "\x1b[35m"), // 紫色
            };

            // 构建结构化日志格式
            let formatted = if self.use_colors {
                format!(
                    "{} [{}] {} | {}{}: {}{}\x1b[0m",
                    timestamp,
                    record.target(),
                    level_str,
                    color_code,
                    level_str,
                    color_code,
                    record.args()
                )
            } else {
                format!(
                    "{} [{}] {} | {}: {}",
                    timestamp,
                    record.target(),
                    level_str,
                    level_str,
                    record.args()
                )
            };

            // 确保原子性写入
            let mut stdout = io::stdout();
            let _ = stdout.write_all(formatted.as_bytes());
            let _ = stdout.write_all(b"\n");
            let _ = stdout.flush();
        }
    }

    fn flush(&self) {
        use std::io::Write;
        let _ = io::stdout().flush();
    }
}

/// 全局监控日志器实例
static mut GLOBAL_MONITOR_LOGGER: Option<MonitorLogger> = None;

/// 初始化全局监控日志器
pub fn init_monitor_logger(level: LevelFilter, use_colors: bool) {
    unsafe {
        GLOBAL_MONITOR_LOGGER = Some(MonitorLogger::new(level, use_colors));
    }
}

/// 获取全局监控日志器
pub fn monitor_logger() -> &'static MonitorLogger {
    unsafe {
        GLOBAL_MONITOR_LOGGER.as_ref().expect("Monitor logger not initialized")
    }
}

