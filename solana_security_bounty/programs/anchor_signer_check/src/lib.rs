use anchor_lang::prelude::*;

declare_id!("wxvuHwcvLZGSAif8yoESUvmWnrWQDMGxAagrHRrXWFF");

#[program]
pub mod anchor_signer_check {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        Ok(())
    }

    // VULNERABLE: Takes 'admin' as UncheckedAccount (or just Account) without checking if it signed.
    // Anyone can call this instruction and pass ANY account as 'admin'.
    // Even if we check `admin.key() == config.admin`, if it's not a signer, 
    // I can pass the admin's public key but I don't have their private key.
    // Wait, if I want to UPDATE the admin.
    // The requirement is: "Only the current admin can update the admin".
    // Vulnerability: Checking the address matches, but NOT checking if it signed.
    pub fn update_admin_insecure(ctx: Context<UpdateAdminInsecure>, new_admin: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        
        // We verify the passed account IS the stored admin address...
        if ctx.accounts.admin.key() != config.admin {
            return err!(ErrorCode::InvalidAdmin);
        }

        // BUT we forgot to check if ctx.accounts.admin.is_signer!
        // So anyone can pass the admin's public key here and bypass the check.

        config.admin = new_admin;
        Ok(())
    }

    pub fn update_admin_secure(ctx: Context<UpdateAdminSecure>, new_admin: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        // Anchor's Signer<'info> type automatically checks:
        // 1. The account is a signer.
        // 2. (In this context context) We also need to check it matches the config state.
        
        if ctx.accounts.admin.key() != config.admin {
             return err!(ErrorCode::InvalidAdmin);
        }
        
        config.admin = new_admin;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 32)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: We just set this as initial admin
    pub admin: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAdminInsecure<'info> {
    #[account(mut)]
    pub config: Account<'info, Config>,
    /// CHECK: VULNERABLE. We accept any account info here.
    /// In the instruction, we check if key matches config.admin,
    /// but we lack the `Signer` check.
    pub admin: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdateAdminSecure<'info> {
    #[account(mut)]
    pub config: Account<'info, Config>,
    // SECURE: This ensures the account signed the transaction.
    pub admin: Signer<'info>,
}

#[account]
pub struct Config {
    pub admin: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid admin address provided.")]
    InvalidAdmin,
}
