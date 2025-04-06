use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warn => "warn",
                LogLevel::Error => "error",
            }
        )
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Text,
    Csv,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OutputFormat::Text => "text",
                OutputFormat::Csv => "csv",
            }
        )
    }
}

/// {n}
/// |---------------------------------------------------|{n}
/// |                    V O Y A G E                    |{n}
/// |---------------------------------------------------|{n}
/// |      Stateful subdomain enumeration toolkit       |{n}
/// |                                                   |{n}
/// |                     voyage.vg                     |{n}
/// |---------------------------------------------------|{n}
#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Specify the domain you want to target.
    /// If a wordlist is not provided, default wordlist will be used.
    /// {n}
    #[arg(short, long, verbatim_doc_comment, required = true)]
    pub domain: Vec<String>,

    /// Specify the wordlist to be used{n}
    #[arg(short, long, default_value_t = format!(""))]
    pub wordlist_path: String,

    /// Specify the interval in ms between each request for a task. Defaults to 0
    #[arg(short, long, default_value_t = 0)]
    pub interval: u64,

    /// Specify the number of tasks to use
    #[arg(short, long, default_value_t = 2)]
    pub tasks: i64,

    /// Starts matching config (if any) from scratch
    #[arg(long, default_value_t = false)]
    pub fresh_start: bool,

    // TODO: implement this feature
    /// Ignore any progress
    // #[arg(long, default_value_t = false)]
    // pub volatile: bool,

    /// Specify whether the user agent should be randomized or remain static for an active enumeration task
    #[arg(long, default_value_t = false)]
    pub active_random_user_agent: bool,

    /// Specify whether the user agent should be randomized or remain static for a passive enumeration task
    #[arg(long, default_value_t = false)]
    pub passive_random_user_agent: bool,

    /// Specify the user agent to be used for active enumeration.
    #[arg(short, long, default_value_t = format!("voyage/{}", env!("CARGO_PKG_VERSION")))]
    pub active_user_agent: String,

    /// Specify the user agent to be used for passive enumeration.
    #[arg(short, long, default_value_t = format!("voyage/{}", env!("CARGO_PKG_VERSION")))]
    pub passive_user_agent: String,

    /// Disable banner display on startup
    #[arg(long, default_value_t = false)]
    pub no_banner: bool,

    /// Delete existing database and start from scratch.
    #[arg(long, default_value_t = false)]
    pub recreate_database: bool,

    /// Specify launch delay in seconds
    #[arg(long, default_value_t = 0)]
    pub launch_delay: i64,

    /// Set minimum log level to debug, info, warn, error
    #[arg(long, value_enum, default_value_t = LogLevel::Debug)]
    pub log_level: LogLevel,

    /// Set output format to text or csv
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub output_format: OutputFormat,

    /// Specify the output file path
    #[arg(short, long, default_value_t = format!(""))]
    pub output_path: String,

    /// Disable passive subdomain enumeration
    #[arg(long, default_value_t = false)]
    pub disable_passive_enum: bool,

    /// Disable active subdomain enumeration
    #[arg(long, default_value_t = false)]
    pub disable_active_enum: bool,

    /// Specify the passive enumeration sources to be excluded from passive enumeration
    #[arg(long)]
    pub exclude_passive_source: Vec<String>,

    /// Specify the active enumeration to be excluded from passive enumeration
    #[arg(long)]
    pub exclude_active_technique: Vec<String>,

    /// Specify ports to be used for http probing
    #[arg(long, default_values_t = vec![80])]
    pub http_probing_port: Vec<u16>,

    /// Specify ports to be used for https probing
    #[arg(long, default_values_t = vec![443])]
    pub https_probing_port: Vec<u16>,
}

pub fn parse() -> Args {
    let args = Args::parse();

    // if both passive and active enumeration are disabled, exit
    if args.disable_passive_enum && args.disable_active_enum {
        eprintln!("[WARN] Cannot proceed. Passive and active enumeration are disabled.");
        std::process::exit(1);
    }

    // sleep for launch delay
    if args.launch_delay > 0 {
        std::thread::sleep(std::time::Duration::from_secs(args.launch_delay as u64));
    }

    args
}
