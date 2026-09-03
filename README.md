# ccnv

[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)](https://www.rust-lang.org/)

`ccnv` is a lightweight command-line utility for converting amounts between
international currencies. This project is a Rust reimplementation of the
[original C++ ccnv utility](https://github.com/victorkryz/ccnv).

## How it works

The application obtains currencies and current exchange rates from the
[Free Currency Exchange Rates API](https://github.com/fawazahmed0/exchange-api).
It sends HTTPS requests to the service, decodes the JSON responses, and uses
the requested rate to calculate and display the converted amount.

## Command-line options

```text
-l, --list             List all available currencies
-f, --from <CURRENCY>  Currency to convert from (usd, eur, ...)
-t, --to <CURRENCY>    Currency to convert to (usd, eur, ...)
-a, --amount <AMOUNT>  Amount to convert [default: 1]
-v, --version          Print version
-h, --help             Print usage
```

Currency codes are passed in lowercase, for example `usd`, `eur`, or `uah`.
Both `--from` and `--to` are required for a conversion.

## Usage examples

```console
ccnv --list
ccnv --from eur --to usd
ccnv --from usd --amount 10 --to eur
ccnv --from usd --amount 25 --to uah
ccnv --from bgn --amount 200 --to uah
```

Listing currencies and converting amounts require an internet connection.

## Build

Install a current stable [Rust toolchain](https://www.rust-lang.org/tools/install),
then build the optimized executable:

```console
cargo build --release
```

The resulting executable is `target/release/ccnv` on Linux and macOS, or
`target\release\ccnv.exe` on Windows.

To build and run directly through Cargo:

```console
cargo run --release -- --from usd --amount 10 --to eur
```

The first `--` separates Cargo's options from the options passed to `ccnv`.

## Install locally

Install the executable from the project directory:

```console
cargo install --path . --locked
```

After Cargo's binary directory is present in `PATH`, run the application as
`ccnv`.

## Test

Run all unit tests:

```console
cargo test --locked
```

Run the same formatting and lint checks used by CI:

```console
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
```
