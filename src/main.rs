use colored::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};
use tokio_stream::StreamExt;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    max_completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ChatResponseChoice {
    delta: Option<Delta>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
    reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Option<Vec<ChatResponseChoice>>,
}

struct DeepSeekClient {
    client: Client,
    api_key: String,
    base_url: String,
    conversation_history: Vec<ChatMessage>,
}

impl DeepSeekClient {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("DEEPSEEK_API_KEY")
            .map_err(|_| "DEEPSEEK_API_KEY environment variable not set")?;

        let client = Client::new();
        let base_url = "https://api.deepseek.com/chat/completions".to_string();

        let system_prompt = r#"
你是一名叫做 DeepSeek Cli Engineer 的精英软件工程师，拥有跨所有编程领域数十年的经验。
你的专业知识涵盖系统设计、算法、测试和最佳实践。
你提供深思熟虑、结构良好的解决方案，同时解释你的推理过程。

重要限制：
- 你无法执行任何文件操作（读取、写入、创建或编辑文件）
- 你无法访问本地文件系统
- 当被问及文件操作时，请礼貌地说明你只支持回答问题和提供代码建议
- 专注于代码分析、解释、最佳实践和理论指导

指导原则：
1. 提供自然的、对话式的回答，解释你的推理过程
2. 如果被要求执行文件操作，请说明限制并提供其他帮助
3. 遵循特定语言的最佳实践
4. 在适当时建议测试或验证步骤
5. 在分析和建议中保持全面性

请记住：你是一名用于代码讨论和建议的高级工程师 - 请保持深思熟虑、精确，并清楚地解释你的推理过程。
"#;

