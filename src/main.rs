//! chatpack-cli: Convert chat exports to LLM-friendly formats
//!
//! A command-line tool for parsing chat exports from Telegram, WhatsApp,
//! Instagram, and Discord, and converting them to CSV, JSON, or JSONL formats.

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use chatpack::prelude::*;

/// Parse and convert chat exports into LLM-friendly formats.
///
/// Supports Telegram, WhatsApp, Instagram, and Discord exports.
/// Outputs to CSV (default), JSON, or JSONL formats optimized for LLM context.
#[derive(Parser, Debug)]
#[command(name = "chatpack")]
#[command(version, about, long_about = None)]
#[command(after_help = "\x1b[1mExamples:\x1b[0m
  chatpack tg export.json                     # Telegram to CSV
  chatpack wa chat.txt -o chat.csv            # WhatsApp to CSV  
  chatpack ig messages.json -f json           # Instagram to JSON
  chatpack dc export.json --after 2024-01-01  # Discord with date filter
  chatpack tg export.json --no-streaming      # Load entire file into memory

\x1b[1mToken Compression:\x1b[0m
  CSV:   ~13x compression (92% savings) - best for LLM context
  JSONL: ~11x compression (91% savings) - good for RAG pipelines
  JSON:  ~8x compression (88% savings)  - keeps full structure")]
struct Cli {
    /// Chat source platform
    #[arg(
        value_enum,
        help = "Source platform: telegram, whatsapp, instagram, discord"
    )]
    source: Source,

    /// Input file path
    #[arg(help = "Path to the exported chat file")]
    input: PathBuf,

    /// Output file path
    #[arg(
        short,
        long,
        default_value = "optimized_chat.csv",
        help = "Output file path"
    )]
    output: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value = "csv", help = "Output format")]
    format: Format,

    /// Include timestamps in output
    #[arg(short, long, help = "Include message timestamps")]
    timestamps: bool,

    /// Include reply references
    #[arg(short, long, help = "Include reply-to references")]
    replies: bool,

    /// Include edit timestamps
    #[arg(short, long, help = "Include edit timestamps")]
    edited: bool,

    /// Include message IDs
    #[arg(long, help = "Include message IDs")]
    ids: bool,

    /// Don't merge consecutive messages from the same sender
    #[arg(long, help = "Disable message merging")]
    no_merge: bool,

    /// Filter: only messages after this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE", help = "Only messages after this date")]
    after: Option<String>,

    /// Filter: only messages before this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE", help = "Only messages before this date")]
    before: Option<String>,

    /// Filter: only messages from specific sender
    #[arg(long, value_name = "USER", help = "Only messages from this sender")]
    from: Option<String>,

    /// Disable streaming mode (load entire file into memory)
    #[arg(long, help = "Load entire file into memory instead of streaming")]
    no_streaming: bool,

    /// Show progress during processing
    #[arg(long, short = 'p', help = "Show processing progress")]
    progress: bool,

    /// Quiet mode: suppress all output except errors
    #[arg(long, short = 'q', help = "Suppress informational output")]
    quiet: bool,
}

/// Supported chat source platforms
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Source {
    /// Telegram (JSON export)
    #[value(alias = "tg")]
    Telegram,
    /// WhatsApp (TXT export)
    #[value(alias = "wa")]
    Whatsapp,
    /// Instagram (JSON export)
    #[value(alias = "ig")]
    Instagram,
    /// Discord (JSON/TXT/CSV export)
    #[value(alias = "dc")]
    Discord,
}

impl Source {
    fn to_platform(self) -> Platform {
        match self {
            Source::Telegram => Platform::Telegram,
            Source::Whatsapp => Platform::WhatsApp,
            Source::Instagram => Platform::Instagram,
            Source::Discord => Platform::Discord,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Source::Telegram => "Telegram",
            Source::Whatsapp => "WhatsApp",
            Source::Instagram => "Instagram",
            Source::Discord => "Discord",
        }
    }
}

/// Output format options
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Format {
    /// CSV format (best for LLM context, ~13x token compression)
    Csv,
    /// JSON array format
    Json,
    /// JSON Lines format (one object per line, for RAG pipelines)
    Jsonl,
}

