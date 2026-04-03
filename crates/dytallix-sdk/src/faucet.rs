//! HTTP client for Dytallix faucet endpoints.

use reqwest::Url;

use crate::error::SdkError;
use crate::{Balance, FaucetStatus, Token};
use dytallix_core::address::DAddr;

/// Client for requesting DGT and DRT from a Dytallix faucet.
#[derive(Debug, Clone)]
pub struct FaucetClient {
    endpoint: String,
    http: reqwest::Client,
}

impl FaucetClient {
    /// Creates a faucet client for the provided endpoint.
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_owned(),
            http: reqwest::Client::new(),
        }
    }

    /// Creates a faucet client for the canonical Dytallix testnet faucet.
    pub fn testnet() -> Self {
        Self::new("https://faucet.dytallix.com")
    }

    /// Requests both DGT and DRT for the provided address.
    pub async fn fund(&self, address: &DAddr) -> Result<Balance, SdkError> {
        self.post_balance("/v1/fund", &FundRequest::both(address.clone()))
            .await
    }

    /// Requests DGT for the provided address.
    pub async fn fund_dgt(&self, address: &DAddr) -> Result<u128, SdkError> {
        let response = self
            .post_amount(
                "/v1/fund/dgt",
                &FundRequest::single(address.clone(), Token::DGT),
            )
            .await?;
        Ok(response.amount)
    }

    /// Requests DRT for the provided address.
    pub async fn fund_drt(&self, address: &DAddr) -> Result<u128, SdkError> {
        let response = self
            .post_amount(
                "/v1/fund/drt",
                &FundRequest::single(address.clone(), Token::DRT),
            )
            .await?;
        Ok(response.amount)
    }

    /// Fetches faucet eligibility and retry information for the provided address.
    pub async fn status(&self, address: &DAddr) -> Result<FaucetStatus, SdkError> {
        let url = self.url(&format!("/v1/status/{address}"))?;
        let response =
            self.http
                .get(url.clone())
                .send()
                .await
                .map_err(|err| SdkError::FaucetUnavailable {
                    endpoint: url.to_string(),
                    reason: err.to_string(),
                })?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|err| SdkError::Serialization(err.to_string()))
        } else if response.status().as_u16() == 429 {
            let retry_after_seconds = retry_after_seconds(response.headers());
            Err(SdkError::FaucetRateLimited {
                retry_after_seconds,
            })
        } else {
            let reason = response
                .text()
                .await
                .unwrap_or_else(|_| "request failed".to_owned());
            Err(SdkError::FaucetUnavailable {
                endpoint: url.to_string(),
                reason,
            })
        }
    }

    async fn post_balance(&self, path: &str, request: &FundRequest) -> Result<Balance, SdkError> {
        let url = self.url(path)?;
        let response = self
            .http
            .post(url.clone())
            .json(request)
            .send()
            .await
            .map_err(|err| SdkError::FaucetUnavailable {
                endpoint: url.to_string(),
                reason: err.to_string(),
            })?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|err| SdkError::Serialization(err.to_string()))
        } else if response.status().as_u16() == 429 {
            let retry_after_seconds = retry_after_seconds(response.headers());
            Err(SdkError::FaucetRateLimited {
                retry_after_seconds,
            })
        } else {
            let reason = response
                .text()
                .await
                .unwrap_or_else(|_| "request failed".to_owned());
            Err(SdkError::FaucetUnavailable {
                endpoint: url.to_string(),
                reason,
            })
        }
    }

    async fn post_amount(
        &self,
        path: &str,
        request: &FundRequest,
    ) -> Result<FundAmountResponse, SdkError> {
        let url = self.url(path)?;
        let response = self
            .http
            .post(url.clone())
            .json(request)
            .send()
            .await
            .map_err(|err| SdkError::FaucetUnavailable {
                endpoint: url.to_string(),
                reason: err.to_string(),
            })?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|err| SdkError::Serialization(err.to_string()))
        } else if response.status().as_u16() == 429 {
            let retry_after_seconds = retry_after_seconds(response.headers());
            Err(SdkError::FaucetRateLimited {
                retry_after_seconds,
            })
        } else {
            let reason = response
                .text()
                .await
                .unwrap_or_else(|_| "request failed".to_owned());
            Err(SdkError::FaucetUnavailable {
                endpoint: url.to_string(),
                reason,
            })
        }
    }

    fn url(&self, path: &str) -> Result<Url, SdkError> {
        let joined = format!("{}{}", self.endpoint, path);
        Url::parse(&joined).map_err(|err| SdkError::Network(err.to_string()))
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct FundRequest {
    address: DAddr,
    token: Option<Token>,
}

impl FundRequest {
    fn both(address: DAddr) -> Self {
        Self {
            address,
            token: None,
        }
    }

    fn single(address: DAddr, token: Token) -> Self {
        Self {
            address,
            token: Some(token),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FundAmountResponse {
    amount: u128,
}

fn retry_after_seconds(headers: &reqwest::header::HeaderMap) -> u64 {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(60)
}
