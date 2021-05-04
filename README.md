# Rainbow Network Secret Transfer Contract

The smart contract is built to allow Polygon(Matic) assets to be transferred over to the Secret Network in the form of SNIP-20 compliant assets.
Note: The currency's admin must be this smart contract's address to allow minting.

## Usage

Users may query some information regarding the contract's coin support and may move their supported coins over to the Polygon network.

### Queries

Get the contract's current admin
```bash
secretcli query compute query <Contract Address> '{"admin": {}}'
```

Get a list of all the supported coins
```bash
secretcli query compute query <Contract Address> '{"coins": {}}'
```

Get more information on one of the supported coins
```bash
secretcli query compute query <Contract Address> '{"coin": {"coin": <coin>}}'
```

### Interactions

Move funds from <user secret address> over to <user polygon address>, remember that you must first give the contract an allowance.

```bash
secretcli tx compute execute <Contract Address> '{"transfer_to_matic_addr": {"recipient": "<user polygon address>", "coin": "<coin>", "amount": "<amount>"}'
```

## Admin Usage

Contract managers get to have more specific powers

Change the current admin

```bash
secretcli tx compute execute <Contract Address> '{"change_admin": {"address": "<new admin secret address>"}'
```

Add a new supported coin

```bash
secretcli tx compute execute <Contract Address> '{"add_coin": {"coin": "<coin>", "secret_addr": "<coin secret address>", "secret_hash": "<coin hash>", "matic_addr": "<coin matic address>"}'
```

Remove a supported coin

```bash
secretcli tx compute execute <Contract Address> '{"remove_coin": {"coin": "<coin>"}'
```

This allows the contract to mint the amount sent over form the bridge

```bash
secretcli tx compute execute <Contract Address> '{"transfer_from_matic_addr": {"recipient": "<user secret address>", "coin": "<coin>", "amount": "<amount>"}'
```