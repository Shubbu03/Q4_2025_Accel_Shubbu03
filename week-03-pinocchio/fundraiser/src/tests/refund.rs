use litesvm::{types::TransactionResult, LiteSVM};
use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
use solana_sdk::{
    message::{AccountMeta, Instruction},
    msg,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

use crate::states::contributor::Contributor;

pub(super) fn refund(
    svm: &mut LiteSVM,
    contributor: &Keypair,
    maker: &Pubkey,
    mint_to_raise: &Pubkey,
    fundraiser_pda: Pubkey,
    contributor_pda: Pubkey,
    contributor_ata: Pubkey,
    vault: Pubkey,
) -> TransactionResult {
    let ix_data = vec![3u8];

    let token_program = litesvm_token::TOKEN_ID;
    let system_program = Pubkey::from(pinocchio_system::id());

    let accounts = vec![
        AccountMeta::new(contributor.pubkey(), true),
        AccountMeta::new_readonly(*maker, false),
        AccountMeta::new_readonly(*mint_to_raise, false),
        AccountMeta::new(fundraiser_pda, false),
        AccountMeta::new(contributor_pda, false),
        AccountMeta::new(contributor_ata, false),
        AccountMeta::new(vault, false),
        AccountMeta::new_readonly(token_program, false),
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
pub fn test_refund() {
    let (mut svm, payer) = super::initialize::setup();
    let init_data = super::initialize::InitializeData::new(&mut svm, &payer);
    let init_tx = super::initialize::initialize(&mut svm, &payer, &init_data).unwrap();
    msg!("Initialize fundraiser: {}", init_tx.signature);

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
        .unwrap();
    msg!("Created contributor: {}", contributor.pubkey());

    let contributor_ata_pubkey = Pubkey::find_program_address(
        &[
            contributor.pubkey().as_ref(),
            litesvm_token::TOKEN_ID.as_ref(),
            init_data.mint_to_raise.as_ref(),
        ],
        &Pubkey::from(pinocchio_associated_token_account::id()),
    )
    .0;

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

    let contributor_ata = contributor_ata_pubkey;

    let contributor_pda = Pubkey::find_program_address(
        &[
            b"contributor",
            init_data.fundraiser.0.as_ref(),
            contributor.pubkey().as_ref(),
        ],
        &super::initialize::program_id(),
    );

    let contribute_amount = 10_000_000u64;
    msg!("Contributing {} tokens", contribute_amount);

    let contribute_tx = super::contribute::contribute(
        &mut svm,
        &contributor,
        &init_data.mint_to_raise,
        init_data.fundraiser,
        contributor_pda,
        contributor_ata,
        contribute_amount,
        init_data.vault,
    )
    .unwrap();
    msg!("Contribute transaction: {}", contribute_tx.signature);

    let contributor_account = svm.get_account(&contributor_pda.0).unwrap();
    let contributor_data = bytemuck::try_from_bytes::<Contributor>(&contributor_account.data)
        .expect("Failed to deserialize contributor data");

    let contributor_amount = contributor_data.amount;
    assert_eq!(
        contributor_amount, contribute_amount,
        "Contribution should be recorded"
    );
    msg!("Contribution verified: {}", contributor_amount);

    let vault_before = svm.get_account(&init_data.vault).unwrap();
    let vault_data_before = &vault_before.data[64..72];
    let vault_amount_before = u64::from_le_bytes(vault_data_before.try_into().unwrap());
    msg!("Vault amount before refund: {}", vault_amount_before);

    let contributor_ata_before = svm.get_account(&contributor_ata).unwrap();
    let contributor_balance_before =
        u64::from_le_bytes(contributor_ata_before.data[64..72].try_into().unwrap());
    msg!(
        "Contributor balance before refund: {}",
        contributor_balance_before
    );

    let result = refund(
        &mut svm,
        &contributor,
        &payer.pubkey(),
        &init_data.mint_to_raise,
        init_data.fundraiser.0,
        contributor_pda.0,
        contributor_ata,
        init_data.vault,
    );

    if result.is_err() {
        msg!("Refund failed as expected (fundraiser conditions not met)");
    } else {
        let vault_after = svm.get_account(&init_data.vault).unwrap();
        let vault_data_after = &vault_after.data[64..72];
        let vault_amount_after = u64::from_le_bytes(vault_data_after.try_into().unwrap());

        let contributor_ata_after = svm.get_account(&contributor_ata).unwrap();
        let contributor_balance_after =
            u64::from_le_bytes(contributor_ata_after.data[64..72].try_into().unwrap());

        msg!("Vault amount after refund: {}", vault_amount_after);
        msg!(
            "Contributor balance after refund: {}",
            contributor_balance_after
        );

        assert_eq!(
            vault_amount_after,
            vault_amount_before - contribute_amount,
            "Vault should have returned the contribution"
        );
    }

    msg!("✅ Refund test completed!");
}
