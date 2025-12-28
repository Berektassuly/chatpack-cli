# chatpack-cli

CLI tool for converting chat exports into LLM-friendly formats. Achieves up to 13x token compression with CSV output.

[![Crates.io](https://img.shields.io/crates/v/chatpack-cli.svg)](https://crates.io/crates/chatpack-cli)
[![CI](https://github.com/Berektassuly/chatpack-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/Berektassuly/chatpack-cli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[Documentation](https://berektassuly.com/chatpack-compress-chat-exports-for-llm-rust) | [Library](https://github.com/Berektassuly/chatpack) | [Web Version](https://chatpack.berektassuly.com)

## Overview

chatpack-cli transforms chat exports from Telegram, WhatsApp, Instagram, and Discord into formats optimized for LLM context windows. The tool handles format-specific edge cases like WhatsApp locale detection and Instagram Mojibake encoding automatically.

**Token compression comparison (34K messages):**

| Format | Tokens | Compression |
|--------|--------|-------------|
| Raw Telegram JSON | 11,177,258 | baseline |
| CSV | 849,915 | 13.2x |
| JSONL | 1,029,130 | 10.9x |
| JSON (clean) | 1,333,586 | 8.4x |

## Installation

### Pre-built binaries

Download from [GitHub Releases](https://github.com/Berektassuly/chatpack-cli/releases):

| Platform | Architecture | Filename |
|----------|--------------|----------|
| Linux | x86_64 | `chatpack-linux-x86_64.tar.gz` |
| Linux | ARM64 | `chatpack-linux-aarch64.tar.gz` |
| Linux | musl (static) | `chatpack-linux-x86_64-musl.tar.gz` |
| macOS | Intel | `chatpack-macos-x86_64.tar.gz` |
| macOS | Apple Silicon | `chatpack-macos-aarch64.tar.gz` |
| Windows | x86_64 | `chatpack-windows-x86_64.zip` |

### From crates.io

```bash
cargo install chatpack-cli
```

### From source

```bash
git clone https://github.com/Berektassuly/chatpack-cli
cd chatpack-cli
cargo install --path .
```

## Quick Start

```bash
chatpack tg result.json           # Telegram
chatpack wa chat.txt              # WhatsApp
chatpack ig message_1.json        # Instagram
chatpack dc export.json           # Discord
```

Output: `optimized_chat.csv`

## Usage

```
chatpack <SOURCE> <INPUT> [OPTIONS]

Arguments:
  <SOURCE>    Chat source: telegram (tg), whatsapp (wa), instagram (ig), discord (dc)
  <INPUT>     Input file path

Options:
  -o, --output <FILE>     Output file [default: optimized_chat.csv]
  -f, --format <FORMAT>   Output format: csv, json, jsonl [default: csv]
  -t, --timestamps        Include timestamps
  -r, --replies           Include reply references
  -e, --edited            Include edit timestamps
      --ids               Include message IDs
      --no-merge          Disable consecutive message merging
      --after <DATE>      Filter: messages after date (YYYY-MM-DD)
      --before <DATE>     Filter: messages before date (YYYY-MM-DD)
      --from <USER>       Filter: messages from specific sender
      --no-streaming      Load entire file into memory
  -p, --progress          Show processing progress
  -q, --quiet             Suppress informational output
  -h, --help              Print help
  -V, --version           Print version
```

## Examples

### Format conversion

```bash
chatpack tg export.json -o chat.csv
chatpack tg export.json -f json -o chat.json
chatpack tg export.json -f jsonl -o chat.jsonl
```

### Filtering

```bash
chatpack tg chat.json --after 2024-01-01 --before 2024-12-31
chatpack tg chat.json --from "Alice"
chatpack tg chat.json --from "Bob" --after 2024-06-01
```

### Metadata options

```bash
chatpack tg chat.json -t                    # with timestamps
chatpack tg chat.json -t -r -e --ids        # all metadata
chatpack tg chat.json --no-merge            # disable merging
```

## Message Merging

By default, consecutive messages from the same sender are merged into single entries:

```
Before (5 messages):          After (2 entries):
Alice: Hey                    Alice: Hey / How are you? / See the project?
Alice: How are you?           Bob: Yeah, looked / Pretty good
Alice: See the project?
Bob: Yeah, looked
Bob: Pretty good
```

This provides ~24% additional token reduction and improves embedding quality for RAG pipelines. Disable with `--no-merge`.

## Supported Platforms

| Platform | Format | Notes |
|----------|--------|-------|
| Telegram | JSON | Full metadata support (IDs, replies, edits, forwards) |
| WhatsApp | TXT | Auto-detects 4 locale-specific date formats |
| Instagram | JSON | Automatic Mojibake encoding fix |
| Discord | JSON | Attachments, stickers, replies |

## Performance

- Speed: 20K+ messages/sec
- Memory: ~3x input file size
- Recommended: files up to 500MB (streaming mode default)

## Library Usage

This CLI wraps the [`chatpack`](https://crates.io/crates/chatpack) library:

```rust
use chatpack::prelude::*;

fn main() -> chatpack::Result<()> {
    let parser = create_parser(Platform::Telegram);
    let messages = parser.parse("export.json".as_ref())?;
    
    let merged = merge_consecutive(messages);
    write_csv(&merged, "output.csv", &OutputConfig::new())?;
    
    Ok(())
}
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
cargo test
cargo fmt --check
cargo clippy
```

## Requirements

Rust 1.85+ (edition 2024)

## License

MIT License. See [LICENSE](LICENSE) for details.

## Links

- [chatpack library](https://github.com/Berektassuly/chatpack)
- [Web version](https://chatpack.berektassuly.com) (WASM, runs locally in browser)
- [Author's article](https://berektassuly.com/chatpack-compress-chat-exports-for-llm-rust)
