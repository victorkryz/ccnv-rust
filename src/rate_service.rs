//! Currency-list and exchange-rate operations.

use std::collections::BTreeMap;

use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::http::{HttpError, HttpTransport, ReqwestTransport};

const CURRENCIES_URL: &str =
    "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies.json";
const RATES_BASE_URL: &str =
    "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/";

pub type CurrencyList = BTreeMap<String, String>;

#[derive(Clone, Debug, PartialEq)]
pub struct CurrencyRate {
    pub from: String,
    pub to: String,
    pub rate: f64,
    pub date: String,
}

#[derive(Debug, Error)]
pub enum RateServiceError {
    #[error("{0}")]
    Transport(#[from] HttpError),

    #[error("HTTP request failed with status {0}")]
    HttpStatus(StatusCode),

    #[error("invalid currency service response: {0}")]
    InvalidResponse(#[from] serde_json::Error),

    #[error("\"{0}\" currency not found!")]
    CurrencyNotFound(String),
}

#[derive(Debug)]
pub struct CurrencyRateService<T> {
    transport: T,
    currencies_url: String,
    rates_base_url: String,
}

impl CurrencyRateService<ReqwestTransport> {
    pub fn new() -> Result<Self, HttpError> {
        Ok(Self::with_transport(ReqwestTransport::new()?))
    }
}

impl<T: HttpTransport> CurrencyRateService<T> {
    pub fn with_transport(transport: T) -> Self {
        Self::with_endpoints(transport, CURRENCIES_URL, RATES_BASE_URL)
    }

    pub fn with_endpoints(
        transport: T,
        currencies_url: impl Into<String>,
        rates_base_url: impl Into<String>,
    ) -> Self {
        Self {
            transport,
            currencies_url: currencies_url.into(),
            rates_base_url: rates_base_url.into(),
        }
    }

    pub fn currencies(&self) -> Result<CurrencyList, RateServiceError> {
        let response = self.transport.get(&self.currencies_url)?;

        if response.status != StatusCode::OK {
            return Err(RateServiceError::HttpStatus(response.status));
        }

        Ok(serde_json::from_str(&response.body)?)
    }

    pub fn rate(&self, from: &str, to: &str) -> Result<CurrencyRate, RateServiceError> {
        let response = self.transport.get(&self.rate_url(from))?;

        if response.status == StatusCode::NOT_FOUND || response.body.is_empty() {
            return Err(RateServiceError::CurrencyNotFound(from.to_owned()));
        }
        if response.status != StatusCode::OK {
            return Err(RateServiceError::HttpStatus(response.status));
        }

        let response: RateResponse = serde_json::from_str(&response.body)?;
        let from_rates = response
            .rates
            .get(from)
            .ok_or_else(|| RateServiceError::CurrencyNotFound(from.to_owned()))?;
        let rate = from_rates
            .get(to)
            .copied()
            .ok_or_else(|| RateServiceError::CurrencyNotFound(to.to_owned()))?;

        Ok(CurrencyRate {
            from: from.to_owned(),
            to: to.to_owned(),
            rate,
            date: response.date,
        })
    }

    fn rate_url(&self, currency: &str) -> String {
        format!("{}{currency}.json", self.rates_base_url)
    }
}

#[derive(Debug, Deserialize)]
struct RateResponse {
    #[serde(default)]
    date: String,

    #[serde(flatten)]
    rates: BTreeMap<String, BTreeMap<String, f64>>,
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::{CurrencyRateService, RateServiceError};
    use crate::http::{HttpError, HttpResponse, HttpTransport};
    use reqwest::StatusCode;

    const LIST_URL: &str = "https://example.test/currencies.json";
    const RATE_BASE_URL: &str = "https://example.test/currencies/";

    #[derive(Debug)]
    struct FakeTransport {
        response: RefCell<Option<Result<HttpResponse, HttpError>>>,
        requested_urls: RefCell<Vec<String>>,
    }

    impl FakeTransport {
        fn responding_with(status: StatusCode, body: &str) -> Self {
            Self {
                response: RefCell::new(Some(Ok(HttpResponse {
                    status,
                    body: body.to_owned(),
                }))),
                requested_urls: RefCell::new(Vec::new()),
            }
        }

        fn failing_with(message: &str) -> Self {
            Self {
                response: RefCell::new(Some(Err(HttpError::new(message)))),
                requested_urls: RefCell::new(Vec::new()),
            }
        }
    }

    impl HttpTransport for FakeTransport {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.requested_urls.borrow_mut().push(url.to_owned());
            self.response
                .borrow_mut()
                .take()
                .expect("fake transport received an unexpected request")
        }
    }

    fn service(transport: FakeTransport) -> CurrencyRateService<FakeTransport> {
        CurrencyRateService::with_endpoints(transport, LIST_URL, RATE_BASE_URL)
    }

    #[test]
    fn returns_an_ordered_currency_list() {
        let service = service(FakeTransport::responding_with(
            StatusCode::OK,
            r#"{"usd":"United States Dollar","eur":"Euro","uah":"Ukrainian Hryvnia"}"#,
        ));

        let currencies = service.currencies().unwrap();

        assert_eq!(
            currencies.keys().map(String::as_str).collect::<Vec<_>>(),
            ["eur", "uah", "usd"]
        );
        assert_eq!(
            service.transport.requested_urls.borrow().as_slice(),
            [LIST_URL]
        );
    }

    #[test]
    fn returns_a_rate_and_composes_the_source_url() {
        let service = service(FakeTransport::responding_with(
            StatusCode::OK,
            r#"{"date":"2026-08-25","usd":{"eur":0.86389992}}"#,
        ));

        let rate = service.rate("usd", "eur").unwrap();

        assert_eq!(rate.from, "usd");
        assert_eq!(rate.to, "eur");
        assert_eq!(rate.rate, 0.86389992);
        assert_eq!(rate.date, "2026-08-25");
        assert_eq!(
            service.transport.requested_urls.borrow().as_slice(),
            ["https://example.test/currencies/usd.json"]
        );
    }

    #[test]
    fn defaults_a_missing_date_to_an_empty_string() {
        let service = service(FakeTransport::responding_with(
            StatusCode::OK,
            r#"{"usd":{"eur":0.86}}"#,
        ));

        assert_eq!(service.rate("usd", "eur").unwrap().date, "");
    }

    #[test]
    fn maps_not_found_and_empty_responses_to_the_source_currency() {
        for (status, body) in [(StatusCode::NOT_FOUND, "missing"), (StatusCode::OK, "")] {
            let service = service(FakeTransport::responding_with(status, body));

            assert!(matches!(
                service.rate("usd", "eur"),
                Err(RateServiceError::CurrencyNotFound(code)) if code == "usd"
            ));
        }
    }

    #[test]
    fn reports_a_missing_source_or_target_currency() {
        let missing_source = service(FakeTransport::responding_with(
            StatusCode::OK,
            r#"{"date":"2026-08-25","eur":{"usd":1.1}}"#,
        ));
        assert!(matches!(
            missing_source.rate("usd", "eur"),
            Err(RateServiceError::CurrencyNotFound(code)) if code == "usd"
        ));

        let missing_target = service(FakeTransport::responding_with(
            StatusCode::OK,
            r#"{"date":"2026-08-25","usd":{"uah":41.7}}"#,
        ));
        assert!(matches!(
            missing_target.rate("usd", "eur"),
            Err(RateServiceError::CurrencyNotFound(code)) if code == "eur"
        ));
    }

    #[test]
    fn reports_malformed_json_and_wrong_value_types() {
        for body in [
            "{not-json}",
            r#"{"date":"2026-08-25","usd":{"eur":"unknown"}}"#,
        ] {
            let service = service(FakeTransport::responding_with(StatusCode::OK, body));
            assert!(matches!(
                service.rate("usd", "eur"),
                Err(RateServiceError::InvalidResponse(_))
            ));
        }
    }

    #[test]
    fn reports_non_success_statuses() {
        let service = service(FakeTransport::responding_with(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failure",
        ));

        assert!(matches!(
            service.currencies(),
            Err(RateServiceError::HttpStatus(
                StatusCode::INTERNAL_SERVER_ERROR
            ))
        ));
    }

    #[test]
    fn reports_transport_failures() {
        let service = service(FakeTransport::failing_with("network unavailable"));

        assert!(matches!(
            service.rate("usd", "eur"),
            Err(RateServiceError::Transport(error))
                if error == HttpError::new("network unavailable")
        ));
    }
}
