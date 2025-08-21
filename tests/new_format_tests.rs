#[cfg(test)]
mod tests {
    use claude_watch::activity::is_claude_active;
    use claude_watch::monitor::{extract_execution_time, check_if_should_skip_llm_call, has_substantial_progress};

    #[test]
    fn test_new_format_esc_interrupt_only() {
        // 测试新格式：只有 (esc to interrupt) 的情况
        let text = "✽ Thinking... (esc to interrupt)";
        assert!(is_claude_active(text));
        
        let text = "✶ Processing... (esc to interrupt)";
        assert!(is_claude_active(text));
        
        let text = "✻ Cogitating... (esc to interrupt)";
        assert!(is_claude_active(text));
    }

    #[test]
    fn test_new_format_with_done_state() {
        // 测试新格式 + Done 状态（应该返回 false）
        let text = "✽ Done... (esc to interrupt)";
        assert!(!is_claude_active(text));
        
        let text = "✶ Done (esc to interrupt)";
        assert!(!is_claude_active(text));
    }

    #[test]
    fn test_mixed_formats_backward_compatibility() {
        // 测试向后兼容性：旧格式仍然应该工作
        let text = "✽ Herding… (343s · ↑ 14.2k tokens · esc to interrupt)";
        assert!(is_claude_active(text));
        
        let text = "✶ Perusing… (28s · ⚒ 414 tokens · esc to interrupt)";
        assert!(is_claude_active(text));
    }

    #[test]
    fn test_new_format_time_extraction() {
        // 测试新格式的时间提取
        let text = "✽ Thinking... (45s esc to interrupt)";
        assert_eq!(extract_execution_time(text), Some(45));
        
        let text = "✶ Processing... (123s · esc to interrupt)";
        assert_eq!(extract_execution_time(text), Some(123));
        
        let text = "✻ Cogitating... (1s esc to interrupt)";
        assert_eq!(extract_execution_time(text), Some(1));
    }

    #[test]
    fn test_skip_llm_call_new_format() {
        // 测试新格式的 LLM 跳过逻辑
        let text = "✽ Thinking... (esc to interrupt)";
        assert!(check_if_should_skip_llm_call(text));
        
        let text = "✶ Cogitating... (esc to interrupt)";
        assert!(check_if_should_skip_llm_call(text));
        
        let text = "✻ Processing... (esc to interrupt)";
        assert!(check_if_should_skip_llm_call(text));
    }

    #[test]
    fn test_skip_llm_call_with_keywords() {
        // 测试新格式 + 活动关键词
        let text = "✽ Cogitating... (esc to interrupt)";
        assert!(check_if_should_skip_llm_call(text));
        
        let text = "✶ Thinking... (esc to interrupt)";
        assert!(check_if_should_skip_llm_call(text));
        
        let text = "✻ Processing... (esc to interrupt)";
        assert!(check_if_should_skip_llm_call(text));
    }

    #[test]
    fn test_substantial_progress_new_format() {
        // 测试新格式的实质性进展检测
        let text = "✽ Thinking... (esc to interrupt)";
        assert!(has_substantial_progress(text));
        
        let text = "✶ Processing... (esc to interrupt)";
        assert!(has_substantial_progress(text));
        
        let text = "✻ Cogitating... (esc to interrupt)";
        assert!(has_substantial_progress(text));
    }

    #[test]
    fn test_new_format_edge_cases() {
        // 测试新格式的边界情况
        let text = "✽ (esc to interrupt)";
        assert!(is_claude_active(text));
        
        let text = "✶ (esc to interrupt)";
        assert!(is_claude_active(text));
        
        let text = "✻ (esc to interrupt)";
        assert!(is_claude_active(text));
    }

    #[test]
    fn test_new_format_with_interruption() {
        // 测试新格式 + 中断状态
        let text = "✽ (esc to interrupt)\nInterrupted by user";
        assert!(is_claude_active(text)); // 仍然检测为活动，但中断状态会在其他地方处理
        
        let text = "Interrupted by user\n✽ (esc to interrupt)";
        assert!(is_claude_active(text));
    }

    #[test]
    fn test_regex_pattern_flexibility() {
        // 测试正则表达式的灵活性
        let text = "✽ Thinking... (45s esc to interrupt)";
        assert!(is_claude_active(text));
        
        let text = "✶ Processing... (123s · some content · esc to interrupt)";
        assert!(is_claude_active(text));
        
        let text = "✻ Cogitating... (1s tokens esc to interrupt)";
        assert!(is_claude_active(text));
    }

    #[test]
    fn test_performance_with_new_format() {
        // 测试新格式的性能
        let text = "✽ Thinking... (esc to interrupt)";
        let start = std::time::Instant::now();
        
        for _ in 0..1000 {
            assert!(is_claude_active(text));
            assert!(check_if_should_skip_llm_call(text));
            assert!(has_substantial_progress(text));
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "Performance test failed: took too long");
    }
}