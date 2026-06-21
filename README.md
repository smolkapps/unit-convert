# unit-convert

A small, fast, dependency-light Rust CLI for converting between physical units.
No network, no external unit database — every unit is defined by an internal
table of `(category, factor-to-base, offset)`.

## Install / build

```sh
cargo build --release
# binary at target/release/unit-convert
```

## Usage

```sh
unit-convert 10 km to mi          # 6.21371 mi
unit-convert 100 C to F           # 212 F
unit-convert 1 GiB to MB          # 1073.74 MB
unit-convert "60 mph" to "km/h"   # 96.5606 km/h
unit-convert 1 acre to m2         # 4046.86 m2
```

The expression is `VALUE UNIT to UNIT`. You can:

- write the value and unit as separate tokens (`10 km`) or glued (`10km`);
- use `to` or `in` as the connective;
- quote a phrase so the shell passes it as one arg (`"60 mph"`).

### Options

| Flag | Description |
| --- | --- |
| `-p, --precision <N>` | Significant figures in the result (default `6`; `0` = no rounding). |
| `--list [CATEGORY]` | List supported units; optionally restrict to one category. |
| `-V, --version` | Print version. |
| `-h, --help` | Print help. |

```sh
unit-convert --precision 3 10 km to mi   # 6.21 mi
unit-convert --list                      # every category and unit
unit-convert --list temperature          # just one category
```

## How conversion works

Each unit maps to its category's **base unit** by a linear relation:

```
base_value   = value * factor + offset
target_value = (base_value - target_offset) / target_factor
```

For every category except temperature, `offset = 0` and this is a pure scale.
**Temperature is affine** (base = Kelvin): Celsius is `K = C + 273.15`,
Fahrenheit is `K = F*(5/9) + (273.15 - 32*5/9)`. Converting `100 C to F` goes
`100 C -> 373.15 K -> 212 F`.

Conversions across categories (`km` to `kg`) are refused with a clear error.

## Data sizes: decimal vs binary are DISTINCT

This is a common source of bugs, so the table keeps them separate:

- **Decimal (SI):** `KB`=1000 B, `MB`=10^6, `GB`=10^9, `TB`=10^12.
- **Binary (IEC):** `KiB`=1024 B, `MiB`=2^20, `GiB`=2^30, `TiB`=2^40.

So `1 GiB = 1073741824 B` but `1 GB = 1000000000 B`. Unit lookup is
case-sensitive for these prefixes so `KB` and `KiB` never collide.

## Supported units

**length** (base m): m, km, cm, mm, mi, yd, ft, in, nmi
**mass** (base g): g, kg, mg, t (tonne), lb, oz, st (stone)
**temperature** (base K): C, F, K
**data-size** (base B): B, KB, MB, GB, TB (decimal); KiB, MiB, GiB, TiB (binary)
**time** (base s): s, min, hr, day, wk
**area** (base m2): m2, km2, ft2, acre, ha
**volume** (base L): L, mL, m3, gal, qt, pt, cup, floz
**speed** (base m/s): m/s, km/h, mph, kn (knots)

Each unit also accepts long-form aliases (e.g. `kilometer`, `pounds`,
`celsius`, `gibibyte`, `milesperhour`). Run `unit-convert --list` for the full
set.

## Tests

```sh
cargo test
```

Library unit tests cover known conversions within epsilon (1 km = 0.621371 mi,
100 C = 212 F, 0 C = 273.15 K, 1 GiB = 1073741824 B, 1 GB = 1000000000 B with
binary != decimal asserted, 60 mph ≈ 96.5606 km/h, 1 acre ≈ 4046.86 m2),
cross-category errors, and A→B→A round-trip identity. Integration tests
(`assert_cmd`) drive the real binary for argument parsing, `--precision`,
`--list`, and non-zero exit on bad input.

## License

MIT — see [LICENSE](LICENSE).
