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
    {
        let maker_ata_state = TokenAccount::from_account_info(&maker_ata)?;
        if maker_ata_state.owner() != maker.key() {
            return Err(ProgramError::IllegalOwner);
        }
        if maker_ata_state.mint() != mint_a.key() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    // validate escrow account and derive pda
    let bump = data[0];
    let seed = [b"escrow".as_ref(), maker.key().as_slice(), &[bump]];
    let escrow_account_pda = derive_address(&seed, None, &crate::ID);

    log(&escrow_account_pda);
    log(&escrow_account.key());
    assert_eq!(escrow_account_pda, *escrow_account.key());

    // Read escrow state immutably
    let amount_to_give = {
        let data = escrow_account.try_borrow_data()?;
        if data.len() != Escrow::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        // Layout: maker[0..32], mint_a[32..64], mint_b[64..96], amount_to_receive[96..104], amount_to_give[104..112], bump[112]
        if &data[0..32] != maker.key().as_ref() {
            return Err(ProgramError::InvalidAccountData);
        }
        if &data[32..64] != mint_a.key().as_ref() {
            return Err(ProgramError::InvalidAccountData);
        }
        if &data[64..96] != mint_b.key().as_ref() {
            return Err(ProgramError::InvalidAccountData);
        }
        let mut amt_bytes = [0u8; 8];
        amt_bytes.copy_from_slice(&data[104..112]);
        u64::from_le_bytes(amt_bytes)
    };

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
    // pinocchio_system::instructions::Transfer {
    //     from: escrow_account,
    //     to: maker,
    //     lamports: escrow_account.lamports(),
    // }
    // .invoke()?;

    Ok(())
}
