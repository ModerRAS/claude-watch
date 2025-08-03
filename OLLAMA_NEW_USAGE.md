# ollama-rs Ollama::new 方法正确用法文档

## 问题背景

在项目中使用 ollama-rs crate 时，遇到了 `Ollama::new` 方法的编译错误。通过分析编译错误和源代码，发现了正确的使用方法。

## 原本的问题代码

```rust
// ❌ 错误用法（会导致编译错误）
let ollama = ollama_rs::Ollama::new(url, None);
```

**编译错误信息：**
```
error[E0308]: mismatched types
   --> src/main.rs:59:46
    |
59  |     let ollama = ollama_rs::Ollama::new(url, None);
    |                  ----------------------      ^^^^ expected `u16`, found `Option<_>`
    |
    = note: expected type `u16`
               found enum `std::option::Option<_>`
note: associated function defined here
   --> /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ollama-rs-0.3.2/src/lib.rs:115:12
    |
115 |     pub fn new(host: impl IntoUrl, port: u16) -> Self {
```

## 正确的 API 签名

从编译错误中可以看到 `Ollama::new` 的正确签名：

```rust
pub fn new(host: impl IntoUrl, port: u16) -> Self
```

**参数说明：**
1. `host: impl IntoUrl` - 主机地址，可以是 `&str` 或 `String` 类型
2. `port: u16` - 端口号，必须是 `u16` 类型（0-65535）
3. 返回值：`Self` - 直接返回 `Ollama` 实例，不是 `Result`

## 正确用法示例

### 基本用法

```rust
// ✅ 正确用法 1: 使用本地默认配置
let ollama = Ollama::new("localhost", 11434);

// ✅ 正确用法 2: 使用远程服务器
let ollama = Ollama::new("192.168.1.100", 11434);

// ✅ 正确用法 3: 使用域名
let ollama = Ollama::new("ollama.example.com", 11434);

// ✅ 正确用法 4: 使用变量
let host = "localhost";
let port = 11434u16;
let ollama = Ollama::new(host, port);
```

### 从 URL 字符串创建连接

由于 API 需要独立的主机和端口参数，我们需要解析 URL 字符串：

```rust
/// 简化的 URL 解析函数
fn parse_url(url: &str) -> Result<(&str, u16), String> {
    // 移除协议前缀
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    
    // 分割主机和端口
    let parts: Vec<&str> = url.split(':').collect();
    match parts.as_slice() {
        [host, port_str] => {
            port_str.parse::<u16>()
                .map(|port| (*host, port))
                .map_err(|_| format!("无效的端口号: {}", port_str))
        }
        [host] => {
            // 默认端口 11434
            Ok((*host, 11434))
        }
        _ => Err(format!("无效的 URL 格式: {}", url)),
    }
}

// 使用示例
let url = "http://localhost:11434";
let (host, port) = parse_url(url)?;
let ollama = Ollama::new(host, port);
```

### 完整的使用示例

```rust
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};

async fn generate_response(prompt: &str, url: &str, model: &str) -> Result<String, String> {
    // 解析 URL
    let (host, port) = parse_url(url)?;
    
    // 创建客户端
    let ollama = Ollama::new(host, port);
    
    // 创建请求
    let request = GenerationRequest::new(
        model.to_string(),
        prompt.to_string(),
    );
    
    // 发送请求
    match ollama.generate(request).await {
        Ok(response) => Ok(response.response),
        Err(e) => Err(format!("Ollama 调用失败: {}", e)),
    }
}

// 使用示例
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompt = "你好！请介绍一下自己。";
    let url = "http://localhost:11434";
    let model = "llama3.2:1b";
    
    match generate_response(prompt, url, model).await {
        Ok(response) => println!("回答: {}", response),
        Err(e) => eprintln!("错误: {}", e),
    }
    
    Ok(())
}
```

### 封装成结构体

```rust
pub struct OllamaClient {
    client: Ollama,
    model: String,
}

impl OllamaClient {
    /// 从 URL 创建客户端
    pub fn from_url(url: &str, model: &str) -> Result<Self, String> {
        let (host, port) = parse_url(url)?;
        let client = Ollama::new(host, port);
        Ok(Self {
            client,
            model: model.to_string(),
        })
    }
    
    /// 使用默认配置
    pub fn default(model: &str) -> Self {
        Self {
            client: Ollama::new("localhost", 11434),
            model: model.to_string(),
        }
    }
    
    /// 生成响应
    pub async fn generate(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());
        let response = self.client.generate(request).await?;
        Ok(response.response)
    }
}

// 使用示例
let client = OllamaClient::from_url("http://localhost:11434", "llama3.2:1b")?;
let response = client.generate("你好！").await?;
```

## 常见错误和解决方法

### 错误 1: 传递 None 作为第二个参数

```rust
// ❌ 错误
let ollama = Ollama::new("localhost", None);
```

**错误信息：** `expected u16, found Option<_>`

**解决方法：** 传递具体的端口号
```rust
// ✅ 正确
let ollama = Ollama::new("localhost", 11434);
```

### 错误 2: 传递完整的 URL 字符串

```rust
// ❌ 错误
let ollama = Ollama::new("http://localhost:11434", 11434);
```

**错误信息：** 可能会连接失败，因为 host 包含协议

**解决方法：** 解析 URL
```rust
// ✅ 正确
let (host, port) = parse_url("http://localhost:11434")?;
let ollama = Ollama::new(host, port);
```

### 错误 3: 端口号超出范围

```rust
// ❌ 错误
let ollama = Ollama::new("localhost", 99999);
```

**错误信息：** 编译时错误，因为 99999 超出 u16 范围

**解决方法：** 使用有效的端口号
```rust
// ✅ 正确
let ollama = Ollama::new("localhost", 11434);
```

## 最佳实践

1. **URL 解析封装**: 将 URL 解析逻辑封装成函数，便于重用
2. **错误处理**: 提供清晰的错误信息，帮助调试
3. **默认值**: 使用默认端口 11434 当 URL 中没有指定时
4. **结构体封装**: 将 Ollama 客户端封装成结构体，提供更友好的 API
5. **配置管理**: 从环境变量或配置文件读取连接参数

## 性能考虑

1. **重用客户端**: 在应用程序中重用 Ollama 实例，避免重复创建
2. **连接池**: ollama-rs 内部使用 reqwest，会自动管理连接池
3. **异步使用**: 使用异步 API 提高性能

## 总结

ollama-rs 的 `Ollama::new` 方法签名很简单，但需要正确理解其参数要求：

- **第一个参数**: 主机地址（`&str` 或 `String`）
- **第二个参数**: 端口号（`u16`）
- **返回值**: 直接返回 `Ollama` 实例

通过正确使用这个 API，你可以轻松地连接到本地或远程的 Ollama 服务，并与各种大语言模型进行交互。