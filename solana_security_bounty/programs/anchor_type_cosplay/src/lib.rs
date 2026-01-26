use anchor_lang::prelude::*;
use std::mem::size_of;

declare_id!("443VtsgVs9WPmDcrZi6TBnwAmDbjUqGw1NxiCER3uuoo");

#[program]
pub mod anchor_type_cosplay {
    use super::*;

    pub fn initialize_user(ctx: Context<InitializeUser>, id: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.id = id;
        user.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn initialize_admin(ctx: Context<InitializeAdmin>, id: u64) -> Result<()> {
        let admin = &mut ctx.accounts.admin;
        admin.id = id;
        admin.special_power = 9000;
        Ok(())
    }

    // VULNERABLE: We manually deserialize the account logic.
    // If the attacker passes an 'Admin' account, but we treat it as 'User',
    // AND if the layouts are compatible (or we just read the first bytes), we get type cosplay.
    // Here 'User' is: [u8; 8] disc + u64 id + Pubkey authority.
    // 'Admin' is: [u8; 8] disc + u64 id + u64 special_power.
    // If we try to deserialize 'Admin' data as 'User', the bytes will be interpreted as User fields.
    // `Admin.id` (u64) -> `User.id`
    // `Admin.special_power` (u64) -> Part of `User.authority` (Pubkey is 32 bytes). 
    // This is a bit messy to demonstrate cleanly with these structs, but the principle holds: 
    // We are pretending the data belongs to 'User' when it belongs to 'Admin'.
    //
    // A better example is just trusting the data without checking discriminator.
    // Or two structs with IDENTICAL layout but different meaning.
    pub fn cosplay_insecure(ctx: Context<CosplayInsecure>) -> Result<()> {
        let account_info = &ctx.accounts.user;
        
        // VULNERABLE: Manual deserialization using try_from_slice which checks size but NOT discriminator
        // unless the struct includes it.
        // Anchor Accounts have an 8-byte discriminator PREPENDED.
        // try_from_slice typically reads from byte 0. 
        // If we define User/Admin with `#[account]`, they have discriminators.
        
        let data = account_info.try_borrow_data()?;
        
        // If we skip the first 8 bytes (discriminator) and try to deserialize the rest:
        let mut _data_slice: &[u8] = &data[8..];
        let user_data: User = AnchorDeserialize::deserialize(&mut _data_slice)?;
        
        // We successfully deserialized 'User' data.
        // But we never checked if the 8 byte discriminator matches 'User' discriminator!
        // So we could have passed an 'Admin' account (which has valid data after 8 bytes if fields align).
        
        msg!("User ID: {}", user_data.id);
        
        Ok(())
    }

    pub fn cosplay_secure(ctx: Context<CosplaySecure>) -> Result<()> {
        let user = &ctx.accounts.user;
        // SECURE: Anchor checks the discriminator automatically here.
        msg!("User ID: {}", user.id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(init, payer = authority, space = 8 + 8 + 32)]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeAdmin<'info> {
    #[account(init, payer = authority, space = 8 + 8 + 8)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CosplayInsecure<'info> {
    /// CHECK: VULNERABLE. Accepting any account.
    pub user: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CosplaySecure<'info> {
    // SECURE: Anchor checks discriminator matches 'User'
    pub user: Account<'info, User>,
}

#[account]
pub struct User {
    pub id: u64,
    pub authority: Pubkey,
}

#[account]
pub struct Admin {
    pub id: u64,
    pub special_power: u64,
}
