//! unit-convert core library.
//!
//! Conversion model: every unit belongs to a [`Category`] and is described by a
//! linear map to that category's canonical base unit:
//!
//! ```text
//! base_value = value * factor + offset
//! ```
//!
//! and the inverse, to render a base value in some target unit:
//!
//! ```text
//! target_value = (base_value - offset) / factor
//! ```
//!
//! For every category except temperature `offset == 0` and the relation is a
//! pure scale. Temperature is *affine*: its base is Kelvin, and e.g. Celsius is
//! `K = C*1 + 273.15`, Fahrenheit is `K = F*(5/9) + (273.15 - 32*5/9)`.
//!
//! Data sizes are intentionally split into decimal (KB = 1000 B) and binary
//! (KiB = 1024 B) units that share the same category (base = byte) but are
//! distinct units, so `1 GiB != 1 GB`.

use anyhow::{anyhow, bail, Result};

/// A physical-quantity category. Conversions are only legal within one category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Length,
    Mass,
    Temperature,
    DataSize,
    Time,
    Area,
    Volume,
    Speed,
}

impl Category {
    /// Stable lowercase name, also used by `--list`.
    pub fn name(self) -> &'static str {
        match self {
            Category::Length => "length",
            Category::Mass => "mass",
            Category::Temperature => "temperature",
            Category::DataSize => "data-size",
            Category::Time => "time",
            Category::Area => "area",
            Category::Volume => "volume",
            Category::Speed => "speed",
        }
    }

    /// The canonical base unit symbol for the category (what `factor`/`offset`
    /// map into). Purely informational for `--list`.
    pub fn base_unit(self) -> &'static str {
        match self {
            Category::Length => "m",
            Category::Mass => "g",
            Category::Temperature => "K",
            Category::DataSize => "B",
            Category::Time => "s",
            Category::Area => "m2",
            Category::Volume => "L",
            Category::Speed => "m/s",
        }
    }

    pub fn all() -> &'static [Category] {
        &[
            Category::Length,
            Category::Mass,
            Category::Temperature,
            Category::DataSize,
            Category::Time,
            Category::Area,
            Category::Volume,
            Category::Speed,
        ]
    }
}

/// A single unit: which category it's in, how it maps to the base, and the set
/// of accepted spellings (`symbols[0]` is the canonical display symbol).
#[derive(Debug, Clone, Copy)]
pub struct Unit {
    pub category: Category,
    /// Multiply a value in this unit by `factor` (then add `offset`) to reach base.
    pub factor: f64,
    /// Affine offset to base. Nonzero only for temperature.
    pub offset: f64,
    /// Accepted spellings. First entry is the canonical symbol shown by `--list`.
    pub symbols: &'static [&'static str],
}

// Handy temperature constants (base = Kelvin).
const F_FACTOR: f64 = 5.0 / 9.0;
// K = F*(5/9) + (273.15 - 32*5/9)
const F_OFFSET: f64 = 273.15 - 32.0 * (5.0 / 9.0);

