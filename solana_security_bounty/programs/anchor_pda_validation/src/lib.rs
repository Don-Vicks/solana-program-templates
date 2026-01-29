use anchor_lang::prelude::*;

declare_id!("3vrTAijwUWFYo4STMGJRC3Xr7TjdJhqwCkRMnkVyG7at");

#[program]
pub mod anchor_pda_validation {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.vault = ctx.accounts.vault.key();
        pool.amount = 0;
        Ok(())
    }

    // HELPER FOR EXPLOIT: Allows creating a 'Pool' account on a random keypair (no seeds).
    // In a real attack, an attacker might find a way to create a valid state 
    // (e.g. if the program allows multiple pools, or if they can init a different PDA).
    // For this template, we explicitly allow "fake" pools to show what happens 
    // if 'deposit_insecure' accepts them.
    pub fn initialize_malicious(ctx: Context<InitializeMalicious>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.vault = ctx.accounts.vault.key();
        pool.amount = 666;
        Ok(())
    }

    // VULNERABLE: Does not validate that 'pool' is the correct PDA.
    // It only checks that 'pool' is of type 'Pool' (correct Owner and Discriminator).
    // Attacker can create a fake Pool account (on a different address) and pass it here.
    pub fn deposit_insecure(ctx: Context<DepositInsecure>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.amount += amount;
        msg!("Deposited {} to pool: {}", amount, pool.key());
        Ok(())
    }

    pub fn deposit_secure(ctx: Context<DepositSecure>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.amount += amount;
         msg!("Deposited {} to CORRECT pool: {}", amount, pool.key());
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 32 + 8, seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,
    /// CHECK: Simulation vault
    pub vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeMalicious<'info> {
    // Malicious initialization: No seeds constraint!
    // Creates a valid 'Pool' account but on a keypair chosen by attacker.
    #[account(init, payer = user, space = 8 + 32 + 8)]
    pub pool: Account<'info, Pool>,
    /// CHECK: Simulation
    pub vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositInsecure<'info> {
    #[account(mut)]
    // VULNERABLE: Missing seeds constraint. Accepts ANY initialized Pool account.
    pub pool: Account<'info, Pool>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct DepositSecure<'info> {
    #[account(
        mut,
        // SECURE: Validates that this account MUST be the one derived from [b"pool"].
        // This ensures nobody can swap it with a fake pool account.
        seeds = [b"pool"], 
        bump
    )]
    pub pool: Account<'info, Pool>,
    pub user: Signer<'info>,
}

#[account]
pub struct Pool {
    pub vault: Pubkey,
    pub amount: u64,
}
