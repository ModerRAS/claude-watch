//! 真实Claude Code界面测试
//! 
//! 基于从真实tmux会话中捕获的Claude Code界面数据进行测试

use claude_watch::testing::*;
use claude_watch::activity::is_claude_active;
use std::time::Duration;

/// 从真实tmux会话中捕获的Claude Code界面数据
const REAL_CLAUDE_INTERFACES: &[&str] = &[
    // 真实界面1: 正在 Philosophising 状态
    "✽ Philosophising… (352s · ↓ 7 tokens · esc to interrupt)
╭────────────────────────────────────────────────────────────────╮
│ >                                                             │
╰────────────────────────────────────────────────────────────────╯
  ? for shortcuts                          Bypassing Permissions",

    // 真实界面2: 时间递增到356s
    "✽ Philosophising… (356s · ↓ 7 tokens · esc to interrupt)
╭────────────────────────────────────────────────────────────────╮
│ >                                                             │
╰────────────────────────────────────────────────────────────────╯
  ? for shortcuts                          Bypassing Permissions",

    // 真实界面3: 时间递增到359s，状态变为 * Philosophising
    "* Philosophising… (359s · ↓ 7 tokens · esc to interrupt)
╭────────────────────────────────────────────────────────────────╮
│ >                                                             │
╰────────────────────────────────────────────────────────────────╯
  ? for shortcuts                          Bypassing Permissions",

    // 真实界面4: 用户输入消息后的状态
    "* Philosophising… (384s · ↓ 7 tokens · esc to interrupt)

  请显示当前时间
╭────────────────────────────────────────────────────────────────╮
│ >Press up to edit queued messages                             │
╰────────────────────────────────────────────────────────────────╯
  ? for shortcuts                          Bypassing Permissions",

    // 真实界面5: 时间递增到399s
    "* Philosophising… (399s · ↓ 7 tokens · esc to interrupt)

  请显示当前时间
╭────────────────────────────────────────────────────────────────╮
│ >Press up to edit queued messages                             │
╰────────────────────────────────────────────────────────────────╯
  ? for shortcuts                          Bypassing Permissions",

    // 真实界面6: 空闲状态（只有命令提示符）
    "╭────────────────────────────────────────────────────────────────╮
│ >                                                             │
╰────────────────────────────────────────────────────────────────╯
  ? for shortcuts                          Bypassing Permissions",

    // 真实界面7: 包含用户消息的界面
    "我要你抓来跑单元测试的，用来做单元测试用例的
╭────────────────────────────────────────────────────────────────╮
│ >                                                             │
╰────────────────────────────────────────────────────────────────╯
  ? for shortcuts                          Bypassing Permissions",
];

/// 真实界面测试用例
#[test]
fn test_real_claude_interfaces() {
    println!("开始测试真实Claude Code界面数据...");
    
    for (i, interface) in REAL_CLAUDE_INTERFACES.iter().enumerate() {
        println!("\n=== 测试界面 {} ===", i + 1);
        println!("界面内容:\n{}", interface);
        
        // 测试活动检测
        let is_active = is_claude_active(interface);
        println!("活动检测结果: {}", is_active);
        
        // 测试时间提取
        let time = extract_execution_time(interface);
        println!("时间提取结果: {:?}", time);
        
        // 测试跳过LLM调用逻辑
        let should_skip = check_if_should_skip_llm_call(interface);
        println!("跳过LLM调用结果: {}", should_skip);
        
        // 测试进展检测
        let has_progress = has_substantial_progress(interface);
        println!("进展检测结果: {}", has_progress);
        
        // 基本验证：真实的工作界面应该被识别为活动状态
        if interface.contains("Philosophising") || interface.contains("tokens") {
            assert!(is_active, "真实工作界面应该被识别为活动状态");
            println!("✅ 界面 {} 活动检测正确", i + 1);
        }
        
        // 基本验证：有时间计数器的界面应该能够提取时间
        if interface.contains("s ·") {
            assert!(time.is_some(), "有时间计数器的界面应该能够提取时间");
            println!("✅ 界面 {} 时间提取正确", i + 1);
        }
        
        println!("---");
    }
}

/// 测试真实界面的时间递增检测
#[test]
fn test_real_interface_time_increasing() {
    setup::reset_global_state();
    
    let pane_id = "real_pane_test";
    
    // 使用真实界面数据测试时间递增检测
    let interfaces_with_time = REAL_CLAUDE_INTERFACES.iter()
        .filter(|&interface| interface.contains("s ·"))
        .collect::<Vec<_>>();
    
    assert!(!interfaces_with_time.is_empty(), "应该至少有一个界面包含时间计数器");
    
    // 测试时间递增逻辑
    for (i, &interface) in interfaces_with_time.iter().enumerate() {
        let is_increasing = is_time_increasing(interface, pane_id);
        println!("界面 {} 时间递增检测结果: {}", i + 1, is_increasing);
        
        // 第一次检测应该返回true
        if i == 0 {
            assert!(is_increasing, "第一次时间检测应该返回true");
        }
    }
}

