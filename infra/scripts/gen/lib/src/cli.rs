//! Command line parsing shared by the generation tools.
//!
//! The Python originals lean on `argparse` plus a hand rolled precedence rule
//! (a bare leading integer is always the address count). This module reproduces
//! that behaviour so the Rust tools accept the same invocations.

use serde::Serialize;

use crate::json;
use crate::mnemonic::DEFAULT_MNEMONIC;

/// The mnemonic and address count accepted by the derivation tools.
#[derive(Debug, Clone)]
pub struct DerivationArgs {
    /// The BIP39 phrase to derive from.
    pub mnemonic: String,
    /// How many addresses to derive.
    pub count: usize,
}

/// Something the caller should print and exit for, rather than a hard failure.
#[derive(Debug)]
pub enum CliError {
    /// Help text; print to stdout and exit successfully.
    Help(String),
    /// A usage problem; print to stderr and exit with a non-zero status.
    Invalid(String),
}

/// The flag names accepted by the derivation tools.
const COUNT_FLAGS: [&str; 2] = ["-n", "--count"];

/// Returns the program name without its directory, mirroring `argparse`'s `prog`.
pub fn program_name() -> String {
    std::env::args()
        .next()
        .map(|arg| {
            std::path::Path::new(&arg)
                .file_name()
                .map_or_else(|| arg.clone(), |name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_default()
}

/// Returns the command line arguments, excluding the program name.
pub fn argv() -> Vec<String> {
    std::env::args().skip(1).collect()
}

/// Parses the mnemonic and count used by every `m-...` tool.
///
/// Supported forms, all matching the Python original:
///
/// * `tool`
/// * `tool 5`
/// * `tool "mnemonic words"`
/// * `tool "mnemonic words" 5`
/// * `tool 5 "mnemonic words"`
/// * `tool -n 5`
/// * `tool -n 5 "mnemonic words"`
pub fn parse_derivation_args(
    program: &str,
    description: &str,
    args: &[String],
) -> std::result::Result<DerivationArgs, CliError> {
    // A bare leading integer is the count; an optional second positional is the mnemonic.
    if let Some(first) = args.first().filter(|arg| !arg.starts_with('-')) {
        if let Some(count) = parse_positive_int(first) {
            let mnemonic = args
                .get(1)
                .filter(|arg| !arg.starts_with('-'))
                .cloned()
                .unwrap_or_else(|| DEFAULT_MNEMONIC.to_string());
            return Ok(DerivationArgs { mnemonic, count });
        }
    }

    // A mnemonic followed by a bare count.
    if let (Some(first), Some(second)) = (args.first(), args.get(1)) {
        if !first.starts_with('-') && !second.starts_with('-') {
            if let Some(count) = parse_positive_int(second) {
                return Ok(DerivationArgs {
                    mnemonic: first.clone(),
                    count,
                });
            }
        }
    }

    parse_flag_style(program, description, args)
}

/// Parses the arguments of the `all_address_types` tool.
///
/// The count may be given either positionally or with `-n`, never both.
pub fn parse_all_types_args(
    program: &str,
    description: &str,
    args: &[String],
) -> std::result::Result<DerivationArgs, CliError> {
    let Parsed {
        count: flag_count,
        positionals,
    } = split_flags(program, description, args)?;

    if positionals.len() > 2 {
        return Err(CliError::Invalid(
            "Use an optional COUNT and one quoted mnemonic phrase.".to_string(),
        ));
    }

    let (count, mnemonic) = match positionals.as_slice() {
        [] => (flag_count.unwrap_or(1), DEFAULT_MNEMONIC.to_string()),
        [single] if looks_like_integer(single) => {
            reject_repeated_count(flag_count)?;
            (parse_int(single)?, DEFAULT_MNEMONIC.to_string())
        }
        [single] => (flag_count.unwrap_or(1), single.clone()),
        [first, second] => {
            if !looks_like_integer(first) {
                return Err(CliError::Invalid(
                    "When supplying two positional arguments, the first must be COUNT.".to_string(),
                ));
            }
            reject_repeated_count(flag_count)?;
            (parse_int(first)?, second.clone())
        }
        _ => unreachable!("the length is checked above"),
    };

    if count < 1 {
        return Err(CliError::Invalid(
            "count must be a positive integer.".to_string(),
        ));
    }

    Ok(DerivationArgs { mnemonic, count })
}

/// Parses the `--strength` flag of the mnemonic generation tool.
///
/// # Errors
/// Returns [`CliError::Help`] for `-h`/`--help` and [`CliError::Invalid`] for
/// anything that is not one of the five BIP39 strengths.
pub fn parse_strength_args(args: &[String]) -> std::result::Result<usize, CliError> {
    const CHOICES: [usize; 5] = [128, 160, 192, 224, 256];

    let mut strength: Option<usize> = None;
    let mut index = 0;

    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "-h" | "--help" => return Err(CliError::Help(strength_usage())),
            "-s" | "--strength" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CliError::Invalid("argument -s/--strength: expected one argument".to_string())
                })?;
                strength = Some(parse_strength(value, &CHOICES)?);
            }
            other => {
                let value = other
                    .strip_prefix("--strength=")
                    .or_else(|| other.strip_prefix("-s").filter(|rest| !rest.is_empty()))
                    .ok_or_else(|| CliError::Invalid(format!("unrecognized arguments: {other}")))?;
                strength = Some(parse_strength(value, &CHOICES)?);
            }
        }
        index += 1;
    }

    Ok(strength.unwrap_or(128))
}

