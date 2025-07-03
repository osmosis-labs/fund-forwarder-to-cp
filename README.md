# Fund Forwarder Contract

A CosmWasm smart contract that automatically forwards funds of a specified denomination to the community pool.

## Overview

This contract is designed to act as a fund forwarder that:
1. Accepts a specific denomination during instantiation
2. Forwards all funds of that denomination to the community pool when the `ForwardFunds` execute message is called
3. Only forwards the specified denomination, ignoring other tokens in the contract's balance

## Features

- **Configurable Denomination**: The contract accepts any denomination during instantiation
- **Community Pool Integration**: Uses the distribution module's `FundCommunityPool` message
- **State Management**: Stores the configured denomination in contract state
- **Query Support**: Allows querying the contract's configuration
- **Comprehensive Testing**: Includes unit tests for all functionality

## Contract Messages

### InstantiateMsg
```rust
pub struct InstantiateMsg {
    pub denom: String,
}
```
- **denom**: The denomination that this contract will forward to the community pool

### ExecuteMsg
```rust
pub enum ExecuteMsg {
    ForwardFunds {},
}
```
- **ForwardFunds**: Forwards all funds of the configured denomination to the community pool

### QueryMsg
```rust
pub enum QueryMsg {
    Config {},
}
```
- **Config**: Returns the contract's configuration including the denomination

### Query Response
```rust
pub struct ConfigResponse {
    pub denom: String,
}
```

## Usage

### 1. Instantiate the Contract
```bash
# Example with USDN denomination
osmosisd tx wasm instantiate <code_id> '{"denom": "ibc/0C39BD03B5C57A1753A9B73164705871A9B549F1A5226CFD7E39BE7BF73CF8CF"}' \
  --label "fund-forwarder" \
  --admin <admin_address> \
  --gas auto \
  --gas-adjustment 1.3 \
  --from <key_name>
```

### 2. Send Funds to Contract
```bash
# Send funds to the contract address
osmosisd tx bank send <sender_address> <contract_address> 1000000ibc/0C39BD03B5C57A1753A9B73164705871A9B549F1A5226CFD7E39BE7BF73CF8CF \
  --gas auto \
  --gas-adjustment 1.3 \
  --from <key_name>
```

### 3. Forward Funds to Community Pool
```bash
# Execute the forward funds message
osmosisd tx wasm execute <contract_address> '{"forward_funds": {}}' \
  --gas auto \
  --gas-adjustment 1.3 \
  --from <key_name>
```

## Building and Testing

### Prerequisites
- Rust 1.70+
- wasm-pack (for building)
- cosmwasm-check (for validation)

### Build the Contract
```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release
```

### Run Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Test Coverage

The contract includes comprehensive tests covering:

1. **Proper Instantiation**: Verifies contract instantiates correctly with denom
2. **Forward Funds Success**: Tests successful forwarding of funds
3. **No Balance Handling**: Tests behavior when contract has no balance
4. **Zero Balance Handling**: Tests behavior with zero balance
5. **Other Tokens Ignored**: Verifies other tokens don't interfere
6. **Query Configuration**: Tests the query functionality

## Contract State

The contract stores its configuration in the following structure:

```rust
pub struct Config {
    pub denom: String,
}
```

## Error Handling

The contract defines the following error types:

- `NoFunds`: Returned when the contract has no balance of the configured denomination

## Security Considerations

1. **Access Control**: The `ForwardFunds` message can be called by anyone
2. **Fund Safety**: Only forwards the specified denomination, preserving other tokens
3. **State Immutability**: The denomination cannot be changed after instantiation
4. **Community Pool**: Funds are sent to the community pool which is managed by governance