/// The full unit table. Factors are exact where the definition is exact
/// (international foot/pound/etc.).
pub const UNITS: &[Unit] = &[
    // ---- Length (base: metre) ----
    Unit {
        category: Category::Length,
        factor: 1.0,
        offset: 0.0,
        symbols: &["m", "meter", "metre", "meters", "metres"],
    },
    Unit {
        category: Category::Length,
        factor: 1000.0,
        offset: 0.0,
        symbols: &["km", "kilometer", "kilometre", "kilometers", "kilometres"],
    },
    Unit {
        category: Category::Length,
        factor: 0.01,
        offset: 0.0,
        symbols: &[
            "cm",
            "centimeter",
            "centimetre",
            "centimeters",
            "centimetres",
        ],
    },
    Unit {
        category: Category::Length,
        factor: 0.001,
        offset: 0.0,
        symbols: &[
            "mm",
            "millimeter",
            "millimetre",
            "millimeters",
            "millimetres",
        ],
    },
    Unit {
        category: Category::Length,
        factor: 1609.344,
        offset: 0.0,
        symbols: &["mi", "mile", "miles"],
    },
    Unit {
        category: Category::Length,
        factor: 0.9144,
        offset: 0.0,
        symbols: &["yd", "yard", "yards"],
    },
    Unit {
        category: Category::Length,
        factor: 0.3048,
        offset: 0.0,
        symbols: &["ft", "foot", "feet"],
    },
    Unit {
        category: Category::Length,
        factor: 0.0254,
        offset: 0.0,
        symbols: &["in", "inch", "inches"],
    },
    Unit {
        category: Category::Length,
        factor: 1852.0,
        offset: 0.0,
        symbols: &["nmi", "nauticalmile", "nauticalmiles"],
    },
    // ---- Mass (base: gram) ----
    Unit {
        category: Category::Mass,
        factor: 1.0,
        offset: 0.0,
        symbols: &["g", "gram", "grams"],
    },
    Unit {
        category: Category::Mass,
        factor: 1000.0,
        offset: 0.0,
        symbols: &["kg", "kilogram", "kilograms"],
    },
    Unit {
        category: Category::Mass,
        factor: 0.001,
        offset: 0.0,
        symbols: &["mg", "milligram", "milligrams"],
    },
    Unit {
        category: Category::Mass,
        factor: 1_000_000.0,
        offset: 0.0,
        symbols: &["t", "tonne", "tonnes", "metricton"],
    },
    Unit {
        category: Category::Mass,
        factor: 453.59237,
        offset: 0.0,
        symbols: &["lb", "lbs", "pound", "pounds"],
    },
    Unit {
        category: Category::Mass,
        factor: 28.349523125,
        offset: 0.0,
        symbols: &["oz", "ounce", "ounces"],
    },
    Unit {
        category: Category::Mass,
        factor: 6350.29318,
        offset: 0.0,
        symbols: &["st", "stone", "stones"],
    },
    // ---- Temperature (base: Kelvin, affine) ----
    Unit {
        category: Category::Temperature,
        factor: 1.0,
        offset: 273.15,
        symbols: &["C", "c", "celsius", "centigrade", "degc"],
    },
    Unit {
        category: Category::Temperature,
        factor: F_FACTOR,
        offset: F_OFFSET,
        symbols: &["F", "f", "fahrenheit", "degf"],
    },
    Unit {
        category: Category::Temperature,
        factor: 1.0,
        offset: 0.0,
        symbols: &["K", "k", "kelvin"],
    },
    // ---- Data size (base: byte). Decimal AND binary, kept distinct. ----
    Unit {
        category: Category::DataSize,
        factor: 1.0,
        offset: 0.0,
        symbols: &["B", "byte", "bytes"],
    },
    Unit {
        category: Category::DataSize,
        factor: 1_000.0,
        offset: 0.0,
        symbols: &["KB", "kB", "kilobyte", "kilobytes"],
    },
    Unit {
        category: Category::DataSize,
        factor: 1_000_000.0,
        offset: 0.0,
        symbols: &["MB", "megabyte", "megabytes"],
    },
    Unit {
        category: Category::DataSize,
        factor: 1_000_000_000.0,
        offset: 0.0,
        symbols: &["GB", "gigabyte", "gigabytes"],
    },
    Unit {
        category: Category::DataSize,
        factor: 1_000_000_000_000.0,
        offset: 0.0,
        symbols: &["TB", "terabyte", "terabytes"],
    },
    Unit {
        category: Category::DataSize,
        factor: 1024.0,
        offset: 0.0,
        symbols: &["KiB", "kibibyte", "kibibytes"],
    },
    Unit {
        category: Category::DataSize,
        factor: 1_048_576.0,
        offset: 0.0,
        symbols: &["MiB", "mebibyte", "mebibytes"],
    },
    Unit {
        category: Category::DataSize,
        factor: 1_073_741_824.0,
        offset: 0.0,
        symbols: &["GiB", "gibibyte", "gibibytes"],
    },
    Unit {
        category: Category::DataSize,
        factor: 1_099_511_627_776.0,
        offset: 0.0,
        symbols: &["TiB", "tebibyte", "tebibytes"],
    },
    // ---- Time (base: second) ----
    Unit {
        category: Category::Time,
        factor: 1.0,
        offset: 0.0,
        symbols: &["s", "sec", "secs", "second", "seconds"],
    },
    Unit {
        category: Category::Time,
        factor: 60.0,
        offset: 0.0,
        symbols: &["min", "mins", "minute", "minutes"],
    },
    Unit {
        category: Category::Time,
        factor: 3600.0,
        offset: 0.0,
        symbols: &["hr", "hrs", "h", "hour", "hours"],
    },
    Unit {
        category: Category::Time,
        factor: 86400.0,
        offset: 0.0,
        symbols: &["day", "days", "d"],
    },
    Unit {
        category: Category::Time,
        factor: 604800.0,
        offset: 0.0,
        symbols: &["wk", "wks", "week", "weeks"],
    },
    // ---- Area (base: square metre) ----
    Unit {
        category: Category::Area,
        factor: 1.0,
        offset: 0.0,
        symbols: &["m2", "m^2", "sqm", "squaremeter", "squaremeters"],
    },
    Unit {
        category: Category::Area,
        factor: 1_000_000.0,
        offset: 0.0,
        symbols: &["km2", "km^2", "sqkm"],
    },
    Unit {
        category: Category::Area,
        factor: 0.09290304,
        offset: 0.0,
        symbols: &["ft2", "ft^2", "sqft", "squarefoot", "squarefeet"],
    },
    Unit {
        category: Category::Area,
        factor: 4046.8564224,
        offset: 0.0,
        symbols: &["acre", "acres"],
    },
    Unit {
        category: Category::Area,
        factor: 10000.0,
        offset: 0.0,
        symbols: &["ha", "hectare", "hectares"],
    },
    // ---- Volume (base: litre) ----
    Unit {
        category: Category::Volume,
        factor: 1.0,
        offset: 0.0,
        symbols: &["L", "l", "liter", "litre", "liters", "litres"],
    },
    Unit {
        category: Category::Volume,
        factor: 0.001,
        offset: 0.0,
        symbols: &[
            "mL",
            "ml",
            "milliliter",
            "millilitre",
            "milliliters",
            "millilitres",
        ],
    },
    Unit {
        category: Category::Volume,
        factor: 1000.0,
        offset: 0.0,
        symbols: &["m3", "m^3", "cubicmeter", "cubicmetre"],
    },
    Unit {
        category: Category::Volume,
        factor: 3.785411784,
        offset: 0.0,
        symbols: &["gal", "gallon", "gallons"],
    },
    Unit {
        category: Category::Volume,
        factor: 0.946352946,
        offset: 0.0,
        symbols: &["qt", "quart", "quarts"],
    },
    Unit {
        category: Category::Volume,
        factor: 0.473176473,
        offset: 0.0,
        symbols: &["pt", "pint", "pints"],
    },
    Unit {
        category: Category::Volume,
        factor: 0.2365882365,
        offset: 0.0,
        symbols: &["cup", "cups"],
    },
    Unit {
        category: Category::Volume,
        factor: 0.0295735295625,
        offset: 0.0,
        symbols: &["floz", "fluidounce", "fluidounces"],
    },
    // ---- Speed (base: metre/second) ----
    Unit {
        category: Category::Speed,
        factor: 1.0,
        offset: 0.0,
        symbols: &["m/s", "mps", "meterspersecond"],
    },
    Unit {
        category: Category::Speed,
        factor: 1000.0 / 3600.0,
        offset: 0.0,
        symbols: &["km/h", "kmh", "kph", "kilometersperhour"],
    },
    Unit {
        category: Category::Speed,
        factor: 1609.344 / 3600.0,
        offset: 0.0,
        symbols: &["mph", "milesperhour"],
    },
    Unit {
        category: Category::Speed,
        factor: 1852.0 / 3600.0,
        offset: 0.0,
        symbols: &["kn", "kt", "knot", "knots"],
    },
];

