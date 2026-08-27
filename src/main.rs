mod cli;
mod error;

use std::process::ExitCode;

use ccnv::currency::Currency;
use ccnv::rate_service::{CurrencyRate, CurrencyRateService};
use cli::{Command, ParseOutcome};
use error::AppError;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match cli::parse() {
        Err(error) => {
            println!(
                "command line arguments parsing error: {}",
                error.to_string().trim()
            );
            ExitCode::FAILURE
        }
        Ok(ParseOutcome::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(ParseOutcome::InvalidUsage) => {
            print_usage();
            ExitCode::FAILURE
        }
        Ok(ParseOutcome::Command(command)) => match run(command) {
            Ok(()) => ExitCode::SUCCESS,
            Err(AppError::InvalidArgument(error)) => {
                eprintln!("Invalid argument: {error}");
                ExitCode::FAILURE
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
    }
}

fn run(command: Command) -> Result<(), AppError> {
    match command {
        Command::List => {
            let service = CurrencyRateService::new().map_err(|error| {
                AppError::Service(format!("failed to initialize HTTP client: {error}"))
            })?;

            for (currency, country) in service.currencies()? {
                println!("{currency} : {country}");
            }
        }
        Command::Convert { from, to, amount } => {
            let service = CurrencyRateService::new().map_err(|error| {
                AppError::Service(format!("failed to initialize HTTP client: {error}"))
            })?;
            let rate = service.rate(&from, &to)?;
            let source = Currency::new(amount, &rate.from);
            let target = Currency::convert_with_default_precision(&source, &rate.to, rate.rate);

            print_conversion(&rate, &source, &target);
        }
        Command::Version => println!("{APP_NAME} version {APP_VERSION}"),
    }

    Ok(())
}

fn print_conversion(rate: &CurrencyRate, from: &Currency, to: &Currency) {
    println!("[{}] [rate: {}] {} -> {} ", rate.date, rate.rate, from, to);
}

fn print_usage() {
    println!("\n{}", cli::help_text());
}