impl Format {
    fn name(self) -> &'static str {
        match self {
            Format::Csv => "CSV",
            Format::Json => "JSON",
            Format::Jsonl => "JSONL",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Validate input file exists
    if !cli.input.exists() {
        bail!(
            "Input file not found: {}\n\nTip: Make sure the path is correct and the file exists.",
            cli.input.display()
        );
    }

    if !cli.quiet {
        eprintln!(
            "📦 Parsing {} export: {}",
            cli.source.name(),
            cli.input.display()
        );
    }

    // Build filter configuration
    let mut filter = FilterConfig::new();

    if let Some(ref after_date) = cli.after {
        filter = filter.with_date_from(after_date).with_context(|| {
            format!(
                "Invalid --after date format: '{}'. Expected YYYY-MM-DD",
                after_date
            )
        })?;
    }

    if let Some(ref before_date) = cli.before {
        filter = filter.with_date_to(before_date).with_context(|| {
            format!(
                "Invalid --before date format: '{}'. Expected YYYY-MM-DD",
                before_date
            )
        })?;
    }

    if let Some(ref sender) = cli.from {
        filter = filter.with_sender(sender);
    }

    // Build output configuration
    let mut output_config = OutputConfig::new();

    if cli.timestamps {
        output_config = output_config.with_timestamps();
    }

    if cli.replies {
        output_config = output_config.with_replies();
    }

    if cli.edited {
        output_config = output_config.with_edited();
    }

    if cli.ids {
        output_config = output_config.with_ids();
    }

    // Parse messages
    let messages = if cli.no_streaming {
        parse_full(&cli)?
    } else {
        parse_streaming(&cli)?
    };

    let total_parsed = messages.len();

    // Apply filters
    let filtered = apply_filters(messages, &filter);
    let filtered_count = filtered.len();

    // Optionally merge consecutive messages
    let processed = if cli.no_merge {
        filtered
    } else {
        merge_consecutive(filtered)
    };

    let final_count = processed.len();

    // Write output
    write_output(&processed, &cli, &output_config)?;

    // Print summary
    if !cli.quiet {
        print_summary(&cli, total_parsed, filtered_count, final_count);
    }

    Ok(())
}

/// Parse using full in-memory loading
fn parse_full(cli: &Cli) -> Result<Vec<Message>> {
    let platform = cli.source.to_platform();
    let parser = create_parser(platform);

    if cli.progress && !cli.quiet {
        eprintln!("⏳ Loading entire file into memory...");
    }

    let messages = parser
        .parse(&cli.input)
        .with_context(|| format!("Failed to parse {} export", cli.source.name()))?;

    if cli.progress && !cli.quiet {
        eprintln!("✓ Loaded {} messages", messages.len());
    }

    Ok(messages)
}

/// Parse using streaming (memory-efficient)
fn parse_streaming(cli: &Cli) -> Result<Vec<Message>> {
    let platform = cli.source.to_platform();
    let parser = create_streaming_parser(platform);

    let mut messages = Vec::new();
    let mut count = 0;

    if cli.progress && !cli.quiet {
        eprintln!("⏳ Streaming messages...");
    }

    let stream = parser
        .stream(&cli.input)
        .with_context(|| format!("Failed to open {} export for streaming", cli.source.name()))?;

    for result in stream {
        let msg = result.with_context(|| format!("Error at message {}", count + 1))?;
        messages.push(msg);
        count += 1;

        if cli.progress && !cli.quiet && count % 10000 == 0 {
            eprint!("\r⏳ Processed {} messages...", count);
        }
    }

    if cli.progress && !cli.quiet && count >= 10000 {
        eprintln!("\r✓ Streamed {} messages    ", count);
    } else if cli.progress && !cli.quiet {
        eprintln!("✓ Streamed {} messages", count);
    }

    Ok(messages)
}

/// Write messages to the output file in the specified format
fn write_output(messages: &[Message], cli: &Cli, config: &OutputConfig) -> Result<()> {
    let output_path = cli
        .output
        .to_str()
        .with_context(|| format!("Invalid output path: {}", cli.output.display()))?;

    match cli.format {
        Format::Csv => {
            write_csv(messages, output_path, config)
                .with_context(|| format!("Failed to write CSV to {}", cli.output.display()))?;
        }
        Format::Json => {
            write_json(messages, output_path, config)
                .with_context(|| format!("Failed to write JSON to {}", cli.output.display()))?;
        }
        Format::Jsonl => {
            write_jsonl(messages, output_path, config)
                .with_context(|| format!("Failed to write JSONL to {}", cli.output.display()))?;
        }
    }

    Ok(())
}

/// Print processing summary
fn print_summary(cli: &Cli, total: usize, filtered: usize, final_count: usize) {
    let has_filters = cli.after.is_some() || cli.before.is_some() || cli.from.is_some();
    let merged = !cli.no_merge && filtered != final_count;

    eprintln!();
    eprintln!("✅ \x1b[1mDone!\x1b[0m");
    eprintln!("   📥 Parsed:   {} messages", total);

    if has_filters {
        eprintln!("   🔍 Filtered: {} messages", filtered);
    }

    if merged {
        eprintln!("   🔀 Merged:   {} → {} entries", filtered, final_count);
    }

    eprintln!(
        "   📤 Output:   {} ({})",
        cli.output.display(),
        cli.format.name()
    );
}
