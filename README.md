# Solana Security Templates - Real-World Exploits

Educational Anchor programs recreating 5 famous Solana exploits for security research and learning.

## Implemented Exploits

| Template                | Inspired By              | Amount Lost | Vulnerability                |
| ----------------------- | ------------------------ | ----------- | ---------------------------- |
| **Infinite Mint**       | Cashio (Mar 2022)        | $52M        | Collateral validation bypass |
| **Oracle Manipulation** | Mango Markets (Oct 2022) | $116M       | Price feed exploitation      |
| **Sysvar Spoofing**     | Wormhole (Feb 2022)      | $325M       | Sysvar account injection     |
| **Reentrancy CPI**      | Crema Finance (Jul 2022) | $9M         | State update ordering        |
| **Access Control**      | Raydium (Dec 2022)       | $4.4M       | Admin key compromise         |

## Quick Start

```bash
# Build all programs
anchor build

# Run tests
anchor test

# Deploy to localnet
solana-test-validator
anchor deploy
```

## Program Structure

Each program contains:

- **Vulnerable instruction** - Demonstrates the original exploit
- **Secure instruction** - Shows the proper fix
- **Detailed comments** - Explains the attack vector and mitigation

## Template Details

### 1. Infinite Mint (Cashio-Style)

The original Cashio hack allowed minting stablecoins against **any token** as collateral.

- `mint_insecure` - Accepts any SPL token as collateral
- `mint_secure` - Validates collateral mint against whitelist

### 2. Oracle Manipulation (Mango-Style)

Mango Markets trusted oracle prices without proper validation, enabling price manipulation attacks.

- `borrow_insecure` - Trusts oracle price blindly
- `borrow_secure` - Validates staleness, confidence interval, and circuit breaker

### 3. Sysvar Spoofing (Wormhole-Style)

Wormhole used a deprecated method to verify the Instructions sysvar, allowing attackers to inject fake accounts.

- `verify_and_mint_insecure` - Accepts any account as sysvar
- `verify_and_mint_secure` - Uses `address = ix_sysvar::ID` constraint

### 4. Reentrancy CPI (Crema-Style)

Crema Finance updated state **after** external CPI calls, allowing state manipulation.

- `swap_insecure` - Updates state after CPI (checks-effects-interactions violation)
- `swap_secure` - Updates state before CPI + reentrancy guard

### 5. Access Control (Raydium-Style)

Raydium had single-key admin functions with no timelock, enabling instant fund drainage.

- `withdraw_insecure` - Single admin key, immediate execution
- `withdraw_secure` - Timelock pattern (initiate → wait → execute)

## Requirements

- Anchor 0.32.1+
- Solana CLI 2.0+
- Node.js 18+

## License

MIT - Educational purposes only.
