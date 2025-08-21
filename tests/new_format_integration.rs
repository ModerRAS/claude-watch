use claude_watch::{is_claude_active, has_substantial_progress, check_if_should_skip_llm_call, extract_execution_time};

#[test]
fn test_new_format_integration_scenario_1() {
    // 场景1: 新格式 - 只有 (esc to interrupt) 的简化格式
    let terminal_output = "✽ Thinking... (esc to interrupt)\n\
                           \n\
                           Processing the user's request...\n\
                           Analyzing the code structure.";
    
    // 应该检测为活动状态（新格式）
    assert!(is_claude_active(terminal_output), "新格式应该检测为活动状态");
    
    // 应该跳过LLM调用（有执行条）
    assert!(check_if_should_skip_llm_call(terminal_output), "新格式应该跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "新格式应该有实质性进展");
}

#[test]
fn test_new_format_integration_scenario_2() {
    // 场景2: 新格式 + 工具调用
    let terminal_output = "Tool use: Reading file\n\
                           ✶ Processing... (esc to interrupt)\n\
                           \n\
                           Reading the contents of src/main.rs...\n\
                           Analyzing the code structure...";
    
    // 应该检测为活动状态
    assert!(is_claude_active(terminal_output), "新格式+工具调用应该检测为活动状态");
    
    // 应该跳过LLM调用
    assert!(check_if_should_skip_llm_call(terminal_output), "新格式+工具调用应该跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "新格式+工具调用应该有实质性进展");
}

#[test]
fn test_new_format_integration_scenario_3() {
    // 场景3: 新格式 + 思考状态
    let terminal_output = "Starting deep analysis...\n\
                           ✻ Cogitating... (esc to interrupt)\n\
                           \n\
                           Working through complex logic...\n\
                           Processing multiple data sources...";
    
    // 应该检测为活动状态
    assert!(is_claude_active(terminal_output), "新格式+思考状态应该检测为活动状态");
    
    // 应该跳过LLM调用
    assert!(check_if_should_skip_llm_call(terminal_output), "新格式+思考状态应该跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "新格式+思考状态应该有实质性进展");
}

#[test]
fn test_new_format_integration_scenario_4() {
    // 场景4: 新格式 + 时间信息（混合格式）
    let terminal_output = "✽ Meandering... (45s esc to interrupt)\n\
                           \n\
                           Time is passing as I process...\n\
                           Analyzing the requirements.";
    
    // 应该检测为活动状态
    assert!(is_claude_active(terminal_output), "混合格式应该检测为活动状态");
    
    // 应该跳过LLM调用
    assert!(check_if_should_skip_llm_call(terminal_output), "混合格式应该跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "混合格式应该有实质性进展");
    
    // 应该能提取时间
    assert_eq!(extract_execution_time(terminal_output), Some(45), "应该能提取时间");
}

#[test]
fn test_new_format_integration_scenario_5() {
    // 场景5: 新格式 + Done 状态
    let terminal_output = "✽ Done... (esc to interrupt)\n\
                           \n\
                           Task has been completed.\n\
                           Ready for new instructions.";
    
    // 应该检测为非活动状态（Done状态）
    assert!(!is_claude_active(terminal_output), "Done状态应该检测为非活动状态");
    
    // 会跳过LLM调用（有执行条格式，即使Done状态也会跳过）
    assert!(check_if_should_skip_llm_call(terminal_output), "Done状态会跳过LLM调用");
    
    // 应该有实质性进展（完成标志）
    assert!(has_substantial_progress(terminal_output), "Done状态应该有实质性进展");
}

#[test]
fn test_new_format_integration_scenario_6() {
    // 场景6: 新格式 + 中断状态
    let terminal_output = "✽ Processing... (esc to interrupt)\n\
                           \n\
                           Interrupted by user\n\
                           > ";
    
    // 应该检测为活动状态（有执行条）
    assert!(is_claude_active(terminal_output), "中断状态应该检测为活动状态");
    
    // 不应该跳过LLM调用（有明确的中断指示）
    assert!(!check_if_should_skip_llm_call(terminal_output), "中断状态不应该跳过LLM调用");
    
    // 应该有实质性进展（中断也算进展）
    assert!(has_substantial_progress(terminal_output), "中断状态应该有实质性进展");
}

#[test]
fn test_new_format_integration_scenario_7() {
    // 场景7: 多行新格式输出
    let terminal_output = "Starting complex task...\n\
                           Tool use: Reading configuration\n\
                           ✽ Cogitating... (esc to interrupt)\n\
                           \n\
                           Analyzing multiple data sources...\n\
                           Processing queue: 3 items\n\
                           \n\
                           Error: Failed to load resource A\n\
                           Retrying with alternative source...";
    
    // 应该检测为活动状态
    assert!(is_claude_active(terminal_output), "多行新格式应该检测为活动状态");
    
    // 应该跳过LLM调用
    assert!(check_if_should_skip_llm_call(terminal_output), "多行新格式应该跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "多行新格式应该有实质性进展");
}

#[test]
fn test_new_format_integration_scenario_8() {
    // 场景8: 新格式 + 命令行提示符
    let terminal_output = "✽ Done... (esc to interrupt)\n\
                           \n\
                           Task completed successfully.\n\
                           $\n\
                           Ready for next command...";
    
    // 应该检测为非活动状态（Done状态）
    assert!(!is_claude_active(terminal_output), "完成+命令提示符应该检测为非活动状态");
    
    // 会跳过LLM调用（有执行条格式，即使Done状态也会跳过）
    assert!(check_if_should_skip_llm_call(terminal_output), "完成+命令提示符会跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "完成+命令提示符应该有实质性进展");
}

#[test]
fn test_new_format_integration_scenario_9() {
    // 场景9: 新格式状态转换
    // 模拟从无活动到新格式活动的状态转换
    
    // 初始状态：无活动
    let initial_state = "Starting monitoring...";
    assert!(!is_claude_active(initial_state), "初始状态应该无活动");
    
    // 转换到：新格式思考
    let thinking_state = "Starting to think...\n✽ Cogitating... (esc to interrupt)";
    assert!(is_claude_active(thinking_state), "新格式思考状态应该有活动");
    assert!(check_if_should_skip_llm_call(thinking_state), "新格式思考状态应该跳过LLM调用");
    assert!(has_substantial_progress(thinking_state), "新格式思考状态应该有进展");
    
    // 转换到：新格式工具调用
    let tool_state = "Tool use: Reading file\n✶ Processing... (esc to interrupt)";
    assert!(is_claude_active(tool_state), "新格式工具调用应该有活动");
    assert!(check_if_should_skip_llm_call(tool_state), "新格式工具调用应该跳过LLM调用");
    assert!(has_substantial_progress(tool_state), "新格式工具调用应该有进展");
    
    // 转换到：完成
    let done_state = "✽ Done... (esc to interrupt)";
    assert!(!is_claude_active(done_state), "新格式完成状态应该无活动");
    assert!(check_if_should_skip_llm_call(done_state), "新格式完成状态会跳过LLM调用");
    assert!(has_substantial_progress(done_state), "新格式完成状态应该有进展");
}

#[test]
fn test_new_format_integration_scenario_10() {
    // 场景10: 新格式边界情况
    let terminal_output = "✽ (esc to interrupt)";
    
    // 应该检测为活动状态（最简格式）
    assert!(is_claude_active(terminal_output), "最简新格式应该检测为活动状态");
    
    // 应该跳过LLM调用
    assert!(check_if_should_skip_llm_call(terminal_output), "最简新格式应该跳过LLM调用");
    
    // 可能有实质性进展（有执行条）
    assert!(has_substantial_progress(terminal_output), "最简新格式应该有进展");
}

#[test]
fn test_new_format_decision_matrix() {
    // 测试新格式决策矩阵
    let test_cases = vec![
        // (terminal_output, should_be_active, should_skip_llm, has_progress, expected_time, description)
        (
            "✽ Thinking... (esc to interrupt)",
            true, true, true, None,
            "新格式基础"
        ),
        (
            "✶ Processing... (esc to interrupt)",
            true, true, true, None,
            "新格式处理状态"
        ),
        (
            "✻ Cogitating... (esc to interrupt)",
            true, true, true, None,
            "新格式思考状态"
        ),
        (
            "✽ Meandering... (45s esc to interrupt)",
            true, true, true, Some(45),
            "新格式+时间"
        ),
        (
            "✽ Done... (esc to interrupt)",
            false, true, true, None,
            "新格式+Done状态"
        ),
        (
            "✽ Processing... (esc to interrupt)\nInterrupted by user",
            true, false, true, None,
            "新格式+中断"
        ),
        (
            "✽ (esc to interrupt)",
            true, true, true, None,
            "新格式最简"
        ),
        (
            "✶ Cogitating... (123s esc to interrupt)",
            true, true, true, Some(123),
            "新格式+长时间"
        ),
    ];
    
    for (input, expected_active, expected_skip, expected_progress, expected_time, description) in test_cases {
        assert_eq!(is_claude_active(input), expected_active, 
                   "活动状态检测失败 - {}: '{}'", description, input);
        assert_eq!(check_if_should_skip_llm_call(input), expected_skip, 
                   "跳过LLM检测失败 - {}: '{}'", description, input);
        assert_eq!(has_substantial_progress(input), expected_progress, 
                   "进展检测失败 - {}: '{}'", description, input);
        assert_eq!(extract_execution_time(input), expected_time, 
                   "时间提取失败 - {}: '{}'", description, input);
    }
}

#[test]
fn test_backward_compatibility_integration() {
    // 测试向后兼容性集成
    
    let old_format = "✽ Herding… (169s · ↑ 8.7k tokens · esc to interrupt)";
    let new_format = "✽ Thinking... (esc to interrupt)";
    let mixed_format = "✽ Meandering... (45s esc to interrupt)";
    
    // 所有格式都应该检测为活动状态
    assert!(is_claude_active(old_format), "旧格式应该工作");
    assert!(is_claude_active(new_format), "新格式应该工作");
    assert!(is_claude_active(mixed_format), "混合格式应该工作");
    
    // 所有格式都应该跳过LLM调用
    assert!(check_if_should_skip_llm_call(old_format), "旧格式应该跳过LLM调用");
    assert!(check_if_should_skip_llm_call(new_format), "新格式应该跳过LLM调用");
    assert!(check_if_should_skip_llm_call(mixed_format), "混合格式应该跳过LLM调用");
    
    // 所有格式都应该有实质性进展
    assert!(has_substantial_progress(old_format), "旧格式应该有进展");
    assert!(has_substantial_progress(new_format), "新格式应该有进展");
    assert!(has_substantial_progress(mixed_format), "混合格式应该有进展");
    
    // 时间提取应该对有时间的格式工作
    assert_eq!(extract_execution_time(old_format), Some(169), "旧格式时间提取");
    assert_eq!(extract_execution_time(new_format), None, "新格式无时间");
    assert_eq!(extract_execution_time(mixed_format), Some(45), "混合格式时间提取");
}

#[test]
fn test_new_format_performance_integration() {
    // 测试新格式的性能集成
    let test_cases = vec![
        "✽ Thinking... (esc to interrupt)",
        "✶ Processing... (esc to interrupt)",
        "✻ Cogitating... (esc to interrupt)",
        "✽ Meandering... (45s esc to interrupt)",
        "✽ (esc to interrupt)",
    ];
    
    let start = std::time::Instant::now();
    let iterations = 1000;
    
    for _ in 0..iterations {
        for test_case in &test_cases {
            assert!(is_claude_active(test_case));
            assert!(check_if_should_skip_llm_call(test_case));
            assert!(has_substantial_progress(test_case));
        }
    }
    
    let duration = start.elapsed();
    let avg_duration = duration.as_millis() as f64 / iterations as f64;
    
    println!("新格式性能测试: {} 次迭代总耗时 {:?}", iterations, duration);
    println!("平均每次测试耗时: {:.2}ms", avg_duration);
    
    // 性能阈值：每次测试应该小于1ms
    assert!(avg_duration < 1.0, "新格式性能测试失败: 平均耗时 {:.2}ms 超过阈值", avg_duration);
}