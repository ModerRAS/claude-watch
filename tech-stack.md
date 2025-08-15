# Claude Watch 测试技术栈和工具选择

## 技术栈概述

claude-watch项目采用Rust生态系统中的测试工具，构建了完整的测试体系。本文档详细说明了测试技术栈的选择理由和配置方案。

## 核心测试技术栈

### 单元测试框架

| 技术 | 选择 | 版本 | 理由 |
|------|------|------|------|
| **Rust内置测试** | ✓ | stable | Rust内置，无需额外依赖，与语言深度集成 |
| **tokio-test** | ✓ | 0.4 | 异步测试支持，与tokio生态系统兼容 |
| **serial_test** | ✓ | 2.0 | 串行测试支持，避免并发测试冲突 |

**配置示例**:
```toml
# Cargo.toml
[dev-dependencies]
tokio-test = "0.4"
serial_test = "2.0"
```

**使用示例**:
```rust
#[cfg(test)]
mod tests {
    use tokio_test::block_on;
    use serial_test::serial;
    
    #[test]
    #[serial] // 确保测试串行执行
    fn test_activity_detection() {
        let result = is_claude_active("* Herding… (343s · ↑ 14.2k tokens)");
        assert!(result);
    }
    
    #[tokio::test]
    #[serial]
    async fn test_async_llm_call() {
        let result = ask_llm_final_status("test", "mock", &config).await;
        assert!(result.is_ok());
    }
}
```

### Mock和Stub框架

| 技术 | 选择 | 版本 | 理由 |
|------|------|------|------|
| **Mockall** | ✓ | 0.12 | 强大的Mock生成，trait mocking |
| **fake** | ✓ | 2.9 | 假数据生成，测试数据构造 |
| **mockito** | ✓ | 1.4 | HTTP请求Mock，LLM API测试 |

**配置示例**:
```toml
# Cargo.toml
[dev-dependencies]
mockall = "0.12"
fake = { version = "2.9", features = ["derive", "uuid"] }
mockito = "1.4"
```

**使用示例**:
```rust
use mockall::*;
use fake::{Fake, Faker};

// Mock trait定义
#[automock]
pub trait TmuxClient {
    fn capture(&mut self, pane: &str) -> String;
    fn send_keys(&mut self, keys: &str, pane: &str);
}

// Mock使用
#[test]
fn test_monitoring_with_mock() {
    let mut mock_tmux = MockTmuxClient::new();
    
    // 设置期望
    mock_tmux.expect_capture()
        .returning(|_| "* Herding… (343s · ↑ 14.2k tokens)".to_string());
    
    mock_tmux.expect_send_keys()
        .withf(|keys, _| keys == "Retry")
        .returning(|_, _| ());
    
    // 执行测试
    let result = monitor_with_mock(&mut mock_tmux);
    assert!(result);
}
```

### 断言和验证库

| 技术 | 选择 | 版本 | 理由 |
|------|------|------|------|
| **assert_cmd** | ✓ | 2.0 | CLI命令测试，集成测试 |
| **assert_fs** | ✓ | 1.1 | 文件系统断言，配置文件测试 |
| **predicates** | ✓ | 3.0 | 复杂条件断言，输出验证 |

**配置示例**:
```toml
# Cargo.toml
[dev-dependencies]
assert_cmd = "2.0"
assert_fs = "1.1"
predicates = "3.0"
```

**使用示例**:
```rust
use assert_cmd::Command;
use assert_fs::NamedTempFile;
use predicates::prelude::*;

#[test]
fn test_cli_integration() {
    let mut cmd = Command::cargo_bin("claude-watch").unwrap();
    
    // 创建临时配置文件
    let config_file = NamedTempFile::new("config.yaml").unwrap();
    let config_content = r#"
    tmux:
      pane: "%0"
    monitoring:
      interval: 2
      stuck_sec: 30
    "#;
    std::fs::write(config_file.path(), config_content).unwrap();
    
    // 执行命令并验证
    cmd.arg("--config")
       .arg(config_file.path())
       .arg("--pane")
       .arg("%0")
       .assert()
       .success()
       .stdout(predicate::str::contains("Claude Watch started"));
}
```

