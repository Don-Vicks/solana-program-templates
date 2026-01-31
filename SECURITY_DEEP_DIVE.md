# Anatomy of $500M in Solana Exploits: A Deep-Dive Security Analysis

> **"Those who fail to learn from history are doomed to repeat it."** — George Santayana

The Solana ecosystem lost over $500 million in 2022 to preventable exploits. Not sophisticated zero-days. Not cryptographic breaks. Simple programming mistakes that frameworks like Anchor didn't catch because **they were never designed to**.

This repository recreates 5 of the most devastating Solana exploits—not as hypothetical examples, but as faithful reproductions of real bugs that cost real money. Each template contains the vulnerable code, the secure fix, and detailed explanations of what went wrong.

**The goal is simple: learn from real disasters, not documentation.**

---

## Table of Contents

1. [Introduction](#introduction)
2. [The Exploits at a Glance](#the-exploits-at-a-glance)
3. [Exploit 1: Infinite Mint (Cashio - $52M)](#exploit-1-infinite-mint-cashio---52m)
4. [Exploit 2: Oracle Manipulation (Mango Markets - $116M)](#exploit-2-oracle-manipulation-mango-markets---116m)
5. [Exploit 3: Sysvar Spoofing (Wormhole - $325M)](#exploit-3-sysvar-spoofing-wormhole---325m)
6. [Exploit 4: Reentrancy via CPI (Crema Finance - $9M)](#exploit-4-reentrancy-via-cpi-crema-finance---9m)
7. [Exploit 5: Access Control (Raydium - $4.4M)](#exploit-5-access-control-raydium---44m)
8. [Common Patterns and Lessons](#common-patterns-and-lessons)
9. [Security Checklist](#security-checklist)
10. [Conclusion](#conclusion)

---

## Introduction

2022 was a brutal year for Solana DeFi. Over $500 million was drained across major protocols—not through sophisticated zero-days or cryptographic breaks, but through **basic programming mistakes** that Anchor's abstractions didn't catch.

The common thread? **Developers assumed their frameworks would handle security automatically.**

This repository provides hands-on, educational recreations of these exploits. Each template contains:

- **Vulnerable code** that mirrors the original bug
- **Secure code** showing the correct implementation
- **Tests** that demonstrate both the exploit and the fix
- **Inline comments** explaining the security reasoning

The goal is simple: **learn from real disasters, not hypothetical examples.**

---

## The Exploits at a Glance

| Exploit             | Protocol      | Date          | Loss  | Root Cause                    |
| ------------------- | ------------- | ------------- | ----- | ----------------------------- |
| Infinite Mint       | Cashio        | March 2022    | $52M  | Missing collateral validation |
| Oracle Manipulation | Mango Markets | October 2022  | $116M | No oracle sanity checks       |
| Sysvar Spoofing     | Wormhole      | February 2022 | $325M | Unchecked sysvar address      |
| Reentrancy via CPI  | Crema Finance | July 2022     | $9M   | State update after CPI        |
| Access Control      | Raydium       | December 2022 | $4.4M | Single key, no timelock       |

**Total: $506.4M in preventable losses.**

---

## Exploit 1: Infinite Mint (Cashio - $52M)

### Background

Cashio was a stablecoin protocol on Solana that allowed users to mint CASH tokens by depositing approved collateral (like USDC). In March 2022, an attacker discovered they could mint unlimited CASH using **any SPL token**—including worthless tokens they created themselves.

### The Vulnerability

The minting function checked that a user had deposited _something_, but never verified **what** they deposited:

```rust
pub fn mint_insecure(ctx: Context<MintInsecure>, amount: u64) -> Result<()> {
    // BUG: We check that collateral exists, but NOT what token it is!
    // Any SPL token is accepted as valid collateral.

    require!(
        ctx.accounts.user_collateral.amount >= amount,
        ErrorCode::InsufficientCollateral
    );

    // Mint stablecoins 1:1 against "collateral"
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.stablecoin_mint.to_account_info(),
                to: ctx.accounts.user_stablecoin.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    Ok(())
}
```

### The Attack

1. Attacker creates a new SPL token with unlimited supply
2. Mints billions of worthless tokens to themselves
3. Deposits these worthless tokens as "collateral"
4. Protocol mints real CASH stablecoins 1:1
5. Attacker sells CASH on DEXs for real USDC
6. Repeat until protocol is drained

### The Fix

Always validate the token mint explicitly:

```rust
pub fn mint_secure(ctx: Context<MintSecure>, amount: u64) -> Result<()> {
    // FIX: Validate the collateral mint matches our approved token
    let collateral_mint = ctx.accounts.user_collateral.mint;
    require!(
        collateral_mint == ctx.accounts.config.approved_collateral,
        ErrorCode::InvalidCollateral
    );

    require!(
        ctx.accounts.user_collateral.amount >= amount,
        ErrorCode::InsufficientCollateral
    );

    // Now safe to mint
    token::mint_to(..., amount)?;

    msg!("Minted {} with validated collateral", amount);
    Ok(())
}
```

### Key Lesson

> **Never assume account contents are valid.** Always check:
>
> - Token account mints (`token_account.mint == expected_mint`)
> - Account owners (`account.owner == expected_program`)
> - PDA seeds and bumps

Anchor's `token::TokenAccount` gives you type safety, but it doesn't know _which_ token you expect.

---

## Exploit 2: Oracle Manipulation (Mango Markets - $116M)

### Background

Mango Markets was a lending protocol that allowed users to borrow against their deposited collateral. The value of collateral was determined by oracle price feeds. In October 2022, an attacker manipulated the MNGO token price to borrow far more than their collateral was worth.

### The Vulnerability

The borrowing function trusted the oracle price without any validation:

```rust
pub fn borrow_insecure(ctx: Context<BorrowInsecure>, amount: u64) -> Result<()> {
    let oracle = &ctx.accounts.oracle;
    let user_position = &ctx.accounts.user_position;

    // BUG: Blindly trust oracle price without ANY validation
    // - No staleness check (price could be hours old)
    // - No confidence interval check (price could be manipulated)
    // - No sanity check (1000x price spike? Sure!)
    let collateral_value = user_position.deposited * oracle.price;

    // Allow 80% LTV
    let max_borrow = collateral_value * 80 / 100;

    require!(amount <= max_borrow, ErrorCode::InsufficientCollateral);

    ctx.accounts.user_position.borrowed += amount;
    Ok(())
}
```

### The Attack

1. Attacker deposits $5M USDC as initial collateral
2. Opens a massive long perpetual position on MNGO
3. Buys MNGO across multiple exchanges, pumping spot price 2,394%
4. Oracle reflects the pumped price
5. Attacker's $5M collateral now "worth" $120M+
6. Borrows $116M against the inflated collateral
7. Withdraws borrowed funds before price crashes
8. Doesn't repay the loans (collateral now worthless)

### The Fix

Implement multiple layers of oracle validation:

```rust
pub fn borrow_secure(ctx: Context<BorrowSecure>, amount: u64) -> Result<()> {
    let oracle = &ctx.accounts.oracle;
    let current_slot = Clock::get()?.slot;

    // FIX 1: Reject stale prices
    // Oracle data older than 10 slots (~4 seconds) is dangerous
    require!(
        current_slot.saturating_sub(oracle.last_update_slot) <= 10,
        ErrorCode::StaleOracle
    );

    // FIX 2: Reject low confidence prices
    // Pyth oracles include confidence intervals - use them!
    // If confidence > 5% of price, something is wrong
    require!(
        oracle.confidence * 100 / oracle.price <= 5,
        ErrorCode::LowConfidence
    );

    // FIX 3: Circuit breaker for extreme price moves
    // If price moved >50% since last check, halt operations
    let price_change = calculate_percentage_change(oracle.price, last_known_price);
    require!(
        price_change <= 50,
        ErrorCode::ExtremePriceMovement
    );

    // Now safe to calculate collateral value
    let collateral_value = user_position.deposited * oracle.price;
    let max_borrow = collateral_value * 80 / 100;

    require!(amount <= max_borrow, ErrorCode::InsufficientCollateral);
    Ok(())
}
```

### Key Lesson

> **Oracles are external dependencies you don't control.** Always validate:
>
> - **Staleness**: Is this price recent?
> - **Confidence**: How certain is the oracle?
> - **Sanity**: Does this price make economic sense?
> - **Source**: Is this the real oracle account?

Consider using TWAPs (time-weighted average prices) for lending operations to smooth out manipulation attempts.

---

## Exploit 3: Sysvar Spoofing (Wormhole - $325M)

### Background

Wormhole is a cross-chain bridge that allows assets to move between Solana and other chains. It uses guardian signatures to verify cross-chain messages. In February 2022, an attacker forged these signatures to mint 120,000 ETH ($325M) on Ethereum.

### The Vulnerability

The signature verification function accepted any account as the Instructions sysvar without validating its address:

```rust
pub fn verify_and_mint_insecure(
    ctx: Context<VerifyInsecure>,
    amount: u64,
) -> Result<()> {
    // BUG: We accept ANY account here and assume it's the Instructions sysvar
    // Attacker can pass a custom account with fake "verified" data
    let instructions_account = &ctx.accounts.instructions;

    // This reads from whatever account was passed
    // If it's attacker-controlled, they control the "verification" result
    let is_verified = verify_guardian_signatures(instructions_account)?;

    require!(is_verified, ErrorCode::InvalidSignature);

    // Mint bridged tokens
    ctx.accounts.bridge.total_minted += amount;
    msg!("Minted {} bridged tokens", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct VerifyInsecure<'info> {
    #[account(mut)]
    pub bridge: Account<'info, Bridge>,

    // BUG: No address constraint! This could be ANY account.
    /// CHECK: No validation - DANGEROUS!
    pub instructions: UncheckedAccount<'info>,

    pub user: Signer<'info>,
}
```

### The Attack

1. Attacker creates a new Solana account
2. Writes crafted data that makes signature verification return `true`
3. Calls `verify_and_mint` with their fake account as "instructions"
4. Verification passes (reading attacker's fake data)
5. 120,000 wETH minted on Solana
6. Bridge these to Ethereum and cash out

### The Fix

Always constrain sysvar addresses explicitly:

```rust
pub fn verify_and_mint_secure(
    ctx: Context<VerifySecure>,
    amount: u64,
) -> Result<()> {
    // FIX: Address is constrained in accounts struct
    // Only the real Instructions sysvar can be passed
    let is_verified = verify_guardian_signatures(&ctx.accounts.instructions)?;

    require!(is_verified, ErrorCode::InvalidSignature);
    ctx.accounts.bridge.total_minted += amount;
    Ok(())
}

#[derive(Accounts)]
pub struct VerifySecure<'info> {
    #[account(mut)]
    pub bridge: Account<'info, Bridge>,

    // FIX: Constrain to the actual Instructions sysvar address
    #[account(address = sysvar::instructions::ID)]
    /// CHECK: Address is verified to be the real Instructions sysvar
    pub instructions: UncheckedAccount<'info>,

    pub user: Signer<'info>,
}
```

Even better, use Anchor's typed sysvar wrapper:

```rust
pub instructions: Sysvar<'info, Instructions>,
```

### Key Lesson

> **Sysvars are accounts like any other—their addresses must be verified.**
>
> When using `UncheckedAccount`, you MUST add `address = <known_address>` or perform manual validation. The `/// CHECK:` comment is not a security measure—it's documentation of what you verified.

---

## Exploit 4: Reentrancy via CPI (Crema Finance - $9M)

### Background

Crema Finance was an automated market maker (AMM) on Solana. In July 2022, an attacker exploited a reentrancy vulnerability where the pool state was updated **after** making external CPI calls, allowing them to drain liquidity by re-entering with stale state.

### The Vulnerability

The swap function made external calls before updating its own state:

```rust
pub fn swap_insecure(ctx: Context<SwapInsecure>, amount_in: u64) -> Result<()> {
    let pool = &ctx.accounts.pool;

    // Calculate output based on CURRENT reserves
    let amount_out = calculate_output(
        amount_in,
        pool.reserve_a,
        pool.reserve_b
    );

    // BUG: External call BEFORE updating state!
    // If the callback re-enters this function, it will still see
    // the OLD reserves and can extract more tokens
    invoke(
        &Instruction {
            program_id: ctx.accounts.callback_program.key(),
            accounts: vec![...],
            data: callback_data,
        },
        &[...],
    )?;

    // State updated AFTER CPI - TOO LATE!
    // By now, attacker may have already re-entered multiple times
    let pool = &mut ctx.accounts.pool;
    pool.reserve_a += amount_in;
    pool.reserve_b -= amount_out;

    Ok(())
}
```

### The Attack

1. Attacker deploys a malicious "callback" program
2. Calls swap on Crema, which triggers the callback
3. Callback immediately calls swap AGAIN
4. Second swap sees original reserves (not yet updated)
5. Calculates favorable rate based on stale state
6. Process repeats recursively
7. Each iteration extracts tokens based on wrong reserves

### The Fix

Follow the **Checks-Effects-Interactions** pattern and add a reentrancy guard:

```rust
pub fn swap_secure(ctx: Context<SwapSecure>, amount_in: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // FIX 1: Reentrancy guard
    // Immediately lock the pool to prevent re-entry
    require!(!pool.is_locked, ErrorCode::Reentrancy);
    pool.is_locked = true;

    // Calculate output
    let amount_out = calculate_output(amount_in, pool.reserve_a, pool.reserve_b);

    // FIX 2: Update state BEFORE any external calls
    // Even if re-entry occurs, state is already correct
    pool.reserve_a += amount_in;
    pool.reserve_b -= amount_out;

    // Now safe to make external call - state is already final
    invoke(
        &Instruction {
            program_id: ctx.accounts.callback_program.key(),
            accounts: vec![...],
            data: callback_data,
        },
        &[...],
    )?;

    // Release the lock
    pool.is_locked = false;

    Ok(())
}
```

### Key Lesson

> **Always follow Checks-Effects-Interactions:**
>
> 1. **Checks**: Validate all preconditions
> 2. **Effects**: Update your state
> 3. **Interactions**: Make external calls LAST
>
> Add reentrancy guards for any function that makes CPI calls.

---

## Exploit 5: Access Control (Raydium - $4.4M)

### Background

Raydium is a major Solana DEX. In December 2022, an attacker gained access to a pool owner's private key and immediately drained funds from multiple liquidity pools. There were no timelocks, multisig requirements, or other safeguards—a single compromised key meant instant loss.

### The Vulnerability

Admin functions allowed immediate, unrestricted execution:

```rust
pub fn withdraw_insecure(ctx: Context<WithdrawInsecure>, amount: u64) -> Result<()> {
    // Only check: is the signer the admin?
    // If the admin key is compromised, attacker wins INSTANTLY
    require!(
        ctx.accounts.signer.key() == ctx.accounts.vault.admin,
        ErrorCode::Unauthorized
    );

    require!(
        ctx.accounts.vault.balance >= amount,
        ErrorCode::InsufficientFunds
    );

    // BUG: Immediate execution with no delay
    // - No timelock (team can't react)
    // - No multisig (single point of failure)
    // - No limits (can drain everything at once)
    ctx.accounts.vault.balance -= amount;

    msg!("Withdrawn {} IMMEDIATELY - no delay!", amount);
    Ok(())
}
```

### The Attack

1. Attacker compromises owner's private key (phishing, malware, etc.)
2. Immediately calls admin withdraw function
3. Drains entire treasury in one transaction
4. Team discovers loss hours later—nothing they can do

### The Fix

Implement timelocked, two-phase admin operations:

```rust
pub fn initiate_withdrawal_secure(
    ctx: Context<InitiateWithdrawal>,
    amount: u64,
    recipient: Pubkey,
) -> Result<()> {
    // Verify admin
    require!(
        ctx.accounts.signer.key() == ctx.accounts.vault.admin,
        ErrorCode::Unauthorized
    );

    // FIX: Don't execute immediately - start a timelock
    const TIMELOCK_SLOTS: u64 = 100; // ~40 seconds minimum delay

    let vault = &mut ctx.accounts.vault;
    vault.pending_withdrawal = Some(PendingWithdrawal {
        amount,
        recipient,
        executable_after: Clock::get()?.slot + TIMELOCK_SLOTS,
        initiated_at: Clock::get()?.unix_timestamp,
    });

    msg!("Withdrawal initiated - executable after {} slots", TIMELOCK_SLOTS);
    Ok(())
}

pub fn cancel_withdrawal_secure(ctx: Context<CancelWithdrawal>) -> Result<()> {
    // FIX: Team can cancel if they detect compromise
    require!(
        ctx.accounts.signer.key() == ctx.accounts.vault.admin,
        ErrorCode::Unauthorized
    );

    ctx.accounts.vault.pending_withdrawal = None;
    msg!("Pending withdrawal CANCELLED");
    Ok(())
}

pub fn execute_withdrawal_secure(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let pending = vault.pending_withdrawal.ok_or(ErrorCode::NoPending)?;

    // FIX: Wait for timelock to expire
    require!(
        Clock::get()?.slot >= pending.executable_after,
        ErrorCode::TimelockNotExpired
    );

    vault.balance -= pending.amount;
    vault.pending_withdrawal = None;

    msg!("Withdrawal executed after timelock");
    Ok(())
}
```

### Key Lesson

> **Timelocks turn instant losses into recoverable situations.**
>
> Design your admin functions assuming the admin key WILL be compromised eventually:
>
> - **Timelock**: Give the team time to react
> - **Multisig**: Require multiple keys for critical operations
> - **Limits**: Cap single-transaction withdrawals
> - **Monitoring**: Alert on any admin action

---

## Common Patterns and Lessons

### 1. Trust Nothing from Outside

Every account, every oracle price, every CPI return value is external input. Validate everything:

- Token mints
- Account owners
- Program IDs
- Sysvar addresses
- Oracle data freshness

### 2. State Before Interactions

Update your program's state BEFORE making any external calls:

```rust
// ❌ WRONG
invoke(&external_instruction, &accounts)?;
state.balance -= amount;

// ✅ CORRECT
state.balance -= amount;
invoke(&external_instruction, &accounts)?;
```

### 3. Defense in Depth

A single check is never enough for critical operations:

```rust
// ❌ Insufficient
require!(signer == admin, Unauthorized);

// ✅ Better
require!(signer == admin, Unauthorized);
require!(amount <= daily_limit, ExceedsLimit);
require!(current_slot >= cooldown_until, Cooldown);
```

### 4. Fail Secure

When something unexpected happens, fail to the safest state:

```rust
// ❌ Dangerous
let price = oracle.price.unwrap_or(0); // Zero price = infinite collateral value!

// ✅ Safe
let price = oracle.price.ok_or(ErrorCode::OracleUnavailable)?;
```

---

## Security Checklist

Use this checklist for every instruction you write:

### Account Validation

- [ ] All token mints validated against expected values
- [ ] Account owners verified (is this the right program?)
- [ ] PDAs derived with correct seeds and bumps
- [ ] Sysvar addresses constrained

### Authority & Access

- [ ] Signer checks for privileged operations
- [ ] Timelocks for admin functions
- [ ] Consider multisig for treasury operations

### External Dependencies

- [ ] Oracle staleness checks
- [ ] Oracle confidence validation
- [ ] Circuit breakers for extreme values
- [ ] CPI return values validated

### State Management

- [ ] State updated BEFORE external calls
- [ ] Reentrancy guards on CPI functions
- [ ] Atomic operations where needed

### Arithmetic

- [ ] Overflow checks (use checked_add, checked_mul)
- [ ] Underflow checks (use checked_sub)
- [ ] Division by zero prevention
- [ ] Precision loss consideration

---

## Conclusion

These five exploits cost the Solana ecosystem over half a billion dollars. Every single one was preventable with basic security practices:

1. **Cashio**: Validate token mints
2. **Mango**: Don't trust oracles blindly
3. **Wormhole**: Constrain sysvar addresses
4. **Crema**: Update state before CPIs
5. **Raydium**: Use timelocks for admin functions

The patterns are simple. The consequences of ignoring them are catastrophic.

**Anchor and Pinocchio provide excellent developer ergonomics, but they don't think about security for you.** Every `require!`, every constraint, every validation check is your responsibility.

Use this repository to practice. Run the tests. Break the vulnerable code. Fix it yourself. **Learn from disasters, not documentation.**

---

## Running the Examples

```bash
# Clone the repository
git clone https://github.com/Don-Vicks/solana-program-templates.git
cd solana-program-templates

# Build all programs
anchor build

# Run tests (demonstrates exploits and fixes)
anchor test

# Start the frontend
cd frontend && npm install && npm run dev
```

---

## Resources

- [Anchor Documentation](https://www.anchor-lang.com/docs)
- [Solana Security Best Practices](https://github.com/coral-xyz/sealevel-attacks)
- [Pyth Oracle Documentation](https://docs.pyth.network/)
- [Solana Cookbook - Security](https://solanacookbook.com/references/security.html)

---

_Built for the SuperteamNG Security Bounty. Open source and free to use for educational purposes._
