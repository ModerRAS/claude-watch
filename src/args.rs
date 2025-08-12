use clap::Parser;

/// 命令行参数配置
/// 
/// 简化实现：使用 clap 解析命令行参数，替代环境变量
/// 支持配置文件路径、tmux 窗格 ID、LLM 后端等参数
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.yaml")]
    pub config: String,

    /// tmux pane ID (例如 %0 或 mysess:1.0)
    #[arg(short, long)]
    pub pane: Option<String>,

    /// LLM 后端选择 [ollama, openai, openrouter, none]
    #[arg(short, long)]
    pub backend: Option<String>,

    /// 检查间隔(秒) [默认: 5]
    #[arg(short, long)]
    pub interval: Option<u64>,

    /// 无变化多久算卡住(秒) [默认: 60]
    #[arg(short, long)]
    pub stuck_sec: Option<u64>,

    /// 最大重试次数 [默认: 10]
    #[arg(short, long)]
    pub max_retry: Option<usize>,
}