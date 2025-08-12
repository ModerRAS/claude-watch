use dotenvy::dotenv;
use std::io;
use std::time::Instant;

mod config;
mod args;
mod tmux;
mod activity;
mod llm;
mod monitor;

use config::Config;
use args::Args;
use monitor::run_monitoring_loop;
use clap::Parser;

fn main() -> io::Result<()> {
    dotenv().ok();
    let args = Args::parse();

    // 加载配置
    let mut config = Config::load(&args.config).unwrap_or_else(|_| {
        eprintln!("无法加载配置文件 {}，使用默认配置", args.config);
        Config::default()
    });

    // 使用命令行参数覆盖配置（如果提供）
    if let Some(pane) = &args.pane {
        config.tmux.pane = pane.clone();
    }
    if let Some(backend) = &args.backend {
        config.llm.backend = backend.clone();
    }
    if let Some(interval) = args.interval {
        config.monitoring.interval = interval;
    }
    if let Some(stuck_sec) = args.stuck_sec {
        config.monitoring.stuck_sec = stuck_sec;
    }
    if let Some(max_retry) = args.max_retry {
        config.monitoring.max_retry = max_retry;
    }

    let mut last_active = Instant::now();
    let mut retry_count = 0usize;

    println!("开始监控 Claude Code 在 tmux pane {} 中的状态", config.tmux.pane);
    println!("使用 LLM 后端: {}", config.llm.backend);

    // 主监控循环
    run_monitoring_loop(&config, &mut last_active, &mut retry_count)
}