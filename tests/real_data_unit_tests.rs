//! 基于真实Claude Code界面数据的单元测试
//! 
//! 使用从真实tmux会话中捕获的Claude Code界面数据进行测试

use std::fs;
use std::path::Path;
use std::time::Duration;
use claude_watch::testing::*;
use claude_watch::activity::is_claude_active;

/// 测试数据目录
const TEST_DATA_DIR: &str = "test_data/claude_interfaces";

/// 从文件加载真实界面数据
fn load_interface_data(filename: &str) -> String {
    let path = Path::new(TEST_DATA_DIR).join(filename);
    fs::read_to_string(path).expect(&format!("无法读取测试数据文件: {}", filename))
}

/// 测试从文件加载的真实界面数据
#[test]
fn test_philosophising_state_interface() {
    let interface_data = load_interface_data("philosophising_state.txt");
    
    println!("=== 测试Philosophising状态界面 ===");
    println!("界面内容:\n{}", interface_data);
    
    // 基本验证
    assert!(!interface_data.is_empty(), "界面数据不应为空");
    assert!(interface_data.contains("Philosophising"), "应该包含Philosophising状态");
    assert!(interface_data.contains("tokens"), "应该包含tokens计数");
    assert!(interface_data.contains("esc to interrupt"), "应该包含中断提示");
    
    // 活动检测
    let is_active = is_claude_active(&interface_data);
    assert!(is_active, "Philosophising状态应该被识别为活动状态");
    println!("✅ 活动检测正确");
    
    // 时间提取
    let time = extract_execution_time(&interface_data);
    assert!(time.is_some(), "应该能够提取时间");
    println!("✅ 时间提取正确: {:?}", time);
    
    // 跳过LLM调用逻辑
    let should_skip = check_if_should_skip_llm_call(&interface_data);
    assert!(should_skip, "Philosophising状态应该跳过LLM调用");
    println!("✅ 跳过LLM调用逻辑正确");
    
    // 进展检测
    let has_progress = has_substantial_progress(&interface_data);
    assert!(has_progress, "Philosophising状态应该有实质性进展");
    println!("✅ 进展检测正确");
}

/// 测试用户输入状态界面
#[test]
fn test_user_input_state_interface() {
    let interface_data = load_interface_data("user_input_state.txt");
    
    println!("\n=== 测试用户输入状态界面 ===");
    println!("界面内容:\n{}", interface_data);
    
    // 基本验证
    assert!(!interface_data.is_empty(), "界面数据不应为空");
    assert!(interface_data.contains("echo 'test message'"), "应该包含用户输入");
    assert!(interface_data.contains("Philosophising"), "应该包含Claude Code状态");
    
    // 活动检测
    let is_active = is_claude_active(&interface_data);
    assert!(is_active, "用户输入状态应该被识别为活动状态");
    println!("✅ 活动检测正确");
    
    // 时间提取
    let time = extract_execution_time(&interface_data);
    assert!(time.is_some(), "应该能够提取时间");
    println!("✅ 时间提取正确: {:?}", time);
    
    // 跳过LLM调用逻辑
    let should_skip = check_if_should_skip_llm_call(&interface_data);
    assert!(should_skip, "用户输入状态应该跳过LLM调用");
    println!("✅ 跳过LLM调用逻辑正确");
}

/// 测试处理响应状态界面
#[test]
fn test_processing_response_interface() {
    let interface_data = load_interface_data("processing_response.txt");
    
    println!("\n=== 测试处理响应状态界面 ===");
    println!("界面内容:\n{}", interface_data);
    
    // 基本验证
    assert!(!interface_data.is_empty(), "界面数据不应为空");
    assert!(interface_data.contains("echo 'test message'"), "应该包含用户输入");
    assert!(interface_data.contains(">"), "应该包含命令提示符");
    
    // 活动检测
    let is_active = is_claude_active(&interface_data);
    println!("活动检测结果: {}", is_active);
    
    // 时间提取
    let time = extract_execution_time(&interface_data);
    println!("时间提取结果: {:?}", time);
    
    // 跳过LLM调用逻辑
    let should_skip = check_if_should_skip_llm_call(&interface_data);
    println!("跳过LLM调用结果: {}", should_skip);
    
    // 进展检测
    let has_progress = has_substantial_progress(&interface_data);
    println!("进展检测结果: {}", has_progress);
}

