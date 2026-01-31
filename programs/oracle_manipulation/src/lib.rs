use anchor_lang::prelude::*;

declare_id!("76AF6gaYLoTANVHFHtEiLAsxaXLYzhMRspQE1f8C4Pyd");

/// # Oracle Price Manipulation (Mango Markets-Style)
///
/// ## The Real Hack (October 2022 - $116M)
/// Mango Markets used on-chain oracles for collateral pricing.
/// The attacker:
/// 1. Deposited $5M USDC as initial collateral
/// 2. Opened a massive MNGO perpetual position
/// 3. Pumped MNGO spot price across exchanges (2,394% increase)
/// 4. Their collateral value skyrocketed based on oracle price
/// 5. Borrowed $116M against the inflated collateral
/// 6. Withdrew everything before price crashed
///
/// ## This Demo
/// - `borrow_insecure`: Trusts oracle price blindly (VULNERABLE)
/// - `borrow_secure`: Validates staleness, confidence, and rate limits (SECURE)
#[program]
pub mod oracle_manipulation {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let lending_pool = &mut ctx.accounts.lending_pool;
        lending_pool.authority = ctx.accounts.authority.key();
        lending_pool.oracle = ctx.accounts.oracle.key();
        lending_pool.total_deposits = 0;
        lending_pool.total_borrows = 0;
        lending_pool.last_known_price = 0;
        lending_pool.bump = ctx.bumps.lending_pool;
        Ok(())
    }

    /// Simulate a deposit for testing
    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        ctx.accounts.lending_pool.total_deposits += amount;
        ctx.accounts.user_position.deposited += amount;
        msg!("Deposited {} tokens", amount);
        Ok(())
    }

    /// Update oracle price (simulated - in real world this is Pyth/Switchboard)
    pub fn update_oracle(ctx: Context<UpdateOracle>, price: u64, confidence: u64) -> Result<()> {
        let oracle = &mut ctx.accounts.oracle;
        oracle.price = price;
        oracle.confidence = confidence;
        oracle.last_update_slot = Clock::get()?.slot;
        msg!("Oracle updated: price={}, confidence={}", price, confidence);
        Ok(())
    }

    /// VULNERABLE: Borrows against collateral using raw oracle price
    ///
    /// Attack vector:
    /// 1. Attacker manipulates low-liquidity token price on exchanges
    /// 2. Oracle reflects manipulated price
    /// 3. Attacker's collateral value is artificially inflated
    /// 4. Attacker borrows maximum against fake collateral value
    /// 5. Price crashes, protocol left with bad debt
    pub fn borrow_insecure(ctx: Context<BorrowInsecure>, amount: u64) -> Result<()> {
        let oracle = &ctx.accounts.oracle;
        let user_position = &ctx.accounts.user_position;

        // BUG: Blindly trust oracle price without ANY validation
        let collateral_value = user_position.deposited * oracle.price;
        
        // Allow 80% LTV
        let max_borrow = collateral_value * 80 / 100;
        
        require!(
            amount <= max_borrow,
            ErrorCode::InsufficientCollateral
        );

        ctx.accounts.user_position.borrowed += amount;
        ctx.accounts.lending_pool.total_borrows += amount;

        msg!(
            "INSECURE: Borrowed {} (collateral value: {}, price: {})",
            amount, collateral_value, oracle.price
        );
        Ok(())
    }

    /// SECURE: Validates oracle data before using it
    ///
    /// Fixes:
    /// 1. Check price staleness (last_update_slot)
    /// 2. Check confidence interval (reject wide spreads)
    /// 3. Check for extreme price movements (circuit breaker)
    pub fn borrow_secure(ctx: Context<BorrowSecure>, amount: u64) -> Result<()> {
        let oracle = &ctx.accounts.oracle;
        let lending_pool = &ctx.accounts.lending_pool;
        let user_position = &ctx.accounts.user_position;
        let current_slot = Clock::get()?.slot;

        // FIX 1: Reject stale prices (older than 10 slots ≈ 4 seconds)
        const MAX_STALENESS: u64 = 10;
        require!(
            current_slot.saturating_sub(oracle.last_update_slot) <= MAX_STALENESS,
            ErrorCode::StaleOracle
        );

        // FIX 2: Reject prices with low confidence (high uncertainty)
        // Confidence should be < 5% of price
        const MAX_CONFIDENCE_RATIO: u64 = 5;
        require!(
            oracle.confidence * 100 / oracle.price <= MAX_CONFIDENCE_RATIO,
            ErrorCode::LowConfidence
        );

        // FIX 3: Circuit breaker - reject extreme price movements (>50% change)
        if lending_pool.last_known_price > 0 {
            let price_change = if oracle.price > lending_pool.last_known_price {
                (oracle.price - lending_pool.last_known_price) * 100 / lending_pool.last_known_price
            } else {
                (lending_pool.last_known_price - oracle.price) * 100 / lending_pool.last_known_price
            };
            
            require!(
                price_change <= 50,
                ErrorCode::ExtremePrice
            );
        }

        let collateral_value = user_position.deposited * oracle.price;
        let max_borrow = collateral_value * 80 / 100;
        
        require!(
            amount <= max_borrow,
            ErrorCode::InsufficientCollateral
        );

        ctx.accounts.user_position.borrowed += amount;
        ctx.accounts.lending_pool.total_borrows += amount;
        ctx.accounts.lending_pool.last_known_price = oracle.price;

        msg!(
            "SECURE: Borrowed {} (validated price: {}, staleness: {} slots)",
            amount, oracle.price, current_slot.saturating_sub(oracle.last_update_slot)
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + LendingPool::INIT_SPACE,
        seeds = [b"lending_pool"],
        bump
    )]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(
        init,
        payer = authority,
        space = 8 + Oracle::INIT_SPACE,
        seeds = [b"oracle"],
        bump
    )]
    pub oracle: Account<'info, Oracle>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    #[account(mut, seeds = [b"lending_pool"], bump = lending_pool.bump)]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserPosition::INIT_SPACE,
        seeds = [b"position", user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    #[account(mut, seeds = [b"oracle"], bump)]
    pub oracle: Account<'info, Oracle>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct BorrowInsecure<'info> {
    #[account(mut, seeds = [b"lending_pool"], bump = lending_pool.bump)]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(seeds = [b"oracle"], bump)]
    pub oracle: Account<'info, Oracle>,

    #[account(
        mut,
        seeds = [b"position", user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct BorrowSecure<'info> {
    #[account(mut, seeds = [b"lending_pool"], bump = lending_pool.bump)]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(seeds = [b"oracle"], bump)]
    pub oracle: Account<'info, Oracle>,

    #[account(
        mut,
        seeds = [b"position", user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,

    pub user: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct LendingPool {
    pub authority: Pubkey,
    pub oracle: Pubkey,
    pub total_deposits: u64,
    pub total_borrows: u64,
    pub last_known_price: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Oracle {
    pub price: u64,
    pub confidence: u64,
    pub last_update_slot: u64,
}

#[account]
#[derive(InitSpace)]
pub struct UserPosition {
    pub deposited: u64,
    pub borrowed: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient collateral for borrow")]
    InsufficientCollateral,
    #[msg("Oracle price is stale (too old)")]
    StaleOracle,
    #[msg("Oracle confidence interval too wide")]
    LowConfidence,
    #[msg("Extreme price movement detected - circuit breaker triggered")]
    ExtremePrice,
}
