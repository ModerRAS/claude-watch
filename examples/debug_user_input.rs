//! 调试用户输入状态的判断逻辑

use claude_watch::monitor::check_if_should_skip_llm_call;

fn main() {
    // 读取用户输入状态数据
    let interface_data = std::fs::read_to_string("test_data/claude_interfaces/user_input_state.txt")
        .expect("无法读取用户输入状态数据");
    
    println!("=== 用户输入状态分析 ===");
    println!("界面内容:\n{}", interface_data);
    
    // 测试跳过LLM调用逻辑
    let should_skip = check_if_should_skip_llm_call(&interface_data);
    println!("是否应该跳过LLM调用: {}", should_skip);
    
    // 分析具体原因
    let lines: Vec<&str> = interface_data.lines().collect();
    let last_lines: Vec<&str> = lines.iter().rev().take(10).cloned().collect();
    let last_content = last_lines.join("\n");
    
    println!("\n=== 最后10行内容 ===");
    println!("{}", last_content);
    
    // 检查命令提示符状态
    let trimmed_content = last_content.trim();
    println!("是否以>结尾: {}", trimmed_content.ends_with('>'));
    println!("是否以$结尾: {}", trimmed_content.ends_with('$'));
    println!("是否以#结尾: {}", trimmed_content.ends_with('#'));
    
    // 检查执行条格式
    let execution_bar_pattern = regex::Regex::new(r"\*[^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)").unwrap();
    let has_execution_bar = execution_bar_pattern.is_match(&last_content);
    println!("是否有执行条格式: {}", has_execution_bar);
    
    // 检查未完成指示符
    let has_incomplete_indicator = last_content.ends_with("...") || 
                                   last_content.ends_with("▪") || 
                                   last_content.ends_with("◦") || 
                                   last_content.ends_with("●") || 
                                   last_content.ends_with("▬");
    println!("是否有未完成指示符: {}", has_incomplete_indicator);
    
    // 检查活动状态关键词
    let active_keywords = [
        "Cogitating", "Herding", "Meandering", "Reticulating", "Thinking",
        "Processing", "Compiling", "Building", "Executing",
        "Reading", "Writing", "Generating", "Creating", "Analyzing",
        "Calling", "Searching", "Browsing", "Loading", "Saving"
    ];
    
    println!("\n=== 活动状态关键词检查 ===");
    for keyword in &active_keywords {
        let contains_keyword = last_content.contains(keyword);
        let has_tokens = last_content.contains("tokens");
        let should_match = (contains_keyword && has_tokens) || 
                          last_content.contains("* Compiling") || 
                          last_content.contains("* Building");
        
        if contains_keyword {
            println!("包含 '{}': {}, 有tokens: {}, 应该匹配: {}", keyword, contains_keyword, has_tokens, should_match);
        }
    }
    
    // 检查Philosophising
    let has_philosophising = last_content.contains("Philosophising");
    let has_tokens = last_content.contains("tokens");
    let philosophising_match = has_philosophising && has_tokens;
    println!("Philosophising检查: 包含={}, 有tokens={}, 应该匹配={}", has_philosophising, has_tokens, philosophising_match);
}