## 性能测试技术栈

### 基准测试框架

| 技术 | 选择 | 版本 | 理由 |
|------|------|------|------|
| **Criterion.rs** | ✓ | 0.5 | Rust标准基准测试框架，统计功能强大 |
| **divan** | ✓ | 0.1 | 轻量级基准测试，适合快速测试 |

**配置示例**:
```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
divan = "0.1"

[[bench]]
name = "activity_detection"
harness = false

[[bench]]
name = "monitor_performance"
harness = false
```

**使用示例**:
```rust
// benches/activity_detection.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_activity_detection(c: &mut Criterion) {
    let test_cases = vec![
        ("standard", "* Herding… (343s · ↑ 14.2k tokens)"),
        ("tool_call", "Tool use: Reading file"),
        ("empty", ""),
    ];
    
    let mut group = c.benchmark_group("activity_detection");
    
    for (name, input) in test_cases {
        group.bench_with_input(BenchmarkId::new("is_claude_active", name), input, |b, input| {
            b.iter(|| is_claude_active(input));
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_activity_detection);
criterion_main!(benches);
```

### 内存分析工具

| 技术 | 选择 | 版本 | 理由 |
|------|------|------|------|
| **dhat** | ✓ | 0.3 | 堆分析，内存泄漏检测 |
| **alloc-counter** | ✓ | 0.1 | 分配计数，内存使用跟踪 |
| **memory-stats** | ✓ | 1.0 | 内存统计，性能监控 |

**配置示例**:
```toml
# Cargo.toml
[dev-dependencies]
dhat = "0.3"
alloc-counter = "0.1"

[features]
dhat-heap = ["dhat"]
```

**使用示例**:
```rust
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[test]
fn test_memory_usage() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    
    // 执行测试代码
    for _ in 0..1000 {
        is_claude_active("* Herding… (343s · ↑ 14.2k tokens)");
    }
    
    #[cfg(feature = "dhat-heap")]
    println!("Memory usage: {}", dhat::HeapStats::get().allocated_bytes);
}
```

## 测试覆盖率工具

### 代码覆盖率

| 技术 | 选择 | 版本 | 理由 |
|------|------|------|------|
| **cargo-llvm-cov** | ✓ | 0.5 | LLVM覆盖率，速度快，精度高 |
| **tarpaulin** | ✓ | 0.25 | Linux/Mac覆盖率，支持分支覆盖 |
| **grcov** | ✓ | 0.8 | 多格式输出，CI/CD友好 |

**配置示例**:
```toml
# Cargo.toml
[dev-dependencies]
cargo-llvm-cov = "0.5"

# .cargo/config.toml
[alias]
cov = "llvm-cov --lcov --output-path coverage-report.lcov"
```

**使用示例**:
```bash
# 生成覆盖率报告
cargo llvm-cov --lcov --output-path coverage-report.lcov
cargo tarpaulin --out Html --output-dir coverage/
```

### 覆盖率配置

```yaml
# .github/workflows/coverage.yml
name: Coverage
on: [push, pull_request]
jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
          components: llvm-tools-preview
      
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      
      - name: Generate coverage report
        run: cargo llvm-cov --lcov --output-path coverage-report.lcov
      
      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          file: ./coverage-report.lcov
```

## 集成测试技术栈

### 容器化测试

| 技术 | 选择 | 版本 | 理由 |
|------|------|------|------|
| **testcontainers** | ✓ | 0.15 | 测试容器，环境隔离 |
| **docker-api** | ✓ | 0.14 | Docker API，容器控制 |
| **nix** | ✓ | 0.26 | Nix包管理，环境一致性 |

**配置示例**:
```toml
# Cargo.toml
[dev-dependencies]
testcontainers = "0.15"
tokio = { version = "1.0", features = ["full"] }
```

**使用示例**:
```rust
use testcontainers::{clients, images::generic::GenericImage};

#[tokio::test]
async fn test_with_real_tmux() {
    let docker = clients::Cli::default();
    
    // 创建tmux容器
    let tmux_container = docker.run(GenericImage::new("tmux:latest"));
    
    // 执行测试
    let result = test_with_real_tmux_container(&tmux_container).await;
    assert!(result);
}
```