/// Look up a unit by any of its accepted spellings.
///
/// Matching is case-sensitive *first* (so `C`=Celsius, `c`=Celsius but `K`/`k`
/// resolve correctly and `m`/`M` stay distinct where it matters), then falls
/// back to a case-insensitive match for ergonomic input like `KM`/`Mph`. The
/// case-sensitive pass is what keeps the data-size SI prefixes unambiguous
/// (`KB` vs `KiB`, `MB` vs `MiB`).
pub fn find_unit(sym: &str) -> Option<&'static Unit> {
    // Exact, case-sensitive first.
    for u in UNITS {
        if u.symbols.iter().any(|s| *s == sym) {
            return Some(u);
        }
    }
    // Case-insensitive fallback. To avoid the KB/KiB collision, only consider
    // this when the lowercased input is unambiguous (matches exactly one unit).
    let low = sym.to_ascii_lowercase();
    let mut hit: Option<&'static Unit> = None;
    for u in UNITS {
        if u.symbols.iter().any(|s| s.to_ascii_lowercase() == low) {
            if hit.is_some() {
                return None; // ambiguous, force the user to be explicit
            }
            hit = Some(u);
        }
    }
    hit
}

/// Convert `value` from unit `from` to unit `to` (both given as strings).
///
/// Errors if either unit is unknown or the two units live in different
/// categories (e.g. `km` -> `kg`).
pub fn convert(value: f64, from: &str, to: &str) -> Result<f64> {
    let fu = find_unit(from).ok_or_else(|| anyhow!("unknown unit: '{from}'"))?;
    let tu = find_unit(to).ok_or_else(|| anyhow!("unknown unit: '{to}'"))?;

    if fu.category != tu.category {
        bail!(
            "cannot convert between '{from}' ({}) and '{to}' ({}): different categories",
            fu.category.name(),
            tu.category.name()
        );
    }

    // value -> base -> target
    let base = value * fu.factor + fu.offset;
    Ok((base - tu.offset) / tu.factor)
}