/// 测试界面尾部数据
#[test]
fn test_philosophising_tail_interface() {
    let interface_data = load_interface_data("philosophising_tail.txt");
    
    println!("\n=== 测试Philosophising状态尾部界面 ===");
    println!("界面内容:\n{}", interface_data);
    
    // 基本验证
    assert!(!interface_data.is_empty(), "界面数据不应为空");
    assert!(interface_data.contains("Philosophising"), "应该包含Philosophising状态");
    assert!(interface_data.contains("tokens"), "应该包含tokens计数");
    
    // 活动检测
    let is_active = is_claude_active(&interface_data);
    assert!(is_active, "Philosophising尾部状态应该被识别为活动状态");
    println!("✅ 活动检测正确");
    
    // 时间提取
    let time = extract_execution_time(&interface_data);
    assert!(time.is_some(), "应该能够提取时间");
    println!("✅ 时间提取正确: {:?}", time);
    
    // 跳过LLM调用逻辑
    let should_skip = check_if_should_skip_llm_call(&interface_data);
    assert!(should_skip, "Philosophising尾部状态应该跳过LLM调用");
    println!("✅ 跳过LLM调用逻辑正确");
}

/// 测试时间递增检测功能
#[test]
fn test_time_increasing_with_real_data() {
    setup::reset_global_state();
    
    let pane_id = "real_test_pane";
    
    // 加载两个不同时间点的界面数据
    let interface1 = load_interface_data("philosophising_state.txt");
    let interface2 = load_interface_data("user_input_state.txt");
    
    // 第一次检测
    let first_detection = is_time_increasing(&interface1, pane_id);
    println!("第一次时间递增检测: {}", first_detection);
    assert!(first_detection, "第一次时间检测应该返回true");
    
    // 第二次检测（相同时间）
    let second_detection = is_time_increasing(&interface1, pane_id);
    println!("第二次时间递增检测: {}", second_detection);
    assert!(!second_detection, "相同时间应该返回false");
    
    // 第三次检测（不同时间）
    let third_detection = is_time_increasing(&interface2, pane_id);
    println!("第三次时间递增检测: {}", third_detection);
    // 注意：这里可能返回true或false，取决于实际的时间值
}

/// 测试内容变化检测功能
#[test]
fn test_content_change_with_real_data() {
    // 加载两个不同的界面数据
    let interface1 = load_interface_data("philosophising_state.txt");
    let interface2 = load_interface_data("user_input_state.txt");
    
    let has_change = has_substantial_content_change(&interface2, &interface1);
    println!("内容变化检测结果: {}", has_change);
    
    // 验证函数能够正常工作
    // 由于真实界面数据复杂，我们不断言具体结果
    // 主要是确保函数不会崩溃
}

/// 测试核心内容提取功能
#[test]
fn test_core_content_extraction_with_real_data() {
    let interface_data = load_interface_data("philosophising_state.txt");
    
    let core_content = extract_core_content(&interface_data);
    println!("提取的核心内容: {}", core_content);
    
    // 验证提取结果
    assert!(!core_content.is_empty(), "提取的核心内容不应为空");
    
    // 时间数字应该被标准化
    assert!(!core_content.contains("475s"), "时间数字应该被标准化");
    
    println!("✅ 核心内容提取正确");
}

