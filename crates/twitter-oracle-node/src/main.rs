mod types;

use std::sync::Arc;

use anyhow::Result;
use inevents_websocket_client::EventStreamClient;
use intear_events::events::log::log_nep297::LogNep297Event;
use json_filter::{Filter, Operator};
use log::{error, info, warn};
use near_api::prelude::{Account, Contract, NetworkConfig};
use near_api::signer::secret_key::SecretKeySigner;
use near_api::signer::Signer;
use near_gas::NearGas;
use near_token::NearToken;
use reqwest::Client;
use types::{OracleRequestEvent, TwitterApiResponse, TwitterRequest, TwitterVerification};

const ORACLE_CONTRACT: &str = "dev-unaudited-v1.oracle.intear.near";
const TWITTER_API_URL: &str = "https://api.twitter.com/2/users/me";

#[derive(Clone)]
struct TwitterNode {
    account: Account,
    contract: Contract,
    signer: Arc<Signer>,
    twitter_client: Client,
}

impl TwitterNode {
    async fn handle_request(&self, event: OracleRequestEvent) -> Result<()> {
        let request: TwitterRequest = serde_json::from_str(&event.request_data)?;
        info!("Processing Twitter verification for account: {}", request.near_account_id);

        let mut attempts = 0;
        let verification = loop {
            attempts += 1;
            match self.verify_twitter_token(&request.oauth_token).await {
                Ok(twitter_user) => {
                    break TwitterVerification {
                        twitter_id: twitter_user.id,
                        twitter_username: twitter_user.username,
                        near_account_id: request.near_account_id.clone(),
                        verified: true,
                    }
                }
                Err(e) => {
                    if attempts >= 3 {
                        error!("Failed to verify Twitter token after 3 attempts: {e:?}");
                        break TwitterVerification {
                            twitter_id: String::new(),
                            twitter_username: String::new(),
                            near_account_id: request.near_account_id.clone(),
                            verified: false,
                        };
                    }
                    warn!("Attempt {attempts}/3 failed to verify Twitter token: {e:?}");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        };

        let tx_hash = self
            .contract
            .call_function(
                "submit",
                serde_json::json!({
                    "request_id": event.request_id,
                    "verification": verification,
                }),
            )?
            .transaction()
            .gas(NearGas::from_tgas(300))
            .deposit(NearToken::from_yoctonear(0))
            .with_signer(self.account.0.clone(), self.signer.clone())
            .with_retries(5)
            .send_to(&NetworkConfig {
                rpc_url: "https://rpc.shitzuapes.xyz".parse().unwrap(),
                ..NetworkConfig::mainnet()
            })
            .await?
            .transaction
            .hash;

        info!("Responded to oracle request: {tx_hash}");
        Ok(())
    }

    async fn verify_twitter_token(&self, token: &str) -> Result<TwitterApiResponse> {
        let response = self
            .twitter_client
            .get(TWITTER_API_URL)
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Twitter API error: {}",
                response.status()
            ));
        }

        let user = response.json::<TwitterApiResponse>().await?;
        Ok(user)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        error!("Failed to load .env file: {}", e);
    }

    env_logger::init();

    let oracle = Arc::new(TwitterNode {
        account: Account(
            std::env::var("ACCOUNT_ID")
                .expect("No ACCOUNT_ID environment variable")
                .parse()
                .expect("ACCOUNT_ID environment variable is invalid"),
        ),
        contract: Contract(
            std::env::var("CONTRACT_ID")
                .expect("No CONTRACT_ID environment variable")
                .parse()
                .expect("CONTRACT_ID environment variable is invalid"),
        ),
        signer: Signer::new(SecretKeySigner::new(
            std::env::var("PRIVATE_KEY")
                .expect("No PRIVATE_KEY environment variable")
                .parse()
                .expect("PRIVATE_KEY environment variable is invalid"),
        ))
        .expect("Failed to create a signer"),
        twitter_client: Client::new(),
    });

    let client = EventStreamClient::default();
    client
        .stream_events::<LogNep297Event, _, _>(
            "log_nep297",
            Some(Operator::And(vec![
                Filter {
                    path: "account_id".to_string(),
                    operator: Operator::Equals(serde_json::Value::String(
                        ORACLE_CONTRACT.to_string(),
                    )),
                },
                Filter {
                    path: "event_standard".to_string(),
                    operator: Operator::Equals(serde_json::Value::String(
                        "intear-oracle".to_string(),
                    )),
                },
                Filter {
                    path: "event_event".to_string(),
                    operator: Operator::Equals(serde_json::Value::String("request".to_string())),
                },
            ])),
            move |event| {
                let oracle = oracle.clone();
                async move {
                    if let Some(event_data) = event.event_data {
                        if let Ok(request) =
                            serde_json::from_value::<OracleRequestEvent>(event_data)
                        {
                            if request.producer_id == oracle.contract.0 {
                                if let Err(err) = oracle.handle_request(request).await {
                                    error!("Failed to handle request: {err:?}");
                                }
                            }
                        }
                    }
                }
            },
        )
        .await;

    Ok(())
}
