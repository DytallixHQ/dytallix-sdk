//! Shared terminal output formatting for the Dytallix CLI.

use std::time::Duration;

use colored::Colorize;
use dytallix_sdk::FeeEstimate;

/// Prints a success line, optionally including an elapsed duration.
pub fn success(message: &str, elapsed: Option<Duration>) {
    println!("{}", format_success(message, elapsed));
}

/// Prints an error line in red.
pub fn error(message: &str) {
    println!("{}", format_error(message).red());
}

/// Prints a warning line in yellow.
pub fn warning(message: &str) {
    println!("{}", format_warning(message).yellow());
}

/// Prints the standard Dytallix divider line.
pub fn divider() {
    println!("{}", divider_string());
}

/// Prints a section header wrapped in divider lines.
pub fn section(title: &str) {
    divider();
    println!("  {title}");
    divider();
}

/// Prints DGT and DRT balances on separate labeled lines.
pub fn balance(dgt: u128, drt: u128) {
    println!("{}", format_balance(dgt, drt));
}

/// Prints a DRT fee estimate with separate compute and bandwidth gas lines.
pub fn fee_breakdown(estimate: &FeeEstimate) {
    println!("{}", format_fee_breakdown(estimate));
}

/// Prints a successful transaction hash line.
pub fn tx_hash(hash: &str) {
    println!("Transaction: {hash}");
}

/// Prints the standard testnet keystore warning in yellow.
pub fn testnet_warning() {
    println!("{}", "⚠  Testnet only. Keystore is unencrypted.".yellow());
    println!("{}", "   Do not use this keypair on mainnet.".yellow());
}

fn divider_string() -> &'static str {
    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

fn format_success(message: &str, elapsed: Option<Duration>) -> String {
    match elapsed {
        Some(duration) => format!("✓ {message}    [{:.1}s]", duration.as_secs_f64()),
        None => format!("✓ {message}"),
    }
}

fn format_error(message: &str) -> String {
    format!("✗ {message}")
}

fn format_warning(message: &str) -> String {
    format!("⚠  {message}")
}

fn format_balance(dgt: u128, drt: u128) -> String {
    format!("  DGT:  {dgt} DGT\n  DRT:  {drt} DRT")
}

fn format_fee_breakdown(estimate: &FeeEstimate) -> String {
    format!(
		"  Fee estimate:\n    Compute (C-Gas):   {} units  {} DGT\n    Bandwidth (B-Gas): {} units  {} DGT\n    Total:             {} DGT",
		estimate.c_gas,
		format_micro_token(estimate.c_gas_cost_drt),
		estimate.b_gas,
		format_micro_token(estimate.b_gas_cost_drt),
		format_micro_token(estimate.total_cost_drt)
	)
}

fn format_micro_token(value: u128) -> String {
    let whole = value / 1_000_000;
    let fractional = value % 1_000_000;
    if fractional == 0 {
        whole.to_string()
    } else {
        format!("{whole}.{fractional:06}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use dytallix_sdk::FeeEstimate;

    use super::{
        divider_string, format_balance, format_error, format_fee_breakdown, format_success,
        format_warning,
    };

    #[test]
    fn formatters_match_expected_strings() {
        assert_eq!(format_success("ready", None), "✓ ready");
        assert_eq!(format_error("failed"), "✗ failed");
        assert_eq!(format_warning("careful"), "⚠  careful");
        assert_eq!(divider_string(), "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        assert!(format_success("ready", Some(Duration::from_millis(150))).contains("[0.1s]"));
    }

    #[test]
    fn balance_output_shows_both_tokens() {
        let rendered = format_balance(1_000, 10_000);
        assert!(rendered.contains("DGT"));
        assert!(rendered.contains("DRT"));
        assert!(rendered.contains("1000"));
        assert!(rendered.contains("10000"));
    }

    #[test]
    fn fee_breakdown_output_shows_both_gas_dimensions() {
        let estimate = FeeEstimate {
            c_gas: 21_000,
            c_gas_cost_drt: 10_000,
            b_gas: 512,
            b_gas_cost_drt: 3_000,
            total_cost_drt: 13_000,
        };
        let rendered = format_fee_breakdown(&estimate);
        assert!(rendered.contains("C-Gas"));
        assert!(rendered.contains("B-Gas"));
        assert!(rendered.contains("Total"));
    }
}