/// Joins every positional argument into a single passphrase.
///
/// Brain wallets read the passphrase as one argument, but joining keeps
/// unquoted invocations such as `brain_wallet correct horse` working.
pub fn join_passphrase(args: &[String]) -> String {
    args.iter()
        .filter(|arg| !arg.starts_with('-'))
        .cloned()
        .collect::<Vec<String>>()
        .join(" ")
}

/// Unwraps a parse result, handling help and usage errors.
///
/// # Panics
/// Does not return when the outcome is [`CliError::Help`] or
/// [`CliError::Invalid`]; it prints the message and exits instead.
pub fn resolve<T>(program: &str, outcome: std::result::Result<T, CliError>) -> T {
    match outcome {
        Ok(value) => value,
        Err(CliError::Help(text)) => {
            println!("{text}");
            std::process::exit(0);
        }
        Err(CliError::Invalid(message)) => {
            eprintln!("{program}: error: {message}");
            eprintln!("Try '{program} --help' for more information.");
            std::process::exit(2);
        }
    }
}

/// Prints a serialisable result as indented JSON, or reports the failure.
///
/// # Panics
/// Does not return when `result` is an error; it prints `Error: ...` to stderr
/// and exits with status 1.
pub fn print_json<T: Serialize>(result: crate::error::Result<T>, indent: u8) {
    match result {
        Ok(value) => println!("{}", json::to_string_pretty(&value, indent)),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

/// Parses the argparse style form: flags plus an optional positional mnemonic.
fn parse_flag_style(
    program: &str,
    description: &str,
    args: &[String],
) -> std::result::Result<DerivationArgs, CliError> {
    let Parsed { count, positionals } = split_flags(program, description, args)?;

    if positionals.len() > 1 {
        return Err(CliError::Invalid(format!(
            "unrecognized arguments: {}",
            positionals[1..].join(" ")
        )));
    }

    Ok(DerivationArgs {
        mnemonic: positionals
            .into_iter()
            .next()
            .unwrap_or_else(|| DEFAULT_MNEMONIC.to_string()),
        count: count.unwrap_or(1),
    })
}

/// The pieces produced by [`split_flags`].
struct Parsed {
    count: Option<usize>,
    positionals: Vec<String>,
}

/// Splits `args` into `-n/--count` values and positionals.
fn split_flags(
    program: &str,
    description: &str,
    args: &[String],
) -> std::result::Result<Parsed, CliError> {
    let label = format!("argument {} COUNT", COUNT_FLAGS.join("/"));
    let mut count: Option<usize> = None;
    let mut positionals: Vec<String> = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let arg = args[index].as_str();

        if arg == "-h" || arg == "--help" {
            return Err(CliError::Help(usage(program, description)));
        }

        if COUNT_FLAGS.contains(&arg) {
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| CliError::Invalid(format!("{label}: expected one argument")))?;
            count = Some(parse_count(value, &label)?);
        } else if let Some(value) = inline_count_value(arg) {
            count = Some(parse_count(value, &label)?);
        } else if arg.starts_with('-') && arg.len() > 1 {
            return Err(CliError::Invalid(format!("unrecognized arguments: {arg}")));
        } else {
            positionals.push(arg.to_string());
        }

        index += 1;
    }

    Ok(Parsed { count, positionals })
}

