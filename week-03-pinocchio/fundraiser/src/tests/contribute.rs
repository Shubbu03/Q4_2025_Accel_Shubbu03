use litesvm::{types::TransactionResult, LiteSVM};
use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
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

use crate::states::{contributor::Contributor, fundraiser::Fundraiser};

pub(super) fn contribute(
    svm: &mut LiteSVM,
    contributor: &Keypair,
    mint_to_raise: &Pubkey,
    fundraiser_pda: (Pubkey, u8),
    contributor_pda: (Pubkey, u8),
    contributor_ata: Pubkey,
    amount: u64,
    vault: Pubkey,
) -> TransactionResult {
    let ix_data = [
        vec![1u8],
        amount.to_le_bytes().to_vec(),
        vec![contributor_pda.1],
    ]
    .concat();

    let system_program = Pubkey::from(pinocchio_system::id());

    let accounts = vec![
        AccountMeta::new(contributor.pubkey(), true),
        AccountMeta::new_readonly(*mint_to_raise, false),
        AccountMeta::new(fundraiser_pda.0, false),
        AccountMeta::new(contributor_pda.0, false),
        AccountMeta::new(contributor_ata, false),
        AccountMeta::new(vault, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(litesvm_token::TOKEN_ID, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let ix = Instruction {
        program_id: super::initialize::program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&contributor.pubkey()),
        &[contributor],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}

#[test]
pub fn test_contribute() {
    let (mut svm, payer) = super::initialize::setup();
    let init_data = super::initialize::InitializeData::new(&mut svm, &payer);
    let init_tx = super::initialize::initialize(&mut svm, &payer, &init_data).unwrap();
    msg!("Initialize fundraiser: {}", init_tx.signature);

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
        .unwrap();
    msg!("Created contributor: {}", contributor.pubkey());

    let contributor_ata =
        CreateAssociatedTokenAccount::new(&mut svm, &contributor, &init_data.mint_to_raise)
            .token_program_id(&litesvm_token::TOKEN_ID)
            .send()
            .unwrap();
    msg!("Created contributor ATA: {}", contributor_ata);

    let mint_amount = 50_000_000u64;
    msg!("Minting {} tokens to contributor", mint_amount);
    MintTo::new(
        &mut svm,
        &payer,
        &init_data.mint_to_raise,
        &contributor_ata,
        mint_amount,
    )
    .send()
    .unwrap();
    msg!("Tokens minted successfully");

    let contributor_pda = Pubkey::find_program_address(
        &[
            b"contributor",
            init_data.fundraiser.0.as_ref(),
            contributor.pubkey().as_ref(),
        ],
        &super::initialize::program_id(),
    );

    let vault_account = svm.get_account(&init_data.vault).unwrap();
    let vault_data = vault_account.data;
    let vault_amount_bytes = &vault_data[64..72];
    let vault_amount_before = u64::from_le_bytes(vault_amount_bytes.try_into().unwrap());
    msg!("Vault amount before: {}", vault_amount_before);

    let contribute_amount = 10_000_000u64;
    msg!("Contributing {} tokens", contribute_amount);

    msg!(
        "Contributor PDA: {}, bump: {}",
        contributor_pda.0,
        contributor_pda.1
    );
    msg!("Contributor ATA: {}", contributor_ata);
    msg!("Vault: {}", init_data.vault);
    msg!("Fundraiser PDA: {}", init_data.fundraiser.0);

    let tx = contribute(
        &mut svm,
        &contributor,
        &init_data.mint_to_raise,
        init_data.fundraiser,
        contributor_pda,
        contributor_ata,
        contribute_amount,
        init_data.vault,
    );

    let tx_result = tx.unwrap();
    msg!("Contribute transaction: {}", tx_result.signature);

    let contributor_account = svm.get_account(&contributor_pda.0).unwrap();
    let contributor_data = bytemuck::try_from_bytes::<Contributor>(&contributor_account.data)
        .expect("Failed to deserialize contributor data");

    let contributor_amount = contributor_data.amount;
    msg!("Contributor account amount: {}", contributor_amount);
    assert_eq!(
        contributor_amount, contribute_amount,
        "Contributor amount should match contribute_amount"
    );

    let fundraiser_account = svm.get_account(&init_data.fundraiser.0).unwrap();
    let fundraiser_data = bytemuck::try_from_bytes::<Fundraiser>(&fundraiser_account.data)
        .expect("Failed to deserialize fundraiser data");

    let current_amount = fundraiser_data.current_amount;
    assert_eq!(
        current_amount, contribute_amount,
        "Fundraiser current_amount should equal contribute_amount"
    );

    let vault_account_after = svm.get_account(&init_data.vault).unwrap();
    let vault_data_after = vault_account_after.data;
    let vault_amount_after_bytes = &vault_data_after[64..72];
    let vault_amount_after = u64::from_le_bytes(vault_amount_after_bytes.try_into().unwrap());
    msg!("Vault amount after: {}", vault_amount_after);

    assert_eq!(
        vault_amount_after,
        vault_amount_before + contribute_amount,
        "Vault should have received the contribution"
    );

    msg!("✅ Contribute test passed!");
}
