mod types;

use intear_oracle::consumer::RequestId;
use intear_oracle::fees::PrepaidFee;
use intear_oracle::producer::{ext_oracle_responder, ProducerContract, Response};
use near_sdk::{
    env, near, store::LookupMap, AccountId, NearToken, PanicOnDefault, Promise,
};
use types::{TwitterLinkage, TwitterRequest, TwitterVerification};

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    oracle_contract: AccountId,
    requests: LookupMap<RequestId, (TwitterRequest, PrepaidFee)>,
    twitter_to_near: LookupMap<String, String>, // twitter_id -> near_account_id
    near_to_twitter: LookupMap<String, TwitterLinkage>, // near_account_id -> TwitterLinkage
}

#[near]
impl Contract {
    #[init]
    pub fn new(oracle_contract: AccountId) -> Self {
        Self {
            oracle_contract,
            requests: LookupMap::new(b"r".to_vec()),
            twitter_to_near: LookupMap::new(b"t2n".to_vec()),
            near_to_twitter: LookupMap::new(b"n2t".to_vec()),
        }
    }

    pub fn submit(&mut self, request_id: RequestId, verification: TwitterVerification) {
        let (request, prepaid_fee) = self.requests.get(&request_id).expect("Request not found");

        // Verify the NEAR account matches the request
        assert_eq!(
            request.near_account_id,
            verification.near_account_id,
            "NEAR account mismatch"
        );

        // If verification successful, store the linkage
        if verification.verified {
            let linkage = TwitterLinkage {
                twitter_id: verification.twitter_id.clone(),
                twitter_username: verification.twitter_username.clone(),
                near_account_id: verification.near_account_id.clone(),
                timestamp: env::block_timestamp(),
            };

            self.twitter_to_near
                .insert(verification.twitter_id.clone(), verification.near_account_id.clone());
            self.near_to_twitter
                .insert(verification.near_account_id.clone(), linkage);
        }

        let prepaid_near = match prepaid_fee {
            PrepaidFee::Near { amount, .. } => amount.as_yoctonear(),
            _ => env::panic_str("Invalid prepaid fee token"),
        };

        // Refund the prepaid fee to the sender
        Promise::new(env::predecessor_account_id())
            .transfer(NearToken::from_yoctonear(prepaid_near));

        // Send response to oracle contract
        ext_oracle_responder::ext(self.oracle_contract.clone()).respond(
            request_id,
            Response {
                response_data: near_sdk::serde_json::to_string(&verification).unwrap(),
                refund_amount: None,
            },
        );
    }

    // View methods
    pub fn get_near_account(&self, twitter_id: String) -> Option<String> {
        self.twitter_to_near.get(&twitter_id).cloned()
    }

    pub fn is_connected(&self, near_account_id: String) -> bool {
        self.near_to_twitter.contains_key(&near_account_id)
    }

    pub fn get_twitter_info(&self, near_account_id: String) -> Option<TwitterLinkage> {
        self.near_to_twitter.get(&near_account_id).cloned()
    }
}

#[near]
impl ProducerContract for Contract {
    fn on_request(&mut self, request_id: RequestId, request_data: String, prepaid_fee: PrepaidFee) {
        if env::predecessor_account_id() != self.oracle_contract {
            env::panic_str("Only oracle contract can call this method");
        }

        let request: TwitterRequest =
            near_sdk::serde_json::from_str(&request_data).expect("Invalid request data");

        self.requests.insert(request_id, (request, prepaid_fee));
    }
}
