use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::{self, log},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::state::TokenAccount;

use crate::state::Escrow;

pub fn process_make_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, escrow_account, maker_ata, escrow_ata, system_program, token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let maker_ata_state = TokenAccount::from_account_info(&maker_ata)?;

    if maker_ata_state.owner() != maker.key() {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }
    if maker_ata_state.mint() != mint_a.key() {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }

    // parsing ix data
    if data.len() != 16 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount_to_receive = {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[0..8]);
        u64::from_le_bytes(bytes)
    };

    let amount_to_give = {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[8..16]);
        u64::from_le_bytes(bytes)
    };

    // pda & bump derivation
    let seed = [b"escrow".as_ref(), maker.key().as_ref()];
    let (escrow_account_pda, initial_bump) = pubkey::find_program_address(&seed, &crate::ID);
    log(&escrow_account_pda);
    log(&escrow_account.key());
    assert_eq!(escrow_account_pda, *escrow_account.key());

    let bump = [initial_bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    if escrow_account.owner() != &crate::ID {
        CreateAccount {
            from: maker,
            to: escrow_account,
            lamports: Rent::get()?.minimum_balance(Escrow::LEN),
            space: Escrow::LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[seeds.clone()])?;

        {
            let escrow_state = Escrow::from_account_info(escrow_account)?;

            escrow_state.set_maker(maker.key());
            escrow_state.set_mint_a(mint_a.key());
            escrow_state.set_mint_b(mint_b.key());
            escrow_state.set_amount_to_receive(amount_to_receive);
            escrow_state.set_amount_to_give(amount_to_give);
            escrow_state.bump = initial_bump;
        }
    } else {
        return Err(ProgramError::IllegalOwner);
    }

    pinocchio_associated_token_account::instructions::Create {
        funding_account: maker,
        account: escrow_ata,
        wallet: escrow_account,
        mint: mint_a,
        token_program: token_program,
        system_program: system_program,
    }
    .invoke()?;

    Ok(())
}

pub fn transfer_token(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Expect exactly 16 bytes => two u64 values; we only need make_amount (second u64)
    if data.len() != 16 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount_to_give = {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[8..16]);
        u64::from_le_bytes(bytes)
    };

    let [maker, _mint_a, _mint_b, _escrow_account, maker_ata, escrow_ata, _system_program, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // from maker to the vault
    pinocchio_token::instructions::Transfer {
        from: maker_ata,
        to: escrow_ata, //vault
        authority: maker,
        amount: amount_to_give,
    }
    .invoke()?;

    Ok(())
}
