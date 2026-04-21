use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use hwp_dvc_core::checker::{CheckLevel, Checker, OutputScope};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::output;
use hwp_dvc_core::spec::DvcSpec;

/// Full version string shown by `--version / -v`.
///
/// Format: `hwp-dvc <semver> (reference-compatible: hancom-io/dvc)`
/// This matches the reference DVC tool's convention of embedding a
/// compatibility note in the version output.
const VERSION_STRING: &str = concat!(
    "hwp-dvc ",
    env!("CARGO_PKG_VERSION"),
    " (reference-compatible: hancom-io/dvc)"
);

/// Validate an HWPX document against a DVC JSON spec.
///
/// Mirrors the reference Hancom DVC CLI option table:
///
///   -j / --format json   JSON output (default)
///   -x / --format xml    XML output
///   -c / --console       Write to stdout (default)
///   --file <PATH>        Write to file instead of stdout
///   -a / --all           Report all errors (default)
///   -s / --simple        Stop at first error
///   -d / --default       Emit all categories (default, same as no flag)
///   -o / --alloption     Emit all categories (explicit)
///   -t / --table         Emit table findings only
///   -i / --tabledetail   Emit per-cell table findings only
///   -p / --shape         Emit shape (CharShape + ParaShape) findings only
///   -y / --style         Emit style findings only
///   -k / --hyperlink     Emit hyperlink findings only
///   -h / --help          Show this help message
///   -v / --version       Show version information
#[derive(Debug, Parser)]
#[command(
    name = "hwp-dvc",
    version = VERSION_STRING,
    about = "HWPX Document Validation Checker",
    long_about = None,
    disable_help_flag = true,
)]
struct Cli {
    /// Path to the DVC spec JSON file (the "checklist"). [-f]
    #[arg(long = "spec", short = 'f')]
    spec: PathBuf,

    /// Path to the HWPX document to validate.
    hwpx: PathBuf,

    /// Output format: json (default) or xml. [-j / -x]
    #[arg(long = "format", short = 'j', value_enum, default_value_t = Format::Json)]
    format: Format,

    /// Write the result to a file instead of stdout. [-c for console is default]
    #[arg(long = "file")]
    file: Option<PathBuf>,

    /// Stop at the first error; default reports all errors. [-s]
    #[arg(long = "simple", short = 's')]
    simple: bool,

    /// Report all errors (default). [-a]
    #[arg(long = "all", short = 'a')]
    all_errors: bool,

    /// Pretty-print JSON output.
    #[arg(long = "pretty")]
    pretty: bool,

    /// Emit all output categories (same as default). [-o]
    #[arg(long = "alloption", short = 'o')]
    all_option: bool,

    /// Emit default output (all categories). [-d]
    #[arg(long = "default", short = 'd')]
    default_scope: bool,

    /// Include table-level findings. [-t]
    #[arg(long = "table", short = 't')]
    table: bool,

    /// Include per-cell table findings. [-i]
    #[arg(long = "tabledetail", short = 'i')]
    table_detail: bool,

    /// Include shape findings (CharShape + ParaShape). [-p]
    #[arg(long = "shape", short = 'p')]
    shape: bool,

    /// Include style findings. [-y]
    #[arg(long = "style", short = 'y')]
    style: bool,

    /// Include hyperlink findings. [-k]
    #[arg(long = "hyperlink", short = 'k')]
    hyperlink: bool,

    /// Show help. [-h]
    #[arg(long = "help", short = 'h', action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Show version information. [-v]
    #[arg(long = "version", short = 'v', action = clap::ArgAction::Version)]
    version: Option<bool>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    /// JSON output (default) [-j]
    Json,
    /// XML output [-x] — only available when the `xml` feature is compiled in.
    #[cfg(feature = "xml")]
    Xml,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    // `-a / --all` and `-d / --default` are accepted for CLI parity with the
    // reference tool but have no additional effect: all-errors and default
    // scope are already the implicit defaults of CheckLevel and OutputScope.
    let _ = (cli.all_errors, cli.default_scope);

    let spec = DvcSpec::from_json_file(&cli.spec)
        .with_context(|| format!("failed to read spec file: {}", cli.spec.display()))?;

    let mut document = Document::open(&cli.hwpx)
        .with_context(|| format!("failed to open HWPX: {}", cli.hwpx.display()))?;

    document
        .parse()
        .with_context(|| format!("failed to parse HWPX: {}", cli.hwpx.display()))?;

    let level = if cli.simple { CheckLevel::Simple } else { CheckLevel::All };
    let scope = OutputScope {
        all: cli.all_option,
        table: cli.table,
        table_detail: cli.table_detail,
        shape: cli.shape,
        style: cli.style,
        hyperlink: cli.hyperlink,
    };
    let checker = Checker { spec: &spec, document: &document, level, scope };

    let errors = checker.run().context("validation run failed")?;

    let rendered = match cli.format {
        Format::Json => output::to_json(&errors, cli.pretty)?,
        #[cfg(feature = "xml")]
        Format::Xml => output::to_xml(&errors, cli.pretty)?,
    };

    if let Some(path) = cli.file.as_ref() {
        std::fs::write(path, rendered)
            .with_context(|| format!("failed to write output file: {}", path.display()))?;
    } else {
        println!("{rendered}");
    }

    Ok(())
}
