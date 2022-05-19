# Fat Contract Workshop

This is a workshop demonstrating how to write a *Fat Contract* with its HTTP request capability on Phala.

## Introduction

Fat Contract is the programming model adopted by Phala Network. Fat Contract is **NOT** smart contract.

Instead, it aims to provide the rich features that ordinary smart contracts cannot offer, including:

- CPU extensive computation: exclusive off-chain execution at the full CPU speed
- Network access: the ability to send the HTTP requests
- Low latency: non-consensus-sensitive operations may not hit the blockchain at all, removing the block latency
- Strong consistency: consensus-sensitive operations remain globally consistent
- Confidentiality: contract state is hidden by default unless you specifically expose it via the read call

Fat Contract is 100% compatible with Substrate's `pallet-contracts`. It fully supports the unmodified ink! smart contracts. Therefore you can still stick to your favorite toolchain including `cargo-contract`,  `@polkadot/contract-api`, and the Polkadot.js Extension.

## About this workshop

This workshop will demonstrate how to use Fat Contract's HTTP request capability to associate a Phala account with a Github user. Such functionality serves as the core for [Decentralized Identity (DID)](https://www.gsma.com/identity/decentralised-identity). Further, we will show how to deploy your contract in [Phala Testnet](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpoc5.phala.network%2Fws#/explorer) and interact with it through our [frontend SDK](https://github.com/Phala-Network/js-sdk).

The Fat Contract "Redeem POAP" links on-chain accounts to Github accounts and then distributes POAP redeem code to the verified users. The simple protocol is listed below:

