# 📦 chatpack-cli

> CLI tool to convert chat exports into LLM-friendly formats. Compress tokens **13x** with CSV output.

[![Crates.io](https://img.shields.io/crates/v/chatpack-cli.svg)](https://crates.io/crates/chatpack-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🚀 **Fast** — 20K+ messages/sec with streaming by default
- 📱 **Multi-platform** — Telegram, WhatsApp, Instagram, Discord
- 🔀 **Smart merge** — Consecutive messages from same sender → one entry
- 🎯 **Powerful filters** — By date range, by sender
- 📄 **Multiple formats** — CSV (13x compression), JSON, JSONL (for RAG)
- 💾 **Memory efficient** — Streaming mode for large files (default)

## Installation

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
# Telegram JSON export
chatpack tg result.json

# WhatsApp TXT export  
chatpack wa chat.txt

# Instagram JSON export
chatpack ig message_1.json

# Discord export
chatpack dc chat.json
```

**Output:** `optimized_chat.csv` — ready to paste into ChatGPT/Claude.

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
      --no-merge          Don't merge consecutive messages
      --after <DATE>      Filter: messages after date (YYYY-MM-DD)
      --before <DATE>     Filter: messages before date (YYYY-MM-DD)
      --from <USER>       Filter: messages from specific sender
      --no-streaming      Load entire file into memory (default: streaming)
  -p, --progress          Show processing progress
  -q, --quiet             Suppress informational output
  -h, --help              Print help
  -V, --version           Print version
```

## Examples

### Basic Usage

```bash
# Convert Telegram export to CSV
chatpack tg export.json

# Specify output file
chatpack wa chat.txt -o conversation.csv

# Use JSON format
chatpack ig messages.json -f json -o output.json
```

### With Filters

```bash
# Messages from 2024
chatpack tg chat.json --after 2024-01-01 --before 2024-12-31

# Messages from specific user
chatpack wa chat.txt --from "Alice"

# Combine filters
chatpack tg chat.json --from "Bob" --after 2024-06-01
```

### With Metadata

```bash
# Include timestamps
chatpack tg chat.json -t

# Include all metadata
chatpack tg chat.json -t -r -e --ids

# Keep messages separate (no merging)
chatpack tg chat.json --no-merge
```

### Memory Management

```bash
# Default: streaming mode (memory efficient)
chatpack tg huge_export.json

# Load entire file into memory (faster for small files)
chatpack tg small_chat.json --no-streaming

# Show progress for large files
chatpack tg huge_export.json -p
```

## Token Compression

| Format | Compression | Best For |
|--------|-------------|----------|
| **CSV** | ~13x (92% savings) | LLM context windows |
| JSONL | ~11x (91% savings) | RAG pipelines |
| JSON | ~8x (88% savings) | API integrations |

## Streaming vs Full Loading

| Mode | Memory Usage | Speed | Use Case |
|------|--------------|-------|----------|
| **Streaming** (default) | Low | Normal | Large files (500MB+) |
| Full (`--no-streaming`) | High | Faster | Small files (<50MB) |

## Supported Platforms

| Platform | Export Format | Features |
|----------|---------------|----------|
| Telegram | JSON | IDs, timestamps, replies, edits, forwarded messages |
| WhatsApp | TXT | Auto-detects 4 locale-specific date formats |
| Instagram | JSON | Fixes Mojibake encoding from Meta exports |
| Discord | JSON/TXT/CSV | Attachments, stickers, replies |

## How to Export Chats

### Telegram
1. Open chat → ⋮ menu → Export Chat History
2. Choose JSON format
3. Run: `chatpack tg result.json`

### WhatsApp
1. Open chat → ⋮ menu → More → Export Chat
2. Choose "Without Media"
3. Run: `chatpack wa chat.txt`

### Instagram
1. Settings → Privacy and Security → Download Data
2. Request JSON format
3. Find `messages/inbox/<chat>/message_1.json`
4. Run: `chatpack ig message_1.json`

### Discord
1. Use DiscordChatExporter or similar tool
2. Export in JSON format
3. Run: `chatpack dc export.json`

## Library Usage

This CLI is built on top of the [`chatpack`](https://crates.io/crates/chatpack) library. 
You can use the library directly in your Rust projects:

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

## Requirements

- Rust 1.85+ (edition 2024)

## License

[MIT](LICENSE) © [Mukhammedali Berektassuly](https://berektassuly.com)

## Related

- [`chatpack`](https://crates.io/crates/chatpack) — The underlying library
- [chatpack.berektassuly.com](https://chatpack.berektassuly.com) — Online version (no installation needed)
