# Fat Contract Workshop

This is a simple workshop demostrating how to write a Phala confidential-preserving ink! smart contract in Phala (thus, _Fat Contract_).

## Prepare

1. [Cargo](https://rustup.rs/)

2. [Binaryen wasm toolchain](https://github.com/WebAssembly/binaryen). Install via a package manager or just put the release binaries to your $PATH environment. You will need at least version 99.

3. ink! contract toolkit (This will install the latest version. As of writing, the latest version is `cargo-contract 0.16.0-unknown-x86_64-linux-gnu`)

    ```sh
    cargo install cargo-contract --force
    ```

## Create a new ink! project

```sh
cargo contract new flipper
```

> Or you can clone this repo instead, but please pay attention to the filename.
>
> ```sh
> git clone https://github.com/Phala-Network/fat-contract-workshop.git
> ```

## Compile

```sh
cd flipper
cargo contract build
```

You will find the compile result at `./target/ink`:

> ```bash
> ~ ls -h target/ink
> flipper.contract  flipper.wasm  metadata.json
> ```

## Deploy

Collect the above three files and create the contract in a local testnet.

### Prepare the local testnet

Please follow the _Environment_ and _Build the Core Blockchain_ section in [this tutorial](https://wiki.phala.network/en-us/docs/developer/run-a-local-development-network/#environment) to build a local testnet, but use the branch **fat-contract-workshop** instead (**important!**).

You should run all the three programs, `phala-node`, `pherry`, and `pruntime`, according to the _Build the Core Blockchain_ section in the tutorial.

![](./static/core-terminal.gif)

### Attach the Polkadot.js browser app to the testnet

1. Enter [Polkadot.js Apps](https://polkadot.js.org/apps).
2. Click the icon at the left top corner to open the side bar
3. Switch to "DEVELOPMENT > Costom" and enter your local `phala-node` ws rpc port (by default: ws://localhost:9944)

You should notice the frontend should load and show the blockchain status.

Now, make sure you have [Polkadot.js Extension](https://polkadot.js.org/extension/) installed and have the test account imported (At least `//Alice` and `//Bob`). You will be able to see some balance in the "Account" page under Alice and Bob. For more detials, please check [the appendix](#Polkadotjs-Extension-and-the-common-seeds).

### Deploy the contract

**One-off job.** Navigate to "Developer > Sudo" and send the following transaction. This only need to be done once in a deployment.

```
pahlaRegistry.registerGatekeeper(0x3a3d45dc55b57bf542f4c6ff41af080ec675317f4ed50ae1d2713bf9f892692d)
```

> The argument is the worker id (worker public key). This is the only (hard-coded) worker in your local deployment.

**First step.** Upload the contract code. Navigate to "Developer > Extrinsics", and select

```
phalaRegistry.uploadCode()
```

You should select the wasm file you got from the "Compile" section. Once it's done, you can navigate to the "Network > Explorer" page and find the `phalaRegistry.CodeUploaded` event with the code hash:

> phalaRegistry.CodeUploaded
> 0x911dd86247a3f196379e70c14357bdbb398b6283842d4bfc2213d44b5680eb2c (example, may vary in your build)

**Next step.** Please note the code hash. Then nativate back to "Developer > Extrinsic" and select

```
phalaRegistry.instantiateContract()
```

- `codeIndex`: WasmCode(0x911dd86247a3f196379e70c14357bdbb398b6283842d4bfc2213d44b5680eb2c) (example)
- `data`: 0xed4b9d1b (the default() function)
- `salt`: 0x
- `deployWorker`: 0x3a3d45dc55b57bf542f4c6ff41af080ec675317f4ed50ae1d2713bf9f892692d

You should be able to see the follow event:

> ```
> phalaRegistry.ContractInstantiated
> 
> SpCoreSr25519Public ([u8;32])
>   0x4c66c3bf1e1b4dd02e7666e578e41570e79a544239d8ec77075651ba7879de5e (contract key)
> PhalaTypesContractContractInfo
>   {
>     deployer: 45R2pfjQUW2s9PQRHU48HQKLKHVMaDja7N3wpBtmF28UYDs2 (Alice)
>     groupId: 1
>     salt: 
>     codeIndex: {
>       WasmCode: 0x911dd86247a3f196379e70c14357bdbb398b6283842d4bfc2213d44b5680eb2c (example)
>     }
>     instantiateData: 0xed4b9d1b
>   }
> ```

To double check, you can also navigate to "Developer > Chain State" check the deployment status:

```
phalaRegistry.contractKey()
```

- include option: off

> ```
> [
>   [
>     [
>       0xcf4b9fd7eb64dc1fe5ca550e715a49fae9f5a2de88afd3c32daa137fcc8ca5b7 (contract id)
>     ]
>     0x4c66c3bf1e1b4dd02e7666e578e41570e79a544239d8ec77075651ba7879de5e (contract key)
>   ]
> ]
> ```

Now the contract is up and running at your worker (0x3a3d45dc55b57bf542f4c6ff41af080ec675317f4ed50ae1d2713bf9f892692d), with the contract id `0xcf4b9fd7eb64dc1fe5ca550e715a49fae9f5a2de88afd3c32daa137fcc8ca5b7`.

Please keep the contract id. It will be used in the next step.

### Interact with the contract 

(WIP)

- Download and build Phala-Network/js-sdk
- Run, enter the contract key, and choose Alice, sign the certificate
- Read `get()`
- Call `flip()`
- Read `get()` again

## Appendex

### Polkadot.js Extension and the common seeds

Phala App only accepts the official [Polkadot.js Extension](https://polkadot.js.org/extension/) as the wallet provider. In the local testnet there are a few built-in well known accounts for testing. In order to access them from the Polkadot.js Extension, you should import them to the extension with their raw seed. It's suggested to import at least Alice and Bob:

| Key       | Raw seed                                                           |
|-----------|--------------------------------------------------------------------|
| //Alice   | 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a |
| //Bob     | 0x398f0c28f98885e046333d4a41c19cee4c37368a9832c6502f6cfd182e2aef89 |
| //Charlie | 0xbc1ede780f784bb6991a585e4f6e61522c14e1cae6ad0895fb57b9a205a8f938 |
| //Dave    | 0x868020ae0687dda7d57565093a69090211449845a7e11453612800b663307246 |
