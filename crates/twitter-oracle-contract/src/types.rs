use near_sdk::near;

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct TwitterRequest {
    pub oauth_token: String,
    pub near_account_id: String,
}

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct TwitterVerification {
    pub twitter_id: String,
    pub twitter_username: String,
    pub near_account_id: String,
    pub verified: bool,
}

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct TwitterLinkage {
    pub twitter_id: String,
    pub twitter_username: String,
    pub near_account_id: String,
    pub timestamp: u64,
}
