use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use hwp_dvc_core::checker::{CheckLevel, Checker, OutputScope};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::output;
use hwp_dvc_core::spec::DvcSpec;

/// Validate an HWPX document against a DVC JSON spec.
///
/// The CLI mirrors the reference implementation
/// (see `references/dvc/README.md` for the original option table):
///
/// * `-j / --format json` (default) | `-x / --format xml` (not yet implemented)
/// * `-c / --console` (default)     | `--file <PATH>`
/// * `-a / --all` (default)         | `-s / --simple`
/// * `-d` default | `-o` all | `-t` table | `-i` tabledetail | `-p` shape | `-y` style | `-k` hyperlink
#[derive(Debug, Parser)]
#[command(
    name = "hwp-dvc",
    version,
    about = "HWPX Document Validation Checker",
    disable_help_flag = true
)]
struct Cli {
    /// Path to the DVC spec JSON file (the "checklist").
    #[arg(long = "spec", short = 'f')]
    spec: PathBuf,

    /// Path to the HWPX document to validate.
    hwpx: PathBuf,

    #[arg(long = "format", short = 'F', value_enum, default_value_t = Format::Json)]
    format: Format,

    /// Write the result to a file instead of stdout.
    #[arg(long = "file")]
    file: Option<PathBuf>,

    /// Stop at the first error (default: report all).
    #[arg(long = "simple", short = 's')]
    simple: bool,

    /// Pretty-print JSON output.
    #[arg(long = "pretty")]
    pretty: bool,

    /// Report every category in the output.
    #[arg(long = "alloption", short = 'o')]
    all_option: bool,

    /// Include table-level findings.
    #[arg(long = "table", short = 't')]
    table: bool,

    /// Include per-cell table findings.
    #[arg(long = "tabledetail", short = 'i')]
    table_detail: bool,

    /// Include shape findings.
    #[arg(long = "shape", short = 'p')]
    shape: bool,

    /// Include style findings.
    #[arg(long = "style", short = 'y')]
    style: bool,

    /// Include hyperlink findings.
    #[arg(long = "hyperlink", short = 'k')]
    hyperlink: bool,

    /// Show help.
    #[arg(long = "help", short = 'h', action = clap::ArgAction::Help)]
    help: Option<bool>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Json,
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

    let spec = DvcSpec::from_json_file(&cli.spec)
        .with_context(|| format!("failed to read spec file: {}", cli.spec.display()))?;

    let mut document = Document::open(&cli.hwpx)
        .with_context(|| format!("failed to open HWPX: {}", cli.hwpx.display()))?;

    // TODO: Document::parse is not yet implemented. Once the OWPML
    // reader is in place this should be called here:
    // document.parse()?;
    let _ = &mut document;

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
