use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

declare_id!("7V2wxk3ZiYucgihoJ7GqBA5rJZmSin1Qp8MV2yKXuUMf");

/// # Infinite Mint Vulnerability (Cashio-Style)
///
/// ## The Real Hack (March 2022 - $52M)
/// Cashio allowed users to mint CASH stablecoins by depositing collateral.
/// The vulnerability: the contract did NOT validate that the deposited token
/// was actually a whitelisted collateral asset.
///
/// An attacker created a worthless fake token and used it as "collateral"
/// to mint 2 billion CASH tokens, which they then swapped for real assets.
///
/// ## This Demo
/// - `mint_insecure`: Accepts ANY token as collateral (VULNERABLE)
/// - `mint_secure`: Validates collateral mint against whitelist (SECURE)
#[program]
pub mod infinite_mint {
    use super::*;

    /// Initialize the stablecoin protocol with approved collateral mints
    pub fn initialize(ctx: Context<Initialize>, approved_collateral: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.stablecoin_mint = ctx.accounts.stablecoin_mint.key();
        config.approved_collateral = approved_collateral;
        config.bump = ctx.bumps.config;
        Ok(())
    }

    /// VULNERABLE: Mints stablecoins without validating collateral type
    ///
    /// Attack vector:
    /// 1. Attacker creates fake token with 0 value
    /// 2. Attacker deposits fake token as "collateral"
    /// 3. Protocol mints real stablecoins against worthless collateral
    pub fn mint_insecure(ctx: Context<MintInsecure>, amount: u64) -> Result<()> {
        // BUG: We check that collateral was deposited, but NOT what token it is!
        // Any SPL token is accepted as valid collateral.
        
        require!(
            ctx.accounts.user_collateral.amount >= amount,
            ErrorCode::InsufficientCollateral
        );

        // Mint stablecoins 1:1 against "collateral"
        let seeds = &[b"config".as_ref(), &[ctx.accounts.config.bump]];
        let signer_seeds = &[&seeds[..]];

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

        msg!("Minted {} stablecoins (INSECURE - no collateral validation)", amount);
        Ok(())
    }

    /// SECURE: Validates that collateral is from the approved mint
    ///
    /// Fix:
    /// - Explicitly check collateral token mint matches approved list
    /// - Reject any unknown collateral types
    pub fn mint_secure(ctx: Context<MintSecure>, amount: u64) -> Result<()> {
        // FIX: Validate the collateral mint matches our approved collateral
        require!(
            ctx.accounts.user_collateral.mint == ctx.accounts.config.approved_collateral,
            ErrorCode::InvalidCollateral
        );

        require!(
            ctx.accounts.user_collateral.amount >= amount,
            ErrorCode::InsufficientCollateral
        );

        let seeds = &[b"config".as_ref(), &[ctx.accounts.config.bump]];
        let signer_seeds = &[&seeds[..]];

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

        msg!("Minted {} stablecoins (SECURE - collateral validated)", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Config::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    /// The stablecoin this protocol issues
    #[account(mut)]
    pub stablecoin_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintInsecure<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    /// User's collateral deposit - BUG: We don't check what mint this is!
    #[account(
        constraint = user_collateral.owner == user.key() @ ErrorCode::InvalidOwner
    )]
    pub user_collateral: Account<'info, TokenAccount>,

    /// Stablecoin mint (authority = config PDA)
    #[account(
        mut,
        constraint = stablecoin_mint.key() == config.stablecoin_mint
    )]
    pub stablecoin_mint: Account<'info, Mint>,

    /// User's stablecoin receiving account
    #[account(mut)]
    pub user_stablecoin: Account<'info, TokenAccount>,

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintSecure<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    /// User's collateral deposit - FIX: Mint checked in instruction logic
    #[account(
        constraint = user_collateral.owner == user.key() @ ErrorCode::InvalidOwner
    )]
    pub user_collateral: Account<'info, TokenAccount>,

    /// Stablecoin mint
    #[account(
        mut,
        constraint = stablecoin_mint.key() == config.stablecoin_mint
    )]
    pub stablecoin_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_stablecoin: Account<'info, TokenAccount>,

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub stablecoin_mint: Pubkey,
    pub approved_collateral: Pubkey,  // The ONLY valid collateral mint
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Deposited collateral is not an approved asset")]
    InvalidCollateral,
    #[msg("Insufficient collateral deposited")]
    InsufficientCollateral,
    #[msg("Token account owner mismatch")]
    InvalidOwner,
}
