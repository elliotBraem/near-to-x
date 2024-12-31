use intear_oracle::{
    consumer::ext_oracle_consumer,
    producer::{ProducerId, Response},
};
use near_sdk::collections::LookupMap;
use near_sdk::{env, near, AccountId, Gas, PanicOnDefault, Promise};

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct PendingLink {
    account_id: AccountId,
    created_at: u64,
}

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct TwitterRequest {
    oauth_token: String,
    linking_id: String,
}

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct TwitterVerification {
    handle: String,
    user_id: String,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    oracle_contract: AccountId,
    producer_id: ProducerId,
    pending_links: LookupMap<String, PendingLink>,
    twitter_links: LookupMap<AccountId, TwitterVerification>,
}

#[near]
impl Contract {
    #[init]
    pub fn new(oracle_contract: AccountId, producer_id: AccountId) -> Self {
        Self {
            oracle_contract,
            producer_id,
            pending_links: LookupMap::new(b"p".to_vec()),
            twitter_links: LookupMap::new(b"t".to_vec()),
        }
    }

    pub fn init_linking(&mut self) -> String {
        let linking_id = self.generate_unique_id();
        self.pending_links.insert(
            &linking_id,
            &PendingLink {
                account_id: env::predecessor_account_id(),
                created_at: env::block_timestamp(),
            },
        );
        linking_id
    }

    pub fn verify_link(&mut self, linking_id: String, oauth_token: String) -> Promise {
        // Verify linking_id exists and belongs to caller
        let pending = self
            .pending_links
            .get(&linking_id)
            .expect("Invalid linking ID");
        assert_eq!(
            pending.account_id,
            env::predecessor_account_id(),
            "Unauthorized"
        );

        let request = TwitterRequest {
            oauth_token,
            linking_id: linking_id.clone(),
        };

        // Call Intear Oracle
        ext_oracle_consumer::ext(self.oracle_contract.clone())
            .with_static_gas(Gas::from_tgas(10))
            .request(
                self.producer_id.clone(),
                near_sdk::serde_json::to_string(&request).unwrap(),
            )
            .then(Self::ext(env::current_account_id()).on_response())
    }

    #[private]
    pub fn on_response(&mut self, #[callback_unwrap] result: Option<Response>) {
        let response = result.expect("Oracle didn't submit a response in time");
        let verification: TwitterVerification =
            near_sdk::serde_json::from_str(&response.response_data).expect("Invalid response data");

        // Get pending link from verification data
        let pending = self
            .pending_links
            .get(&verification.handle)
            .expect("Request ID not found");

        // Store verification
        self.twitter_links
            .insert(&pending.account_id, &verification);

        // Clean up pending link
        self.pending_links.remove(&verification.handle);
    }

    fn generate_unique_id(&self) -> String {
        let random_bytes = env::random_seed();
        hex::encode(&random_bytes[..16])
    }

    pub fn get_twitter_link(&self, account_id: AccountId) -> Option<TwitterVerification> {
        self.twitter_links.get(&account_id).map(|v| v.clone())
    }
}
