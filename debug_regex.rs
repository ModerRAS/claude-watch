use regex::Regex;

fn main() {
    let text = "● Update Todos\n  ⎿  ☒ 完成创建复杂的示例Rust项目结构\n     ☒ 分析项目模块和依赖关系\n     ☒ 生成详细的项目文档\n\n╭───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮\n│ >                                                                                                                                                                                                            │\n╰───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯\n  ? for shortcuts                                                                                                                                                                         Bypassing Permissions";
    
    // 测试执行条格式
    let execution_bar_pattern = Regex::new(r"\*[^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)").unwrap();
    println!("执行条格式匹配: {}", execution_bar_pattern.is_match(text));
    
    // 测试时间tokens模式
    let time_tokens_pattern = Regex::new(r"\([^)]*\d+s[^)]*tokens[^)]*\)").unwrap();
    println!("时间tokens模式匹配: {}", time_tokens_pattern.is_match(text));
    
    // 查看具体匹配了什么
    for cap in time_tokens_pattern.captures_iter(text) {
        println!("匹配到: {}", cap.get(0).unwrap().as_str());
    }
}