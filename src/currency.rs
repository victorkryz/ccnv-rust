//! Currency values represented as integer minor units.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

const DEFAULT_CODE: &str = "usd";
const DEFAULT_PRECISION: u32 = 2;

#[derive(Clone, Debug)]
pub struct Currency {
    minor_units: i64,
    code: String,
    precision: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyMismatch;

impl fmt::Display for CurrencyMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Mismatched currency or precision")
    }
}

impl std::error::Error for CurrencyMismatch {}

impl Currency {
    pub fn new(amount: f64, code: impl Into<String>) -> Self {
        Self::with_precision(amount, code, DEFAULT_PRECISION)
    }

    pub fn with_precision(amount: f64, code: impl Into<String>, precision: u32) -> Self {
        Self {
            minor_units: (amount * scale(precision)).round() as i64,
            code: code.into(),
            precision,
        }
    }

    pub fn from_minor_units(minor_units: i64, code: impl Into<String>, precision: u32) -> Self {
        Self {
            minor_units,
            code: code.into(),
            precision,
        }
    }

    pub fn amount(&self) -> f64 {
        self.minor_units as f64 / scale(self.precision)
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn precision(&self) -> u32 {
        self.precision
    }

    pub fn minor_units(&self) -> i64 {
        self.minor_units
    }

    pub fn convert(from: &Self, to_code: impl Into<String>, rate: f64, to_precision: u32) -> Self {
        let to_code = to_code.into();
        if from.code == to_code {
            return from.clone();
        }

        Self::with_precision(from.amount() * rate, to_code, to_precision)
    }

    pub fn convert_with_default_precision(
        from: &Self,
        to_code: impl Into<String>,
        rate: f64,
    ) -> Self {
        Self::convert(from, to_code, rate, DEFAULT_PRECISION)
    }

    pub fn checked_add(&self, other: &Self) -> Result<Self, CurrencyMismatch> {
        self.ensure_compatible(other)?;
        Ok(Self::from_minor_units(
            self.minor_units + other.minor_units,
            self.code.clone(),
            self.precision,
        ))
    }

    pub fn checked_sub(&self, other: &Self) -> Result<Self, CurrencyMismatch> {
        self.ensure_compatible(other)?;
        Ok(Self::from_minor_units(
            self.minor_units - other.minor_units,
            self.code.clone(),
            self.precision,
        ))
    }

    pub fn checked_cmp(&self, other: &Self) -> Result<Ordering, CurrencyMismatch> {
        self.ensure_compatible(other)?;
        Ok(self.minor_units.cmp(&other.minor_units))
    }

    fn ensure_compatible(&self, other: &Self) -> Result<(), CurrencyMismatch> {
        if self.code == other.code && self.precision == other.precision {
            Ok(())
        } else {
            Err(CurrencyMismatch)
        }
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self::new(1.0, DEFAULT_CODE)
    }
}

impl PartialEq for Currency {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.minor_units == other.minor_units
    }
}

impl Eq for Currency {}

impl PartialOrd for Currency {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.checked_cmp(other)
                .expect("cannot compare incompatible currencies"),
        )
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:.*} {}",
            self.precision as usize,
            self.amount(),
            self.code
        )
    }
}

impl Add<&Currency> for &Currency {
    type Output = Currency;

    fn add(self, other: &Currency) -> Self::Output {
        self.checked_add(other)
            .expect("cannot add incompatible currencies")
    }
}

impl Sub<&Currency> for &Currency {
    type Output = Currency;

    fn sub(self, other: &Currency) -> Self::Output {
        self.checked_sub(other)
            .expect("cannot subtract incompatible currencies")
    }
}

impl AddAssign<&Currency> for Currency {
    fn add_assign(&mut self, other: &Currency) {
        self.ensure_compatible(other)
            .expect("cannot add incompatible currencies");
        self.minor_units += other.minor_units;
    }
}

