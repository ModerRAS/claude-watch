use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

/// Main configuration structure for claude-watch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLM backend configuration
    pub llm: LlmConfig,
    
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    
    /// Tmux configuration
    pub tmux: TmuxConfig,
}

/// LLM backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Backend type: ollama, openrouter, openai, none
    pub backend: String,
    
    /// Ollama configuration
    pub ollama: Option<OllamaConfig>,
    
    /// OpenAI configuration
    pub openai: Option<OpenAiConfig>,
    
    /// OpenRouter configuration
    pub openrouter: Option<OpenRouterConfig>,
}

/// Ollama configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama server URL
    pub url: String,
    
    /// Model to use
    pub model: String,
}

/// OpenAI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    /// OpenAI API key
    pub api_key: String,
    
    /// OpenAI API base URL
    pub api_base: String,
    
    /// Model to use
    pub model: String,
}

/// OpenRouter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    /// OpenRouter API key
    pub api_key: String,
    
    /// Model to use
    pub model: String,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Check interval in seconds
    pub interval: u64,
    
    /// Stuck timeout in seconds
    pub stuck_sec: u64,
    
    /// Maximum retry attempts
    pub max_retry: usize,
}

/// Tmux configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxConfig {
    /// Tmux pane ID
    pub pane: String,
}

impl Config {
    /// Load configuration from file or create default
    pub fn load(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(content) = fs::read_to_string(config_path) {
            let config: Config = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default configuration
            Ok(Config::default())
        }
    }
    
    /// Create configuration from command line arguments
    pub fn from_args(args: &crate::args::Args) -> Self {
        let backend = args.backend.clone().unwrap_or("ollama".to_string());
        
        Config {
            llm: LlmConfig {
                backend: backend.clone(),
                ollama: if backend == "ollama" {
                    Some(OllamaConfig {
                        url: env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
                        model: "qwen2.5:3b".to_string(),
                    })
                } else {
                    None
                },
                openai: if backend == "openai" {
                    Some(OpenAiConfig {
                        api_key: env::var("OPENAI_API_KEY").unwrap_or_default(),
                        api_base: env::var("OPENAI_API_BASE").unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
                        model: "gpt-4o".to_string(),
                    })
                } else {
                    None
                },
                openrouter: if backend == "openrouter" {
                    Some(OpenRouterConfig {
                        api_key: env::var("OPENROUTER_KEY").unwrap_or_default(),
                        model: env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "qwen/qwen-2.5-7b-instruct".to_string()),
                    })
                } else {
                    None
                },
            },
            monitoring: MonitoringConfig {
                interval: args.interval.unwrap_or(5),
                stuck_sec: args.stuck_sec.unwrap_or(60),
                max_retry: args.max_retry.unwrap_or(10),
            },
            tmux: TmuxConfig {
                pane: args.pane.clone().unwrap_or("%0".to_string()),
            },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            llm: LlmConfig {
                backend: "ollama".to_string(),
                ollama: Some(OllamaConfig {
                    url: "http://localhost:11434".to_string(),
                    model: "qwen2.5:3b".to_string(),
                }),
                openai: Some(OpenAiConfig {
                    api_key: "".to_string(),
                    api_base: "https://api.openai.com/v1".to_string(),
                    model: "gpt-4o".to_string(),
                }),
                openrouter: Some(OpenRouterConfig {
                    api_key: "".to_string(),
                    model: "qwen/qwen-2.5-7b-instruct".to_string(),
                }),
            },
            monitoring: MonitoringConfig {
                interval: 5,
                stuck_sec: 60,
                max_retry: 10,
            },
            tmux: TmuxConfig {
                pane: "%0".to_string(),
            },
        }
    }
}