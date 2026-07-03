//! Integration tests: drive the real `unit-convert` binary and check stdout,
//! stderr, and exit codes — the actual arg-parsing trigger path that the unit
//! tests over the library can't exercise.

use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("unit-convert").unwrap()
}

#[test]
fn length_basic() {
    bin()
        .args(["10", "km", "to", "mi"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("6.21371"))
        .stdout(predicate::str::contains("mi"));
}

#[test]
fn temperature_boiling() {
    bin()
        .args(["100", "C", "to", "F"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("212"))
        .stdout(predicate::str::contains("F"));
}

#[test]
fn data_binary_to_decimal() {
    // 1 GiB to MB = 1073.741824 MB
    bin()
        .args(["1", "GiB", "to", "MB"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("1073.74"))
        .stdout(predicate::str::contains("MB"));
}

#[test]
fn data_binary_distinct_from_decimal() {
    // 1 GiB -> B is 1073741824, NOT 1000000000
    bin()
        .args(["1", "GiB", "to", "B"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1073741824"));
    bin()
        .args(["1", "GB", "to", "B"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1000000000"));
}

#[test]
fn quoted_phrase_args() {
    // mirrors: unit-convert "60 mph" to "km/h"
    bin()
        .args(["60 mph", "to", "km/h"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("96.5606"))
        .stdout(predicate::str::contains("km/h"));
}

#[test]
fn glued_value_unit() {
    bin()
        .args(["10km", "to", "mi"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("6.21371"));
}

#[test]
fn precision_flag() {
    bin()
        .args(["--precision", "3", "10", "km", "to", "mi"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("6.21").and(predicate::str::contains("6.2137").not()));
}

#[test]
fn cross_category_errors_nonzero_exit() {
    bin()
        .args(["10", "km", "to", "kg"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("different categories"))
        .stderr(predicate::str::contains("length"))
        .stderr(predicate::str::contains("mass"));
}

#[test]
fn unknown_unit_errors() {
    bin()
        .args(["10", "furlong", "to", "m"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown unit"));
}

#[test]
fn missing_to_errors() {
    bin()
        .args(["10", "km", "mi"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing 'to'"));
}

#[test]
fn no_args_errors() {
    bin()
        .assert()
        .failure()
        .stderr(predicate::str::contains("no expression"));
}

#[test]
fn list_all() {
    bin()
        .args(["--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("length"))
        .stdout(predicate::str::contains("temperature"))
        .stdout(predicate::str::contains("data-size"))
        .stdout(predicate::str::contains("speed"))
        .stdout(predicate::str::contains("KiB"))
        .stdout(predicate::str::contains("mph"));
}

#[test]
fn list_one_category() {
    bin()
        .args(["--list", "length"])
        .assert()
        .success()
        .stdout(predicate::str::contains("length"))
        .stdout(predicate::str::contains("nmi"))
        .stdout(predicate::str::contains("temperature").not());
}

#[test]
fn list_unknown_category_errors() {
    bin()
        .args(["--list", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown category"));
}

#[test]
fn list_subcommand_all() {
    // `list` subcommand mirrors `--list`.
    bin()
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("length"))
        .stdout(predicate::str::contains("pressure"))
        .stdout(predicate::str::contains("psi"))
        .stdout(predicate::str::contains("KiB"));
}

#[test]
fn list_subcommand_one_category() {
    bin()
        .args(["list", "pressure"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pressure"))
        .stdout(predicate::str::contains("atm"))
        .stdout(predicate::str::contains("length").not());
}

#[test]
fn list_subcommand_unknown_category_errors() {
    bin()
        .args(["list", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown category"));
}

#[test]
fn pressure_atm_to_kpa() {
    bin()
        .args(["1", "atm", "to", "kPa"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("101.325"))
        .stdout(predicate::str::contains("kPa"));
}

#[test]
fn pressure_bar_to_psi() {
    bin()
        .args(["1", "bar", "to", "psi"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("14.5038"));
}

#[test]
fn negative_temperature() {
    // 0 K to C = -273.15
    bin()
        .args(["0", "K", "to", "C"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("-273.15"));
}

#[test]
fn speed_roundtrip_via_cli() {
    // 60 mph -> km/h, separate tokens
    bin()
        .args(["60", "mph", "to", "km/h"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("96.5606"));
}
