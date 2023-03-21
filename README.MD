# ChatGPT-Rust v0.1.0 README

ChatGPT-Rust is a Rust implementation of OpenAI's ChatGPT. This project aims to bring the power of the ChatGPT model to the Rust ecosystem and eventually deploy it on WebAssembly (WASM) for seamless integration into web applications. The project is being developed iteratively, with several versions planned, including a version that supports ChatGPT's permanent memory feature.

## Getting Started

### Example
![image](https://raw.githubusercontent.com/RevAtN/chatGPT-rust/main/example.gif)

### Prerequisites

To run this project, you will need:

- Rust: Install the latest version of Rust from the [official website](https://www.rust-lang.org/tools/install).
- Cargo: Cargo is included with Rust by default.

### Setup

1. Clone the repository:
git clone https://github.com/your-username/chatGPT-rust.git
2. Change the working directory to the project folder:
cd chatGPT-rust
3. Set up your OpenAI API key as an environment variable:
export OPENAI_API_KEY=sk-...
4. Run the project using Cargo:
cargo run

### API
gpt-3.5-turbo:
```
        let payload = serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": messages,
            "temperature": 1.0,
            "top_p": 1.0,
            "n": 1,
            "stream": true,
            "presence_penalty": 0,
            "frequency_penalty": 0
        });
```
text-embedding-ada-002
```
        let payload_embedding = serde_json::json!({
            "model": "text-embedding-ada-002",
            "input": result_text
        });
```

## Features

- Stream text output (`stream=true`): The current version (v0.1.0) supports streaming text output for a more interactive and responsive experience.
- Terminal-based: ChatGPT-Rust runs directly in your terminal, providing a simple and straightforward user interface.
- WebAssembly (WASM) support (coming soon): Future versions of ChatGPT-Rust will be deployable to WASM, enabling seamless integration with web applications.

## Contributing

We welcome contributions from the community! Please submit issues and pull requests on the GitHub repository, and follow the guidelines provided in the [CONTRIBUTING.md](CONTRIBUTING.md) file.

## License

This project is licensed under the [MIT License](LICENSE.md).

## Acknowledgments

- OpenAI for the development of the ChatGPT model and API.
- The Rust community for providing guidance and resources.

## Credits

- Architecture inspired by [Dave Shapiro's](https://www.patreon.com/daveshap) work
- Dave Shapiro's YT video Series:
  - [Tutorial: DIY ChatGPT with Long Term Memories](https://www.youtube.com/watch?v=c3aiCrk0F0U)
  - [DIY ChatGPT: Enhancing RAVEN's long-term memories and starting to work on self reflection](https://www.youtube.com/watch?v=QGLF3UbDf7g)
  - [RAVEN Dream Sequence - Memory Consolidation and Insight Extraction for AGI or cognitive architecture](https://www.youtube.com/watch?v=QGLF3UbDf7g)


