use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use claude_watch::*;
use claude_watch::testing::*;
use std::time::Duration;

fn bench_activity_detection(c: &mut Criterion) {
    let test_cases = vec![
        "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)",
        "* Cogitating… (169s · ↓ 8.7k tokens · esc to interrupt)",
        "* Processing… (56s · ↑ 2.1k tokens · esc to interrupt)",
        "Tool use: Reading file",
        "Interrupted by user",
        ">",
        "Error: something went wrong",
        "",
    ];
    
    let mut group = c.benchmark_group("activity_detection");
    
    for (i, test_case) in test_cases.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("is_claude_active", i), test_case, |b, content| {
            b.iter(|| activity::is_claude_active(black_box(content)));
        });
    }
    
    group.finish();
}

fn bench_monitor_functions(c: &mut Criterion) {
    let test_content = "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)";
    
    let mut group = c.benchmark_group("monitor_functions");
    
    group.bench_function("extract_execution_time", |b| {
        b.iter(|| monitor::extract_execution_time(black_box(test_content)));
    });
    
    group.bench_function("check_if_should_skip_llm_call", |b| {
        b.iter(|| monitor::check_if_should_skip_llm_call(black_box(test_content)));
    });
    
    group.bench_function("has_substantial_progress", |b| {
        b.iter(|| monitor::has_substantial_progress(black_box(test_content)));
    });
    
    group.bench_function("is_just_time_counter", |b| {
        b.iter(|| monitor::is_just_time_counter(black_box("104s")));
    });
    
    group.bench_function("has_substantial_content_change", |b| {
        b.iter(|| monitor::has_substantial_content_change(
            black_box("* Herding… (100s · ↑ 14.2k tokens · esc to interrupt)"),
            black_box("* Herding… (101s · ↑ 14.2k tokens · esc to interrupt)")
        ));
    });
    
    group.bench_function("extract_core_content", |b| {
        b.iter(|| monitor::extract_core_content(black_box(test_content)));
    });
    
    group.finish();
}

fn bench_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_operations");
    
    group.bench_function("config_default", |b| {
        b.iter(|| config::Config::default());
    });
    
    group.bench_function("config_from_args", |b| {
        let args = args::Args::try_parse_from(&[
            "claude-watch",
            "--pane", "%6",
            "--backend", "openai",
            "--interval", "10",
            "--stuck-sec", "60",
            "--max-retry", "5"
        ]).unwrap();
        
        b.iter(|| config::Config::from_args(black_box(&args)));
    });
    
    // Test YAML serialization/deserialization
    let config = config::Config::default();
    let yaml = serde_yaml::to_string(&config).unwrap();
    
    group.bench_function("config_serialize", |b| {
        b.iter(|| serde_yaml::to_string(black_box(&config)));
    });
    
    group.bench_function("config_deserialize", |b| {
        b.iter(|| serde_yaml::from_str::<config::Config>(black_box(&yaml)));
    });
    
    group.finish();
}

fn bench_args_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("args_parsing");
    
    let test_args = vec![
        vec!["claude-watch"],
        vec!["claude-watch", "--pane", "%6"],
        vec!["claude-watch", "--backend", "openai", "--interval", "10"],
        vec!["claude-watch", "--config", "custom.yaml", "--stuck-sec", "120", "--max-retry", "5"],
        vec!["claude-watch", "-p", "%6", "-b", "ollama", "-i", "5", "-s", "30", "-m", "3"],
    ];
    
    for (i, args) in test_args.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("parse_args", i), args, |b, args_vec| {
            b.iter(|| args::Args::try_parse_from(black_box(args_vec)));
        });
    }
    
    group.finish();
}

fn bench_llm_heuristics(c: &mut Criterion) {
    let test_cases = vec![
        ("✅ All checks passed", llm::TaskStatus::Done),
        ("Error: something went wrong", llm::TaskStatus::Stuck),
        ("* Cogitating… (100s · ↑ 14.2k tokens · esc to interrupt)", llm::TaskStatus::Stuck),
        ("Interrupted by user", llm::TaskStatus::Stuck),
        ("Tool use: Reading file", llm::TaskStatus::Stuck),
        ("Processing...", llm::TaskStatus::Stuck),
    ];
    
    let mut group = c.benchmark_group("llm_heuristics");
    
    for (i, (content, _expected)) in test_cases.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("simple_heuristic_check", i), content, |b, content| {
            b.iter(|| llm::simple_heuristic_check(black_box(content)));
        });
    }
    
    group.finish();
}

fn bench_regex_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("regex_operations");
    
    let time_regex = regex::Regex::new(r"\((\d+)s\)").unwrap();
    let execution_bar_regex = regex::Regex::new(r"\*[^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)").unwrap();
    let tokens_regex = regex::Regex::new(r"(\d+)\s*tokens?").unwrap();
    
    let test_content = "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)";
    
    group.bench_function("time_regex_capture", |b| {
        b.iter(|| time_regex.captures(black_box(test_content)));
    });
    
    group.bench_function("execution_bar_regex_is_match", |b| {
        b.iter(|| execution_bar_regex.is_match(black_box(test_content)));
    });
    
    group.bench_function("tokens_regex_capture", |b| {
        b.iter(|| tokens_regex.captures(black_box(test_content)));
    });
    
    group.finish();
}