### 端到端测试

| 技术 | 选择 | 版本 | 理由 |
|------|------|------|------|
| **rstest** | ✓ | 0.18 | 参数化测试，数据驱动 |
| **test-case** | ✓ | 3.1 | 测试用例装饰器，简化测试 |
| **once_cell** | ✓ | 1.19 | 懒加载，共享测试资源 |

**配置示例**:
```toml
# Cargo.toml
[dev-dependencies]
rstest = "0.18"
test-case = "3.1"
once_cell = "1.19"
```

**使用示例**:
```rust
use rstest::rstest;
use test_case::test_case;
use once_cell::sync::Lazy;

// 共享测试资源
static TEST_CONFIG: Lazy<Config> = Lazy::new(|| {
    Config {
        tmux: TmuxConfig { pane: "%0".to_string() },
        monitoring: MonitoringConfig {
            interval: 1,
            stuck_sec: 2,
            max_retry: 1,
        },
        llm: LlmConfig::default(),
    }
});

#[rstest]
#[case("* Herding… (343s · ↑ 14.2k tokens)", true)]
#[case("Tool use: Reading file", true)]
#[case("Interrupted by user\n>", false)]
#[case("", false)]
fn test_activity_detection_cases(#[case] input: &str, #[case] expected: bool) {
    assert_eq!(is_claude_active(input), expected);
}

#[test_case("standard")]
#[test_case("tool_call")]
#[test_case("empty")]
fn test_performance_scenarios(scenario: &str) {
    let input = get_test_input(scenario);
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        is_claude_active(input);
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Performance too slow for {}", scenario);
}
```

## 测试数据管理

### 测试数据生成

| 技术 | 选择 | 版本 | 理由 |
|------|------|------|------|
| **fake** | ✓ | 2.9 | 假数据生成，测试数据构造 |
| **rand** | ✓ | 0.8 | 随机数生成，测试变体 |
| **proptest** | ✓ | 1.4 | 属性测试，边界情况 |

**配置示例**:
```toml
# Cargo.toml
[dev-dependencies]
fake = { version = "2.9", features = ["derive", "uuid", "chrono"] }
rand = "0.8"
proptest = "1.4"
```

**使用示例**:
```rust
use fake::{Fake, Faker};
use fake::faker::lorem::en::*;
use proptest::prelude::*;

// 生成测试数据
fn generate_test_data() -> Vec<String> {
    let mut data = Vec::new();
    
    // 生成标准格式
    for _ in 0..10 {
        let time: u64 = (1..1000).fake();
        let tokens: f64 = (1.0..100.0).fake();
        let status: String = Sentence(1..3).fake();
        
        data.push(format!("* {}… ({}s · ↑ {:.1}k tokens)", status, time, tokens));
    }
    
    data
}

// 属性测试
proptest! {
    #[test]
    fn test_activity_detection_property(
        time in 1u64..1000u64,
        tokens in 1.0f64..100.0f64,
        has_activity in prop::bool::ANY,
    ) {
        let input = if has_activity {
            format!("* Herding… ({}s · ↑ {:.1}k tokens)", time, tokens)
        } else {
            "Just some text".to_string()
        };
        
        let result = is_claude_active(&input);
        
        if has_activity {
            assert!(result, "Should detect activity for: {}", input);
        } else {
            assert!(!result, "Should not detect activity for: {}", input);
        }
    }
}
```

### 测试数据存储

```yaml
# test_data/scenarios.yaml
scenarios:
  - name: "normal_working"
    description: "Claude Code正常工作状态"
    test_cases:
      - input: "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)"
        expected: true
        category: "standard_format"
      - input: "* Cogitating… (169s · ↓ 8.7k tokens · esc to interrupt)"
        expected: true
        category: "standard_format"
  
  - name: "tool_calls"
    description: "工具调用状态"
    test_cases:
      - input: "Tool use: Reading file"
        expected: true
        category: "tool_call"
      - input: "Calling tool: function_name"
        expected: true
        category: "tool_call"
  
  - name: "edge_cases"
    description: "边界情况"
    test_cases:
      - input: ""
        expected: false
        category: "empty"
      - input: "* Herding… (not-a-number s · tokens)"
        expected: false
        category: "invalid_format"
```

