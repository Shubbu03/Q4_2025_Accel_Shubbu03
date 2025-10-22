use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::log,
    ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::state::TokenAccount;

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [taker, maker, mint_a, mint_b, escrow_account, taker_ata_a, taker_ata_b, maker_ata_b, _escrow_ata, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // validate taker's ata for mint_a (receive)
    let taker_ata_a_state = TokenAccount::from_account_info(&taker_ata_a)?;

    if taker_ata_a_state.owner() != taker.key() {
        return Err(ProgramError::IllegalOwner);
    }
    if taker_ata_a_state.mint() != mint_a.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // validate taker's ata for mint_b (send)
    let taker_ata_b_state = TokenAccount::from_account_info(&taker_ata_b)?;

    if taker_ata_b_state.owner() != taker.key() {
        return Err(ProgramError::IllegalOwner);
    }
    if taker_ata_b_state.mint() != mint_b.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // validate maker's ata for mint_b
    let maker_ata_b_state = TokenAccount::from_account_info(&maker_ata_b)?;

    if maker_ata_b_state.owner() != maker.key() {
        return Err(ProgramError::IllegalOwner);
    }
    if maker_ata_b_state.mint() != mint_b.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // validate ix data and derive escrow pda
    if data.len() != 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let bump = data[0];
    let seed = [b"escrow".as_ref(), maker.key().as_slice(), &[bump]];
    let escrow_account_pda = derive_address(&seed, None, &crate::ID);

    log(&escrow_account_pda);
    log(&escrow_account.key());
    assert_eq!(escrow_account_pda, *escrow_account.key());

    // load escrow state
    let escrow_state = Escrow::from_account_info(escrow_account)?;

    // validate escrow state
    if escrow_state.maker() != *maker.key() {
        return Err(ProgramError::InvalidAccountData);
    }
    if escrow_state.mint_a() != *mint_a.key() {
        return Err(ProgramError::InvalidAccountData);
    }
    if escrow_state.mint_b() != *mint_b.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    let escrow_state = Escrow::from_account_info(escrow_account)?;
    if escrow_state.bump != bump {
        return Err(ProgramError::InvalidInstructionData);
    }

    Ok(())
}

pub fn maker_transfer(accounts: &[AccountInfo]) -> ProgramResult {
    let [taker, _maker, _mint_a, _mint_b, _escrow_account, _taker_ata_a, taker_ata_b, maker_ata_b, _escrow_ata, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // transfering mint_b from taker to maker - no pda
    let escrow_state = {
        // we don't have the escrow account in this slice binding; amounts known from state require it
        // But accounts order includes it at index 4; safe to index due to earlier destructuring shape
        let escrow_account = &accounts[4];
        Escrow::from_account_info(escrow_account)?
    };
    let amount_to_receive = escrow_state.amount_to_receive();

    pinocchio_token::instructions::Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: amount_to_receive,
    }
    .invoke()?;

    Ok(())
}

pub fn taker_transfer(accounts: &[AccountInfo]) -> ProgramResult {
    let [_taker, maker, _mint_a, _mint_b, escrow_account, taker_ata_a, _taker_ata_b, _maker_ata_b, escrow_ata, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Reload escrow state and amounts
    let escrow_state = Escrow::from_account_info(escrow_account)?;
    let amount_to_give = escrow_state.amount_to_give();
    let bump_array = [escrow_state.bump];

    // signer seeds
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump_array),
    ];
    let signers = &[Signer::from(&seed)];

    // transfering mint_a from escrow to taker using PDA signer
    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: taker_ata_a,
        authority: escrow_account,
        amount: amount_to_give,
    }
    .invoke_signed(signers)?;

    // When running this code gives err - transfer-from-must-not-carry-data
    // Close escrow account to maker after settlement
    // pinocchio_system::instructions::Transfer {
    //     from: escrow_account,
    //     to: &accounts[1], // maker at index 1
    //     lamports: escrow_account.lamports(),
    // }
    // .invoke_signed(signers)?;

    Ok(())
}
