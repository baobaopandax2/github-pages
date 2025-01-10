use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::{instruction, state};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let token_program = next_account_info(accounts_iter)?;
    let mint = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;

    // Check if the token program is correct
    if token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    match instruction_data[0] {
        // Initialize mint
        0 => {
            msg!("Initializing Bao Bao Panda Token");
            let mint_data = state::Mint::unpack_unchecked(&mint.data.borrow())?;
            if mint_data.supply != 0 {
                return Err(ProgramError::InvalidInstructionData);
            }

            // Set up mint with 10 trillion tokens (10^13 with 9 decimals)
            instruction::initialize_mint(
                token_program.key,
                mint.key,
                mint_authority.key,
                None, // No freeze authority
                9,    // 9 decimals for precision
            )?;

            // Set token name and symbol
            let name = b"Bao Bao Panda";
            let symbol = b"BaoBao";
            let mint_data = state::Mint {
                mint_authority: None, // No mint authority after initialization
                supply: 10_000_000_000_000,
                decimals: 9,
                is_initialized: true,
                freeze_authority: None, // No freeze authority
            };
            state::Mint::pack(mint_data, &mut mint.data.borrow_mut()[..])?;

            // Write name and symbol to account data
            mint.data.borrow_mut()[8..8 + name.len()].copy_from_slice(name);
            mint.data.borrow_mut()[8 + name.len()..8 + name.len() + symbol.len()].copy_from_slice(symbol);

            // Mint the total supply
            instruction::mint_to(
                token_program.key,
                mint.key,
                mint.key, // Minting to itself as there's no specific account to mint to initially
                mint_authority.key,
                &[],
                10_000_000_000_000, // 10 trillion tokens with 9 decimals
            )?;

            // Close the mint authority so no one can mint more tokens
            instruction::set_authority(
                token_program.key,
                mint.key,
                Some(&spl_token::id()),
                spl_token::instruction::AuthorityType::MintTokens,
                mint_authority.key,
                &[],
            )?;
        },
        // Transfer tokens
        1 => {
            let source = next_account_info(accounts_iter)?;
            let destination = next_account_info(accounts_iter)?;
            let authority = next_account_info(accounts_iter)?;

            if !authority.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

            instruction::transfer(
                token_program.key,
                source.key,
                destination.key,
                authority.key,
                &[],
                amount,
            )?;
        },
        _ => {
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    Ok(())
}