/// Round `value` to `sig` significant figures (used for display only).
///
/// `sig == 0` is treated as "no rounding" and returns the value unchanged.
pub fn round_sig(value: f64, sig: u32) -> f64 {
    if sig == 0 || value == 0.0 || !value.is_finite() {
        return value;
    }
    let d = value.abs().log10().floor() as i32;
    let power = (sig as i32) - 1 - d;
    let factor = 10f64.powi(power);
    (value * factor).round() / factor
}

/// Format a value for display: significant-figure rounding, then a compact
/// rendering that drops a redundant trailing `.0` and trailing zeros.
///
/// Exact whole-number results (e.g. `1 GiB to B` = `1073741824`) are printed in
/// full and are *not* truncated by the significant-figure setting — sig-figs
/// govern the precision of fractional results, not the magnitude of an integer
/// count. Only the fractional part is subject to rounding.
pub fn format_value(value: f64, sig: u32) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if !value.is_finite() {
        return value.to_string();
    }

    // A result that is (essentially) a whole number prints exactly. This keeps
    // large exact byte/second counts intact instead of being rounded to N sig
    // figs (which would turn 1073741824 into 1073740000 at the default 6).
    if value.fract().abs() < 1e-9 || (value - value.round()).abs() < 1e-6 * value.abs() {
        let rounded = value.round();
        // Within i64 range, render via integer to avoid float-to-string
        // scientific notation / precision artifacts.
        if rounded.abs() < 9.0e18 {
            return format!("{}", rounded as i64);
        }
        return format!("{:.0}", rounded);
    }

    let v = round_sig(value, sig);
    // Use enough precision to print the rounded value, then trim trailing zeros.
    let mut s = format!("{:.*}", (sig.max(1) + 4) as usize, v);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