/// Extracts the value of `--count=5` or `-n5` style flags.
fn inline_count_value(arg: &str) -> Option<&str> {
    arg.strip_prefix("--count=")
        .or_else(|| arg.strip_prefix("-n").filter(|value| !value.is_empty()))
}

fn parse_count(value: &str, label: &str) -> std::result::Result<usize, CliError> {
    match value.parse::<i64>() {
        Ok(parsed) if parsed >= 1 => Ok(parsed as usize),
        Ok(_) => Err(CliError::Invalid(format!(
            "{label}: number of addresses must be at least 1"
        ))),
        Err(_) => Err(CliError::Invalid(format!(
            "{label}: invalid positive_count value: '{value}'"
        ))),
    }
}

fn parse_strength(value: &str, choices: &[usize]) -> std::result::Result<usize, CliError> {
    let list = choices
        .iter()
        .map(usize::to_string)
        .collect::<Vec<String>>()
        .join(", ");

    value
        .parse::<usize>()
        .ok()
        .filter(|parsed| choices.contains(parsed))
        .ok_or_else(|| {
            CliError::Invalid(format!(
                "argument -s/--strength: invalid choice: {value} (choose from {list})"
            ))
        })
}

fn reject_repeated_count(flag_count: Option<usize>) -> std::result::Result<(), CliError> {
    if flag_count.is_some() {
        return Err(CliError::Invalid(
            "Specify the address count only once: as COUNT or with --count.".to_string(),
        ));
    }
    Ok(())
}

fn parse_positive_int(value: &str) -> Option<usize> {
    value
        .parse::<i64>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed as usize)
}

fn parse_int(value: &str) -> std::result::Result<usize, CliError> {
    value
        .parse::<i64>()
        .map(|parsed| parsed.max(0) as usize)
        .map_err(|_| CliError::Invalid(format!("invalid count value: '{value}'")))
}

fn looks_like_integer(value: &str) -> bool {
    let digits = value.trim_start_matches(['+', '-']);
    !digits.is_empty() && digits.chars().all(|char| char.is_ascii_digit())
}

fn usage(program: &str, description: &str) -> String {
    format!(
        "usage: {program} [-h] [-n COUNT] [--count COUNT] [mnemonic]\n\n\
         {description}\n\n\
         positional arguments:\n  \
         mnemonic            quoted 12- or 24-word BIP39 mnemonic; defaults to the\n                       \
         public test mnemonic\n\n\
         options:\n  \
         -h, --help           show this help message and exit\n  \
         -n COUNT, --count COUNT\n                       \
         number of addresses to derive (default: 1)\n"
    )
}

