# Twitter (X) Oracle Contract

A NEAR smart contract that serves as an oracle for Twitter (X) account verification. This contract enables linking Twitter accounts to NEAR accounts through OAuth token verification. The oracle verifies Twitter account ownership and maintains a secure mapping between Twitter and NEAR accounts.

## Features

- Twitter account ownership verification via OAuth tokens
- Secure bidirectional mapping between Twitter and NEAR accounts
- Query methods to:
  - Get linked NEAR account for a Twitter ID
  - Check if a NEAR account is connected to Twitter
  - Get Twitter account details for a linked NEAR account

## How it Works

1. A user initiates a link request through a consumer contract, providing their Twitter OAuth token
2. The oracle node verifies the token with Twitter's API
3. Upon successful verification, the contract stores the linkage
4. The verification result is returned to the consumer contract

## Usage

Consumer contracts can request Twitter account verification by calling the oracle contract with:

- OAuth token from Twitter
- NEAR account to link

The oracle will verify ownership and store the linkage if successful.