/// Parse an expression of the form `VALUE UNIT to UNIT`, returning
/// `(value, from_unit, to_unit)`.
///
/// Accepts the value and from-unit either as separate tokens (`10 km to mi`) or
/// glued (`10km to mi`). The keyword between source and target may be `to` or
/// `in` (case-insensitive). Surrounding quotes on tokens are tolerated by the
/// caller (the CLI strips them); this function works on already-split tokens.
pub fn parse_expr(tokens: &[String]) -> Result<(f64, String, String)> {
    // Normalize: split a glued leading "10km" into "10" "km", and split any
    // token that itself contains the connective like "60mph".
    let mut flat: Vec<String> = Vec::new();
    for t in tokens {
        for piece in t.split_whitespace() {
            flat.push(piece.to_string());
        }
    }
    if flat.is_empty() {
        bail!("empty expression; expected: VALUE UNIT to UNIT");
    }

    // Find the connective ("to" / "in"), case-insensitive.
    let conn_pos = flat
        .iter()
        .position(|t| {
            let l = t.to_ascii_lowercase();
            l == "to" || l == "in"
        })
        .ok_or_else(|| anyhow!("missing 'to' in expression; expected: VALUE UNIT to UNIT"))?;

    let lhs = &flat[..conn_pos];
    let rhs = &flat[conn_pos + 1..];

    if rhs.is_empty() {
        bail!("missing target unit after 'to'");
    }
    // Target unit: join remaining tokens with no space (handles "km / h" edge).
    let to_unit = rhs.join("");

    // LHS is either ["10", "km"] or ["10km"].
    let (value, from_unit) = match lhs.len() {
        0 => bail!("missing value and source unit before 'to'"),
        1 => split_value_unit(&lhs[0])?,
        _ => {
            let value = lhs[0]
                .parse::<f64>()
                .map_err(|_| anyhow!("invalid number: '{}'", lhs[0]))?;
            let from_unit = lhs[1..].join("");
            (value, from_unit)
        }
    };

    Ok((value, from_unit, to_unit))
}

/// Split a glued `"10km"` / `"-3.5e2C"` into its numeric prefix and unit suffix.
fn split_value_unit(tok: &str) -> Result<(f64, String)> {
    // Walk to the end of the numeric prefix (digits, sign, decimal, exponent).
    let bytes = tok.as_bytes();
    let mut i = 0;
    let mut seen_digit = false;
    while i < bytes.len() {
        let c = bytes[i] as char;
        let is_num = c.is_ascii_digit()
            || c == '.'
            || ((c == '+' || c == '-') && (i == 0 || matches!(bytes[i - 1] as char, 'e' | 'E')))
            || ((c == 'e' || c == 'E') && seen_digit);
        if c.is_ascii_digit() {
            seen_digit = true;
        }
        if is_num {
            i += 1;
        } else {
            break;
        }
    }
    if i == 0 || !seen_digit {
        bail!("expected a number at start of '{tok}'");
    }
    let (num, unit) = tok.split_at(i);
    let value = num
        .parse::<f64>()
        .map_err(|_| anyhow!("invalid number: '{num}'"))?;
    if unit.is_empty() {
        bail!("missing source unit in '{tok}'");
    }
    Ok((value, unit.to_string()))
}

/// List all units, optionally filtered to one category. Returns a printable
/// multi-line string.
pub fn list_units(category: Option<Category>) -> String {
    let mut out = String::new();
    for cat in Category::all() {
        if let Some(want) = category {
            if *cat != want {
                continue;
            }
        }
        out.push_str(&format!("{} (base: {}):\n", cat.name(), cat.base_unit()));
        for u in UNITS {
            if u.category == *cat {
                let canonical = u.symbols[0];
                let aliases: Vec<&str> = u.symbols[1..].to_vec();
                if aliases.is_empty() {
                    out.push_str(&format!("  {}\n", canonical));
                } else {
                    out.push_str(&format!(
                        "  {}  (aliases: {})\n",
                        canonical,
                        aliases.join(", ")
                    ));
                }
            }
        }
        out.push('\n');
    }
    out
}

