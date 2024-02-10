# Repyh PoW Blockchain 🏗️⛓️

_by Yann Bolliger_

This project implements a very simple proof-of-work blockchain in Rust.

## How to run

### Node
```sh
cargo run --bin node
```
This will start a single node without any known network peers.
To connect nodes, start more nodes and give them at least one address of a peer:
```sh
cargo run --bin node -- 127.0.0.1:7000
```
You can provide as many space-separated addresses as you like. 
However, one is usually enough as the nodes gossip their addresses among each other.

### Client/Wallet

To run the client application that proposes some random transactions to a node:
```sh
cargo run --bin client -- 127.0.0.1:7000
```
Again, you can provide as many space-separated addresses as you like, but the nodes also propagate transactions
in the network, so one should be good.

### Logging/printing

Both binaries print some status and actions to the standard output, so you can see what's happening.

## Implementation

The library implements the [`BlockChain`](./src/chain.rs) data structure which in turn consists of:
 - [`Block`](./src/block.rs)s that contain
 - [`Transactions`](./src/tx.rs).

Single SHA256 hashing is used. The protocol is described by [`Message`](./src/msg.rs) and fully implemented in
the [node](./src/bin/node.rs) binary.

## Limitations

Most limitations are highlighted in the code with `TODO` comments. Some of the bigger simplifications are:

- There is no orphan block nor synchronisation mechanism. I.e. a node has to participate in the network from the first
  block that is being mined. Otherwise, it has no chance of synchronising old blocks and will reject new ones.

- Transactions are just a plain data structure. There is _no_ built-in integrity, no signing, no double-spending 
  prevention... the validity of a transaction is not even defined. Consequently, transactions are not checked for
  validity before being possibly applied to a block/the chain.

- The proof-of-work difficulty is given by a constant. This should be changed to be a function of the block height
  or the hashrate of the network. Also, blocks should then be validated to contain the correct difficulty.

- The client/wallet application currently just creates 10 more or less random transactions and proposes them to the
  network. It would be nice to let the user specify transactions to propose (i.e. in a JSON file or similar).