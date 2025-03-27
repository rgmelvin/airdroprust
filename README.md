# Turbin3 Prerequisite - Rust Scripts

This repository contains a Rust-based Solana DevNet workflow for the **Turbin3** prerequisite tasks, including:
1. Generating and storing a new Solana keypair.
2. Airdropping DevNet SOL.
3. Transferring SOL.
4. Emptying a dev wallet.
5. Converting private keys between Base58 and byte array formats.
6. Enrolling in the Turbin3 on-chain program via a custom IDL.

It uses [`solana-idlgen`](#https://github.com/deanmlittle/solana-idlgen) to parse an **Anchor-style** IDL, calling the "complete" instruction to finalize enrollment.

---

## Table of Contents

- [Project Layout](#project-layout)

- [Prerequisites](#prerequisites)

- [Build & Test](#build--test)

- [Scripts Overview](#scripts-overview)

- [IDL Troubleshooting](#idl-troubleshooting)

- [Keeping Keys Safe](#keeping-keys-safe)

---

## Project Layout
```lua
airdroprust/
|- Cargo.toml
|- .gitignore
|- README.md  <-- This file
|_ src/
    |- lib.rs
    |_ programs/
       |- mod.rs
       |_Turbin3_prereq.rs
```

- `Cargo.toml` - Lists Rust dependencies, version, etc.
- `src/lib.rs` - Contains the `[test]` functions for each step (keygen, airdrop, etc.).
- `src/programs/Turbin3_prereq.rs` - Contains the IDL (`idlgen!({ ... })) for the Turbin3 program and derived Rust code.

You also have:
- `dev-wallet.json` - A local dev wallet for testing (ignored by Git).
- `Turbin3-wallet.json` - The wallet used for final enrollment (also ignored).

---

## Prerequisites

1. **Rust**(1.65+ recommended).
2. **Solana CLI** (optional, but helpful for checks).
3. A GitHub account for storing your code.
4. Familiarity with basic Rust commands (`cargo build`, `cargo test`).

Ensure that you have Rust installed:
```bash
rustc --version
# If not installed, see https://www.rust-lang.org/tools/install
```

---

## Build & Test

1. **Clone** this repository and enter the directory:
```bash
git clone https://github.com/rgmelvin/airdroprust.git
cd airdroprust
```
2. **Install** dependencies (as per `Cargo.toml`) and build:
```bash
cargo build
```
3. **Test** each [test] function individually, e.g.:
```bash
cargo test keygen -- --nocapture
cargo test base58_to_wallet -- --nocapture
cargo test wallet_to_base58 -- --nocapture
cargo test airdrop -- --nocapture
carfo test transfer_sol -- --nocapture
cargo test empty_wallet -- --nocapture
cargo test enroll -- --nocapture
```
- the `-- --nocapture` flag shows `println!` output in your terminal.

---

## Scripts Overview

## 1. Key Generation(`keygen`)

- **Purpose**: Generate a new Solana keypair.
- **Usage**:
```bash
cargo test keygen -- --nocapture
```
- **Output**:
    - A **public key** (base58)
    - A **64-byte array** for the private key
        Copy the array into `dev-wallet.json` for future use.

## 2. Base58 Conversions (`base58_to_wallet` / `wallet_to_base58)

- **Purpose**: Convert **base58**-encoded private keys to **byte array** format, or vice vesa.
- **Usage**:
```bash
cargo test base58_to_wallet -- --nocapture
cargo test wallet_to_base58 -- --nocapture
```
- **Prompt**: You paste your private key string or array at runtime.

## 3. Airdrop Dev Wallet (`airdrop`)

- **Purpose**: Request **2 SOL** worth of lamports from the Solana DevNet faucet into the dev wallet.
- **Usage**:
```bash
cargo test airdrop -- --nocapture
```
- **Requires**: `dev-wallet.json` with your key.

## 4. Transfer SOL (`transfer_sol`)

- **Purpose**: Send 0.1 SOL (or 0.001, depending on the code) form `dev-wallet.json` to your Turbin3 address on DevNet.
- **Usage**:
```bash
cargo test transfer_sol -- --nocapture
```
- **Output**: A transaction link on [Solana Explorer (DevNet)](#https://explorer.solana.com/?cluster=devnet).

## 5. Empty Dev Wallet (`empty_wallet`)

- **Purpose**: Send the **entire** dev wallet balance to a specified address (like your Turbin3 or any wallet).
- **Usage**:
```bash
cargo test empty_wallet -- --nocapture
```
- **Process**:
    - Fetch wallet balance.
    - Estimate transaction fee.
    - Transfer everything except the fee to free up the dev wallet.

## 6. Enroll (`enroll`)
- **Purpose**: Calls the **Turbin3** program's `complete` instruction to confirm your enrollment.
- **Usage**:
```bash
cargo test enroll -- --nocapture
```
- **Implementation**:
    - We import an IDL in `Turbin3_prereq.rs` via `solana-idlgen`.
    - It generates an Anchor-like client to derive the `prereq` PDA from seeds `["prereq", signer_pubkey].
    - We sign with out `Turbin3-wallet.json`.
    - Displays a **DevNet** transaction link.

---

## IDL Troubleshooting

One of the biggest learning points in this exercise is dealing with **Anchor-style IDLs** under `solana-idlgen`. Here are the **most common issues** encountered:
1. **Missing Fields** (`"missing field \ isMut"` or `"missingfield\ isSigner"`)
    - In **Anchor**-style IDLs, every account in the instructions **must** have `"isMut"` and `"isSigner"` (boolean).
    - For example:
    ```jsonc
    {
        "name": "signer",
        "isMut": true,
        "isSigner": true,
        "type": { "kind": "account" }
    }
    ```
2. `"type": { "kind": "account" }` **vs.** "type": { "kind": "program" }
    - Normal accounts (like your dev wallet or "prereq") use "kind": "account".
    - Program accounts (e.g., `system_program`) need `"kind": "program"`.
3. `"version"` **and** `"name"` **at the Top level**
    - Anchor expects a **top-level** `"version"` (e.g., `"0.1.0"`) and `"name"` (e.g., `"turbin3_prereq"`).
    - Not ust under `"metadata"`.
4. **Inline vs. Defined Structs**
    - If you have a custom account struct (`Solanacohort5Account`) at the top level, you can do `"type": { "kind": "struct","fields":[...] }"` **inline** or reference it via `"type": { "defined":"SolanaCohort5Account"}` and define it under `"types"`.
    - Make sure you only do one or the other, not both.
5. **Detailed JSON***
- Watch out for trailing commas, missing commas, or keys spelled incorrectly.

I recommend tackling these issues step by step if you run into them (better for learning); each error message typically points to a specific missing field. By manually updating `pda`, `isMut`, `isSigner`, or `type` fields in the IDL, you can learn how Anchor-like IDLs are structured. Once you get a successful parse, the rest flows very smoothly.

---

## Keeping Keys Safe

1. `.gitignore` excludes any files named `*wallet.json` so you don't accidentaly commit your private keys.
2. **Don't** push real mainnet keys into GitHub.
3. consider rotaing to new dev keys if you ever accidentally commit them.

Happy coding!