/// Resolve a category name (as accepted by `--list <category>`).
pub fn category_by_name(name: &str) -> Option<Category> {
    let n = name.to_ascii_lowercase();
    for c in Category::all() {
        if c.name() == n {
            return Some(*c);
        }
    }
    // a couple of friendly synonyms
    match n.as_str() {
        "temp" => Some(Category::Temperature),
        "data" | "datasize" | "data_size" => Some(Category::DataSize),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert two floats are within a relative+absolute epsilon.
    fn close(a: f64, b: f64) -> bool {
        let diff = (a - b).abs();
        diff <= 1e-9 || diff <= 1e-6 * a.abs().max(b.abs())
    }

    macro_rules! assert_close {
        ($a:expr, $b:expr) => {{
            let a = $a;
            let b = $b;
            assert!(
                close(a, b),
                "expected {} ≈ {} (diff {})",
                a,
                b,
                (a - b).abs()
            );
        }};
    }

    // ---------- Length ----------
    #[test]
    fn km_to_mi() {
        assert_close!(convert(1.0, "km", "mi").unwrap(), 0.621371);
    }

    #[test]
    fn ten_km_to_mi() {
        assert_close!(convert(10.0, "km", "mi").unwrap(), 6.2137119224);
    }

    #[test]
    fn ft_to_in_exact() {
        assert_close!(convert(1.0, "ft", "in").unwrap(), 12.0);
    }

    #[test]
    fn mi_to_km() {
        assert_close!(convert(1.0, "mi", "km").unwrap(), 1.609344);
    }

    // ---------- Temperature (affine) ----------
    #[test]
    fn c_to_f_boiling() {
        assert_close!(convert(100.0, "C", "F").unwrap(), 212.0);
    }

    #[test]
    fn c_to_f_freezing() {
        assert_close!(convert(0.0, "C", "F").unwrap(), 32.0);
    }

    #[test]
    fn c_to_k_freezing() {
        assert_close!(convert(0.0, "C", "K").unwrap(), 273.15);
    }

    #[test]
    fn f_to_c_98_6() {
        assert_close!(convert(98.6, "F", "C").unwrap(), 37.0);
    }

    #[test]
    fn k_to_c_absolute_zero() {
        assert_close!(convert(0.0, "K", "C").unwrap(), -273.15);
    }

    #[test]
    fn f_to_k() {
        // 32 F = 273.15 K
        assert_close!(convert(32.0, "F", "K").unwrap(), 273.15);
        // -40 F = -40 C = 233.15 K (the famous crossover)
        assert_close!(convert(-40.0, "F", "C").unwrap(), -40.0);
    }

    // ---------- Data size: decimal vs binary kept DISTINCT ----------
    #[test]
    fn gib_to_bytes() {
        assert_close!(convert(1.0, "GiB", "B").unwrap(), 1_073_741_824.0);
    }

    #[test]
    fn gb_to_bytes() {
        assert_close!(convert(1.0, "GB", "B").unwrap(), 1_000_000_000.0);
    }

    #[test]
    fn binary_not_equal_decimal() {
        let gib = convert(1.0, "GiB", "B").unwrap();
        let gb = convert(1.0, "GB", "B").unwrap();
        assert_ne!(gib, gb, "1 GiB must differ from 1 GB");
        assert_close!(gib - gb, 73_741_824.0);
    }

    #[test]
    fn gib_to_mb() {
        // 1 GiB = 1073741824 B = 1073.741824 MB (decimal MB)
        assert_close!(convert(1.0, "GiB", "MB").unwrap(), 1073.741824);
    }

    #[test]
    fn kib_vs_kb_lookup_distinct() {
        // case-sensitive lookup must keep KB (1000) and KiB (1024) apart
        assert_close!(convert(1.0, "KB", "B").unwrap(), 1000.0);
        assert_close!(convert(1.0, "KiB", "B").unwrap(), 1024.0);
        assert_close!(convert(1.0, "MiB", "B").unwrap(), 1_048_576.0);
        assert_close!(convert(1.0, "TiB", "B").unwrap(), 1_099_511_627_776.0);
    }

    // ---------- Speed ----------
    #[test]
    fn mph_to_kmh() {
        assert_close!(convert(60.0, "mph", "km/h").unwrap(), 96.56064);
    }

    #[test]
    fn ms_to_kmh() {
        assert_close!(convert(10.0, "m/s", "km/h").unwrap(), 36.0);
    }

    #[test]
    fn knots_to_kmh() {
        assert_close!(convert(1.0, "kn", "km/h").unwrap(), 1.852);
    }

    // ---------- Area ----------
    #[test]
    fn acre_to_m2() {
        assert_close!(convert(1.0, "acre", "m2").unwrap(), 4046.8564224);
    }

    #[test]
    fn ha_to_m2() {
        assert_close!(convert(1.0, "ha", "m2").unwrap(), 10000.0);
    }

    #[test]
    fn km2_to_m2() {
        assert_close!(convert(1.0, "km2", "m2").unwrap(), 1_000_000.0);
    }

    // ---------- Mass ----------
    #[test]
    fn kg_to_lb() {
        assert_close!(convert(1.0, "kg", "lb").unwrap(), 2.2046226218);
    }

    #[test]
    fn lb_to_oz() {
        assert_close!(convert(1.0, "lb", "oz").unwrap(), 16.0);
    }

    #[test]
    fn stone_to_lb() {
        assert_close!(convert(1.0, "st", "lb").unwrap(), 14.0);
    }

    #[test]
    fn t_to_kg() {
        assert_close!(convert(1.0, "t", "kg").unwrap(), 1000.0);
    }

    // ---------- Time ----------
    #[test]
    fn hr_to_min() {
        assert_close!(convert(1.0, "hr", "min").unwrap(), 60.0);
    }

    #[test]
    fn day_to_hr() {
        assert_close!(convert(1.0, "day", "hr").unwrap(), 24.0);
    }

    #[test]
    fn wk_to_day() {
        assert_close!(convert(1.0, "wk", "day").unwrap(), 7.0);
    }

    // ---------- Volume ----------
    #[test]
    fn gal_to_l() {
        assert_close!(convert(1.0, "gal", "L").unwrap(), 3.785411784);
    }

    #[test]
    fn l_to_ml() {
        assert_close!(convert(1.0, "L", "mL").unwrap(), 1000.0);
    }

    #[test]
    fn gal_to_qt() {
        assert_close!(convert(1.0, "gal", "qt").unwrap(), 4.0);
    }

    #[test]
    fn cup_to_floz() {
        assert_close!(convert(1.0, "cup", "floz").unwrap(), 8.0);
    }

    // ---------- Cross-category must error ----------
    #[test]
    fn cross_category_errors() {
        assert!(convert(1.0, "km", "kg").is_err());
        assert!(convert(1.0, "C", "m").is_err());
        assert!(convert(1.0, "GB", "s").is_err());
        assert!(convert(1.0, "L", "acre").is_err());
    }

    #[test]
    fn cross_category_error_message_mentions_categories() {
        let e = convert(1.0, "km", "kg").unwrap_err().to_string();
        assert!(e.contains("length"), "msg should name source category: {e}");
        assert!(e.contains("mass"), "msg should name target category: {e}");
    }

    #[test]
    fn unknown_unit_errors() {
        assert!(convert(1.0, "furlong", "m").is_err());
        assert!(convert(1.0, "m", "smoot").is_err());
    }

    // ---------- Round-trip A -> B -> A ≈ identity ----------
    #[test]
    fn round_trip_identity() {
        let cases = [
            (12.5, "km", "mi"),
            (37.0, "C", "F"),
            (3.0, "GiB", "MB"),
            (60.0, "mph", "km/h"),
            (2.5, "acre", "ha"),
            (5.0, "kg", "lb"),
            (90.0, "min", "hr"),
            (2.0, "gal", "L"),
            (273.15, "K", "F"),
        ];
        for (v, a, b) in cases {
            let there = convert(v, a, b).unwrap();
            let back = convert(there, b, a).unwrap();
            assert_close!(back, v);
        }
    }

    // ---------- Parsing ----------
    #[test]
    fn parse_separate_tokens() {
        let toks = vec![
            "10".to_string(),
            "km".to_string(),
            "to".to_string(),
            "mi".to_string(),
        ];
        let (v, f, t) = parse_expr(&toks).unwrap();
        assert_eq!((v, f.as_str(), t.as_str()), (10.0, "km", "mi"));
    }

    #[test]
    fn parse_glued_value_unit() {
        let toks = vec!["10km".to_string(), "to".to_string(), "mi".to_string()];
        let (v, f, t) = parse_expr(&toks).unwrap();
        assert_eq!((v, f.as_str(), t.as_str()), (10.0, "km", "mi"));
    }

    #[test]
    fn parse_quoted_phrase_tokens() {
        // simulating `"60 mph" to "km/h"` after the shell splits quotes off
        let toks = vec!["60 mph".to_string(), "to".to_string(), "km/h".to_string()];
        let (v, f, t) = parse_expr(&toks).unwrap();
        assert_eq!((v, f.as_str(), t.as_str()), (60.0, "mph", "km/h"));
    }

    #[test]
    fn parse_in_keyword() {
        let toks = vec![
            "100".to_string(),
            "C".to_string(),
            "in".to_string(),
            "F".to_string(),
        ];
        let (v, f, t) = parse_expr(&toks).unwrap();
        assert_eq!((v, f.as_str(), t.as_str()), (100.0, "C", "F"));
    }

    #[test]
    fn parse_negative_and_scientific() {
        let toks = vec!["-3.5e2C".to_string(), "to".to_string(), "F".to_string()];
        let (v, f, t) = parse_expr(&toks).unwrap();
        assert_eq!((v, f.as_str(), t.as_str()), (-350.0, "C", "F"));
    }

    #[test]
    fn parse_missing_to_errors() {
        let toks = vec!["10".to_string(), "km".to_string(), "mi".to_string()];
        assert!(parse_expr(&toks).is_err());
    }

    #[test]
    fn parse_missing_target_errors() {
        let toks = vec!["10".to_string(), "km".to_string(), "to".to_string()];
        assert!(parse_expr(&toks).is_err());
    }

    // ---------- Formatting / precision ----------
    #[test]
    fn round_sig_basic() {
        assert_close!(round_sig(6.2137119224, 6), 6.21371);
        assert_close!(round_sig(1234.5678, 2), 1200.0);
        assert_close!(round_sig(0.00123456, 3), 0.00123);
    }

    #[test]
    fn format_trims_trailing_zeros() {
        assert_eq!(format_value(36.0, 6), "36");
        assert_eq!(format_value(1.5, 6), "1.5");
        assert_eq!(format_value(0.0, 6), "0");
    }

    #[test]
    fn format_respects_precision() {
        // 6.2137119224 to 3 sig figs => 6.21
        assert_eq!(format_value(6.2137119224, 3), "6.21");
    }

    // ---------- Lookup ergonomics ----------
    #[test]
    fn case_insensitive_fallback() {
        // KM not in table but unambiguous lowercased -> km
        assert!(find_unit("KM").is_some());
        assert!(find_unit("Mph").is_some());
    }

    #[test]
    fn list_contains_all_categories() {
        let all = list_units(None);
        for c in Category::all() {
            assert!(all.contains(c.name()), "list missing category {}", c.name());
        }
        // and a filtered list only contains the one
        let only_len = list_units(Some(Category::Length));
        assert!(only_len.contains("length"));
        assert!(!only_len.contains("temperature"));
    }

    #[test]
    fn category_by_name_resolves() {
        assert_eq!(category_by_name("length"), Some(Category::Length));
        assert_eq!(category_by_name("data-size"), Some(Category::DataSize));
        assert_eq!(category_by_name("temp"), Some(Category::Temperature));
        assert_eq!(category_by_name("nonsense"), None);
    }
}
