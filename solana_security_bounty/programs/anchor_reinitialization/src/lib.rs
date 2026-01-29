use anchor_lang::prelude::*;

declare_id!("5RCjxGdhxajLEsxRRZtvtw4Qo2qmSbLLxP5xVHMigvJA");

#[program]
pub mod anchor_reinitialization {
    use super::*;

    // VULNERABLE: Initializes an account but fails to check if it's already initialized.
    // It assumes the caller passed a fresh account.
    // Ideally, we should check `if user.discriminator != 0 { return error }`.
    // But here we just overwrite data.
    pub fn initialize_insecure(ctx: Context<InitializeInsecure>, id: u64) -> Result<()> {
        let user = &mut ctx.accounts.user; // UncheckedAccount
        
        // We write the discriminator MANUALLY because UncheckedAccount doesn't do it.
        // Or we deserialize?
        // Let's say we just write data.
        
        let mut data = user.try_borrow_mut_data()?;
        
        // This simulates "initializing" by writing a discriminator and data.
        let dst: &mut [u8] = &mut data;
        let mut writer = std::io::Cursor::new(dst);
        
        let discriminator: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8]; // Fake discriminator for 'User'
        
        use anchor_lang::AnchorSerialize;
        discriminator.serialize(&mut writer)?;
        id.serialize(&mut writer)?;
        
        msg!("Initialized User Account with ID: {}", id);
        
        Ok(())
    }

    // SECURE: Uses 'init' constraint.
    // Anchor guarantees the account is strictly new (owned by system program, no data).
    // If we call this on an already initialized account, it fails.
    pub fn initialize_secure(ctx: Context<InitializeSecure>, id: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.id = id;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeInsecure<'info> {
    #[account(mut)]
    /// CHECK: VULNERABLE. Accepting any account. We don't check owner or state.
    /// The caller usually creates the account via SystemProgram before calling this.
    /// But if they call it TWICE, we just overwrite.
    pub user: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct InitializeSecure<'info> {
    #[account(
        init,
        payer = payer, 
        space = 8 + 8,
        // SECURE: 'init' checks that account is NOT initialized.
        // It consumes the account creation logic.
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct User {
    pub id: u64,
}
