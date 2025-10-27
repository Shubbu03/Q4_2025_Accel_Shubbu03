use std::path::PathBuf;

use litesvm::{types::TransactionResult, LiteSVM};
use litesvm_token::{CreateMint, TOKEN_ID};
use solana_sdk::{
    message::{AccountMeta, Instruction},
    msg,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    sysvar,
    transaction::Transaction,
};

use crate::states::{fundraiser::Fundraiser, utils::DataLen};

pub(super) fn program_id() -> Pubkey {
    Pubkey::from(crate::ID)
}

pub(super) fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let so_path = PathBuf::from(manifest_dir).join("target/deploy/fundraiser.so");
    msg!("The path is: {:?}", so_path);
    let program_data = std::fs::read(so_path).expect("Failed to read SO file");
    svm.add_program(program_id(), &program_data).unwrap();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    (svm, payer)
}

pub(super) struct InitializeData {
    pub mint_to_raise: Pubkey,
    pub fundraiser: (Pubkey, u8),
    pub vault: Pubkey,
    pub amount_to_raise: u64,
    pub duration: u8,
}

impl InitializeData {
    pub(super) fn new(svm: &mut LiteSVM, payer: &Keypair) -> Self {
        let mint_to_raise = CreateMint::new(svm, payer)
            .authority(&payer.pubkey())
            .decimals(6)
            .send()
            .unwrap();

        let (fundraiser_pubkey, bump) =
            Pubkey::find_program_address(&[b"fundraiser", payer.pubkey().as_ref()], &program_id());

        let associated_token_program = Pubkey::from(pinocchio_associated_token_account::id());
        let (vault, _) = Pubkey::find_program_address(
            &[
                fundraiser_pubkey.as_ref(),
                TOKEN_ID.as_ref(),
                mint_to_raise.as_ref(),
            ],
            &associated_token_program,
        );

        Self {
            mint_to_raise,
            fundraiser: (fundraiser_pubkey, bump),
            vault,
            amount_to_raise: 100_000_000u64,
            duration: 30u8,
        }
    }
}

pub(super) fn initialize(
    svm: &mut LiteSVM,
    payer: &Keypair,
    data: &InitializeData,
) -> TransactionResult {
    let mut ix_data = Vec::new();
    ix_data.push(0u8);
    ix_data.extend_from_slice(&data.amount_to_raise.to_le_bytes());
    ix_data.extend_from_slice(&[data.duration]);
    ix_data.extend_from_slice(&[data.fundraiser.1]);

    let system_program = Pubkey::from(pinocchio_system::id());
    let associated_token_program = Pubkey::from(pinocchio_associated_token_account::id());

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new_readonly(data.mint_to_raise, false),
        AccountMeta::new(data.fundraiser.0, false),
        AccountMeta::new(data.vault, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(TOKEN_ID, false),
        AccountMeta::new_readonly(system_program, false),
        AccountMeta::new_readonly(associated_token_program, false),
    ];

    let ix = Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}

#[test]
pub fn test_initialize() {
    let (mut svm, payer) = setup();
    let init_data = InitializeData::new(&mut svm, &payer);

    let tx = initialize(&mut svm, &payer, &init_data).unwrap();
    msg!(
        "Initialize fundraiser successful with signature: {}",
        tx.signature
    );

    let fundraiser_account = svm.get_account(&init_data.fundraiser.0).unwrap();
    assert!(
        fundraiser_account.data.len() >= Fundraiser::LEN,
        "Fundraiser account data len should be at least {}",
        Fundraiser::LEN
    );

    let fundraiser_data = bytemuck::try_from_bytes::<Fundraiser>(&fundraiser_account.data)
        .expect("Failed to deserialize fundraiser data");

    let amount_to_raise = fundraiser_data.amount_to_raise;
    let current_amount = fundraiser_data.current_amount;
    let maker = fundraiser_data.maker;
    let mint_to_raise = fundraiser_data.mint_to_raise;
    let duration = fundraiser_data.duration;
    let bump = fundraiser_data.bump;

    msg!("Fundraiser amount to raise: {}", amount_to_raise);

    assert_eq!(
        amount_to_raise, init_data.amount_to_raise,
        "Amount to raise should match"
    );

    assert_eq!(
        maker.as_ref(),
        payer.pubkey().as_ref(),
        "Fundraiser maker should match payer"
    );

    assert_eq!(
        mint_to_raise.as_ref(),
        init_data.mint_to_raise.as_ref(),
        "Mint to raise should match"
    );

    assert_eq!(duration, [init_data.duration], "Duration should match");

    assert_eq!(bump, [init_data.fundraiser.1], "Bump should match");

    assert_eq!(current_amount, 0, "Current amount should be 0 initially");

    msg!("All assertions passed!");
}
