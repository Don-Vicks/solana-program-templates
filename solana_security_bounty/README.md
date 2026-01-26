# Solana Security Templates (Anchor & Pinocchio)

This repository contains **5 Solana programs** demonstrating common security vulnerabilities and their fixes using the Anchor framework, plus a **Pinocchio** comparison to highlight manual security checks.

Built for the **Superteam Nigeria Intermediate Developer Challenge**.

## 🛡️ Vulnerability Templates

Each template allows you to run a test exploit against the "insecure" instruction and verifies the fix in the "secure" instruction.

### 1. Missing Signer Check (`anchor_signer_check`)

- **Vulnerability**: Creating an `update_admin` instruction that accepts an `admin` account but fails to verify it signed the transaction. Anyone can pass the admin's public key (address) to valid the key check, but without the private key signature.
- **Fix**: Use Anchor's `Signer<'info>` type instead of `UncheckedAccount` or `Account`. Anchor (and the runtime) guarantees that any account marked as `Signer` has signed the transaction.
- **Pinocchio Equivalent**: You must explicitly check `assert!(account.is_signer())`.

### 2. Arbitrary CPI (`anchor_arbitrary_cpi`)

- **Vulnerability**: Invoking a Cross-Program Invocation (CPI) to a program passed by the user without validating the program ID. An attacker can pass a malicious program (or System Program) in place of the Token Program.
- **Fix**: Use Anchor's `Program<'info, Token>` wrapper. It explicitly validates that `token_program.key() == token::ID`.

### 3. Type Cosplay / Discriminator Mismatch (`anchor_type_cosplay`)

- **Vulnerability**: Manually deserializing account data (e.g., using `try_from_slice`) without checking the 8-byte Anchor discriminator. This allows an attacker to pass an account of type `Admin` (valid data) where `User` was expected, potentially leading to privilege escalation if fields align.
- **Fix**: Always use `Account<'info, User>`. Anchor automatically checks the discriminator matches the `User` struct type.

### 4. PDA Validation (`anchor_pda_validation`)

- **Vulnerability**: Using `Account<'info, Pool>` without `seeds` constraints. This verifies the account is indeed a `Pool` owned by the program, but ANY valid Pool. An attacker can create a _fake_ Pool (initialized on a different address) and trick the program into using it.
- **Fix**: Use `#[account(seeds = [b"pool"], bump)]`. This forces the account to be the specific PDA derived from those seeds.

### 5. Re-initialization (`anchor_reinitialization`)

- **Vulnerability**: writing to an account (via `UncheckedAccount` or `mut`) with an "initialize" function that doesn't check if the account is already initialized. Calling it twice overwrites data.
- **Fix**: Use `#[account(init, ...)]`. Anchor ensures the account is strictly new (owned by System Program / uninitialized).

## 🧩 Pinocchio Comparison

A `pinocchio_comparison` program is included to demonstrate the "under the hood" work Anchor does. It implements **all 5 vulnerabilities** using:

- **Zero-Copy Serialization**: Manually parsing byte slices instead of Borsh.
- **Manual Validations**: Explicitly checking `account.is_signer()`, `program_id`, and derivations.
- **State Mutation**: Directly writing bytes to account data.

This serves as a high-performance reference implementation showing the verbosity required to achieve what Anchor does with a single line.

## 🚀 How to Run

**Prerequisites**: Solan CLI, Anchor CLI, Yarn.

1. **Install dependencies**:

   ```bash
   yarn install
   ```

2. **Build the programs**:

   ```bash
   anchor build
   ```

3. **Run the security tests**:

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
   _(Note: The frontend interactions currently mock the network calls until you successfully run `anchor build` to generate the IDLs)_.

---

_Note: If build fails due to network issues (Rust toolchain download), please ensure you have a stable internet connection and try `anchor build` again._