1. The user should create a Github Gist with a special format with the account id: "This gist is owned by address: 0x01234...."
2. The user submits the raw file URL to the Fat Contract by a _query_ ("https://gist.githubusercontent.com/[username]/[gist-id]/raw/...")
3. The contract then
    - Verifies the URL is valid
    - Fetches the content of the URL (by Fat Contract's HTTP in Query feature)
    - Verifies the account claim in the gist is valid
    - Extracts the Github username from the URL and the account id from the claim
    - Produces an `Attestation` struct with the Github username and the account id, and signs it with a contract-owned key (by Fat Contract's confidentiality and cryptographic feature)
4. The user receives the `SignedAttestation`, and then submit the attestation to the contract by a `redeem` _command_
5. The contract verifies the attestation, and then links the Github username and the account id
6. If the user didn't claim any POAP, it reveals a POAP code to the user after the verification

## Get Things Ready

To get your application running, you will need to download our frontend and backend demo code:

- The backend code, a.k.a. the demo fat contract, is available in our [Fat Contract Workshop Repository](https://github.com/Phala-Network/fat-contract-workshop);
- The frontend SDK ([JS-SDK](https://github.com/Phala-Network/js-sdk)), which already contains the frontend to interact with contract above as an example.

After your contract is ready, you should upload it to some Contract Cluster in [Phala Testnet](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpoc5.phala.network%2Fws#/explorer) and instantiate it.

- Available Cluster: `0x0000000000000000000000000000000000000000000000000000000000000000`

Your contract will be finally deployed to our off-chain Secure Workers. You will need two endpoints to config your frontend to interact with your contract:

- A Phala node WebSocket endpoint to read the on-chain data and send Command: `wss://poc5.phala.network/ws`
- Secure Worker endpoints to send Query to your deployed contract and get results, now we deploy one Secure Worker for our Testnet
  - Worker `0x94a2ded4c77fbb910943f7e452e4d243ee5b60bf1a838a911acf2ffd4bae9b63`
    - Endpoint `https://poc5.phala.network/tee-api-1`

> Note that the node endpoint is generally stable, while the available cluster and Secure Workers can change due to Testnet update. Report in the #dev channel of our [Discord](https://discord.gg/myBmQu5) if they are down.

## Environment Preparation

An operating system of macOS or Linux systems like Ubuntu 18.04/20.04 is recommended for the workshop.
- For macOS users, we recommend to use the *Homebrew* package manager to install the dependencies
- For other Linux distribution users, use the package manager with the system like Apt/Yum

The following toolchains are needed:

- Rust toolchain
    - Install rustup, rustup is the "package manager" of different versions of Rust compilers: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    - This will install `rustup` and `cargo`
- Ink! Contract toolchain
    - Install [binaryen](https://github.com/WebAssembly/binaryen) with
        - Homebrew for macOS: `brew install binaryen`
        - Apt for Ubuntu: `sudo apt install binaryen`
        - or download the [release](https://github.com/WebAssembly/binaryen/releases/tag/version_105) and put it under your `$PATH`
    - Install dylint-link toolchain: `cargo install cargo-dylint dylint-link`
    - Install contract toolchain: `cargo install cargo-contract --force`
    - For macOS M1 chip users: `rustup component add rust-src --toolchain nightly-aarch64-apple-darwin`
- Install frontend toolchain
    - Node.js (>=v16), follow the [official tutorial](https://nodejs.org/en/download/package-manager/)
    - Yarn (v1): `npm install --global yarn`

Check your installation with

```bash
$ rustup toolchain list
# stable-x86_64-unknown-linux-gnu (default)
# nightly-x86_64-unknown-linux-gnu

$ cargo --version
# cargo 1.58.0 (f01b232bc 2022-01-19)

$ cargo contract --version
# cargo-contract 0.17.0-unknown-x86_64-linux-gnu

$ node --version
# v17.5.0

$ yarn --version
# 1.22.17
```

## Create Polkadot Account to Use Phala Testnet

Our Testnet is launched in the `dev` mode, so there are some pre-defined development accounts with enough tokens for you to use.

![](./static/tutor-accounts.png)

Also you can install Polkadot.js extension and create/import the Phala gas account following the [official tutorial](https://wiki.polkadot.network/docs/learn-account-generation#polkadotjs-browser-extension)
- Connect to Phala Testnet
    - Open <https://polkadot.js.org/apps/>;
    - Click left top to switch network;
    - Choose `Test Networks` - `Phala(PoC 5)` and click `Switch` at the top;
- Send some coins to your own account (limited, don't be evil);
    - Send some coins from the development accounts above .
    ![](https://i.imgur.com/l3I14ri.png)

## Compile the Contract

In the `fat-contract-workshop` folder, run the following commands

```bash
cargo +nightly contract build
```

Also test to ensure everything is fine

```bash
cargo +nightly contract test
```

You will find the compile result at `./target/ink`:

```bash
$ ls -h target/ink
# fat_sample.wasm  metadata.json ...
```

`fat_sample.wasm` contains the compiled contract code, and will be uploaded for execution; `metadata.json` contains the information of your contract, especially all the *interfaces* and their *4-byte selectors*. You will use these selectors when interacting with the contract so it will know which function to call.

## Deploy

Collect the above two files and create the contract in Phala Testnet (PoC 5). The contract deployment can be divided into two steps: code upload and contract instantiation.

> We recommend to keep a tab for *Explorer* so you will not miss any historical events.
>
> ![](./static/tutor-explorer.png)

### Code Upload

Choose `Developer` - `Extrinsics`, and select the extrinsic `phalaFatContracts` and `uploadCodeToCluster`.

- Drag the `fat_sample.wasm` file;
- Use the available Cluster mentioned [above](#get-things-ready);

and send the transaction.

![](./static/tutor-upload.png)

A event of `phalaFatContracts.CodeUploaded` should be observed in the block explorer with the code hash, record the code hash for future use.

![](./static/tutor-uploaded.png)

> Code upload could failed if there are *illegal* instructions in your compiled wasm. Report in the #dev channel of our [Discord](https://discord.gg/myBmQu5) and we will help you find the reason.

### Contract Instantiation

Choose `Developer` - `Extrinsics`, and select the extrinsic `phalaFatContracts` and `instantiateContract`. We explain the arguments as follow:
- `codeIndex`: the code to use, choose `WasmCode` and type in the hash of you uploaded code
- `data`: the instantiation argument. We shall call the constructor function of the contract will the specific function selector, This can be found in the `metadata.json` (in this case, `0xed4b9d1b`)
```json
...
    "constructors": [
    {
        "args": [],
        "docs": [],
        "label": "default",
        "payable": false,
        "selector": "0xed4b9d1b"
    }
],
...
```
- `salt`: some random bytes to prevent collision, like `0x0` or `0x1234`
- `clusterId`: this must be the same as the one when you [upload your code](#code-upload), since the code is stored in the cluster level

There are three events to observe, all these events contain your Contract ID

- `phalaFatContracts.Instantiating`, the chain has receive your request and start instanting;
- `phalaFatContracts.ContractPubkeyAvailable`, the gatekeeper has generated the contract key to encrypt its state and input/output;
- `phalaFatContracts.Instantiated`, your contract is successfully instantiated.

You can go to `Developer` - `Chain state` and select the extrinsic `phalaFatContracts` and `contracts` to see all the contracts.

> ### Handle Instantiation Failure
>
> For now, the contract execution log is not directly available to the developers. Join our [Discord](https://discord.gg/myBmQu5) and we can help forward the Worker logs if necessary.

## Run the Frontend

Phala provides [js-sdk](https://github.com/Phala-Network/js-sdk) to simplified the frontend development. It already contains the frontend for the demo contract, check its [example folder](https://github.com/Phala-Network/js-sdk/tree/main/packages/example).

Follow the steps to run the frontend

1. Download Phala-Network/js-sdk

    ```bash
    git clone --branch ethdenver-2022 https://github.com/Phala-Network/js-sdk.git
    ```

2. Compile and run the frontend. By default it will serve the app at <http://localhost:3000>:

    ```bash
    cd js-sdk

    yarn
    yarn dev
    ```

## Play with Your Contract

Open the Frontend, by default it's in <http://localhost:3000>

![](https://i.imgur.com/h761Y6C.png)

 1. Connect your wallet;
 2. Load the deployed contract;
     - Substrate endpoint: `wss://poc5.phala.network/ws`
     - Secure Worker endpoint: `https://poc5.phala.network/tee-api-1`
     - Contract ID: available during [Contract Instantiation](#contract-instantiation)
     - ABI: copy from `metadata.json` ([prebuilt copy](https://gist.githubusercontent.com/h4x3rotab/4a55b7c6ac9ad50f2f803a1edc93456e/raw/12e0b2d8afacba2fb8de664744fb64c6ed507290/metadata.json))
 3. Click `Sign a Certificate`, this will generate a certificate to encrypt your traffic to/from the contract;
 4. Follow the instruction, copy the contents and create a *public* Github Gist with it;
 5. Open the **RAW** file, and copy the link;
 ![](https://i.imgur.com/YqgesNm.png)
 1. Paste the link to the box and click `Verify`;
 2. The redeem code box in will refresh every 5s. It should show your code once the verification is successful.

> ### Attention
>
> By default, the `poap_code` pool is empty, so the users can only get empty string even if they have passed the verification. The contract admin need to invoke the `admin_set_poap_code()` first to fill in the redeem code pool so the users can really get something.

## Challenge: Fill in the Missing Code

We leave two challenges (labeled by `TODO`) for you to explore.
- The first is about adding the necessary access control to the update redeem code function, you will learn about how to access the contract metadata through the `self.env()` function;
- The second is about check to prevent a double redemption of the code.

### Solution

Please check the `ethdenver-2022-solution` branch.

## Appendix

### Resources

- [JS SDK (with the UI scaffold)](https://github.com/Phala-Network/js-sdk)
    - [SDK docs](https://github.com/Phala-Network/js-sdk/tree/main/packages/sdk)
    - [Scalffold docs](https://github.com/Phala-Network/js-sdk/tree/main/packages/example)
- [ink! Docs](https://paritytech.github.io/ink-docs/)
- [Polkadot.js Docs](https://polkadot.js.org/docs)
- Join [Discord](https://discord.gg/phala) #dev and #hackathon groups!
### Endpoints

- Chain: `wss://poc5.phala.network/ws`
    - Polkadot.js quick link: <https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpoc5.phala.network%2Fws#/explorer>
- Workers (with their identity key)
  - <https://poc5.phala.network/tee-api-1>
      - 0x94a2ded4c77fbb910943f7e452e4d243ee5b60bf1a838a911acf2ffd4bae9b63
  - ~~https://poc5.phala.network/tee-api-2~~
      - ~~0x50ede2dd7c65716a2d55bb945dfa28d951879154f832e049851d7882c288db76~~
  - ~~https://poc5.phala.network/tee-api-3~~
      - ~~0xfe26077a6030e505136855100f335503ca40f6e8afa149b0c6c618e81c1cb53b~~
  - ~~https://poc5.phala.network/tee-api-4~~
      - ~~0x6cfc1282880305c7691f0941b98089b9da17acde43b66ef2220022797bb3e370~~
  - ~~https://poc5.phala.network/tee-api-5~~
      - ~~0xbed94c30d660a1de5a499e38f9f3afe9ccc1ef5f901530efd48de641679fbc7d~~