fn strength_usage() -> String {
    "usage: generate_mnemonic [-h] [-s {128,160,192,224,256}]\n\n\
     Generate a BIP39 mnemonic, seed, and BIP32 root key.\n\n\
     options:\n  \
     -h, --help            show this help message and exit\n  \
     -s {128,160,192,224,256}, --strength {128,160,192,224,256}\n                       \
     Strength of the mnemonic in bits (default: 128 for 12 words, use 256 for 24 words)\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().copied().map(String::from).collect()
    }

    #[test]
    fn defaults_to_one_address_from_the_test_mnemonic() {
        let parsed = parse_derivation_args("tool", "desc", &[]).expect("parses");

        assert_eq!(parsed.mnemonic, DEFAULT_MNEMONIC);
        assert_eq!(parsed.count, 1);
    }

    #[test]
    fn accepts_a_bare_leading_count() {
        let parsed = parse_derivation_args("tool", "desc", &args(&["5"])).expect("parses");

        assert_eq!(parsed.mnemonic, DEFAULT_MNEMONIC);
        assert_eq!(parsed.count, 5);
    }

    #[test]
    fn accepts_a_bare_leading_count_then_a_mnemonic() {
        let parsed = parse_derivation_args("tool", "desc", &args(&["3", "word"])).expect("parses");

        assert_eq!(parsed.mnemonic, "word");
        assert_eq!(parsed.count, 3);
    }

    #[test]
    fn accepts_a_mnemonic_then_a_bare_count() {
        let parsed = parse_derivation_args("tool", "desc", &args(&["word", "3"])).expect("parses");

        assert_eq!(parsed.mnemonic, "word");
        assert_eq!(parsed.count, 3);
    }

    #[test]
    fn accepts_long_count_flags() {
        let parsed = parse_derivation_args("tool", "desc", &args(&["--count", "4", "word"]))
            .expect("parses");

        assert_eq!(parsed.mnemonic, "word");
        assert_eq!(parsed.count, 4);

        assert_eq!(
            parse_derivation_args("tool", "desc", &args(&["-n7"]))
                .expect("parses")
                .count,
            7
        );
    }

    #[test]
    fn rejects_a_count_below_one() {
        assert!(matches!(
            parse_derivation_args("tool", "desc", &args(&["-n", "0"])),
            Err(CliError::Invalid(_))
        ));
    }

    #[test]
    fn help_is_returned_for_the_help_flag() {
        assert!(matches!(
            parse_derivation_args("tool", "desc", &args(&["--help"])),
            Err(CliError::Help(_))
        ));
    }

    #[test]
    fn all_types_rejects_a_duplicated_count() {
        assert!(matches!(
            parse_all_types_args("tool", "desc", &args(&["-n", "2", "3"])),
            Err(CliError::Invalid(_))
        ));
    }

    #[test]
    fn all_types_accepts_a_positional_count() {
        let parsed = parse_all_types_args("tool", "desc", &args(&["3"])).expect("parses");

        assert_eq!(parsed.count, 3);
        assert_eq!(parsed.mnemonic, DEFAULT_MNEMONIC);
    }

    #[test]
    fn all_types_accepts_count_then_mnemonic() {
        let parsed = parse_all_types_args("tool", "desc", &args(&["3", "word"])).expect("parses");

        assert_eq!(parsed.count, 3);
        assert_eq!(parsed.mnemonic, "word");
    }

    #[test]
    fn all_types_accepts_a_mnemonic_on_its_own() {
        let parsed = parse_all_types_args("tool", "desc", &args(&["word"])).expect("parses");

        assert_eq!(parsed.count, 1);
        assert_eq!(parsed.mnemonic, "word");
    }

    #[test]
    fn strength_defaults_to_128_and_rejects_other_values() {
        assert_eq!(parse_strength_args(&[]).expect("parses"), 128);
        assert_eq!(
            parse_strength_args(&args(&["-s", "256"])).expect("parses"),
            256
        );
        assert!(parse_strength_args(&args(&["-s", "129"])).is_err());
    }
}