/// 测试真实界面数据的性能
#[test]
fn test_real_data_performance() {
    let mut profiler = PerformanceProfiler::new();
    
    // 加载所有测试数据
    let test_files = vec![
        "philosophising_state.txt",
        "user_input_state.txt", 
        "processing_response.txt",
        "philosophising_tail.txt",
        "interrupted_state.txt",
        "working_state.txt",
        "processing_state.txt",
        "completed_state.txt",
    ];
    
    // 测试处理所有真实数据的性能
    profiler.start_measurement("real_data_processing");
    
    for _ in 0..5 {
        for filename in &test_files {
            let interface_data = load_interface_data(filename);
            let _ = is_claude_active(&interface_data);
            let _ = extract_execution_time(&interface_data);
            let _ = check_if_should_skip_llm_call(&interface_data);
            let _ = has_substantial_progress(&interface_data);
        }
    }
    
    profiler.end_measurement("real_data_processing");
    
    let processing_time = profiler.get_measurement("real_data_processing").unwrap();
    println!("处理真实数据5次耗时: {:?}", processing_time);
    
    // 性能应该在合理范围内
    assert!(processing_time < Duration::from_millis(2000), "真实数据处理时间过长");
    println!("✅ 性能测试通过");
}

/// 测试测试数据文件的完整性
#[test]
fn test_test_data_files_exist() {
    let test_files = vec![
        "philosophising_state.txt",
        "user_input_state.txt",
        "processing_response.txt", 
        "philosophising_tail.txt",
        "interrupted_state.txt",
        "working_state.txt",
        "processing_state.txt",
        "completed_state.txt",
    ];
    
    for filename in &test_files {
        let path = Path::new(TEST_DATA_DIR).join(filename);
        assert!(path.exists(), "测试数据文件应该存在: {}", filename);
        
        let content = fs::read_to_string(path).expect(&format!("无法读取文件: {}", filename));
        assert!(!content.is_empty(), "文件内容不应为空: {}", filename);
        
        println!("✅ 测试数据文件 {} 验证通过", filename);
    }
}

/// 测试中断状态界面
#[test]
fn test_interrupted_state_interface() {
    let interface_data = load_interface_data("interrupted_state.txt");
    
    println!("\n=== 测试中断状态界面 ===");
    println!("界面内容:\n{}", interface_data);
    
    // 基本验证
    assert!(!interface_data.is_empty(), "界面数据不应为空");
    assert!(interface_data.contains("Combobulating"), "应该包含Claude Code状态");
    assert!(interface_data.contains("没让你测试claude-watch运行"), "应该包含用户对话内容");
    
    // 活动检测
    let is_active = is_claude_active(&interface_data);
    assert!(is_active, "中断状态应该被识别为活动状态");
    println!("✅ 活动检测正确");
    
    // 时间提取
    let time = extract_execution_time(&interface_data);
    assert!(time.is_some(), "应该能够提取时间");
    println!("✅ 时间提取正确: {:?}", time);
    
    // 跳过LLM调用逻辑 - 有执行条格式应该跳过
    let should_skip = check_if_should_skip_llm_call(&interface_data);
    assert!(should_skip, "有执行条格式应该跳过LLM调用");
    println!("✅ 中断状态判断正确");
}

/// 测试工作状态界面
#[test]
fn test_working_state_interface() {
    let interface_data = load_interface_data("working_state.txt");
    
    println!("\n=== 测试工作状态界面 ===");
    println!("界面内容:\n{}", interface_data);
    
    // 基本验证
    assert!(!interface_data.is_empty(), "界面数据不应为空");
    assert!(interface_data.contains("请写一个简单的Python程序"), "应该包含用户输入");
    assert!(interface_data.contains("Combobulating"), "应该包含Claude Code状态");
    
    // 活动检测
    let is_active = is_claude_active(&interface_data);
    assert!(is_active, "工作状态应该被识别为活动状态");
    println!("✅ 活动检测正确");
    
    // 时间提取
    let time = extract_execution_time(&interface_data);
    assert!(time.is_some(), "应该能够提取时间");
    println!("✅ 时间提取正确: {:?}", time);
    
    // 跳过LLM调用逻辑 - 工作状态应该跳过
    let should_skip = check_if_should_skip_llm_call(&interface_data);
    assert!(should_skip, "工作状态应该跳过LLM调用");
    println!("✅ 工作状态判断正确");
}

