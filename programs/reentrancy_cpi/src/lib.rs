use anchor_lang::prelude::*;

declare_id!("Adrnw2U3aWSws16yAt9nrCsGaRXaDDbKvVzhhahSyfTe");

/// # Reentrancy via CPI (Crema Finance-Style)
///
/// ## The Real Hack (July 2022 - $9M)
/// Crema Finance was a concentrated liquidity DEX on Solana.
/// The attacker exploited a flash loan + state ordering issue:
/// 1. Flash loaned tokens from the pool
/// 2. Before the pool's state was updated, made a CPI call
/// 3. The CPI callback re-entered the vulnerable function
/// 4. State was read before it was updated, allowing double-spending
///
/// ## Solana "Reentrancy"
/// Unlike EVM, Solana doesn't have direct reentrancy via callbacks.
/// BUT: Programs can be exploited when:
/// - State is updated AFTER a CPI call instead of BEFORE
/// - External programs can be called mid-execution
/// - Attacker controls the called program
///
/// ## This Demo
/// - `swap_insecure`: Updates state AFTER CPI (VULNERABLE)
/// - `swap_secure`: Updates state BEFORE CPI + lock flag (SECURE)
#[program]
pub mod reentrancy_cpi {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, initial_reserve: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.reserve_a = initial_reserve;
        pool.reserve_b = initial_reserve;
        pool.is_locked = false;
        pool.bump = ctx.bumps.pool;
        Ok(())
    }

    /// VULNERABLE: Updates pool state AFTER external CPI
    ///
    /// Attack vector (Checks-Effects-Interactions violation):
    /// 1. Attacker initiates swap
    /// 2. External CPI is made (e.g., to attacker's program)
    /// 3. Attacker's program re-calls swap before state updates
    /// 4. Pool reads stale state, attacker gets tokens twice
    pub fn swap_insecure(
        ctx: Context<SwapInsecure>,
        amount_in: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        
        // Calculate output based on current reserves
        let amount_out = calculate_output(amount_in, pool.reserve_a, pool.reserve_b);
        
        msg!(
            "INSECURE: Swapping {} for {} (reserves: {}/{})",
            amount_in, amount_out, pool.reserve_a, pool.reserve_b
        );

        // BUG: We make the "external call" BEFORE updating state!
        // In a real exploit, this would call an attacker-controlled program
        // that re-enters this function with stale state
        simulate_external_cpi(&ctx.accounts.callback_program)?;

        // State is updated AFTER the CPI - TOO LATE!
        // An attacker could have re-entered and used old reserve values
        let pool = &mut ctx.accounts.pool;
        pool.reserve_a += amount_in;
        pool.reserve_b -= amount_out;

        Ok(())
    }

    /// SECURE: Updates state BEFORE CPI + uses reentrancy guard
    ///
    /// Fixes:
    /// 1. Check-Effects-Interactions: Update state BEFORE any external calls
    /// 2. Reentrancy lock: Prevent any re-entry while locked
    pub fn swap_secure(
        ctx: Context<SwapSecure>,
        amount_in: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        // FIX 1: Reentrancy guard - fail if already in a swap
        require!(!pool.is_locked, ErrorCode::ReentrancyDetected);
        pool.is_locked = true;

        // FIX 2: Update state BEFORE any external interaction
        let amount_out = calculate_output(amount_in, pool.reserve_a, pool.reserve_b);
        pool.reserve_a += amount_in;
        pool.reserve_b -= amount_out;

        msg!(
            "SECURE: Swapping {} for {} (new reserves: {}/{})",
            amount_in, amount_out, pool.reserve_a, pool.reserve_b
        );

        // Now it's safe to make external calls - state is already updated
        simulate_external_cpi(&ctx.accounts.callback_program)?;

        // Release the lock
        let pool = &mut ctx.accounts.pool;
        pool.is_locked = false;

        Ok(())
    }
}

/// Simulates making an external CPI call
/// In a real attack, this would invoke an attacker-controlled program
fn simulate_external_cpi(_program: &AccountInfo) -> Result<()> {
    msg!("Simulating external CPI call...");
    // In reality: invoke(&instruction, &accounts)?;
    Ok(())
}

/// Simple constant product AMM formula: x * y = k
fn calculate_output(amount_in: u64, reserve_in: u64, reserve_out: u64) -> u64 {
    // (amount_in * reserve_out) / (reserve_in + amount_in)
    let numerator = (amount_in as u128) * (reserve_out as u128);
    let denominator = (reserve_in as u128) + (amount_in as u128);
    (numerator / denominator) as u64
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Pool::INIT_SPACE,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SwapInsecure<'info> {
    #[account(
        mut,
        seeds = [b"pool"],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,

    /// The external program to callback (could be attacker-controlled)
    /// CHECK: Intentionally unchecked for vulnerability demonstration
    pub callback_program: AccountInfo<'info>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct SwapSecure<'info> {
    #[account(
        mut,
        seeds = [b"pool"],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: External program, but we're protected by reentrancy guard
    pub callback_program: AccountInfo<'info>,

    pub user: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub is_locked: bool,  // Reentrancy guard
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Reentrancy detected - swap already in progress")]
    ReentrancyDetected,
}
