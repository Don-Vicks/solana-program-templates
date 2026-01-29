use anchor_lang::prelude::*;
use anchor_lang::solana_program;

declare_id!("6SFE4yeHZVRb89ZADeLHQjKoe7oCRkR3sdeccxEhzJ4T");

#[program]
pub mod anchor_arbitrary_cpi {
    use super::*;

    pub fn cpi_insecure(ctx: Context<CpiInsecure>, amount: u64) -> Result<()> {
        // VULNERABLE: We are invoking a CPI to the 'token_program' passed in the context.
        // However, 'token_program' is defined as UncheckedAccount (or just AccountInfo).
        // The attacker can pass a malicious program in place of the Token Program.
        
        let ix = solana_program::instruction::Instruction {
            program_id: ctx.accounts.token_program.key(), // malicious program ID
            accounts: vec![
                solana_program::instruction::AccountMeta::new(ctx.accounts.source.key(), false),
                solana_program::instruction::AccountMeta::new(ctx.accounts.destination.key(), false),
                solana_program::instruction::AccountMeta::new_readonly(ctx.accounts.authority.key(), true),
            ],
            data: {
                let mut d = vec![3u8]; // 3 = Transfer
                d.extend_from_slice(&amount.to_le_bytes());
                d
            },
        };

        solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.source.to_account_info(),
                ctx.accounts.destination.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn cpi_secure(ctx: Context<CpiSecure>, amount: u64) -> Result<()> {
        // SECURE: We use the `anchor_spl::token::transfer` helper which requires
        // the program to be the real Token Program (checked by Anchor's Program type).
        
        // Alternatively, if using invoke manually, we would check:
        // if ctx.accounts.token_program.key() != anchor_spl::token::ID { return err ... }

        use anchor_spl::token::{self, Transfer};

        let cpi_accounts = Transfer {
            from: ctx.accounts.source.to_account_info(),
            to: ctx.accounts.destination.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        // Anchor checks that ctx.accounts.token_program IS the SPL Token Program
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CpiInsecure<'info> {
    #[account(mut)]
    /// CHECK: VULNERABLE. We accept any account as source.
    pub source: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: VULNERABLE. We accept any account as destination.
    pub destination: UncheckedAccount<'info>,
    pub authority: Signer<'info>,
    /// CHECK: VULNERABLE. We accept any program here.
    pub token_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CpiSecure<'info> {
    #[account(mut)]
    /// CHECK: In a real app we'd use Account<'info, TokenAccount> but for Cpi demo Unchecked is fine ONLY IF we trust the token program to check validity? 
    /// Actually, if we use anchor_spl::token::transfer, it will fail if accounts are invalid for the token program.
    /// But typically we should use Account<'info, TokenAccount>. Keeping it simple for demo.
    pub source: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Recipient account
    pub destination: UncheckedAccount<'info>,
    pub authority: Signer<'info>,
    // SECURE: This ensures it IS the token program.
    pub token_program: Program<'info, anchor_spl::token::Token>,
}
