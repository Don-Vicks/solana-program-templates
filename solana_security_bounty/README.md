# Solana Security Templates (Anchor & Pinocchio)

This repository contains **5 Solana programs** demonstrating common security vulnerabilities and their fixes using the Anchor framework, plus a **Pinocchio** comparison to highlight manual security checks.

Built for the **Superteam Nigeria Intermediate Developer Challenge**.

## 🛡️ Vulnerability Deep-Dive

Below is a detailed explanation of each vulnerability implemented in this repository, its implications, and how to fix it.

### 1. Missing Signer Check (`anchor_signer_check`)

**The Vulnerability:**
In Solana, simply passing an account's public key to a program does **not** prove ownership or authorization. A program can read any account's data. A "Missing Signer Check" occurs when an instruction performs a privileged action (like changing an admin) based solely on the presence of an account, without verifying that the account holder signed the transaction.

**The Implication:**
If an instruction `update_admin(new_admin)` takes an `current_admin` account but doesn't check if it's a signer, **attacker** can call this instruction passing the _real_ admin's public key (which is public knowledge). The program sees the correct address and proceeds, allowing the hacker to seize control of the protocol without the real admin's private key.

**Secure Pattern:**
Use Anchor's `Signer<'info>` type wrapper.

```rust
// INSECURE
pub admin: UncheckedAccount<'info>, // No check that admin signed!

// SECURE
pub admin: Signer<'info>, // Anchor enforces account.is_signer == true
```

---

### 2. Arbitrary CPI (`anchor_arbitrary_cpi`)

**The Vulnerability:**
Cross-Program Invocations (CPIs) allow programs to call other programs (e.g., calling the Token Program to transfer tokens). An "Arbitrary CPI" vulnerability exists when a program invokes an instruction on a program ID passed by the user without verifying it.

**The Implication:**
An attacker can pass a malicious program (or essentially any other program like the System Program) in place of the expected SPL Token Program. If your program calls `token::transfer` on this malicious program, the malicious program can succeed (do nothing) or behave unexpectedly. More critically, if you are relying on the _result_ of that CPI (e.g., "I burned 10 tokens, so now give me 10 SOL"), using a fake program allows the attacker to bypass the cost (burning nothing) and steal the reward.

**Secure Pattern:**
Use Anchor's `Program<'info, Token>` wrapper.

```rust
// INSECURE
pub token_program: UncheckedAccount<'info>, // Could be any program!

// SECURE
pub token_program: Program<'info, Token>, // Checks key == standard Token Program ID
```

---

### 3. Type Cosplay / Discriminator Mismatch (`anchor_type_cosplay`)

**The Vulnerability:**
Solana accounts are just byte arrays. Anchor solves this by adding an 8-byte "discriminator" (hash of the struct name) to the start of the account data. "Type Cosplay" happens when a program deserializes account data entirely manually (e.g., trying to read raw bytes as a specific struct) without checking this discriminator.

**The Implication:**
If you have a `User` struct and an `Admin` struct that happen to have similar byte layouts (e.g., both start with a `u64` balance), an attacker can create a `User` account and pass it to an instruction expecting an `Admin`. If the program only reads the bytes without checking _what_ type of account it is, it might interpret the `User`'s data as `Admin` data. This allows an attacker to "cosplay" as an admin using a regular user account.

**Secure Pattern:**
Always use standard Anchor accounts, which check the discriminator automatically.

```rust
// INSECURE
// Manually parsing bytes without checking discriminator (unsafe)

// SECURE
pub user: Account<'info, User>, // Anchor verifies the 8-byte discriminator matches "User"
```

---

### 4. PDA Validation (`anchor_pda_validation`)

**The Vulnerability:**
Program Derived Addresses (PDAs) are essential for deterministic account ownership (e.g., a "Pool" belonging to specific "Mint"). Vulnerability arises when an account accepts a generic account (like `Account<'info, Pool>`) but does not constrain _which_ Pool it is via seeds.

**The Implication:**
Without seed validation, the program checks "Is this account owned by me?" and "Is it a Pool?". Both are true for _any_ Pool created by the program. An attacker can create their _own_ Pool (where they are the admin) and pass it to a global function. The program thinks it's interacting with the official protocol Pool, allowing the attacker to drain funds or corrupt state using their fake Pool.

**Secure Pattern:**
Enforce PDA derivation using `seeds`.

```rust
// INSECURE
#[account]
pub pool: Account<'info, Pool>, // Any pool works

// SECURE
#[account(
    seeds = [b"pool"], // Must be THE pool derived from these exact seeds
    bump
)]
pub pool: Account<'info, Pool>,
```

---

### 5. Re-initialization (`anchor_reinitialization`)

**The Vulnerability:**
On Solana, accounts are permanent until closed. A "Re-initialization" attack occurs when an instruction meant to `initialize` an account (set initial state) can be called on an account that has _already_ been initialized.

**The Implication:**
If an attacker can call `initialize` again on an active account, they can reset its data. For example, they could reset a "Token Vault" balance to 0, or overwrite the "Owner" field of a multisig wallet to their own public key. This effectively allows complete takeover or destruction of the account's state.

**Secure Pattern:**
Use the `init` constraint, which fails if the account strictly already exists/has a discriminator.

```rust
// INSECURE
#[account(mut)]
pub user: Account<'info, User>, // Can calculate fields and overwrite existing data

// SECURE
#[account(init, payer = authority, space = ...)]
pub user: Account<'info, User>, // Fails if account already has a defined discriminator
```

---

## 🧩 Pinocchio Comparison

A `pinocchio_comparison` program is included to demonstrate the "under the hood" work Anchor does. It implements **all 5 vulnerabilities** using:

- **Zero-Copy Serialization**: Manually parsing byte slices instead of Borsh.
- **Manual Validations**: Explicitly checking `account.is_signer()`, `program_id`, and derivations.
- **State Mutation**: Directly writing bytes to account data.

This serves as a high-performance reference implementation showing the verbosity required to achieve what Anchor does with a single line.

## 🚀 How to Run

**Prerequisites**: Solana CLI, Anchor CLI (v0.32.1 recommended), Yarn.

1. **Install dependencies**:

   ```bash
   yarn install
   ```

2. **Build the programs**:

   ```bash
   anchor build
   ```

   _Note: This builds the 5 Anchor templates. The Pinocchio comparison program is isolated to avoid dependency conflicts._

3. **Build Pinocchio Comparison (Optional)**:

   ```bash
   cd reference_programs/pinocchio_comparison
   cargo build-sbf
   cd ../..
   ```

4. **Run the security tests**:

   ```bash
   anchor test
   ```

   The tests are designed to:

   - **Pass** explicitly when exploiting the vulnerable instruction (proving the bug).
   - **Fail** (via expected error) when attacking the secure instruction (proving the fix).

### 🖥️ Frontend Dashboard

A Next.js frontend is included to demonstrate the vulnerabilities interactively.

1. **Navigate to app directory**:

   ```bash
   cd app
   ```

2. **Run the development server**:

   ```bash
   yarn dev
   ```

3. **Open browser**:
   Visit `http://localhost:3000`. You can connect your wallet (configured for Localnet/Devnet) and toggle between "Vulnerable" and "Secure" modes for each template.
   _Note: Ensure `solana-test-validator` is running and `anchor deploy` has been executed._

---

_Note: If build fails due to network issues (Rust toolchain download), please ensure you have a stable internet connection and try `anchor build` again._
