#[cfg(test)]
mod tests {
    use {
        anchor_lang::{prelude::msg, AccountDeserialize, InstructionData, ToAccountMetas},
        anchor_spl::associated_token::spl_associated_token_account,
        litesvm::LiteSVM,
        solana_instruction::{AccountMeta, Instruction},
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
    };

    static PROGRAM_ID: Pubkey = crate::ID;
    static TRANSFER_HOOK_PROGRAM_ID: Pubkey =
        solana_pubkey::pubkey!("2Bc7QG4A4sxTsEhefSRBQRVuWcgJvHA5jd4FcKZ5TDxm");
    static TOKEN_2022_PROGRAM_ID: Pubkey =
        solana_pubkey::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

    fn setup() -> (LiteSVM, Keypair) {
        let mut program = LiteSVM::new();
        let payer = Keypair::new();

        program
            .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        program
            .add_program_from_file(PROGRAM_ID, "../../target/deploy/vault.so")
            .expect("Failed to load vault program");

        program
            .add_program_from_file(
                TRANSFER_HOOK_PROGRAM_ID,
                "../../../whitelist-transfer-hook/target/deploy/whitelist_transfer_hook.so",
            )
            .expect("Failed to load transfer hook program");

        (program, payer)
    }

    struct VaultTestContext {
        program: LiteSVM,
        payer: Keypair,
        admin: Pubkey,
        vault_config: Pubkey,
    }

    impl VaultTestContext {
        fn new() -> Self {
            let (program, payer) = setup();
            let admin = payer.pubkey();

            let vault_config = Pubkey::find_program_address(&[b"vault_config"], &PROGRAM_ID).0;
            msg!("Vault Config PDA: {}\n", vault_config);

            Self {
                program,
                payer,
                admin,
                vault_config,
            }
        }

        fn execute_init_vault(&mut self) -> (Pubkey, Pubkey) {
            let mint = Keypair::new();
            let vault = spl_associated_token_account::get_associated_token_address_with_program_id(
                &self.vault_config,
                &mint.pubkey(),
                &TOKEN_2022_PROGRAM_ID,
            );

            msg!("Mint: {}", mint.pubkey());
            msg!("Vault: {}\n", vault);

            let init_vault_ix = Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::InitVault {
                    admin: self.admin,
                    vault_config: self.vault_config,
                    mint: mint.pubkey(),
                    transfer_hook_program: TRANSFER_HOOK_PROGRAM_ID,
                    vault,
                    associated_token_program: spl_associated_token_account::ID,
                    token_program: TOKEN_2022_PROGRAM_ID,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::InitializeVault {}.data(),
            };

            let message = Message::new(&[init_vault_ix], Some(&self.payer.pubkey()));
            let transaction = Transaction::new(
                &[&self.payer, &mint],
                message,
                self.program.latest_blockhash(),
            );
            let tx = self.program.send_transaction(transaction).unwrap();

            msg!("Init vault transaction successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}\n", tx.signature);

            (mint.pubkey(), vault)
        }

        fn execute_mint_token(&mut self, mint: &Pubkey, user: &Pubkey, amount: u64) -> Pubkey {
            let user_ata =
                spl_associated_token_account::get_associated_token_address_with_program_id(
                    user,
                    mint,
                    &TOKEN_2022_PROGRAM_ID,
                );

            let mint_token_ix = Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::MintToken {
                    admin: self.admin,
                    user: *user,
                    mint: *mint,
                    user_ata,
                    associated_token_program: spl_associated_token_account::ID,
                    token_program: TOKEN_2022_PROGRAM_ID,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::MintToken { amount }.data(),
            };

            let message = Message::new(&[mint_token_ix], Some(&self.payer.pubkey()));
            let transaction =
                Transaction::new(&[&self.payer], message, self.program.latest_blockhash());
            let tx = self.program.send_transaction(transaction).unwrap();

            msg!("Mint token transaction successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}\n", tx.signature);

            user_ata
        }

