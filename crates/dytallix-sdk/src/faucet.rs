//! HTTP client for Dytallix faucet endpoints.

use reqwest::StatusCode;
use reqwest::Url;

use crate::error::SdkError;
use crate::{Balance, FaucetStatus};
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
        Self::new("https://dytallix.com/api/faucet")
    }

    /// Requests both DGT and DRT for the provided address.
    pub async fn fund(&self, address: &DAddr) -> Result<Balance, SdkError> {
        let limits = self.get_limits().await?;
        let response = self
            .post_request(
                "/request",
                &FundRequest::new(address.clone(), limits.dgt, limits.drt),
            )
            .await?;

        if response.balances.dgt > 0 || response.balances.drt > 0 {
            Ok(response.balances)
        } else {
            Ok(Balance {
                dgt: response.funded.dgt,
                drt: response.funded.drt,
            })
        }
    }

    /// Requests DGT for the provided address.
    pub async fn fund_dgt(&self, address: &DAddr) -> Result<u128, SdkError> {
        let limits = self.get_limits().await?;
        let response = self
            .post_request(
                "/request",
                &FundRequest::new(address.clone(), limits.dgt, 0),
            )
            .await?;

        Ok(if response.funded.dgt > 0 {
            response.funded.dgt
        } else {
            limits.dgt
        })
    }

    /// Requests DRT for the provided address.
    pub async fn fund_drt(&self, address: &DAddr) -> Result<u128, SdkError> {
        let limits = self.get_limits().await?;
        let response = self
            .post_request(
                "/request",
                &FundRequest::new(address.clone(), 0, limits.drt),
            )
            .await?;

        Ok(if response.funded.drt > 0 {
            response.funded.drt
        } else {
            limits.drt
        })
    }

    /// Fetches faucet eligibility and retry information for the provided address.
    pub async fn status(&self, address: &DAddr) -> Result<FaucetStatus, SdkError> {
        let url = self.url(&format!("/check/{address}"))?;
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
            let status: FaucetCheckResponse = response
                .json()
                .await
                .map_err(|err| SdkError::Serialization(err.to_string()))?;

            Ok(FaucetStatus {
                can_request: status.can_request,
                retry_after_seconds: status.time_until_next.map(|minutes| minutes * 60),
            })
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

    async fn post_request(
        &self,
        path: &str,
        request: &FundRequest,
    ) -> Result<FundResponse, SdkError> {
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
            let status = response.status();
            let reason = response
                .text()
                .await
                .unwrap_or_else(|_| "request failed".to_owned());
            if path == "/request" && faucet_request_route_missing(status) {
                return self.post_base_request(request).await;
            }
            Err(SdkError::FaucetUnavailable {
                endpoint: url.to_string(),
                reason,
            })
        }
    }

    async fn post_base_request(&self, request: &FundRequest) -> Result<FundResponse, SdkError> {
        let url = self.url("")?;
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

    async fn get_limits(&self) -> Result<FaucetLimits, SdkError> {
        let url = self.url("/status")?;
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
            let status: FaucetStatusResponse = response
                .json()
                .await
                .map_err(|err| SdkError::Serialization(err.to_string()))?;

            Ok(status.limits)
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

fn faucet_request_route_missing(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::NOT_FOUND | StatusCode::METHOD_NOT_ALLOWED
    )
}

#[derive(Debug, Clone, serde::Serialize)]
struct FundRequest {
    address: DAddr,
    dgt_amount: u128,
    drt_amount: u128,
}

impl FundRequest {
    fn new(address: DAddr, dgt_amount: u128, drt_amount: u128) -> Self {
        Self {
            address,
            dgt_amount,
            drt_amount,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FaucetStatusResponse {
    limits: FaucetLimits,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FaucetLimits {
    dgt: u128,
    drt: u128,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct FaucetCheckResponse {
    can_request: bool,
    time_until_next: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FundResponse {
    #[serde(default = "default_balance")]
    balances: Balance,
    #[serde(default)]
    funded: FundedAmounts,
}

fn default_balance() -> Balance {
    Balance { dgt: 0, drt: 0 }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct FundedAmounts {
    #[serde(default)]
    dgt: u128,
    #[serde(default)]
    drt: u128,
}

fn retry_after_seconds(headers: &reqwest::header::HeaderMap) -> u64 {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(60)
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::FaucetClient;
    use crate::Balance;
    use dytallix_core::address::DAddr;
    use dytallix_core::keypair::DytallixKeypair;

    #[tokio::test]
    async fn fund_falls_back_to_base_faucet_route_when_request_route_is_missing() {
        let server = TestFaucetServer::spawn(vec![
            ExpectedRequest::new("GET", "/status", 200, r#"{"limits":{"dgt":0,"drt":100}}"#),
            ExpectedRequest::new("POST", "/request", 404, r#"{"error":"not found"}"#),
            ExpectedRequest::new("POST", "/", 200, r#"{"balances":{"dgt":0,"drt":100}}"#),
        ])
        .await;
        let keypair = DytallixKeypair::generate();
        let address = DAddr::from_public_key(keypair.public_key()).unwrap();

        let balance = FaucetClient::new(&server.endpoint)
            .fund(&address)
            .await
            .unwrap();

        assert_eq!(balance, Balance { dgt: 0, drt: 100 });
    }

    struct TestFaucetServer {
        endpoint: String,
    }

    struct ExpectedRequest {
        method: &'static str,
        path: &'static str,
        status: u16,
        body: &'static str,
    }

    impl ExpectedRequest {
        fn new(method: &'static str, path: &'static str, status: u16, body: &'static str) -> Self {
            Self {
                method,
                path,
                status,
                body,
            }
        }
    }

    impl TestFaucetServer {
        async fn spawn(expected: Vec<ExpectedRequest>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let endpoint = format!("http://{}", listener.local_addr().unwrap());
            tokio::spawn(async move {
                for expected in expected {
                    let (mut stream, _) = listener.accept().await.unwrap();
                    let mut buffer = [0u8; 8192];
                    let read = stream.read(&mut buffer).await.unwrap();
                    let request = String::from_utf8_lossy(&buffer[..read]);
                    let request_line = request.lines().next().unwrap_or_default();
                    let expected_prefix = format!("{} {} ", expected.method, expected.path);
                    assert!(
                        request_line.starts_with(&expected_prefix),
                        "expected request line prefix `{expected_prefix}`, got `{request_line}`"
                    );

                    let status_text = match expected.status {
                        200 => "OK",
                        404 => "Not Found",
                        _ => "Status",
                    };
                    let response = format!(
                        "HTTP/1.1 {} {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        expected.status,
                        status_text,
                        expected.body.len(),
                        expected.body
                    );
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
            });

            Self { endpoint }
        }
    }
}
