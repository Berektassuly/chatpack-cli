//! chatpack-cli: Convert chat exports to LLM-friendly formats
//!
//! A command-line tool for parsing chat exports from Telegram, WhatsApp,
//! Instagram, and Discord, and converting them to CSV, JSON, or JSONL formats.

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

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
  chatpack tg export.json --all-metadata      # Include all optional metadata
  chatpack dc export.jsonl -f ndjson          # JSONL/NDJSON for RAG

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

    /// Include all optional metadata fields
    #[arg(long, help = "Include timestamps, IDs, replies, and edit timestamps")]
    all_metadata: bool,

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

    /// Stop on invalid records instead of skipping them during streaming
    #[arg(long, help = "Fail on invalid records during streaming")]
    strict: bool,

    /// Streaming read buffer size in bytes
    #[arg(long, value_name = "BYTES", help = "Streaming read buffer size")]
    buffer_size: Option<usize>,

    /// Maximum single message size in bytes during streaming
    #[arg(
        long,
        value_name = "BYTES",
        help = "Maximum single message size during streaming"
    )]
    max_message_size: Option<usize>,

    /// Keep WhatsApp system messages
    #[arg(long, help = "Keep WhatsApp system messages")]
    keep_system_messages: bool,

    /// Disable Instagram mojibake encoding fix
    #[arg(long, help = "Disable Instagram encoding repair")]
    no_fix_encoding: bool,

    /// Show progress during processing
    #[arg(long, short = 'p', help = "Show processing progress")]
    progress: bool,

    /// Report progress every N messages
    #[arg(
        long,
        value_name = "N",
        default_value_t = 10_000,
        help = "Message interval for progress updates"
    )]
    progress_interval: usize,

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
    #[value(alias = "ndjson")]
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

impl From<Format> for OutputFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::Csv => OutputFormat::Csv,
            Format::Json => OutputFormat::Json,
            Format::Jsonl => OutputFormat::Jsonl,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    validate_options(&cli)?;

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
    let mut output_config = if cli.all_metadata {
        OutputConfig::all()
    } else {
        OutputConfig::new()
    };

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
    let messages = if should_use_streaming(&cli) {
        parse_streaming(&cli)?
    } else {
        parse_full(&cli)?
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
    let parser = create_configured_parser(cli, false);

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
    let parser = create_configured_parser(cli, true);

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

        if cli.progress && !cli.quiet && count % cli.progress_interval == 0 {
            eprint!("\r⏳ Processed {} messages...", count);
        }
    }

    if cli.progress && !cli.quiet && count >= cli.progress_interval {
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

    write_to_format(messages, output_path, cli.format.into(), config).with_context(|| {
        format!(
            "Failed to write {} to {}",
            cli.format.name(),
            cli.output.display()
        )
    })?;

    Ok(())
}

fn validate_options(cli: &Cli) -> Result<()> {
    if matches!(cli.buffer_size, Some(0)) {
        bail!("--buffer-size must be greater than 0");
    }

    if matches!(cli.max_message_size, Some(0)) {
        bail!("--max-message-size must be greater than 0");
    }

    if cli.progress_interval == 0 {
        bail!("--progress-interval must be greater than 0");
    }

    if cli.keep_system_messages && cli.source != Source::Whatsapp {
        bail!("--keep-system-messages is only supported for WhatsApp exports");
    }

    if cli.no_fix_encoding && cli.source != Source::Instagram {
        bail!("--no-fix-encoding is only supported for Instagram exports");
    }

    Ok(())
}

fn should_use_streaming(cli: &Cli) -> bool {
    if cli.no_streaming {
        return false;
    }

    if cli.source == Source::Whatsapp && cli.keep_system_messages {
        return false;
    }

    if cli.source == Source::Instagram && cli.no_fix_encoding {
        return false;
    }

    if cli.source == Source::Discord {
        return is_streamable_discord_input(&cli.input);
    }

    true
}

fn is_streamable_discord_input(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            ["jsonl", "ndjson"]
                .iter()
                .any(|candidate| ext.eq_ignore_ascii_case(candidate))
        })
}

fn create_configured_parser(cli: &Cli, streaming: bool) -> Box<dyn chatpack::parser::Parser> {
    match cli.source {
        Source::Telegram => {
            let mut config = if streaming {
                TelegramConfig::streaming()
            } else {
                TelegramConfig::new()
            }
            .with_skip_invalid(!cli.strict);

            if let Some(buffer_size) = cli.buffer_size {
                config = config.with_buffer_size(buffer_size);
            }

            if let Some(max_message_size) = cli.max_message_size {
                config = config.with_max_message_size(max_message_size);
            }

            Box::new(TelegramParser::with_config(config))
        }
        Source::Whatsapp => {
            let mut config = if streaming {
                WhatsAppConfig::streaming()
            } else {
                WhatsAppConfig::new()
            }
            .with_skip_invalid(!cli.strict)
            .with_skip_system_messages(!cli.keep_system_messages);

            if let Some(buffer_size) = cli.buffer_size {
                config = config.with_buffer_size(buffer_size);
            }

            Box::new(WhatsAppParser::with_config(config))
        }
        Source::Instagram => {
            let mut config = if streaming {
                InstagramConfig::streaming()
            } else {
                InstagramConfig::new()
            }
            .with_skip_invalid(!cli.strict)
            .with_fix_encoding(!cli.no_fix_encoding);

            if let Some(buffer_size) = cli.buffer_size {
                config = config.with_buffer_size(buffer_size);
            }

            if let Some(max_message_size) = cli.max_message_size {
                config = config.with_max_message_size(max_message_size);
            }

            Box::new(InstagramParser::with_config(config))
        }
        Source::Discord => {
            let mut config = if streaming {
                DiscordConfig::streaming()
            } else {
                DiscordConfig::new()
            }
            .with_skip_invalid(!cli.strict);

            if let Some(buffer_size) = cli.buffer_size {
                config = config.with_buffer_size(buffer_size);
            }

            if let Some(max_message_size) = cli.max_message_size {
                config = config.with_max_message_size(max_message_size);
            }

            Box::new(DiscordParser::with_config(config))
        }
    }
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
