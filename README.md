# DeepSeek Cli Engineer - Rust 实现

## 功能特点

- 支持流式响应
- 保留对话历史上下文
- 自动修剪过长的对话历史
- 友好的终端交互界面
- 支持中文交互

## 安装与运行

1. 确保已安装 Rust 工具链 (rustc, cargo)
2. 克隆项目:
   ```bash
   git clone https://github.com/utilsrc/deepseek-cli-engineer-rust.git
   cd deepseek-cli-engineer-rust
   ```
3. 配置环境变量:
   ```bash
   echo 'export DEEPSEEK_API_KEY="your-api-key"' > init.sh
   source init.sh
   ```
4. 运行程序:
   ```bash
   cargo run
   ```

## 环境变量

必须设置 `DEEPSEEK_API_KEY` 环境变量，可以通过以下方式:

1. 直接设置:
   ```bash
   export DEEPSEEK_API_KEY="your-api-key"
   ```
2. 或通过 init.sh 文件自动加载

## 使用示例

```bash
🔵 You> 如何优化Rust代码性能?
🐋 Seeking...

💭 Reasoning:
分析性能优化需要考虑...
(详细的技术分析)

🤖 Assistant> 优化Rust代码性能的几个关键点:
1. 使用合适的集合类型...
2. 避免不必要的克隆...
3. 利用并行处理...
```

## 注意事项

- 此版本不支持文件操作
- 需要有效的 DeepSeek API key
- 对话历史最多保留20条消息
- 按 Ctrl+C 可中断当前响应
- 输入 "exit" 或 "quit" 退出程序

## 依赖项

- reqwest (HTTP客户端)
- tokio (异步运行时)
- serde (JSON序列化)
- colored (终端颜色)

## 许可证

MIT License
