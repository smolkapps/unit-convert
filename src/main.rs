//! Thin CLI over the `unit_convert` library.
//!
//! Usage:
//!   unit-convert 10 km to mi
//!   unit-convert 100 C to F
//!   unit-convert 1 GiB to MB
//!   unit-convert "60 mph" to "km/h"
//!   unit-convert --precision 3 10 km to mi
//!   unit-convert list
//!   unit-convert list length
//!   unit-convert --list length

use anyhow::Result;
use clap::Parser;
use unit_convert::{category_by_name, convert, format_value, list_units, parse_expr, Category};

#[derive(Parser, Debug)]
#[command(
    name = "unit-convert",
    version,
    about = "Convert between physical units",
    long_about = "Convert between physical units.\n\n\
                  Examples:\n  \
                  unit-convert 10 km to mi\n  \
                  unit-convert 100 C to F\n  \
                  unit-convert 1 GiB to MB\n  \
                  unit-convert \"60 mph\" to \"km/h\"\n  \
                  unit-convert list\n  \
                  unit-convert list length"
)]
struct Cli {
    /// Significant figures for the result (default 6). 0 means no rounding.
    #[arg(short, long, default_value_t = 6)]
    precision: u32,

    /// List supported units (optionally for a single category) and exit.
    #[arg(long, value_name = "CATEGORY", num_args = 0..=1, default_missing_value = "")]
    list: Option<String>,

    /// The conversion expression: VALUE UNIT to UNIT
    #[arg(
        value_name = "EXPR",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    expr: Vec<String>,
}

/// Resolve an optional category name into the filter passed to `list_units`.
/// An empty/`None` argument lists every category; anything else must name a
/// known category.
fn resolve_list_category(arg: Option<&str>) -> Result<Option<Category>> {
    match arg {
        None | Some("") => Ok(None),
        Some(name) => category_by_name(name).map(Some).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown category: '{name}'. Try one of: length, mass, \
                 temperature, data-size, time, area, volume, speed, pressure"
            )
        }),
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // --list [category]
    if let Some(cat_arg) = cli.list {
        let cat = resolve_list_category(Some(cat_arg.as_str()))?;
        print!("{}", list_units(cat));
        return Ok(());
    }

    // `list [category]` subcommand — same output as `--list`, nicer to type.
    if let Some(first) = cli.expr.first() {
        if first.eq_ignore_ascii_case("list") {
            let cat = resolve_list_category(cli.expr.get(1).map(String::as_str))?;
            print!("{}", list_units(cat));
            return Ok(());
        }
    }

    if cli.expr.is_empty() {
        anyhow::bail!(
            "no expression given.\nUsage: unit-convert VALUE UNIT to UNIT \
             (e.g. `unit-convert 10 km to mi`)\nUse --list to see supported units."
        );
    }

    let (value, from, to) = parse_expr(&cli.expr)?;
    let result = convert(value, &from, &to)?;
    println!("{} {}", format_value(result, cli.precision), to);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
