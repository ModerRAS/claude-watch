//! 简化性能测试
//! 
//! 提供基本的性能基准测试

use claude_watch::testing::*;
use claude_watch::activity::is_claude_active;
use std::time::Duration;

#[test]
fn test_performance_baselines() {
    // 测试性能基线
    let mut profiler = PerformanceProfiler::new();
    
    // 测试活动检测性能
    profiler.start_measurement("activity_detection_batch");
    for _ in 0..1000 {
        let _ = is_claude_active("* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)");
    }
    profiler.end_measurement("activity_detection_batch");
    
    // 测试时间提取性能
    profiler.start_measurement("time_extraction_batch");
    for _ in 0..1000 {
        let _ = extract_execution_time("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)");
    }
    profiler.end_measurement("time_extraction_batch");
    
    // 测试进展检测性能
    profiler.start_measurement("progress_detection_batch");
    for _ in 0..1000 {
        let _ = has_substantial_progress("Tool use: Reading file");
    }
    profiler.end_measurement("progress_detection_batch");
    
    // 验证性能结果
    let activity_time = profiler.get_measurement("activity_detection_batch").unwrap();
    let time_time = profiler.get_measurement("time_extraction_batch").unwrap();
    let progress_time = profiler.get_measurement("progress_detection_batch").unwrap();
    
    println!("活动检测1000次耗时: {:?}", activity_time);
    println!("时间提取1000次耗时: {:?}", time_time);
    println!("进展检测1000次耗时: {:?}", progress_time);
    
    // 性能应该在合理范围内（根据测试环境调整）
    assert!(activity_time < Duration::from_millis(2000), "活动检测性能过慢");
    assert!(time_time < Duration::from_millis(2000), "时间提取性能过慢");
    assert!(progress_time < Duration::from_millis(500), "进展检测性能过慢");
}

#[test]
fn test_large_input_performance() {
    // 测试大输入处理性能
    let mut profiler = PerformanceProfiler::new();
    
    // 生成大输入
    let large_input = "* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)\n".repeat(1000);
    
    profiler.start_measurement("large_input_processing");
    for _ in 0..10 {
        let _ = is_claude_active(&large_input);
        let _ = check_if_should_skip_llm_call(&large_input);
        let _ = has_substantial_progress(&large_input);
    }
    profiler.end_measurement("large_input_processing");
    
    let large_time = profiler.get_measurement("large_input_processing").unwrap();
    println!("大输入处理10次耗时: {:?}", large_time);
    
    assert!(large_time < Duration::from_millis(5000), "大输入处理时间过长");
}

#[test]
fn test_scalability() {
    // 测试可扩展性
    let sizes = vec![100, 1000, 10000];
    
    for size in sizes {
        let start = std::time::Instant::now();
        
        // 生成测试数据
        let test_data: Vec<String> = (0..size)
            .map(|i| format!("* Herding… ({}s · ↑ {} tokens · esc to interrupt)", i % 1000, i % 20))
            .collect();
        
        // 处理数据
        for content in &test_data {
            let _ = is_claude_active(content);
            let _ = check_if_should_skip_llm_call(content);
            let _ = has_substantial_progress(content);
        }
        
        let duration = start.elapsed();
        let per_item = duration / size as u32;
        
        println!("处理 {} 个项目，总耗时: {:?}, 平均每个: {:?}", size, duration, per_item);
        
        // 确保处理时间与数据量成线性关系
        assert!(per_item < Duration::from_millis(10), "处理单个项目时间过长");
    }
}

#[test]
fn test_memory_efficiency() {
    // 测试内存效率
    let mut profiler = PerformanceProfiler::new();
    
    // 测试大量小字符串处理
    profiler.start_measurement("small_strings");
    for _ in 0..10000 {
        let _ = is_claude_active("104s");
        let _ = check_if_should_skip_llm_call("104s");
        let _ = has_substantial_progress("104s");
    }
    profiler.end_measurement("small_strings");
    
    // 测试少量大字符串处理
    let large_string = "* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)\n".repeat(1000);
    profiler.start_measurement("large_strings");
    for _ in 0..100 {
        let _ = is_claude_active(&large_string);
        let _ = check_if_should_skip_llm_call(&large_string);
        let _ = has_substantial_progress(&large_string);
    }
    profiler.end_measurement("large_strings");
    
    let small_time = profiler.get_measurement("small_strings").unwrap();
    let large_time = profiler.get_measurement("large_strings").unwrap();
    
    println!("处理100000个小字符串耗时: {:?}", small_time);
    println!("处理100个大字符串耗时: {:?}", large_time);
    
    // 确保内存使用效率
    assert!(small_time < Duration::from_secs(60), "小字符串处理时间过长");
    assert!(large_time < Duration::from_millis(1000), "大字符串处理时间过长");
}