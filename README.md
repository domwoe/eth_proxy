# MetaMask Proxy Canister PoC

This Proof of Concept shows how MetaMask could be used on the Internet Computer, by using a proxy canister that acts as JSON RPC Provider.
The proxy can be used to translate certain EVM contract calls to calls on the Internet Computer. This demo aims to implement this for DIP20 contract calls.

## Architecture


## Demo Flow

## Status and To Dos

[x] Proxy canister as MetaMask JSON RPC Provider 
[x] Send and receive ICP
[x] Fake ERC20 related calls in proxy canister, in particular `transfer`


## Issues


### Mapping from Ethereum addresses to principals

### Authentication of calls

All calls to the proxy canister are made using the anonymous identity, i.e. are unauthenticated. This is not a super big issue, since the actual transactions are signed and can be verified inside the proxy canister.
However, there are still some update calls, that are a bit problematic, i.e. RPC calls to get the current block number or the current transaction count. MetaMask seems to keep track of the current block height and makes frequent calls to the RPC provider to get the latest block height. If we don't increase our fake block height MetaMask won't do any other calls. In our current naive implementation, we just increase the block height by one every time this request is made. We do the same for the transaction count request, which determines the nonce MetaMask uses when creating a transaction. Ideally, these calls should only be allowed by the owner. A quick fix would be to introduce an API key that is added as a query parameter.

### Usage of the raw domain


