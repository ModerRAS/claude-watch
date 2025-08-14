//! 直接测试时间提取功能

use claude_watch::monitor::extract_execution_time;

fn main() {
    // 测试原始数据
    let test_text = "✻ Philosophising… (475s · ↓ 7 tokens · esc to interrupt)";
    println!("=== 原始数据测试 ===");
    println!("测试文本: '{}'", test_text);
    println!("文本长度: {}", test_text.len());
    
    // 手动测试正则表达式
    let time_pattern = regex::Regex::new(r"\((\d+)s\)").unwrap();
    println!("正则表达式模式: {}", time_pattern.as_str());
    
    if let Some(caps) = time_pattern.captures(test_text) {
        if let Some(time_str) = caps.get(1) {
            println!("找到时间字符串: '{}'", time_str.as_str());
            if let Ok(time) = time_str.as_str().parse::<u64>() {
                println!("解析成功: {}", time);
            } else {
                println!("解析失败");
            }
        } else {
            println!("没有找到时间组");
        }
    } else {
        println!("没有匹配到时间模式");
        
        // 尝试不同的模式
        println!("尝试不同的正则表达式模式...");
        let patterns = vec![
            r"\((\d+)s\)",
            r"\((\d+)\s*s\)",
            r"\((\d+)\s*s\s*\)",
        ];
        
        for pattern in patterns {
            let test_pattern = regex::Regex::new(pattern).unwrap();
            println!("测试模式 '{}': {:?}", pattern, test_pattern.captures(test_text));
        }
    }
    
    // 测试函数
    let result = extract_execution_time(test_text);
    println!("函数结果: {:?}", result);
    
    // 测试更简单的格式
    let simple_text = "(475s)";
    let simple_result = extract_execution_time(simple_text);
    println!("简单格式结果: {:?}", simple_result);
    
    // 测试ASCII版本
    let ascii_text = "* Philosophising... (475s) tokens esc to interrupt";
    println!("\n=== ASCII版本测试 ===");
    println!("ASCII文本: '{}'", ascii_text);
    let ascii_result = extract_execution_time(ascii_text);
    println!("ASCII结果: {:?}", ascii_result);
    
    // 测试文件读取的数据
    println!("\n=== 文件数据测试 ===");
    match std::fs::read_to_string("test_data/claude_interfaces/philosophising_state.txt") {
        Ok(file_content) => {
            let file_result = extract_execution_time(&file_content);
            println!("文件数据结果: {:?}", file_result);
            
            // 寻找包含时间的行
            for (i, line) in file_content.lines().enumerate() {
                if line.contains("(475s)") {
                    println!("第{}行包含时间: '{}'", i + 1, line);
                    let line_result = extract_execution_time(line);
                    println!("行内提取结果: {:?}", line_result);
                }
            }
        }
        Err(e) => println!("无法读取文件: {}", e),
    }
}