        fn execute_deposit(&mut self, user: &Keypair, mint: &Pubkey, vault: &Pubkey, amount: u64) {
            let amount_pda =
                Pubkey::find_program_address(&[b"amount", user.pubkey().as_ref()], &PROGRAM_ID).0;

            let user_ata =
                spl_associated_token_account::get_associated_token_address_with_program_id(
                    &user.pubkey(),
                    mint,
                    &TOKEN_2022_PROGRAM_ID,
                );

            let mut accounts = crate::accounts::Deposit {
                user: user.pubkey(),
                amount_pda,
                vault_config: self.vault_config,
                mint: *mint,
                user_ata,
                vault: *vault,
                associated_token_program: spl_associated_token_account::ID,
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None);

            // Add extra accounts for transfer hook (order matters!)
            let extra_account_meta_list = Pubkey::find_program_address(
                &[b"extra-account-metas", mint.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            let source_whitelist = Pubkey::find_program_address(
                &[b"whitelist", mint.as_ref(), user.pubkey().as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            let dest_whitelist = Pubkey::find_program_address(
                &[b"whitelist", mint.as_ref(), self.vault_config.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            // Required order for transfer hook: program, extra_meta_list, then resolved accounts
            accounts.push(AccountMeta::new_readonly(TRANSFER_HOOK_PROGRAM_ID, false));
            accounts.push(AccountMeta::new_readonly(extra_account_meta_list, false));
            accounts.push(AccountMeta::new_readonly(source_whitelist, false));
            accounts.push(AccountMeta::new_readonly(dest_whitelist, false));

            let deposit_ix = Instruction {
                program_id: PROGRAM_ID,
                accounts,
                data: crate::instruction::Deposit { amount }.data(),
            };

            let message = Message::new(&[deposit_ix], Some(&user.pubkey()));
            let transaction = Transaction::new(&[user], message, self.program.latest_blockhash());
            let tx = self.program.send_transaction(transaction).unwrap();

            msg!("Deposit transaction successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}\n", tx.signature);
        }

        fn execute_withdraw(&mut self, user: &Keypair, mint: &Pubkey, vault: &Pubkey, amount: u64) {
            let amount_pda =
                Pubkey::find_program_address(&[b"amount", user.pubkey().as_ref()], &PROGRAM_ID).0;

            let user_ata =
                spl_associated_token_account::get_associated_token_address_with_program_id(
                    &user.pubkey(),
                    mint,
                    &TOKEN_2022_PROGRAM_ID,
                );

            let mut accounts = crate::accounts::Withdraw {
                user: user.pubkey(),
                amount_pda,
                vault_config: self.vault_config,
                mint: *mint,
                user_ata,
                vault: *vault,
                associated_token_program: spl_associated_token_account::ID,
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None);

            // Add extra accounts for transfer hook (order matters!)
            let extra_account_meta_list = Pubkey::find_program_address(
                &[b"extra-account-metas", mint.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            let source_whitelist = Pubkey::find_program_address(
                &[b"whitelist", mint.as_ref(), self.vault_config.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            let dest_whitelist = Pubkey::find_program_address(
                &[b"whitelist", mint.as_ref(), user.pubkey().as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            // Required order for transfer hook: program, extra_meta_list, then resolved accounts
            accounts.push(AccountMeta::new_readonly(TRANSFER_HOOK_PROGRAM_ID, false));
            accounts.push(AccountMeta::new_readonly(extra_account_meta_list, false));
            accounts.push(AccountMeta::new_readonly(source_whitelist, false));
            accounts.push(AccountMeta::new_readonly(dest_whitelist, false));

            let withdraw_ix = Instruction {
                program_id: PROGRAM_ID,
                accounts,
                data: crate::instruction::Withdraw { amount }.data(),
            };

            let message = Message::new(&[withdraw_ix], Some(&user.pubkey()));
            let transaction = Transaction::new(&[user], message, self.program.latest_blockhash());
            let tx = self.program.send_transaction(transaction).unwrap();

            msg!("Withdraw transaction successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}\n", tx.signature);
        }

        fn get_token_balance(&mut self, ata: &Pubkey) -> u64 {
            let account = self.program.get_account(ata).unwrap();
            let amount_offset = 64;
            u64::from_le_bytes(
                account.data[amount_offset..amount_offset + 8]
                    .try_into()
                    .unwrap(),
            )
        }

        fn assert_account_closed(&mut self, pubkey: &Pubkey, name: &str) {
            if let Some(account) = self.program.get_account(pubkey) {
                assert_eq!(
                    account.lamports, 0,
                    "{} should have 0 lamports after closure",
                    name
                );
            }
        }

        // Transfer Hook Helper Methods
        fn execute_initialize_transfer_hook(&mut self, mint: &Pubkey) -> Pubkey {
            let extra_account_meta_list = Pubkey::find_program_address(
                &[b"extra-account-metas", mint.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            msg!("Extra Account Meta List: {}\n", extra_account_meta_list);

            // Calculate discriminator using Anchor's method
            let discriminator =
                anchor_lang::solana_program::hash::hash(b"global:initialize_transfer_hook");
            let mut data = vec![];
            data.extend_from_slice(&discriminator.to_bytes()[..8]);

            let init_hook_ix = Instruction {
                program_id: TRANSFER_HOOK_PROGRAM_ID,
                accounts: vec![
                    anchor_lang::prelude::AccountMeta::new(self.payer.pubkey(), true),
                    anchor_lang::prelude::AccountMeta::new(extra_account_meta_list, false),
                    anchor_lang::prelude::AccountMeta::new_readonly(*mint, false),
                    anchor_lang::prelude::AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                ],
                data,
            };

            let message = Message::new(&[init_hook_ix], Some(&self.payer.pubkey()));
            let transaction =
                Transaction::new(&[&self.payer], message, self.program.latest_blockhash());
            let tx = self.program.send_transaction(transaction).unwrap();

            msg!("Initialize transfer hook successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}\n", tx.signature);

            extra_account_meta_list
        }

        fn execute_add_to_whitelist(&mut self, mint: &Pubkey, user: &Pubkey) -> Pubkey {
            let whitelist_entry = Pubkey::find_program_address(
                &[b"whitelist", mint.as_ref(), user.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            msg!("Whitelisting: {} -> {}\n", user, whitelist_entry);

            // Calculate discriminator using Anchor's method
            let discriminator = anchor_lang::solana_program::hash::hash(b"global:add_to_whitelist");
            let mut data = vec![];
            data.extend_from_slice(&discriminator.to_bytes()[..8]);
            data.extend_from_slice(user.as_ref());

            let add_whitelist_ix = Instruction {
                program_id: TRANSFER_HOOK_PROGRAM_ID,
                accounts: vec![
                    anchor_lang::prelude::AccountMeta::new(self.payer.pubkey(), true),
                    anchor_lang::prelude::AccountMeta::new_readonly(*mint, false),
                    anchor_lang::prelude::AccountMeta::new(whitelist_entry, false),
                    anchor_lang::prelude::AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                ],
                data,
            };

            let message = Message::new(&[add_whitelist_ix], Some(&self.payer.pubkey()));
            let transaction =
                Transaction::new(&[&self.payer], message, self.program.latest_blockhash());
            let tx = self.program.send_transaction(transaction).unwrap();

            msg!("Add to whitelist successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}\n", tx.signature);

            whitelist_entry
        }

        fn execute_remove_from_whitelist(&mut self, mint: &Pubkey, user: &Pubkey) {
            let whitelist_entry = Pubkey::find_program_address(
                &[b"whitelist", mint.as_ref(), user.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            // Calculate discriminator using Anchor's method
            let discriminator =
                anchor_lang::solana_program::hash::hash(b"global:remove_from_whitelist");
            let mut data = vec![];
            data.extend_from_slice(&discriminator.to_bytes()[..8]);
            data.extend_from_slice(user.as_ref());

            let remove_whitelist_ix = Instruction {
                program_id: TRANSFER_HOOK_PROGRAM_ID,
                accounts: vec![
                    anchor_lang::prelude::AccountMeta::new(self.payer.pubkey(), true),
                    anchor_lang::prelude::AccountMeta::new_readonly(*mint, false),
                    anchor_lang::prelude::AccountMeta::new(whitelist_entry, false),
                ],
                data,
            };

            let message = Message::new(&[remove_whitelist_ix], Some(&self.payer.pubkey()));
            let transaction =
                Transaction::new(&[&self.payer], message, self.program.latest_blockhash());
            let tx = self.program.send_transaction(transaction).unwrap();

            msg!("Remove from whitelist successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}\n", tx.signature);
        }

        fn execute_emergency_transfer(
            &mut self,
            mint: &Pubkey,
            from_account: &Pubkey,
            to_account: &Pubkey,
            amount: u64,
        ) {
            // Get owners from token accounts
            let from_account_data = self.program.get_account(from_account).unwrap();
            let owner_offset = 32;
            let from_owner =
                Pubkey::try_from(&from_account_data.data[owner_offset..owner_offset + 32]).unwrap();

            let to_account_data = self.program.get_account(to_account).unwrap();
            let to_owner =
                Pubkey::try_from(&to_account_data.data[owner_offset..owner_offset + 32]).unwrap();

            let mut accounts = crate::accounts::EmergencyTransfer {
                admin: self.admin,
                vault_config: self.vault_config,
                mint: *mint,
                from_account: *from_account,
                to_account: *to_account,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None);

            // Add extra accounts for transfer hook (order matters!)
            let extra_account_meta_list = Pubkey::find_program_address(
                &[b"extra-account-metas", mint.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            let source_whitelist = Pubkey::find_program_address(
                &[b"whitelist", mint.as_ref(), from_owner.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            let dest_whitelist = Pubkey::find_program_address(
                &[b"whitelist", mint.as_ref(), to_owner.as_ref()],
                &TRANSFER_HOOK_PROGRAM_ID,
            )
            .0;

            // Required order for transfer hook: program, extra_meta_list, then resolved accounts
            accounts.push(AccountMeta::new_readonly(TRANSFER_HOOK_PROGRAM_ID, false));
            accounts.push(AccountMeta::new_readonly(extra_account_meta_list, false));
            accounts.push(AccountMeta::new_readonly(source_whitelist, false));
            accounts.push(AccountMeta::new_readonly(dest_whitelist, false));

            let emergency_transfer_ix = Instruction {
                program_id: PROGRAM_ID,
                accounts,
                data: crate::instruction::EmergencyTransfer { amount }.data(),
            };

            let message = Message::new(&[emergency_transfer_ix], Some(&self.payer.pubkey()));
            let transaction =
                Transaction::new(&[&self.payer], message, self.program.latest_blockhash());
            let tx = self.program.send_transaction(transaction).unwrap();

            msg!("Emergency transfer successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}\n", tx.signature);
        }
    }

    #[test]
    pub fn test_init_vault_with_hook() {
        let mut ctx = VaultTestContext::new();
        let (mint, vault) = ctx.execute_init_vault();

        // Initialize transfer hook
        ctx.execute_initialize_transfer_hook(&mint);

        // Whitelist the vault PDA (critical!)
        let vault_config = ctx.vault_config;
        ctx.execute_add_to_whitelist(&mint, &vault_config);

        let admin = ctx.admin;

        // Verify vault_config state
        let vault_config_account = ctx.program.get_account(&vault_config).unwrap();
        let vault_config_data =
            crate::state::VaultConfig::try_deserialize(&mut vault_config_account.data.as_ref())
                .unwrap();

        assert_eq!(vault_config_data.admin, admin);
        assert_eq!(vault_config_data.vault, vault);
        assert_eq!(vault_config_data.mint, mint);

        msg!("✓ Vault initialized successfully");
        msg!("✓ Transfer hook initialized");
        msg!("✓ Vault PDA whitelisted");
        msg!("✓ VaultConfig state verified");
    }

    // #[test]
    // pub fn test_deposit_with_whitelist() {
    //     let mut ctx = VaultTestContext::new();
    //     let (mint, vault) = ctx.execute_init_vault();

    //     // Initialize transfer hook and whitelist vault
    //     ctx.execute_initialize_transfer_hook(&mint);
    //     let vault_config = ctx.vault_config;
    //     ctx.execute_add_to_whitelist(&mint, &vault_config);

    //     let user = Keypair::new();
    //     ctx.program
    //         .airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
    //         .unwrap();
    //     msg!("User: {}\n", user.pubkey());

    //     // Whitelist user before they can interact with vault
    //     ctx.execute_add_to_whitelist(&mint, &user.pubkey());

    //     // Mint tokens to user
    //     let user_ata = ctx.execute_mint_token(&mint, &user.pubkey(), 1_000_000);
    //     let balance_before = ctx.get_token_balance(&user_ata);
    //     msg!("User balance before deposit: {}\n", balance_before);
    //     assert_eq!(balance_before, 1_000_000);

    //     // Deposit tokens
    //     let deposit_amount = 500_000u64;
    //     ctx.execute_deposit(&user, &mint, &vault, deposit_amount);

    //     // Verify balances
    //     let user_balance_after = ctx.get_token_balance(&user_ata);
    //     let vault_balance = ctx.get_token_balance(&vault);
    //     msg!("User balance after deposit: {}", user_balance_after);
    //     msg!("Vault balance: {}\n", vault_balance);

    //     assert_eq!(user_balance_after, 500_000);
    //     assert_eq!(vault_balance, 500_000);

    //     // Verify amount_pda state
    //     let amount_pda =
    //         Pubkey::find_program_address(&[b"amount", user.pubkey().as_ref()], &PROGRAM_ID).0;
    //     let amount_pda_account = ctx.program.get_account(&amount_pda).unwrap();
    //     let amount_pda_data =
    //         crate::state::Amount::try_deserialize(&mut amount_pda_account.data.as_ref()).unwrap();

    //     assert_eq!(amount_pda_data.amount, deposit_amount);
    //     msg!("✓ Deposit successful with whitelist validation");
    //     msg!("✓ Amount PDA tracking correct");
    // }

    // #[test]
    // pub fn test_withdraw_with_whitelist() {
    //     let mut ctx = VaultTestContext::new();
    //     let (mint, vault) = ctx.execute_init_vault();

    //     // Initialize hook and whitelist
    //     ctx.execute_initialize_transfer_hook(&mint);
    //     let vault_config = ctx.vault_config;
    //     ctx.execute_add_to_whitelist(&mint, &vault_config);

    //     let user = Keypair::new();
    //     ctx.program
    //         .airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
    //         .unwrap();

    //     ctx.execute_add_to_whitelist(&mint, &user.pubkey());

    //     // Mint and deposit tokens
    //     let user_ata = ctx.execute_mint_token(&mint, &user.pubkey(), 1_000_000);
    //     ctx.execute_deposit(&user, &mint, &vault, 800_000);

    //     let user_balance_before = ctx.get_token_balance(&user_ata);
    //     let vault_balance_before = ctx.get_token_balance(&vault);
    //     msg!("User balance before withdraw: {}", user_balance_before);
    //     msg!("Vault balance before withdraw: {}\n", vault_balance_before);

    //     // Withdraw tokens
    //     let withdraw_amount = 800_000u64;
    //     ctx.execute_withdraw(&user, &mint, &vault, withdraw_amount);

    //     // Verify balances
    //     let user_balance_after = ctx.get_token_balance(&user_ata);
    //     let vault_balance_after = ctx.get_token_balance(&vault);
    //     msg!("User balance after withdraw: {}", user_balance_after);
    //     msg!("Vault balance after withdraw: {}\n", vault_balance_after);

    //     assert_eq!(user_balance_after, 1_000_000);
    //     assert_eq!(vault_balance_after, 0);

    //     // Verify amount_pda is closed
    //     let amount_pda =
    //         Pubkey::find_program_address(&[b"amount", user.pubkey().as_ref()], &PROGRAM_ID).0;
    //     ctx.assert_account_closed(&amount_pda, "Amount PDA");

    //     msg!("✓ Withdraw successful with whitelist validation");
    //     msg!("✓ Amount PDA closed");
    // }

    // #[test]
    // pub fn test_multiple_deposits_with_whitelist() {
    //     let mut ctx = VaultTestContext::new();
    //     let (mint, vault) = ctx.execute_init_vault();

    //     // Initialize hook and whitelist
    //     ctx.execute_initialize_transfer_hook(&mint);
    //     let vault_config = ctx.vault_config;
    //     ctx.execute_add_to_whitelist(&mint, &vault_config);

    //     let user = Keypair::new();
    //     ctx.program
    //         .airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
    //         .unwrap();

    //     ctx.execute_add_to_whitelist(&mint, &user.pubkey());
    //     ctx.execute_mint_token(&mint, &user.pubkey(), 1_000_000);

    //     // First deposit
    //     ctx.execute_deposit(&user, &mint, &vault, 200_000);
    //     let amount_pda =
    //         Pubkey::find_program_address(&[b"amount", user.pubkey().as_ref()], &PROGRAM_ID).0;
    //     let amount_data_1 = ctx.program.get_account(&amount_pda).unwrap();
    //     let amount_1 =
    //         crate::state::Amount::try_deserialize(&mut amount_data_1.data.as_ref()).unwrap();
    //     assert_eq!(amount_1.amount, 200_000);

    //     // Second deposit
    //     ctx.execute_deposit(&user, &mint, &vault, 300_000);
    //     let amount_data_2 = ctx.program.get_account(&amount_pda).unwrap();
    //     let amount_2 =
    //         crate::state::Amount::try_deserialize(&mut amount_data_2.data.as_ref()).unwrap();
    //     assert_eq!(amount_2.amount, 500_000);

    //     let vault_balance = ctx.get_token_balance(&vault);
    //     assert_eq!(vault_balance, 500_000);

    //     msg!("✓ Multiple deposits accumulate correctly with whitelist");
    // }

    // #[test]
    // pub fn test_partial_withdraw_with_whitelist() {
    //     let mut ctx = VaultTestContext::new();
    //     let (mint, vault) = ctx.execute_init_vault();

    //     // Initialize hook and whitelist
    //     ctx.execute_initialize_transfer_hook(&mint);
    //     let vault_config = ctx.vault_config;
    //     ctx.execute_add_to_whitelist(&mint, &vault_config);

    //     let user = Keypair::new();
    //     ctx.program
    //         .airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
    //         .unwrap();

    //     ctx.execute_add_to_whitelist(&mint, &user.pubkey());

    //     let user_ata = ctx.execute_mint_token(&mint, &user.pubkey(), 1_000_000);
    //     ctx.execute_deposit(&user, &mint, &vault, 800_000);

    //     // Partial withdraw
    //     ctx.execute_withdraw(&user, &mint, &vault, 300_000);

    //     let user_balance = ctx.get_token_balance(&user_ata);
    //     let vault_balance = ctx.get_token_balance(&vault);

    //     assert_eq!(user_balance, 500_000);
    //     assert_eq!(vault_balance, 500_000);

    //     // Verify amount_pda is NOT closed (still has balance)
    //     let amount_pda =
    //         Pubkey::find_program_address(&[b"amount", user.pubkey().as_ref()], &PROGRAM_ID).0;
    //     let amount_data = ctx.program.get_account(&amount_pda).unwrap();
    //     let amount = crate::state::Amount::try_deserialize(&mut amount_data.data.as_ref()).unwrap();
    //     assert_eq!(amount.amount, 500_000);

    //     msg!("✓ Partial withdraw successful with whitelist");
    //     msg!("✓ Amount PDA still active");
    // }

    // #[test]
    // #[should_panic(expected = "SourceNotWhitelisted")]
    // pub fn test_deposit_fails_without_whitelist() {
    //     let mut ctx = VaultTestContext::new();
    //     let (mint, vault) = ctx.execute_init_vault();

    //     // Initialize hook and whitelist vault, but NOT the user
    //     ctx.execute_initialize_transfer_hook(&mint);
    //     let vault_config = ctx.vault_config;
    //     ctx.execute_add_to_whitelist(&mint, &vault_config);

    //     let user = Keypair::new();
    //     ctx.program
    //         .airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
    //         .unwrap();

    //     // User is NOT whitelisted
    //     ctx.execute_mint_token(&mint, &user.pubkey(), 1_000_000);

    //     // This should fail - user not whitelisted
    //     ctx.execute_deposit(&user, &mint, &vault, 500_000);
    // }

    // #[test]
    // #[should_panic(expected = "DestinationNotWhitelisted")]
    // pub fn test_withdraw_fails_after_removal_from_whitelist() {
    //     let mut ctx = VaultTestContext::new();
    //     let (mint, vault) = ctx.execute_init_vault();

    //     // Initialize hook and whitelist
    //     ctx.execute_initialize_transfer_hook(&mint);
    //     let vault_config = ctx.vault_config;
    //     ctx.execute_add_to_whitelist(&mint, &vault_config);

    //     let user = Keypair::new();
    //     ctx.program
    //         .airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
    //         .unwrap();

    //     ctx.execute_add_to_whitelist(&mint, &user.pubkey());
    //     ctx.execute_mint_token(&mint, &user.pubkey(), 1_000_000);
    //     ctx.execute_deposit(&user, &mint, &vault, 500_000);

    //     // Remove user from whitelist
    //     ctx.execute_remove_from_whitelist(&mint, &user.pubkey());

    //     // This should fail - user no longer whitelisted
    //     ctx.execute_withdraw(&user, &mint, &vault, 500_000);
    // }

    // #[test]
    // pub fn test_emergency_transfer_with_permanent_delegate() {
    //     let mut ctx = VaultTestContext::new();
    //     let (mint, _vault) = ctx.execute_init_vault();

    //     // Initialize hook and whitelist
    //     ctx.execute_initialize_transfer_hook(&mint);
    //     let vault_config = ctx.vault_config;
    //     ctx.execute_add_to_whitelist(&mint, &vault_config);

    //     let user1 = Keypair::new();
    //     let user2 = Keypair::new();

    //     ctx.program
    //         .airdrop(&user1.pubkey(), 10 * LAMPORTS_PER_SOL)
    //         .unwrap();
    //     ctx.program
    //         .airdrop(&user2.pubkey(), 10 * LAMPORTS_PER_SOL)
    //         .unwrap();

    //     // Whitelist both users
    //     ctx.execute_add_to_whitelist(&mint, &user1.pubkey());
    //     ctx.execute_add_to_whitelist(&mint, &user2.pubkey());

    //     // Mint tokens to user1
    //     let user1_ata = ctx.execute_mint_token(&mint, &user1.pubkey(), 1_000_000);
    //     let user2_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
    //         &user2.pubkey(),
    //         &mint,
    //         &TOKEN_2022_PROGRAM_ID,
    //     );

    //     // Create user2's ATA
    //     ctx.execute_mint_token(&mint, &user2.pubkey(), 0);

    //     let user1_balance_before = ctx.get_token_balance(&user1_ata);
    //     let user2_balance_before = ctx.get_token_balance(&user2_ata);

    //     msg!(
    //         "User1 balance before emergency transfer: {}",
    //         user1_balance_before
    //     );
    //     msg!(
    //         "User2 balance before emergency transfer: {}",
    //         user2_balance_before
    //     );

    //     // Admin uses permanent delegate to transfer from user1 to user2 (without user1's signature)
    //     let emergency_amount = 300_000u64;
    //     ctx.execute_emergency_transfer(&mint, &user1_ata, &user2_ata, emergency_amount);

    //     // Verify balances after emergency transfer
    //     let user1_balance_after = ctx.get_token_balance(&user1_ata);
    //     let user2_balance_after = ctx.get_token_balance(&user2_ata);

    //     msg!(
    //         "User1 balance after emergency transfer: {}",
    //         user1_balance_after
    //     );
    //     msg!(
    //         "User2 balance after emergency transfer: {}",
    //         user2_balance_after
    //     );

    //     assert_eq!(user1_balance_after, 700_000);
    //     assert_eq!(user2_balance_after, 300_000);

    //     msg!("✓ Emergency transfer successful using Permanent Delegate");
    //     msg!(
    //         "✓ Admin transferred {} tokens without user signature",
    //         emergency_amount
    //     );
    // }
}