fn bench_validation_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_operations");
    
    group.bench_function("pane_validation", |b| {
        b.iter(|| {
            let mut validator = TestValidator::new();
            validator.validate_string_matches_regex("pane", "%6", r"^%\d+$");
            validator.is_valid()
        });
    });
    
    group.bench_function("url_parsing", |b| {
        b.iter(|| llm::parse_ollama_url(black_box("http://localhost:11434")));
    });
    
    group.bench_function("number_range_validation", |b| {
        b.iter(|| {
            let mut validator = TestValidator::new();
            validator.validate_number_range("interval", 5, 1, 3600);
            validator.is_valid()
        });
    });
    
    group.finish();
}

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");
    
    let test_strings = vec![
        "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)",
        "Interrupted by user",
        "Tool use: Reading file /path/to/file",
        "Error: compilation failed",
        "✅ Task completed successfully",
        "Building...",
        "Processing data",
        "Downloading dependencies",
        "Running tests",
        "Generating documentation",
    ];
    
    for (i, test_string) in test_strings.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("string_contains", i), test_string, |b, s| {
            b.iter(|| s.contains(black_box("tokens")));
        });
    }
    
    group.bench_function("string_lines_collection", |b| {
        b.iter(|| {
            let content = "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)\nInterrupted by user\n>";
            let lines: Vec<&str> = content.lines().collect();
            lines.len()
        });
    });
    
    group.bench_function("string_split_whitespace", |b| {
        b.iter(|| {
            let command = "tmux capture-pane -p -t %0";
            let parts: Vec<&str> = command.split_whitespace().collect();
            parts.len()
        });
    });
    
    group.finish();
}

fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");
    
    // Test allocation of large strings
    group.bench_function("large_string_allocation", |b| {
        b.iter(|| {
            let s = "a".repeat(10000);
            s.len()
        });
    });
    
    // Test vector operations
    group.bench_function("vector_operations", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..1000 {
                vec.push(format!("item_{}", i));
            }
            vec.len()
        });
    });
    
    // Test hash map operations
    group.bench_function("hashmap_operations", |b| {
        b.iter(|| {
            use std::collections::HashMap;
            let mut map = HashMap::new();
            for i in 0..1000 {
                map.insert(format!("key_{}", i), format!("value_{}", i));
            }
            map.len()
        });
    });
    
    group.finish();
}

fn bench_integration_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("integration_scenarios");
    
    // Test complete monitoring cycle
    group.bench_function("full_monitoring_cycle", |b| {
        b.iter(|| {
            let content = "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)";
            
            // Simulate complete monitoring workflow
            let _is_active = activity::is_claude_active(content);
            let _time = monitor::extract_execution_time(content);
            let _should_skip = monitor::check_if_should_skip_llm_call(content);
            let _has_progress = monitor::has_substantial_progress(content);
            
            // Simulate config creation
            let args = args::Args::try_parse_from(&["claude-watch", "--pane", "%6"]).unwrap();
            let _config = config::Config::from_args(&args);
            
            // Simulate validation
            let mut validator = TestValidator::new();
            validator.validate_string_matches_regex("pane", "%6", r"^%\d+$");
            validator.is_valid()
        });
    });
    
    // Test configuration parsing and validation
    group.bench_function("config_validation_cycle", |b| {
        b.iter(|| {
            let yaml_content = r#"
pane: "%6"
stuck_sec: 30
interval: 2
llm:
  backend: "openai"
  openai:
    api_key: "test-key"
    api_base: "https://api.openai.com/v1"
    model: "gpt-4"
"#;
            
            let config: config::Config = serde_yaml::from_str(yaml_content).unwrap();
            
            let mut validator = TestValidator::new();
            validator.validate_number_range("interval", config.monitoring.interval, 1, 3600);
            validator.validate_number_range("stuck_sec", config.monitoring.stuck_sec, 5, 7200);
            validator.validate_number_range("max_retry", config.monitoring.max_retry, 1, 100);
            validator.validate_string_matches_regex("pane", &config.tmux.pane, r"^%\d+$");
            
            validator.is_valid()
        });
    });
    
    group.finish();
}

fn bench_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_cases");
    
    // Test with empty strings
    group.bench_function("empty_string_handling", |b| {
        b.iter(|| {
            let _ = activity::is_claude_active("");
            let _ = monitor::extract_execution_time("");
            let _ = monitor::check_if_should_skip_llm_call("");
        });
    });
    
    // Test with very large strings
    let large_content = "a".repeat(100000) + "\n* Herding… (100s · ↑ 14.2k tokens · esc to interrupt)";
    group.bench_function("large_string_handling", |b| {
        b.iter(|| {
            let _ = activity::is_claude_active(&large_content);
            let _ = monitor::extract_execution_time(&large_content);
            let _ = monitor::check_if_should_skip_llm_call(&large_content);
        });
    });
    
    // Test with many regex operations
    group.bench_function("many_regex_operations", |b| {
        b.iter(|| {
            for i in 0..100 {
                let content = format!("* Processing… ({}s · ↓ 2.1k tokens · esc to interrupt)", i);
                let _ = activity::is_claude_active(&content);
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_activity_detection,
    bench_monitor_functions,
    bench_config_operations,
    bench_args_parsing,
    bench_llm_heuristics,
    bench_regex_operations,
    bench_validation_operations,
    bench_string_operations,
    bench_memory_operations,
    bench_integration_scenarios,
    bench_edge_cases
);

criterion_main!(benches);