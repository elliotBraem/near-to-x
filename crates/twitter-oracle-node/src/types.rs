use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterRequest {
    pub oauth_token: String,
    pub near_account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterVerification {
    pub twitter_id: String,
    pub twitter_username: String,
    pub near_account_id: String,
    pub verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct OracleRequestEvent {
    pub producer_id: String,
    #[allow(unused)]
    pub consumer_id: String,
    pub request_id: String,
    pub request_data: String,
}

#[derive(Debug, Deserialize)]
pub struct TwitterApiResponse {
    pub id: String,
    pub username: String,
}
