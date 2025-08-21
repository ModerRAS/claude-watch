use claude_watch::activity::is_claude_active;

fn main() {
    let test_case = "* Processing… (56s · esc to interrupt)";
    let result = is_claude_active(test_case);
    println!("Input: '{}'", test_case);
    println!("Result: {}", result);
    println!("Expected: true");
    
    let trimmed = test_case.trim();
    println!("Contains 'Processing': {}", trimmed.contains("Processing"));
    println!("Contains 'esc to interrupt': {}", trimmed.contains("(esc to interrupt)"));
    println!("Contains 'tokens': {}", trimmed.contains("tokens"));
}