# MetaMask Proxy Canister PoC

This Proof of Concept shows how MetaMask could be used on the Internet Computer, by using a proxy canister that acts as JSON RPC Provider.
The proxy can be used to translate certain EVM contract calls to calls on the Internet Computer. This demo aims to implement this for fungible token contract calls.

## Architecture

There are two possible architectures.
1) A shared proxy canister that can serve all Ethereum addresses
2) Individual user-owned proxy canisters

For simplicity, this PoC uses the former approach with a shared proxy canister.


## Demo

See https://emrov-eyaaa-aaaap-qatxq-cai.ic0.app

## Status and To Dos

- [x] Proxy canister as MetaMask JSON RPC Provider 
- [x] Send and receive ICP
- [x] Translate ERC20 related calls in proxy canister, in particular, `transfer`
- [ ] Full demo flow with swapping GoerliETH to ckETH
- [ ] Proxy frontend to send assets to other principals
- [ ] Translate ERC721 related calls



