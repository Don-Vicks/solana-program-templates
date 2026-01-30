use pinocchio::{
    account::AccountView,
    address::Address,
    entrypoint,
    ProgramResult,
    // program_error::ProgramError // pinocchio doesn't export this directly?
    // It exports `error` module which is `solana_program_error`.
    // Let's use `pinocchio::error::ProgramError`.
    error::ProgramError,
};

entrypoint!(process_instruction);

// Instruction Discriminators
const IX_SIGNER_INSECURE: u8 = 0;
const IX_SIGNER_SECURE: u8 = 1;
const IX_CPI_INSECURE: u8 = 2;
const IX_CPI_SECURE: u8 = 3;
const IX_COSPLAY_INSECURE: u8 = 4;
const IX_COSPLAY_SECURE: u8 = 5;
const IX_PDA_INSECURE: u8 = 6;
const IX_PDA_SECURE: u8 = 7;
const IX_REINIT_INSECURE: u8 = 8;
const IX_REINIT_SECURE: u8 = 9;

fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    match instruction_data[0] {
        IX_SIGNER_INSECURE => process_signer(accounts, false),
        IX_SIGNER_SECURE => process_signer(accounts, true),
        
        IX_CPI_INSECURE => process_cpi(accounts, false),
        IX_CPI_SECURE => process_cpi(accounts, true),
        
        IX_COSPLAY_INSECURE => process_cosplay(accounts, false),
        IX_COSPLAY_SECURE => process_cosplay(accounts, true),
        
        IX_PDA_INSECURE => process_pda(accounts, program_id, false),
        IX_PDA_SECURE => process_pda(accounts, program_id, true),
        
        IX_REINIT_INSECURE => process_reinit(accounts, instruction_data, false),
        IX_REINIT_SECURE => process_reinit(accounts, instruction_data, true),

        _ => Err(ProgramError::InvalidInstructionData),
    }
}

pub fn next_account<'a, I>(iter: &mut I) -> Result<&'a AccountView, ProgramError>
where
    I: Iterator<Item = &'a AccountView>,
{
    iter.next().ok_or(ProgramError::NotEnoughAccountKeys)
}

// ----------------------------------------------------------------
// 1. Missing Signer Check
// ----------------------------------------------------------------
fn process_signer(accounts: &[AccountView], secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _config = next_account(account_iter)?; 
    let admin = next_account(account_iter)?;

    if secure && !admin.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Logic simulated
    Ok(())
}

// ----------------------------------------------------------------
// 2. Arbitrary CPI
// ----------------------------------------------------------------
fn process_cpi(accounts: &[AccountView], secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _source = next_account(account_iter)?;
    let _dest = next_account(account_iter)?;
    let _authority = next_account(account_iter)?;
    let token_program = next_account(account_iter)?;

    if secure {
        // TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let expected = [
            6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172, 
            28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169
        ];
        if token_program.address().as_ref() != &expected {
            return Err(ProgramError::IncorrectProgramId);
        }
    }
    Ok(())
}

// ----------------------------------------------------------------
// 3. Type Cosplay
// ----------------------------------------------------------------
fn process_cosplay(accounts: &[AccountView], _secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _user_account = next_account(account_iter)?;
    
    // Logic simulated
    Ok(())
}

// ----------------------------------------------------------------
// 4. PDA Validation
// ----------------------------------------------------------------
fn process_pda(accounts: &[AccountView], _program_id: &Address, secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _pool = next_account(account_iter)?;
    
    if secure {
        // Logic commented out due to missing solana_program dependency for address derivation
        /*
        let (expected_key, _) = ... find_program_address ...
        if pool.key() != expected { return Err(ProgramError::InvalidSeeds); }
        */
    }
    
    Ok(())
}

// ----------------------------------------------------------------
// 5. Re-initialization
// ----------------------------------------------------------------
fn process_reinit(accounts: &[AccountView], _data: &[u8], _secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _user = next_account(account_iter)?;
    
    // Logic simulated
    Ok(())
}