        let conversation_history = vec![ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        }];

        Ok(DeepSeekClient {
            client,
            api_key,
            base_url,
            conversation_history,
        })
    }

    async fn send_message(&mut self, user_input: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Add user message to conversation history
        self.conversation_history.push(ChatMessage {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        // Prepare the request
        let request = ChatRequest {
            model: "deepseek-reasoner".to_string(),
            messages: self.conversation_history.clone(),
            stream: true,
            max_completion_tokens: 64000,
        };

        // Make the API request
        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API Error: {}", error_text).into());
        }

        // Handle streaming response
        let mut stream = response.bytes_stream();
        let mut assistant_content = String::new();
        let mut reasoning_started = false;
        let mut response_started = false;

        println!("\n{}", "🐋 思考中...".bright_blue().bold());

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            // Parse each line that starts with "data: "
            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let json_str = &line[6..]; // Remove "data: " prefix

                    if json_str == "[DONE]" {
                        break;
                    }

                    if let Ok(parsed) = serde_json::from_str::<ChatResponse>(json_str) {
                        if let Some(choices) = parsed.choices {
                            for choice in choices {
                                if let Some(delta) = choice.delta {
                                    // Handle reasoning content
                                    if let Some(reasoning) = delta.reasoning_content {
                                        if !reasoning_started {
                                            println!("\n{}", "💭 推理过程:".blue().bold());
                                            reasoning_started = true;
                                        }
                                        print!("{}", reasoning);
                                        io::stdout().flush().unwrap();
                                    }

                                    // Handle regular content
                                    if let Some(content) = delta.content {
                                        if reasoning_started && !response_started {
                                            println!("\n");
                                            print!("{} ", "🤖 助手>".bright_blue().bold());
                                            response_started = true;
                                        } else if !response_started {
                                            print!("{} ", "🤖 助手>".bright_blue().bold());
                                            response_started = true;
                                        }
                                        print!("{}", content);
                                        io::stdout().flush().unwrap();
                                        assistant_content.push_str(&content);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!(); // New line after streaming

        // Add assistant response to conversation history
        if !assistant_content.is_empty() {
            self.conversation_history.push(ChatMessage {
                role: "assistant".to_string(),
                content: assistant_content,
            });
        }

        // Trim conversation history if it gets too long
        self.trim_conversation_history();

        Ok(())
    }

    fn trim_conversation_history(&mut self) {
        if self.conversation_history.len() <= 20 {
            return;
        }

        // Keep system message and last 15 messages
        let system_msg = self.conversation_history[0].clone();
        let recent_messages: Vec<_> = self
            .conversation_history
            .iter()
            .skip(1)
            .rev()
            .take(15)
            .rev()
            .cloned()
            .collect();

        self.conversation_history = vec![system_msg];
        self.conversation_history.extend(recent_messages);
    }
}

fn print_welcome() {
    let welcome = r#"
╔══════════════════════════════════════════════════════════════════╗
║                    🐋 DeepSeek Cli Engineer - Rust               ║
║                   由 DeepSeek-R1 推理模型驱动                    ║
╚══════════════════════════════════════════════════════════════════╝
"#;

    println!("{}", welcome.bright_blue().bold());

    let instructions = r#"
💡 使用方法:
  • 自然地提出任何编程问题
  • 获得详细的解释和代码建议
  • 注意：此版本不支持文件操作
  
🎯 命令:
  • 输入 'exit' 或 'quit' 结束会话
  • 按 Ctrl+C 中断
"#;

    println!("{}", instructions.blue());
}

fn get_user_input() -> Result<String, Box<dyn std::error::Error>> {
    print!("{}", "🔵 你> ".bright_cyan().bold());
    io::stdout().flush()?;

    // 设置终端为原始模式
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;
    stdout.flush()?;

    let mut input = String::new();
    for c in stdin.keys() {
        match c? {
            termion::event::Key::Char('\n') => break,
            termion::event::Key::Char(c) => {
                input.push(c);
                print!("{}", c);
                stdout.flush()?;
            },
            termion::event::Key::Backspace => {
                if !input.is_empty() {
                    // 获取要删除的字符
                    if let Some(last_char) = input.chars().last() {
                        // 删除字符串中的最后一个字符
                        input.pop();
                        
                        // 计算该字符的显示宽度
                        let char_width = last_char.width().unwrap_or(1);
                        
                        // 根据字符宽度删除相应数量的终端显示位置
                        for _ in 0..char_width {
                            print!("\x08 \x08");
                        }
                        stdout.flush()?;
                    }
                }
            },
            _ => {}
        }
    }

    Ok(input)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if it exists
    if let Err(_) = dotenvy::dotenv() {
        // .env file doesn't exist or can't be read, that's okay
    }

    print_welcome();

    let mut client = match DeepSeekClient::new() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("{} {}", "❌ 错误:".red().bold(), e);
            eprintln!("{}", "请设置 DEEPSEEK_API_KEY 环境变量。".yellow());
            std::process::exit(1);
        }
    };

    loop {
        match get_user_input() {
            Ok(input) => {
                if input.is_empty() {
                    continue;
                }

                if input.to_lowercase() == "exit" || input.to_lowercase() == "quit" {
                    println!("{}", "👋 再见！编程愉快！".bright_blue().bold());
                    break;
                }

                // Check for file operation attempts
                if input.contains("/add")
                    || input.to_lowercase().contains("read file")
                    || input.to_lowercase().contains("create file")
                    || input.to_lowercase().contains("write file")
                    || input.contains("读取文件")
                    || input.contains("创建文件")
                    || input.contains("写入文件")
                    || input.contains("文件操作")
                {
                    println!("{}", "\n⚠️  此版本不支持文件操作。".yellow().bold());
                    println!(
                        "{}",
                        "我可以帮助您解答代码问题、提供解释和最佳实践建议！".yellow()
                    );
                    continue;
                }

                if let Err(e) = client.send_message(&input).await {
                    eprintln!("{} {}", "❌ 错误:".red().bold(), e);
                }
            }
            Err(e) => {
                eprintln!("{} {}", "❌ 输入错误:".red().bold(), e);
                break;
            }
        }
    }

    println!(
        "{}",
        "✨ 会话结束。感谢使用 DeepSeek Cli Engineer！".blue().bold()
    );
    Ok(())
}