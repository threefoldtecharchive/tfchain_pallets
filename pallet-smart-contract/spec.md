# Smart Contract for IT on the blockchain

## Overview

The current smart contract 4 IT on the Threefold Grid is centralized. It is owned by the *explorer* and all it's data sits in a MongoDB. We want to decentralize the way a User and a Farmer aggree on what needs to be deployed on their nodes.

## Current architecture

The TFexplorer is responsible for deploying, decomissioning, refunding, .. of workloads. It is the intermediate party between the User and the Farmer on the Threefold Grid. For more details on the workings of the current model: https://manual2.threefold.io/#/smartcontract_details

The issue with this architecture is that there is a single point of failure. In this case it can be the machines where the explorers are running on.

We, as a company, also promote decentralization in any way possible. The very way we run this critical component centralized is contradictionary.

## Proposed architecture

Two main components will play a role in achieving a decentralised consensus between a user and a farmer.

1: TFGrid Substrate Database [Pallet TFGrid](../pallet-tfgrid/readme.md)
2: TFGrid Smart Contract

The TFGrid Substrate Database will keep a record of all users, twins, nodes and farmers in the TF Grid network. This makes it easy to integrate the Smart Contract on Substrate as well since we can read from that storage in Runtime.

The Smart Contract on Substrate will work as following:

## 1: The user wants to deploy a workload, he interacts with this smart contract pallet and calls: `create_contract` with the input being:

json
```
contract {
    "workload": "encrypted_workload_data",
    "node_id": "some_node_address"
}
```

This pallet saves this data to storage and returns the user a `contract_id`.

## 2: The user sends the contractID through the RMB to the destination Node.

The Node reads from the [RMB](https://github.com/threefoldtech/rmb) and sees a deploy command, it reads the contractID and fetches that Contract from this pallet's storage. It decodes the workload and does validation before it deploys the contents. If successfull it sets the Contract to state `deployed` on the chain. Else the contract is removed.

## 3: The Node sends capacity reports to the chain

The Node periodically sends capacity reports back to the chain for each deployed contract. The chain will compute how much is being used and will bill the user based on the farmers prices (the chain can read these prices by quering the farmers storage and reading the pricing data). Billing will be done in Database Tokens and will be send to the corresponding farmer. If the user runs out of funds the chain will set the contract state to `canceled` or it will be removed from storage. The Node needs to act on this. 

The main currency of this chain. More information on this is explained here: TODO


## Footnote

Sending the workloads encrypted to the chain makes sure that nobody except the destination Node can read the deployment's information as this can contain sensitive data. This way we also don't need to convert all the Zero OS primitive types to a Rust implementation and we can keep it relatively simple.