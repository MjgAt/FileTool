use clap::{Parser, Subcommand, ValueEnum};
use fileutil::commands;
use serde_json::Value;
use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Parser)]
#[command(name = "fileutil")]
#[command(about = "Tool to examine the file system")]
struct Cli {
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,
    #[command(subcommand)]
    command: Commands,
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
    let data = match cli.command {
        Commands::Ls(args) => commands::ls::run(args),
        Commands::Enum(args) => commands::r#enum::run(args),
        // Handle other built-ins
        // For external subcommands, add cases like: Commands::Custom => handle_external("custom", serde_json::to_string(args).unwrap())
    };
    render_output(data, &cli.format);
}

fn render_output(data: Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
        OutputFormat::Table => {
            render_table(&data);
        }
        OutputFormat::Csv => {
            render_csv(&data);
        }
    }
}

fn render_table(data: &Value) {
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
                    let row: Vec<String> = headers.iter().map(|h| {
                        obj.get(h).map(|v| v.to_string()).unwrap_or_default()
                    }).collect();
                    table.add_row(row);
                }
            }
        }
        println!("{}", table);
    }
}

fn render_csv(data: &Value) {
    if let Value::Array(arr) = data {
        if arr.is_empty() {
            return;
        }
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        if let Value::Object(obj) = &arr[0] {
            let headers: Vec<String> = obj.keys().cloned().collect();
            wtr.write_record(&headers).unwrap();
            for item in arr {
                if let Value::Object(obj) = item {
                    let record: Vec<String> = headers.iter().map(|h| {
                        match obj.get(h) {
                            Some(Value::String(s)) => s.clone(),
                            Some(Value::Bool(b)) => b.to_string(),
                            Some(Value::Number(n)) => n.to_string(),
                            Some(Value::Null) => "".to_string(),
                            _ => "".to_string(),
                        }
                    }).collect();
                    wtr.write_record(&record).unwrap();
                }
            }
        }
        wtr.flush().unwrap();
    }
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
            eprintln!("No built-in or external plugin found for subcommand '{}'", subcommand_name);
        }
    }
}