## CI/CD 集成

### GitHub Actions 配置

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust-version: [stable, beta]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust-version }}
          override: true
          components: rustfmt, clippy
      
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Install dependencies
        run: |
          cargo install cargo-llvm-cov
          cargo install cargo-tarpaulin
      
      - name: Format check
        run: cargo fmt --all -- --check
      
      - name: Clippy lints
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Generate coverage
        run: cargo llvm-cov --lcov --output-path coverage-report.lcov
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          file: ./coverage-report.lcov
      
      - name: Run benchmarks
        run: cargo bench --all-features
      
      - name: Upload benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: benchmark.json
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

### 本地开发工具

```bash
#!/bin/bash
# scripts/test.sh

set -e

echo "🧪 Running all tests..."

# 格式检查
echo "📝 Checking formatting..."
cargo fmt --all -- --check

# Lint检查
echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# 单元测试
echo "🧪 Running unit tests..."
cargo test --lib

# 集成测试
echo "🔗 Running integration tests..."
cargo test --test "*"

# 文档测试
echo "📚 Running doc tests..."
cargo test --doc

# 覆盖率报告
echo "📊 Generating coverage report..."
cargo llvm-cov --lcov --output-path coverage-report.lcov

# 基准测试
echo "⚡ Running benchmarks..."
cargo bench --all-features

echo "✅ All tests passed!"
```

## 质量保证工具

### 静态分析

| 工具 | 用途 | 配置 |
|------|------|------|
| **clippy** | Lint检查 | `cargo clippy` |
| **rustfmt** | 代码格式化 | `cargo fmt` |
| **cargo-deny** | 依赖审计 | `cargo deny check` |
| **cargo-audit** | 安全检查 | `cargo audit` |

**配置示例**:
```toml
# .cargo/config.toml
[alias]
lint = "clippy --all-targets --all-features -- -D warnings"
format = "fmt --all"
audit = "audit"
deny = "deny check"
```

### 测试质量检查

```rust
// tests/quality_checks.rs
#[test]
fn test_all_tests_have_assertions() {
    use std::fs;
    use std::path::Path;
    
    let test_files = vec![
        "tests/activity_detection.rs",
        "tests/monitor_logic.rs",
        "tests/config_tests.rs",
    ];
    
    for file in test_files {
        let content = fs::read_to_string(file).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut has_assertions = false;
        let mut in_test = false;
        
        for line in lines {
            if line.trim().starts_with("#[test]") {
                in_test = true;
                has_assertions = false;
            } else if in_test && line.trim().starts_with("fn ") {
                // 新的函数，重置状态
                in_test = false;
            } else if in_test && (line.contains("assert!") || line.contains("assert_eq") || line.contains("assert_ne")) {
                has_assertions = true;
            }
        }
        
        assert!(has_assertions, "Test file {} should have assertions", file);
    }
}
```

## 性能监控

### 持续性能监控

```yaml
# .github/workflows/performance.yml
name: Performance
on:
  schedule:
    - cron: '0 0 * * *'  # 每天运行
  push:
    branches: [main]

jobs:
  performance:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Run benchmarks
        run: |
          cargo bench --all-features -- --output-format bencher | tee benchmark.txt
      
      - name: Compare with baseline
        run: |
          # 这里可以添加与基准性能比较的逻辑
          echo "Benchmark results:"
          cat benchmark.txt
      
      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: benchmark.txt
```

## 总结

claude-watch项目的测试技术栈选择基于以下原则：

1. **实用主义**: 选择Rust生态系统中最成熟、最广泛使用的工具
2. **全面覆盖**: 从单元测试到E2E测试，从性能测试到安全检查
3. **开发体验**: 工具易于使用，集成良好，反馈及时
4. **CI/CD友好**: 所有工具都适合在自动化环境中运行
5. **可维护性**: 配置简单，文档丰富，社区活跃

这个技术栈为claude-watch项目提供了强大的测试保障，确保代码质量和系统稳定性。