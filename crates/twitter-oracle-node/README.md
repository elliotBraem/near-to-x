# Twitter (X) Oracle Node

A node service that monitors the Twitter (X) Oracle contract on NEAR Protocol and processes Twitter account verification requests. This node verifies Twitter OAuth tokens and submits verification results back to the contract.

## Features

- Monitors oracle contract events for verification requests
- Verifies Twitter OAuth tokens using Twitter's API
- Submits verification results back to the contract
- Automatic retries for failed verifications
- Robust error handling and logging

## Prerequisites

1. Twitter API Access
   - Create a Twitter Developer Account
   - Create a project and app at https://developer.twitter.com
   - Get your API keys and OAuth 2.0 credentials

2. NEAR Account
   - Create a NEAR account to run the oracle node
   - Generate a full access key for contract interactions

## Environment Variables

Required environment variables:

- `ACCOUNT_ID`: The account ID of the oracle producer
- `PRIVATE_KEY`: The private key for the oracle producer account
- `CONTRACT_ID`: The contract ID of the Twitter Oracle contract (not to confuse with the Intear Oracle main contract)

Optional environment variables:

- `NEAR_ENV`: Network to use (defaults to 'mainnet')
- `RPC_URL`: Custom RPC URL (defaults to 'https://rpc.shitzuapes.xyz')

## Running

1. Install dependencies:

```bash
cargo build
```

2. Create a `.env` file with required environment variables:

```env
ACCOUNT_ID=your-oracle.near
PRIVATE_KEY=ed25519:...
CONTRACT_ID=twitter-oracle.near
```

3. Run the node:

```bash
cargo run
```

## Monitoring

The node provides detailed logging of:

- Received verification requests
- Twitter API interactions
- Verification results
- Contract interactions
- Any errors or retries

Use `RUST_LOG=info` for standard logging or `RUST_LOG=debug` for more detailed output.
