use clap::{Parser, Subcommand, ValueEnum};
use fileutil::commands;
use fileutil::schema;
use serde_json::Value;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(name = "fileutil")]
#[command(about = "Tool to examine the file system")]
struct Cli {
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// Write (or update) the csvdb column-schema file for all built-in subtools.
    /// Stale or missing table entries are replaced with the current definition.
    /// Tables registered by external plugins are left untouched.
    #[arg(long)]
    update_schema: bool,

    /// Path to the csvdb schema file.  Defaults to `schema.csvdb.json` in the
    /// current working directory.
    #[arg(long, default_value = "schema.csvdb.json")]
    schema_path: PathBuf,

    /// Redirect subcommand output to this file instead of stdout.
    /// The file is created or truncated.  Required when --output is provided.
    #[arg(long, value_name = "PATH")]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Table,
    Csv,
    Json,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Ls(commands::ls::LsArgs),
    Enum(commands::r#enum::EnumArgs),
    // Add more built-in subcommands here
    // For external subcommands, add variants here and dispatch accordingly
}

fn main() {
    let cli = Cli::parse();

    // Schema update runs first (before any subcommand), so callers can do
    // `fileutil --update-schema` without specifying a subcommand.
    if cli.update_schema {
        match schema::update_schema(&cli.schema_path) {
            Ok(n) => {
                if n > 0 {
                    eprintln!(
                        "fileutil: schema updated ({} table{} changed) → {}",
                        n,
                        if n == 1 { "" } else { "s" },
                        cli.schema_path.display()
                    );
                } else {
                    eprintln!(
                        "fileutil: schema is already up to date → {}",
                        cli.schema_path.display()
                    );
                }
            }
            Err(e) => {
                eprintln!("fileutil: failed to update schema: {e}");
                std::process::exit(1);
            }
        }
    } else if schema::is_schema_stale(&cli.schema_path) {
        // Resolve to an absolute path so the warning is unambiguous regardless
        // of the caller's working directory.
        let abs =
            std::fs::canonicalize(&cli.schema_path).unwrap_or_else(|_| cli.schema_path.clone());
        eprintln!(
            "fileutil: warning: schema file is out of date with the current fileutil schema: {}\n\
             fileutil:          run `fileutil --update-schema` to update it.",
            abs.display()
        );
    }

    // Run the subcommand if one was given.
    if let Some(command) = cli.command {
        let data = match command {
            Commands::Ls(args) => commands::ls::run(args),
            Commands::Enum(args) => commands::r#enum::run(args),
            // Handle other built-ins here
        };
        // Open the output destination: file when --output is given, else stdout.
        let mut writer: Box<dyn Write> = match &cli.output {
            Some(path) => {
                let file = std::fs::File::create(path).unwrap_or_else(|e| {
                    eprintln!(
                        "fileutil: cannot open output file '{}': {e}",
                        path.display()
                    );
                    std::process::exit(1);
                });
                Box::new(BufWriter::new(file))
            }
            None => Box::new(BufWriter::new(std::io::stdout())),
        };
        render_output(data, &cli.format, &mut *writer);
    } else if !cli.update_schema {
        // Neither a subcommand nor --update-schema was supplied — print help.
        use clap::CommandFactory;
        Cli::command().print_help().ok();
        eprintln!();
        std::process::exit(0);
    }
}

fn render_output(data: Value, format: &OutputFormat, out: &mut dyn Write) {
    match format {
        OutputFormat::Json => {
            writeln!(out, "{}", serde_json::to_string_pretty(&data).unwrap()).unwrap();
        }
        OutputFormat::Table => {
            render_table(&data, out);
        }
        OutputFormat::Csv => {
            render_csv(&data, out);
        }
    }
}

fn render_table(data: &Value, out: &mut dyn Write) {
    if let Value::Array(arr) = data {
        if arr.is_empty() {
            return;
        }
        let mut table = comfy_table::Table::new();
        table.load_preset(comfy_table::presets::UTF8_FULL);
        // Assume first item has keys
        if let Value::Object(obj) = &arr[0] {
            let headers: Vec<String> = obj.keys().cloned().collect();
            table.set_header(headers.clone());
            for item in arr {
                if let Value::Object(obj) = item {
                    let row: Vec<String> = headers
                        .iter()
                        .map(|h| obj.get(h).map(|v| v.to_string()).unwrap_or_default())
                        .collect();
                    table.add_row(row);
                }
            }
        }
        writeln!(out, "{}", table).unwrap();
    }
}

fn render_csv(data: &Value, out: &mut dyn Write) {
    if let Value::Array(arr) = data {
        if arr.is_empty() {
            return;
        }
        let mut wtr = csv::Writer::from_writer(out);
        if let Value::Object(obj) = &arr[0] {
            let headers: Vec<String> = obj.keys().cloned().collect();
            wtr.write_record(&headers).unwrap();
            for item in arr {
                if let Value::Object(obj) = item {
                    let record: Vec<String> = headers
                        .iter()
                        .map(|h| match obj.get(h) {
                            Some(Value::String(s)) => strip_datetime_tz(s).to_string(),
                            Some(Value::Bool(b)) => b.to_string(),
                            Some(Value::Number(n)) => n.to_string(),
                            Some(Value::Null) => "".to_string(),
                            _ => "".to_string(),
                        })
                        .collect();
                    wtr.write_record(&record).unwrap();
                }
            }
        }
        wtr.flush().unwrap();
    }
}

/// Strip the timezone offset from an RFC 3339 datetime string for CSV output.
///
/// By convention, CSV datetime values are expressed in the local locale without
/// an explicit timezone marker.  Consumers of the file are expected to know
/// that all timestamps are in the machine-local timezone.  Keeping the offset
/// would bloat every datetime cell with redundant information that is identical
/// across every row, and it conflicts with how most spreadsheet applications
/// (Excel, LibreOffice Calc) default-parse datetime strings.
///
/// The JSON and table renderers are unaffected; they retain the full RFC 3339
/// string with offset so that machine consumers get unambiguous timestamps.
///
/// Algorithm: find `T` (the date/time separator) then scan forward for the
/// first `+` or `-` that begins the UTC offset.  Everything before that sign
/// is returned.  Non-datetime strings pass through unchanged.
fn strip_datetime_tz(s: &str) -> &str {
    if let Some(t_pos) = s.find('T') {
        let after_t = &s[t_pos + 1..];
        if let Some(tz_offset) = after_t.find(|c| c == '+' || c == '-') {
            return &s[..t_pos + 1 + tz_offset];
        }
    }
    s
}

// Placeholder for handling external subcommands via protocol
// External plugins should be binaries named "fileutil-<subcommand>" that read JSON from stdin and write to stdout
#[allow(dead_code)]
fn handle_external_subcommand(subcommand_name: &str, args_json: String) {
    let plugin_name = format!("fileutil-{}", subcommand_name);
    match Command::new(&plugin_name)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                if stdin.write_all(args_json.as_bytes()).is_ok() {
                    stdin.flush().ok();
                }
            }
            if let Ok(output) = child.wait_with_output() {
                if output.status.success() {
                    if let Ok(response) = String::from_utf8(output.stdout) {
                        println!("{}", response);
                    }
                } else {
                    eprintln!("Plugin failed: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
        }
        Err(_) => {
            eprintln!(
                "No built-in or external plugin found for subcommand '{}'",
                subcommand_name
            );
        }
    }
}
