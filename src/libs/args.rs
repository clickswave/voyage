use clap::{Parser, ValueEnum};
use crate::libs;

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

    /// Specify the interval in ms between each request. Defaults to 0
    #[arg(short, long, default_value_t = 0)]
    pub interval: i64,

    /// Specify the number of tasks to use. Defaults to 2
    #[arg(short, long, default_value_t = 2)]
    pub tasks: i64,

    /// Starts matching config (if any) from scratch
    #[arg(long, default_value_t = false)]
    pub fresh_start: bool,

    /// Ignore any progress
    #[arg(long, default_value_t = false)]
    pub volatile: bool,

    /// Specify the user-agent
    #[arg(short, long, default_value_t = format!("voyage/0.0.0"))]
    pub agent: String,

    /// Disable banner display on startup
    #[arg(long, default_value_t = false)]
    pub no_banner: bool,

    /// Specify launch delay in seconds
    #[arg(long, default_value_t = 0)]
    pub launch_delay: i64,

    /// Set log level to debug, info, warn, error. Defaults to warn
    #[arg(long, value_enum, default_value_t = LogLevel::Warn)]
    pub log_level: LogLevel,

    /// Set output format to text or csv. Defaults to warn
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub output_format: OutputFormat,

    /// Specify the output file path
    #[arg(short, long, default_value_t = format!(""))]
    pub output_path: String,
}

pub fn parse() -> Args {
    let args = Args::parse();

    // display banner unless disabled
    if !args.no_banner {
        libs::banner::full();
    }

    // sleep for launch delay
    if args.launch_delay > 0 {
        std::thread::sleep(std::time::Duration::from_secs(args.launch_delay as u64));
    }

    args
}
