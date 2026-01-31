use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as ix_sysvar;

declare_id!("ocPDubsmZoeEE7LiiDiFJvp16ifeqNjNScLDHEkjB9p");

/// # Sysvar Spoofing Vulnerability (Wormhole-Style)
///
/// ## The Real Hack (February 2022 - $325M)
/// Wormhole's Solana contract needed to verify guardian signatures.
/// It used a deprecated function that loaded the Instructions sysvar
/// from an account passed by the user, NOT from the actual sysvar.
///
/// The attacker:
/// 1. Created a fake account with spoofed "signature verification" data
/// 2. Passed this fake account as if it were the Instructions sysvar
/// 3. The contract read the fake data and thought signatures were valid
/// 4. Minted 120,000 wETH without any real guardian approval
///
/// ## This Demo
/// - `verify_insecure`: Accepts any account as "sysvar" (VULNERABLE)
/// - `verify_secure`: Uses proper Anchor Sysvar wrapper (SECURE)
#[program]
pub mod sysvar_spoofing {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.authority = ctx.accounts.authority.key();
        bridge.guardian_set = ctx.accounts.authority.key(); // Simplified
        bridge.total_minted = 0;
        bridge.bump = ctx.bumps.bridge;
        Ok(())
    }

    /// VULNERABLE: Accepts any account as the "instructions sysvar"
    ///
    /// Attack vector:
    /// 1. Attacker creates account with fake "verified signature" data
    /// 2. Passes fake account where Instructions sysvar is expected
    /// 3. Program reads fake data, thinks signatures are valid
    /// 4. Attacker mints tokens without real authorization
    pub fn verify_and_mint_insecure(
        ctx: Context<VerifyInsecure>,
        amount: u64,
    ) -> Result<()> {
        // BUG: We accept ANY account as the instructions sysvar!
        // We don't verify it's actually the real sysvar.
        let instructions_account = &ctx.accounts.instructions;
        
        // Simulated "verification" - in real code this would parse the account
        // The attacker can make this return anything they want
        let is_verified = simulate_signature_check(instructions_account)?;
        
        require!(is_verified, ErrorCode::InvalidSignature);

        ctx.accounts.bridge.total_minted += amount;
        msg!(
            "INSECURE: Minted {} tokens (used unchecked instructions account)",
            amount
        );
        Ok(())
    }

    /// SECURE: Uses proper sysvar constraint to verify account address
    ///
    /// Fix:
    /// - Use `address = ix_sysvar::ID` constraint on the instructions account
    /// - This ensures the account IS the real Instructions sysvar
    /// - Alternatively, use Anchor's Sysvar wrapper types
    pub fn verify_and_mint_secure(
        ctx: Context<VerifySecure>,
        amount: u64,
    ) -> Result<()> {
        // The account is guaranteed to be the real Instructions sysvar
        // because of the `address = ix_sysvar::ID` constraint
        let instructions_account = &ctx.accounts.instructions;
        
        let is_verified = simulate_signature_check(instructions_account)?;
        
        require!(is_verified, ErrorCode::InvalidSignature);

        ctx.accounts.bridge.total_minted += amount;
        msg!(
            "SECURE: Minted {} tokens (verified sysvar address)",
            amount
        );
        Ok(())
    }
}

/// Simulates reading signature verification data from an account
/// In the real Wormhole, this would parse guardian signatures
fn simulate_signature_check(account: &AccountInfo) -> Result<bool> {
    // In real implementation, this would deserialize and verify
    // For demo: if account has data and first byte is 1, "verified"
    if account.data_len() > 0 {
        let data = account.try_borrow_data()?;
        Ok(data[0] == 1)
    } else {
        Ok(false)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Bridge::INIT_SPACE,
        seeds = [b"bridge"],
        bump
    )]
    pub bridge: Account<'info, Bridge>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyInsecure<'info> {
    #[account(
        mut,
        seeds = [b"bridge"],
        bump = bridge.bump
    )]
    pub bridge: Account<'info, Bridge>,

    /// BUG: No address check! Attacker can pass ANY account here.
    /// CHECK: This is intentionally unsafe for demonstration
    pub instructions: AccountInfo<'info>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifySecure<'info> {
    #[account(
        mut,
        seeds = [b"bridge"],
        bump = bridge.bump
    )]
    pub bridge: Account<'info, Bridge>,

    /// FIX: Constrain to the actual Instructions sysvar address
    /// CHECK: Address is verified via constraint below
    #[account(address = ix_sysvar::ID)]
    pub instructions: AccountInfo<'info>,

    pub user: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Bridge {
    pub authority: Pubkey,
    pub guardian_set: Pubkey,
    pub total_minted: u64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Guardian signature verification failed")]
    InvalidSignature,
}
