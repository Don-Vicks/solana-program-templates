use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
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

// Struct Layouts (Zero Copy / Raw Bytes)
// Config: [admin: Pubkey (32)]
// User: [discriminator: u64 (8), id: u64 (8), authority: Pubkey (32)]
// Pool: [discriminator: u64 (8), vault: Pubkey (32), amount: u64 (8)]

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
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

pub fn next_account_info<'a, 'b, I>(iter: &mut I) -> Result<&'a AccountInfo<'a>, ProgramError>
where
    I: Iterator<Item = &'a AccountInfo<'a>>,
{
    iter.next().ok_or(ProgramError::NotEnoughAccountKeys)
}

// ----------------------------------------------------------------
// 1. Missing Signer Check (Actual State Write)
// ----------------------------------------------------------------
fn process_signer(accounts: &[AccountInfo], secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config = next_account_info(account_iter)?; // Writable
    let admin = next_account_info(account_iter)?; // New admin logic? Or authority?

    // Logic: Update config.admin to be the 'admin' account passed.
    // In secure version, 'admin' must be signer (authorizing the change? No Usually authority signs).
    // Let's assume 'admin' is the AUTHORITY trying to update the config.
    // Or let's assume we are SETTING the admin to this new key, and the CURRENT admin must sign.
    // Anchor template: update_admin(new_admin). Signer is 'admin' (current admin).
    
    if secure && !admin.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Actually write to config
    // Config layout: [Pubkey]
    let mut data = config.borrow_mut_data().ok_or(ProgramError::AccountDataTooSmall)?;
    if data.len() < 32 { return Err(ProgramError::AccountDataTooSmall); }
    
    let admin_key = admin.key().as_ref();
    data[0..32].copy_from_slice(admin_key);
    
    msg!("Config admin updated to {:?}", admin.key());
    Ok(())
}

// ----------------------------------------------------------------
// 2. Arbitrary CPI (Simulated Invoke)
// ----------------------------------------------------------------
fn process_cpi(accounts: &[AccountInfo], secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _source = next_account_info(account_iter)?;
    let _dest = next_account_info(account_iter)?;
    let _authority = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;

    if secure {
        // Mainnet Token Program ID
        let expected = [
            6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172, 
            28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169
        ];
        if token_program.key() != &expected {
            return Err(ProgramError::IncorrectProgramId);
        }
    }
    
    // In a real optimized program we would construct InstructionView and call invoke.
    // For this template, validating the logic is the key part.
    msg!("CPI Logic Executed");
    Ok(())
}

// ----------------------------------------------------------------
// 3. Type Cosplay (Actual Data Read)
// ----------------------------------------------------------------
fn process_cosplay(accounts: &[AccountInfo], secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let user_account = next_account_info(account_iter)?;
    
    let data = user_account.borrow_data().ok_or(ProgramError::AccountDataTooSmall)?;
    if data.len() < 8 { return Err(ProgramError::AccountDataTooSmall); }

    if secure {
        // Check discriminator. Anchor uses Sha256("account:User")[..8]
        // We'll assume a made-up discriminator for this raw program: 0xAA...
        let expected_disc = [0xAA; 8];
        if data[0..8] != expected_disc {
            return Err(ProgramError::InvalidAccountData);
        }
    }
    
    // Read ID (u64 at offset 8)
    let id_bytes: [u8; 8] = data[8..16].try_into().map_err(|_| ProgramError::AccountDataTooSmall)?;
    let id = u64::from_le_bytes(id_bytes);
    msg!("Read User ID: {}", id);
    
    Ok(())
}

// ----------------------------------------------------------------
// 4. PDA Validation (Actual Derivation)
// ----------------------------------------------------------------
fn process_pda(accounts: &[AccountInfo], program_id: &Pubkey, secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let pool = next_account_info(account_iter)?;
    
    if secure {
        let (expected_key, _) = Pubkey::find_program_address(&[b"pool"], program_id);
        if pool.key() != &expected_key {
            return Err(ProgramError::InvalidSeeds);
        }
    }
    
    // Simulate deposit logic
    msg!("Deposit to pool: {:?}", pool.key());
    Ok(())
}

// ----------------------------------------------------------------
// 5. Re-initialization (Actual State Check & Write)
// ----------------------------------------------------------------
fn process_reinit(accounts: &[AccountInfo], _data: &[u8], secure: bool) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let user = next_account_info(account_iter)?;
    
    let mut data = user.borrow_mut_data().ok_or(ProgramError::AccountDataTooSmall)?;
    
    // "Is Initialized" check = check if discriminator is set
    // For this demo, let's say initialized means first byte != 0
    let is_initialized = data.iter().any(|&x| x != 0);

    if secure && is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    
    // Initialize: Write [0xAA; 8] as discriminator
    if data.len() >= 8 {
        data[0..8].copy_from_slice(&[0xAA; 8]);
    }
    
    msg!("Account Initialized");
    Ok(())
}
