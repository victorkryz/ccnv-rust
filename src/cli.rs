//! Command-line argument definitions and command selection.

use clap::{CommandFactory, Parser};

const USAGE_EXAMPLES: &str = "Command line samples:
    ccnv -l 
    ccnv -f eur -t usd
    ccnv -f usd -a 10 -t eur 
    ccnv -f usd -a 25 -t uah";

#[derive(Debug, Parser)]
#[command(
    name = "ccnv",
    about = "Currency converter",
    disable_help_flag = true,
    disable_version_flag = true,
    after_help = USAGE_EXAMPLES
)]
struct Cli {
    /// List all available currencies.
    #[arg(short, long)]
    list: bool,

    /// Currency to convert from (usd, eur, ...).
    #[arg(short, long, value_name = "CURRENCY")]
    from: Option<String>,

    /// Currency to convert to (usd, eur, ...).
    #[arg(short, long, value_name = "CURRENCY")]
    to: Option<String>,

    /// Amount to convert (10, 50, 100, ...).
    #[arg(short, long, default_value_t = 1.0, value_name = "AMOUNT")]
    amount: f64,

    /// Print version.
    #[arg(short, long)]
    version: bool,

    /// Print usage.
    #[arg(short, long)]
    help: bool,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    List,
    Convert {
        from: String,
        to: String,
        amount: f64,
    },
    Version,
}

#[derive(Debug, PartialEq)]
pub enum ParseOutcome {
    Command(Command),
    Help,
    InvalidUsage,
}

pub fn parse() -> Result<ParseOutcome, clap::Error> {
    parse_from(std::env::args_os())
}

fn parse_from<I, T>(arguments: I) -> Result<ParseOutcome, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(arguments)?;

    if cli.help {
        return Ok(ParseOutcome::Help);
    }

    if cli.version {
        return Ok(ParseOutcome::Command(Command::Version));
    }

    if cli.list {
        return Ok(ParseOutcome::Command(Command::List));
    }

    match (cli.from, cli.to) {
        (Some(from), Some(to)) => Ok(ParseOutcome::Command(Command::Convert {
            from,
            to,
            amount: cli.amount,
        })),
        _ => Ok(ParseOutcome::InvalidUsage),
    }
}

pub fn help_text() -> String {
    Cli::command().render_help().to_string()
}

#[cfg(test)]
mod tests {
    use super::{Command, ParseOutcome, parse_from};

    #[test]
    fn defaults_to_invalid_usage() {
        assert_eq!(parse_from(["ccnv"]).unwrap(), ParseOutcome::InvalidUsage);
    }

    #[test]
    fn selects_conversion_and_default_amount() {
        assert_eq!(
            parse_from(["ccnv", "-f", "usd", "-t", "eur"]).unwrap(),
            ParseOutcome::Command(Command::Convert {
                from: "usd".into(),
                to: "eur".into(),
                amount: 1.0,
            })
        );
    }

    #[test]
    fn honors_reference_option_precedence() {
        assert_eq!(
            parse_from(["ccnv", "--help", "--version", "--list"]).unwrap(),
            ParseOutcome::Help
        );
        assert_eq!(
            parse_from(["ccnv", "--version", "--list"]).unwrap(),
            ParseOutcome::Command(Command::Version)
        );
        assert_eq!(
            parse_from(["ccnv", "--list", "--from", "usd"]).unwrap(),
            ParseOutcome::Command(Command::List)
        );
    }

    #[test]
    fn rejects_invalid_amount() {
        assert!(parse_from(["ccnv", "--amount", "many"]).is_err());
    }
}