/// 测试处理状态界面
#[test]
fn test_processing_state_interface() {
    let interface_data = load_interface_data("processing_state.txt");
    
    println!("\n=== 测试处理状态界面 ===");
    println!("界面内容:\n{}", interface_data);
    
    // 基本验证
    assert!(!interface_data.is_empty(), "界面数据不应为空");
    assert!(interface_data.contains("请写一个简单的Python程序"), "应该包含用户输入");
    assert!(interface_data.contains("Combobulating"), "应该包含Claude Code状态");
    
    // 活动检测
    let is_active = is_claude_active(&interface_data);
    assert!(is_active, "处理状态应该被识别为活动状态");
    println!("✅ 活动检测正确");
    
    // 时间提取
    let time = extract_execution_time(&interface_data);
    assert!(time.is_some(), "应该能够提取时间");
    println!("✅ 时间提取正确: {:?}", time);
    
    // 跳过LLM调用逻辑 - 处理状态应该跳过
    let should_skip = check_if_should_skip_llm_call(&interface_data);
    assert!(should_skip, "处理状态应该跳过LLM调用");
    println!("✅ 处理状态判断正确");
}

/// 测试完成状态界面
#[test]
fn test_completed_state_interface() {
    let interface_data = load_interface_data("completed_state.txt");
    
    println!("\n=== 测试完成状态界面 ===");
    println!("界面内容:\n{}", interface_data);
    
    // 基本验证
    assert!(!interface_data.is_empty(), "界面数据不应为空");
    
    // 活动检测 - 完成状态可能不活跃
    let is_active = is_claude_active(&interface_data);
    println!("✅ 活动检测结果: {}", is_active);
    
    // 时间提取
    let time = extract_execution_time(&interface_data);
    println!("✅ 时间提取结果: {:?}", time);
    
    // 跳过LLM调用逻辑
    let should_skip = check_if_should_skip_llm_call(&interface_data);
    println!("✅ 完成状态跳过判断: {}", should_skip);
}

/// 测试真实界面数据的边界情况
#[test]
fn test_real_data_edge_cases() {
    // 测试处理所有真实数据时不会崩溃
    let test_files = vec![
        "philosophising_state.txt",
        "user_input_state.txt",
        "processing_response.txt",
        "philosophising_tail.txt",
    ];
    
    for filename in &test_files {
        let interface_data = load_interface_data(filename);
        
        // 确保所有函数都能正常处理真实数据
        let _ = is_claude_active(&interface_data);
        let _ = extract_execution_time(&interface_data);
        let _ = check_if_should_skip_llm_call(&interface_data);
        let _ = has_substantial_progress(&interface_data);
        let _ = extract_core_content(&interface_data);
        let _ = is_just_time_counter(&interface_data);
        
        println!("✅ 文件 {} 边界情况测试通过", filename);
    }
}

/// 集成测试：使用真实数据模拟完整的监控流程
#[test]
fn test_full_monitoring_flow_with_real_data() {
    let test_files = vec![
        "philosophising_state.txt",
        "user_input_state.txt",
        "processing_response.txt",
        "philosophising_tail.txt",
    ];
    
    for filename in &test_files {
        let interface_data = load_interface_data(filename);
        
        // 模拟完整的监控流程
        println!("\n=== 模拟监控流程: {} ===", filename);
        
        // 1. 捕获界面
        println!("1. 捕获界面: {} 字符", interface_data.len());
        
        // 2. 检测活动状态
        let is_active = is_claude_active(&interface_data);
        println!("2. 活动检测: {}", is_active);
        
        // 3. 提取时间
        let time = extract_execution_time(&interface_data);
        println!("3. 时间提取: {:?}", time);
        
        // 4. 判断是否跳过LLM调用
        let should_skip = check_if_should_skip_llm_call(&interface_data);
        println!("4. 跳过LLM调用: {}", should_skip);
        
        // 5. 检测进展
        let has_progress = has_substantial_progress(&interface_data);
        println!("5. 进展检测: {}", has_progress);
        
        // 6. 提取核心内容
        let core_content = extract_core_content(&interface_data);
        println!("6. 核心内容: {} 字符", core_content.len());
        
        println!("✅ 监控流程模拟完成: {}", filename);
    }
}