/// 测试真实界面的内容变化检测
#[test]
fn test_real_interface_content_change() {
    // 使用前两个真实界面测试内容变化检测
    if REAL_CLAUDE_INTERFACES.len() >= 2 {
        let interface1 = REAL_CLAUDE_INTERFACES[0];
        let interface2 = REAL_CLAUDE_INTERFACES[1];
        
        let has_change = has_substantial_content_change(interface2, interface1);
        println!("真实界面内容变化检测结果: {}", has_change);
        
        // 由于时间数字变化，但其他内容相似，可能不会被认为是实质性变化
        // 这个测试主要是验证函数不会崩溃
    }
}

/// 测试真实界面的执行条格式识别
#[test]
fn test_real_interface_execution_bar() {
    for (i, interface) in REAL_CLAUDE_INTERFACES.iter().enumerate() {
        // 检查是否包含Claude Code的标准执行条格式
        let has_execution_bar = interface.contains("tokens") && 
                               (interface.contains("✽") || interface.contains("*")) &&
                               interface.contains("esc to interrupt");
        
        println!("界面 {} 执行条格式检测结果: {}", i + 1, has_execution_bar);
        
        if has_execution_bar {
            // 有执行条格式的界面应该被正确识别
            let is_active = is_claude_active(interface);
            let should_skip = check_if_should_skip_llm_call(interface);
            
            assert!(is_active, "有执行条格式的界面应该被识别为活动状态");
            assert!(should_skip, "有执行条格式的界面应该跳过LLM调用");
            
            println!("✅ 界面 {} 执行条格式处理正确", i + 1);
        }
    }
}

/// 测试真实界面的边界情况
#[test]
fn test_real_interface_edge_cases() {
    // 测试包含特殊字符的界面
    for interface in REAL_CLAUDE_INTERFACES {
        // 确保所有真实界面都不会导致函数崩溃
        let _ = is_claude_active(interface);
        let _ = extract_execution_time(interface);
        let _ = check_if_should_skip_llm_call(interface);
        let _ = has_substantial_progress(interface);
        let _ = extract_core_content(interface);
    }
}

/// 测试真实界面的性能
#[test]
fn test_real_interface_performance() {
    let mut profiler = PerformanceProfiler::new();
    
    // 测试处理所有真实界面的性能
    profiler.start_measurement("real_interfaces_processing");
    
    for _ in 0..10 {
        for interface in REAL_CLAUDE_INTERFACES {
            let _ = is_claude_active(interface);
            let _ = extract_execution_time(interface);
            let _ = check_if_should_skip_llm_call(interface);
            let _ = has_substantial_progress(interface);
        }
    }
    
    profiler.end_measurement("real_interfaces_processing");
    
    let processing_time = profiler.get_measurement("real_interfaces_processing").unwrap();
    println!("处理真实界面数据10次耗时: {:?}", processing_time);
    
    // 性能应该在合理范围内
    assert!(processing_time < Duration::from_millis(1000), "真实界面处理时间过长");
}

/// 测试真实界面的多行内容处理
#[test]
fn test_real_interface_multiline() {
    for (i, interface) in REAL_CLAUDE_INTERFACES.iter().enumerate() {
        // 测试多行内容处理
        let lines: Vec<&str> = interface.lines().collect();
        let line_count = lines.len();
        
        println!("界面 {} 行数: {}", i + 1, line_count);
        
        // 确保多行内容能够正确处理
        if line_count > 1 {
            let is_active = is_claude_active(interface);
            println!("多行界面 {} 活动检测结果: {}", i + 1, is_active);
        }
    }
}

/// 测试真实界面的命令提示符状态识别
#[test]
fn test_real_interface_prompt_detection() {
    for (i, interface) in REAL_CLAUDE_INTERFACES.iter().enumerate() {
        // 检查是否包含命令提示符
        let has_prompt = interface.contains('>') || interface.contains('$') || interface.contains('#');
        
        if has_prompt {
            println!("界面 {} 包含命令提示符", i + 1);
            
            // 测试命令提示符状态的处理
            let should_skip = check_if_should_skip_llm_call(interface);
            println!("命令提示符界面 {} 跳过LLM调用结果: {}", i + 1, should_skip);
        }
    }
}