impl SubAssign<&Currency> for Currency {
    fn sub_assign(&mut self, other: &Currency) {
        self.ensure_compatible(other)
            .expect("cannot subtract incompatible currencies");
        self.minor_units -= other.minor_units;
    }
}

fn scale(precision: u32) -> f64 {
    10_f64.powf(f64::from(precision))
}

#[cfg(test)]
mod tests {
    use super::{Currency, CurrencyMismatch};

    struct CurrencyDescriptor {
        minor_units: i64,
        major_units: f64,
        precision: u32,
        code: &'static str,
    }

    #[test]
    fn initializes_from_minor_units() {
        let cases = [
            CurrencyDescriptor {
                minor_units: 2_500,
                major_units: 25.0,
                precision: 2,
                code: "usd",
            },
            CurrencyDescriptor {
                minor_units: 4_500,
                major_units: 45.0,
                precision: 2,
                code: "uah",
            },
            CurrencyDescriptor {
                minor_units: 37,
                major_units: 0.37,
                precision: 2,
                code: "eur",
            },
        ];

        for case in cases {
            let currency = Currency::from_minor_units(case.minor_units, case.code, case.precision);
            assert_eq!(currency.minor_units(), case.minor_units);
            assert_eq!(currency.amount(), case.major_units);
            assert_eq!(currency.code(), case.code);
            assert_eq!(currency.precision(), case.precision);
        }
    }

    #[test]
    fn converts_using_static_rates() {
        let cases = [
            (2_500, "usd", 2, 41.71458864, "uah", 2, 104_286, 1_042.86),
            (1_000, "usd", 2, 0.86389992, "eur", 2, 864, 8.64),
        ];

        for (
            units,
            code,
            precision,
            rate,
            target_code,
            target_precision,
            expected_units,
            expected_amount,
        ) in cases
        {
            let source = Currency::from_minor_units(units, code, precision);
            let target = Currency::convert(&source, target_code, rate, target_precision);
            assert_eq!(target.minor_units(), expected_units);
            assert_eq!(target.amount(), expected_amount);
            assert_eq!(target.code(), target_code);
            assert_eq!(target.precision(), target_precision);
        }
    }

    #[test]
    fn rounds_half_away_from_zero() {
        assert_eq!(Currency::new(1.125, "usd").minor_units(), 113);
        assert_eq!(Currency::new(-1.125, "usd").minor_units(), -113);
    }

    #[test]
    fn same_currency_conversion_returns_source_unchanged() {
        let source = Currency::from_minor_units(1_234, "usd", 3);
        let converted = Currency::convert(&source, "usd", 99.0, 2);
        assert_eq!(converted.minor_units(), 1_234);
        assert_eq!(converted.precision(), 3);
    }

    #[test]
    fn supports_arithmetic_assignment_comparison_and_formatting() {
        let first = Currency::new(10.25, "usd");
        let second = Currency::new(2.50, "usd");

        assert_eq!((&first + &second).minor_units(), 1_275);
        assert_eq!((&first - &second).minor_units(), 775);
        assert_eq!(first.to_string(), "10.25 usd");
        assert!(first > second);

        let mut total = first.clone();
        total += &second;
        assert_eq!(total.minor_units(), 1_275);
        total -= &second;
        assert_eq!(total, first);
    }

    #[test]
    fn checked_operations_reject_incompatible_values() {
        let dollars = Currency::new(1.0, "usd");
        let euros = Currency::new(1.0, "eur");

        assert_eq!(dollars.checked_add(&euros), Err(CurrencyMismatch));
        assert_eq!(dollars.checked_sub(&euros), Err(CurrencyMismatch));
        assert_eq!(dollars.checked_cmp(&euros), Err(CurrencyMismatch));
    }

    #[test]
    fn defaults_to_one_us_dollar() {
        let currency = Currency::default();
        assert_eq!(currency.minor_units(), 100);
        assert_eq!(currency.code(), "usd");
        assert_eq!(currency.precision(), 2);
    }
}
