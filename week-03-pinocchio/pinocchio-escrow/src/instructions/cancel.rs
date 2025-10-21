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

pub fn process_cancel_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, escrow_account, maker_ata, escrow_ata, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // validate maker's ata for mint_a
    let maker_ata_state = TokenAccount::from_account_info(&maker_ata)?;

    if maker_ata_state.owner() != maker.key() {
        return Err(ProgramError::IllegalOwner);
    }
    if maker_ata_state.mint() != mint_a.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // validate escrow account and deriving pda
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

    let amount_to_give = escrow_state.amount_to_give();

    let bump_array = [bump];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump_array),
    ];

    let signers = &[Signer::from(&seed)];

    // 1. transfer mint_a tokens from escrow back to maker
    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: maker_ata,
        authority: escrow_account,
        amount: amount_to_give,
    }
    .invoke_signed(signers)?;

    // 2. close escrow account and transfer lamports to maker
    pinocchio_system::instructions::Transfer {
        from: escrow_account,
        to: maker,
        lamports: escrow_account.lamports(),
    }
    .invoke()?;

    Ok(())
}
