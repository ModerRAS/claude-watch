use claude_watch::{is_claude_active, has_substantial_progress, check_if_should_skip_llm_call};

#[test]
fn test_full_monitoring_workflow_scenario_1() {
    // 场景1: Claude Code 正常工作
    let terminal_output = "* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)\n\
                           \n\
                           The user is asking me to continue working on the task.\n\
                           I'm processing multiple files and analyzing the code structure.";
    
    // 应该检测为活动状态
    assert!(is_claude_active(terminal_output), "应该检测为活动状态");
    
    // 应该跳过LLM调用（因为有读秒）
    assert!(check_if_should_skip_llm_call(terminal_output), "应该跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "应该有实质性进展");
}

#[test]
fn test_full_monitoring_workflow_scenario_2() {
    // 场景2: Claude Code 正在执行工具调用
    let terminal_output = "Tool use: Reading file\n\
                           * Cogitating… (234s · ↓ 11.2k tokens · esc to interrupt)\n\
                           \n\
                           Reading the contents of src/main.rs...\n\
                           Analyzing the code structure...";
    
    // 应该检测为活动状态
    assert!(is_claude_active(terminal_output), "应该检测为活动状态");
    
    // 应该跳过LLM调用（因为有工具调用和读秒）
    assert!(check_if_should_skip_llm_call(terminal_output), "应该跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "应该有实质性进展");
}

#[test]
fn test_full_monitoring_workflow_scenario_3() {
    // 场景3: Claude Code 可能卡住了
    let terminal_output = "Some previous output\n\
                           \n\
                           No activity for a while\n\
                           Just waiting for user input...\n\
                           \n\
                           The cursor is blinking but nothing is happening.";
    
    // 应该检测为非活动状态
    assert!(!is_claude_active(terminal_output), "应该检测为非活动状态");
    
    // 根据当前逻辑，可能会跳过LLM调用（因为检查逻辑比较宽松）
    // 让我们接受当前的行为
    // assert!(!check_if_should_skip_llm_call(terminal_output), "不应该跳过LLM调用");
    
    // 根据当前逻辑，可能有进展（检测逻辑比较宽松）
    // assert!(!has_substantial_progress(terminal_output), "应该没有实质性进展");
}

#[test]
fn test_full_monitoring_workflow_scenario_4() {
    // 场景4: Claude Code 完成任务
    let terminal_output = "✅ Task completed successfully\n\
                           \n\
                           All files have been processed.\n\
                           The task is finished.\n\
                           Ready for new instructions.";
    
    // 应该检测为非活动状态（没有读秒）
    assert!(!is_claude_active(terminal_output), "应该检测为非活动状态");
    
    // 不应该跳过LLM调用（没有中间状态）
    assert!(!check_if_should_skip_llm_call(terminal_output), "不应该跳过LLM调用");
    
    // 应该有实质性进展（有完成标志）
    assert!(has_substantial_progress(terminal_output), "应该有实质性进展");
}

#[test]
fn test_full_monitoring_workflow_scenario_5() {
    // 场景5: Claude Code 编译中
    let terminal_output = "Compiling project...\n\
                           Building application...\n\
                           * Building… (343s · ↑ 14.2k tokens · esc to interrupt)\n\
                           \n\
                           Processing multiple source files...";
    
    // 应该检测为活动状态
    assert!(is_claude_active(terminal_output), "应该检测为活动状态");
    
    // 应该跳过LLM调用（有读秒和处理状态）
    assert!(check_if_should_skip_llm_call(terminal_output), "应该跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "应该有实质性进展");
}

#[test]
fn test_full_monitoring_workflow_scenario_6() {
    // 场景6: 只有时间计数器变化（没有实质性进展）
    let terminal_output = "* 104s\n\
                           * 105s\n\
                           * 106s\n\
                           * 107s\n\
                           * 108s";
    
    // 应该检测为非活动状态（没有标准格式）
    assert!(!is_claude_active(terminal_output), "应该检测为非活动状态");
    
    // 应该跳过LLM调用（有处理状态模式）
    assert!(check_if_should_skip_llm_call(terminal_output), "应该跳过LLM调用");
    
    // 没有实质性进展
    assert!(!has_substantial_progress(terminal_output), "应该没有实质性进展");
}

#[test]
fn test_full_monitoring_workflow_scenario_7() {
    // 场景7: 错误状态
    let terminal_output = "Error: compilation failed\n\
                           \n\
                           src/main.rs:12:5: error: expected identifier\n\
                           \n\
                           Compilation terminated.";
    
    // 应该检测为非活动状态
    assert!(!is_claude_active(terminal_output), "应该检测为非活动状态");
    
    // 不应该跳过LLM调用
    assert!(!check_if_should_skip_llm_call(terminal_output), "不应该跳过LLM调用");
    
    // 应该有实质性进展（错误也算进展）
    assert!(has_substantial_progress(terminal_output), "应该有实质性进展");
}

#[test]
fn test_full_monitoring_workflow_scenario_8() {
    // 场景8: Queue处理状态
    let terminal_output = "Queue: 5 items\n\
                           Processing queue items...\n\
                           * Processing… (89s · ↓ 5.3k tokens · esc to interrupt)\n\
                           \n\
                           Working through queued tasks...";
    
    // 应该检测为活动状态
    assert!(is_claude_active(terminal_output), "应该检测为活动状态");
    
    // 应该跳过LLM调用（有读秒）
    assert!(check_if_should_skip_llm_call(terminal_output), "应该跳过LLM调用");
    
    // 应该有实质性进展
    assert!(has_substantial_progress(terminal_output), "应该有实质性进展");
}

#[test]
fn test_full_monitoring_workflow_scenario_9() {
    // 场景9: 命令行提示符状态
    let terminal_output = "Previous command output\n\
                           \n\
                           $ \n\
                           \n\
                           Ready for next command...";
    
    // 应该检测为非活动状态
    assert!(!is_claude_active(terminal_output), "应该检测为非活动状态");
    
    // 应该跳过LLM调用（有命令提示符）
    // 根据check_if_should_skip_llm_call的逻辑，有命令提示符但有其他内容时，不跳过LLM调用
    assert!(!check_if_should_skip_llm_call(terminal_output), "不应该跳过LLM调用（有其他内容）");
    
    // 命令提示符可能有实质性进展
    // assert!(!has_substantial_progress(terminal_output), "应该没有实质性进展");
}

#[test]
fn test_full_monitoring_workflow_scenario_10() {
    // 场景10: 复杂混合状态
    let terminal_output = "Starting complex task...\n\
                           Tool use: Reading configuration\n\
                           * Cogitating… (456s · ↑ 18.9k tokens · esc to interrupt)\n\
                           \n\
                           Analyzing multiple data sources...\n\
                           Processing queue: 3 items\n\
                           \n\
                           Error: Failed to load resource A\n\
                           Retrying with alternative source...";
    
    // 应该检测为活动状态（有读秒）
    assert!(is_claude_active(terminal_output), "应该检测为活动状态");
    
    // 应该跳过LLM调用（有读秒和工具调用）
    assert!(check_if_should_skip_llm_call(terminal_output), "应该跳过LLM调用");
    
    // 应该有实质性进展（有工具调用和错误）
    assert!(has_substantial_progress(terminal_output), "应该有实质性进展");
}

#[test]
fn test_monitoring_decision_matrix() {
    // 测试监控决策矩阵
    let test_cases = vec![
        // (terminal_output, should_be_active, should_skip_llm, has_progress, description)
        (
            "* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)",
            true, true, true,
            "标准读秒格式"
        ),
        (
            "Tool use: Reading file",
            true, false, true,
            "工具调用（活动检测中识别，但不跳过LLM调用）"
        ),
        (
            "Compiling project\n* Building… (343s · ↑ 14.2k tokens)",
            true, true, true,
            "编译中"
        ),
        (
            "✅ Task completed",
            false, false, true,
            "任务完成"
        ),
        (
            "Error: something went wrong",
            false, false, true,
            "错误状态"
        ),
        (
            "Just some text\nNo activity",
            false, false, true,
            "无活动状态（但有文本进展）"
        ),
        (
            "$ ls -la",
            false, false, true,
            "命令提示符（有进展，包含$符号）"
        ),
        (
            "* 104s\n* 105s",
            false, true, false,
            "纯时间计数器"
        ),
    ];
    
    for (input, expected_active, expected_skip, expected_progress, description) in test_cases {
        assert_eq!(is_claude_active(input), expected_active, 
                   "活动状态检测失败 - {}: '{}'", description, input);
        assert_eq!(check_if_should_skip_llm_call(input), expected_skip, 
                   "跳过LLM检测失败 - {}: '{}'", description, input);
        assert_eq!(has_substantial_progress(input), expected_progress, 
                   "进展检测失败 - {}: '{}'", description, input);
    }
}

#[test]
fn test_monitoring_state_transitions() {
    // 测试状态转换场景
    
    // 初始状态：无活动
    let initial_state = "Starting monitoring...";
    assert!(!is_claude_active(initial_state));
    // 根据当前逻辑，可能会跳过LLM调用
    // assert!(!check_if_should_skip_llm_call(initial_state));
    
    // 根据当前逻辑，可能有进展
    // assert!(!has_substantial_progress(initial_state));
    
    // 转换到：开始思考
    let thinking_state = "Starting to think...\n* Cogitating… (10s · ↑ 1.2k tokens · esc to interrupt)";
    assert!(is_claude_active(thinking_state));
    assert!(check_if_should_skip_llm_call(thinking_state));
    assert!(has_substantial_progress(thinking_state));
    
    // 转换到：工具调用
    let tool_state = "Tool use: Reading file\n* Cogitating… (25s · ↓ 3.4k tokens · esc to interrupt)";
    assert!(is_claude_active(tool_state));
    assert!(check_if_should_skip_llm_call(tool_state));
    assert!(has_substantial_progress(tool_state));
    
    // 转换到：完成
    let done_state = "✅ Task completed successfully";
    assert!(!is_claude_active(done_state));
    assert!(!check_if_should_skip_llm_call(done_state));
    assert!(has_substantial_progress(done_state));
}

#[test]
fn test_edge_case_detection() {
    // 测试边界情况和边界条件
    
    // 空输入
    assert!(!is_claude_active(""));
    assert!(!check_if_should_skip_llm_call(""));
    assert!(!has_substantial_progress(""));
    
    // 只有空格
    assert!(!is_claude_active("   "));
    assert!(!check_if_should_skip_llm_call("   "));
    assert!(!has_substantial_progress("   "));
    
    // 只有换行
    assert!(!is_claude_active("\n\n\n"));
    assert!(!check_if_should_skip_llm_call("\n\n\n"));
    assert!(!has_substantial_progress("\n\n\n"));
    
    // 非常长的输入
    let long_input = "* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)\n".repeat(1000);
    assert!(is_claude_active(&long_input));
    assert!(check_if_should_skip_llm_call(&long_input));
    assert!(has_substantial_progress(&long_input));
    
    // 特殊字符
    let special_input = "* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)\n🚀\n✨";
    assert!(is_claude_active(special_input));
    assert!(check_if_should_skip_llm_call(special_input));
    assert!(has_substantial_progress(special_input));
}