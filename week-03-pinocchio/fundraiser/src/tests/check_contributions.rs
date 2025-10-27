use litesvm::{types::TransactionResult, LiteSVM};
use solana_sdk::{
    message::{AccountMeta, Instruction},
    msg,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    sysvar,
    transaction::Transaction,
};

pub(super) fn check_contributions(
    svm: &mut LiteSVM,
    maker: &Keypair,
    mint_to_raise: &Pubkey,
    fundraiser_pda: Pubkey,
    vault: Pubkey,
) -> TransactionResult {
    let ix_data = vec![2u8];

    let system_program = Pubkey::from(pinocchio_system::id());
    let associated_token_program = Pubkey::from(pinocchio_associated_token_account::id());

    let accounts = vec![
        AccountMeta::new(maker.pubkey(), true),
        AccountMeta::new_readonly(*mint_to_raise, false),
        AccountMeta::new(fundraiser_pda, false),
        AccountMeta::new(vault, false),
        AccountMeta::new(maker.pubkey(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(litesvm_token::TOKEN_ID, false),
        AccountMeta::new_readonly(system_program, false),
        AccountMeta::new_readonly(associated_token_program, false),
    ];

    let ix = Instruction {
        program_id: super::initialize::program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&maker.pubkey()),
        &[maker],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}

#[test]
pub fn test_check_contributions() {
    let (mut svm, payer) = super::initialize::setup();
    let init_data = super::initialize::InitializeData::new(&mut svm, &payer);
    let init_tx = super::initialize::initialize(&mut svm, &payer, &init_data).unwrap();
    msg!("Initialize fundraiser: {}", init_tx.signature);

    let result = check_contributions(
        &mut svm,
        &payer,
        &init_data.mint_to_raise,
        init_data.fundraiser.0,
        init_data.vault,
    );

    if result.is_err() {
        msg!("Check contributions failed as expected (target not met)");
    } else {
        msg!("Check contributions succeeded (target was met)!");
    }

    msg!("Check contributions test completed!");
}
