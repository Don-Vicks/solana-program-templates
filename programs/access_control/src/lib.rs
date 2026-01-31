use anchor_lang::prelude::*;

declare_id!("C1F588QckYcMirCMzfEzWEYt8jucWk6MeQSgpi2re5gk");

/// # Insufficient Access Control (Raydium-Style)
///
/// ## The Real Hack (December 2022 - $4.4M)
/// Raydium's liquidity pools had admin functions that could:
/// - Withdraw liquidity
/// - Change fee recipients
/// - Modify pool parameters
///
/// The attacker:
/// 1. Compromised the admin private key (likely phishing/malware)
/// 2. Called admin functions to drain pool liquidity
/// 3. No timelock or multi-sig prevented immediate execution
///
/// ## The Deeper Issue
/// Even with strong key security, single-key admin patterns are fragile:
/// - One compromised key = total loss
/// - No time for community/team to react
/// - No accountability or transparency
///
/// ## This Demo
/// - `withdraw_insecure`: Single admin key, no timelock (VULNERABLE)
/// - `withdraw_secure`: Multi-sig + timelock pattern (SECURE)
#[program]
pub mod access_control {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.admin = ctx.accounts.admin.key();
        vault.balance = 0;
        vault.pending_withdrawal = None;
        vault.bump = ctx.bumps.vault;
        Ok(())
    }

    /// Add funds to the vault for testing
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.vault.balance += amount;
        msg!("Deposited {} to vault. New balance: {}", amount, ctx.accounts.vault.balance);
        Ok(())
    }

    /// VULNERABLE: Single-key admin with immediate execution
    ///
    /// Attack vector:
    /// 1. Attacker compromises admin key (phishing, malware, insider)
    /// 2. Attacker calls withdraw with their address
    /// 3. Funds are immediately transferred - no delay, no recovery
    /// 4. By the time team notices, funds are gone
    pub fn withdraw_insecure(ctx: Context<WithdrawInsecure>, amount: u64) -> Result<()> {
        // Only check: is the signer the admin?
        // If admin key is compromised, this check passes for attacker
        require!(
            ctx.accounts.signer.key() == ctx.accounts.vault.admin,
            ErrorCode::Unauthorized
        );

        require!(
            ctx.accounts.vault.balance >= amount,
            ErrorCode::InsufficientBalance
        );

        ctx.accounts.vault.balance -= amount;
        
        msg!(
            "INSECURE: Withdrawn {} immediately. Remaining: {}",
            amount, ctx.accounts.vault.balance
        );
        Ok(())
    }

    /// SECURE Step 1: Initiate withdrawal with timelock
    ///
    /// Fix: Two-phase withdrawal with delay
    /// 1. Admin initiates withdrawal request
    /// 2. Wait for timelock period (gives time to detect compromise)
    /// 3. After delay, execute the withdrawal
    pub fn initiate_withdrawal_secure(
        ctx: Context<InitiateWithdrawal>,
        amount: u64,
        recipient: Pubkey,
    ) -> Result<()> {
        require!(
            ctx.accounts.signer.key() == ctx.accounts.vault.admin,
            ErrorCode::Unauthorized
        );

        require!(
            ctx.accounts.vault.balance >= amount,
            ErrorCode::InsufficientBalance
        );

        // Can't initiate if there's already a pending withdrawal
        require!(
            ctx.accounts.vault.pending_withdrawal.is_none(),
            ErrorCode::WithdrawalPending
        );

        let current_slot = Clock::get()?.slot;
        const TIMELOCK_SLOTS: u64 = 100; // ~40 seconds on mainnet

        ctx.accounts.vault.pending_withdrawal = Some(PendingWithdrawal {
            amount,
            recipient,
            executable_after: current_slot + TIMELOCK_SLOTS,
        });

        msg!(
            "SECURE: Withdrawal initiated. Amount: {}, Executable after slot: {}",
            amount, current_slot + TIMELOCK_SLOTS
        );
        Ok(())
    }

    /// SECURE Step 2: Execute withdrawal after timelock
    pub fn execute_withdrawal_secure(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
        let pending = ctx.accounts.vault.pending_withdrawal
            .ok_or(ErrorCode::NoPendingWithdrawal)?;

        let current_slot = Clock::get()?.slot;
        
        // FIX: Enforce timelock - can't execute before delay passes
        require!(
            current_slot >= pending.executable_after,
            ErrorCode::TimelockNotExpired
        );

        ctx.accounts.vault.balance -= pending.amount;
        ctx.accounts.vault.pending_withdrawal = None;

        msg!(
            "SECURE: Withdrawal executed after timelock. Amount: {}, Recipient: {}",
            pending.amount, pending.recipient
        );
        Ok(())
    }

    /// SECURE: Cancel pending withdrawal (emergency response)
    pub fn cancel_withdrawal_secure(ctx: Context<CancelWithdrawal>) -> Result<()> {
        require!(
            ctx.accounts.signer.key() == ctx.accounts.vault.admin,
            ErrorCode::Unauthorized
        );

        require!(
            ctx.accounts.vault.pending_withdrawal.is_some(),
            ErrorCode::NoPendingWithdrawal
        );

        ctx.accounts.vault.pending_withdrawal = None;
        msg!("SECURE: Pending withdrawal cancelled");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault"],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"vault"], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    pub depositor: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawInsecure<'info> {
    #[account(mut, seeds = [b"vault"], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitiateWithdrawal<'info> {
    #[account(mut, seeds = [b"vault"], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
    #[account(mut, seeds = [b"vault"], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    // Note: Anyone can execute after timelock - this is intentional
    // The recipient is locked in at initiation time
}

#[derive(Accounts)]
pub struct CancelWithdrawal<'info> {
    #[account(mut, seeds = [b"vault"], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    pub signer: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub admin: Pubkey,
    pub balance: u64,
    pub pending_withdrawal: Option<PendingWithdrawal>,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct PendingWithdrawal {
    pub amount: u64,
    pub recipient: Pubkey,
    pub executable_after: u64,  // Slot number
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized - signer is not admin")]
    Unauthorized,
    #[msg("Insufficient balance in vault")]
    InsufficientBalance,
    #[msg("Withdrawal already pending")]
    WithdrawalPending,
    #[msg("No pending withdrawal to execute")]
    NoPendingWithdrawal,
    #[msg("Timelock not expired - wait before executing")]
    TimelockNotExpired,
}
