//! Thin CLI over the `unit_convert` library.
//!
//! Usage:
//!   unit-convert 10 km to mi
//!   unit-convert 100 C to F
//!   unit-convert 1 GiB to MB
//!   unit-convert "60 mph" to "km/h"
//!   unit-convert --precision 3 10 km to mi
//!   unit-convert --list
//!   unit-convert --list length

use anyhow::Result;
use clap::Parser;
use unit_convert::{category_by_name, convert, format_value, list_units, parse_expr};

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
                  unit-convert --list length"
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

fn run() -> Result<()> {
    let cli = Cli::parse();

    // --list [category]
    if let Some(cat_arg) = cli.list {
        if cat_arg.is_empty() {
            print!("{}", list_units(None));
        } else {
            let cat = category_by_name(&cat_arg).ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown category: '{cat_arg}'. Try one of: length, mass, \
                     temperature, data-size, time, area, volume, speed"
                )
            })?;
            print!("{}", list_units(Some(cat)));
        }
        return Ok(());
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
