#[cfg(test)]
mod tests {

    use {
        anchor_lang::{
            prelude::msg, solana_program::program_pack::Pack, AccountDeserialize, InstructionData,
            ToAccountMetas,
        },
        anchor_spl::{
            associated_token::{self, spl_associated_token_account},
            token::spl_token,
        },
        litesvm::LiteSVM,
        litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
        },
        solana_account::Account,
        solana_address::Address,
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_rpc_client::rpc_client::RpcClient,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::str::FromStr,
    };

    static PROGRAM_ID: Pubkey = crate::ID;

    fn setup() -> (LiteSVM, Keypair) {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let payer = Keypair::new();

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Load program SO file
        program
            .add_program_from_file(PROGRAM_ID, "../../target/deploy/escrow_litesvm.so")
            .expect("Failed to load program");

        // Example on how to Load an account from devnet
        let rpc_client = RpcClient::new("https://api.devnet.solana.com");
        let account_address =
            Address::from_str("DRYvf71cbF2s5wgaJQvAGkghMkRcp5arvsK2w97vXhi2").unwrap();
        let fetched_account = rpc_client
            .get_account(&account_address)
            .expect("Failed to fetch account from devnet");

        program
            .set_account(
                payer.pubkey(),
                Account {
                    lamports: fetched_account.lamports,
                    data: fetched_account.data,
                    owner: Pubkey::from(fetched_account.owner.to_bytes()),
                    executable: fetched_account.executable,
                    rent_epoch: fetched_account.rent_epoch,
                },
            )
            .unwrap();

        msg!("Lamports of fetched account: {}", fetched_account.lamports);

        // Return the LiteSVM instance and payer keypair
        (program, payer)
    }

    struct EscrowTestContext {
        program: LiteSVM,
        payer: Keypair,
        maker: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        maker_ata_a: Pubkey,
        escrow: Pubkey,
        vault: Pubkey,
        seed: u64,
    }

    impl EscrowTestContext {
        fn new() -> Self {
            let (mut program, payer) = setup();
            let maker = payer.pubkey();
            let seed = 123u64;

            // Create mints
            let mint_a = CreateMint::new(&mut program, &payer)
                .decimals(6)
                .authority(&maker)
                .send()
                .unwrap();
            msg!("Mint A: {}\n", mint_a);

            let mint_b = CreateMint::new(&mut program, &payer)
                .decimals(6)
                .authority(&maker)
                .send()
                .unwrap();
            msg!("Mint B: {}\n", mint_b);

            // Create maker's ATA
            let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
                .owner(&maker)
                .send()
                .unwrap();
            msg!("Maker ATA A: {}\n", maker_ata_a);

            // Derive PDAs
            let escrow = Pubkey::find_program_address(
                &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
                &PROGRAM_ID,
            )
            .0;
            msg!("Escrow PDA: {}\n", escrow);

            let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
            msg!("Vault PDA: {}\n", vault);

            // Mint initial tokens to maker
            MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1_000_000_000)
                .send()
                .unwrap();

            Self {
                program,
                payer,
                maker,
                mint_a,
                mint_b,
                maker_ata_a,
                escrow,
                vault,
                seed,
            }
        }

        fn execute_make(&mut self, deposit: u64, receive: u64, min_accept_lockin_time: i64) {
            let make_ix = Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::Make {
                    maker: self.maker,
                    mint_a: self.mint_a,
                    mint_b: self.mint_b,
                    maker_ata_a: self.maker_ata_a,
                    escrow: self.escrow,
                    vault: self.vault,
                    associated_token_program: spl_associated_token_account::ID,
                    token_program: TOKEN_PROGRAM_ID,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::Make {
                    deposit,
                    seed: self.seed,
                    receive,
                    min_accept_lockin_time,
                }
                .data(),
            };

            let message = Message::new(&[make_ix], Some(&self.payer.pubkey()));
            let transaction =
                Transaction::new(&[&self.payer], message, self.program.latest_blockhash());
            self.program.send_transaction(transaction).unwrap();
            msg!("Make transaction successful\n");
        }

        fn execute_refund(&mut self) {
            let refund_ix = Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::Refund {
                    maker: self.maker,
                    mint_a: self.mint_a,
                    maker_ata_a: self.maker_ata_a,
                    escrow: self.escrow,
                    vault: self.vault,
                    token_program: TOKEN_PROGRAM_ID,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::Refund {}.data(),
            };

            let message = Message::new(&[refund_ix], Some(&self.payer.pubkey()));
            let transaction =
                Transaction::new(&[&self.payer], message, self.program.latest_blockhash());
            let tx = self.program.send_transaction(transaction).unwrap();
            msg!("\n\nRefund transaction successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}", tx.signature);
        }

        fn get_token_balance(&mut self, ata: &Pubkey) -> u64 {
            let account = self.program.get_account(ata).unwrap();
            let token_account = spl_token::state::Account::unpack(&account.data).unwrap();
            token_account.amount
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

        fn execute_take(
            &mut self,
            taker: &Keypair,
            taker_ata_a: Pubkey,
            taker_ata_b: Pubkey,
            maker_ata_b: Pubkey,
        ) -> Result<(), String> {
            let take_ix = Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::Take {
                    taker: taker.pubkey(),
                    maker: self.maker,
                    mint_a: self.mint_a,
                    mint_b: self.mint_b,
                    taker_ata_a,
                    taker_ata_b,
                    maker_ata_b,
                    escrow: self.escrow,
                    vault: self.vault,
                    associated_token_program: spl_associated_token_account::ID,
                    token_program: TOKEN_PROGRAM_ID,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::Take {}.data(),
            };

            let message = Message::new(&[take_ix], Some(&taker.pubkey()));
            let transaction = Transaction::new(&[taker], message, self.program.latest_blockhash());
            let tx = self
                .program
                .send_transaction(transaction)
                .map_err(|e| format!("{:?}", e))?;

            msg!("\n\nTake transaction successful");
            msg!("CUs Consumed: {}", tx.compute_units_consumed);
            msg!("Tx Signature: {}", tx.signature);

            Ok(())
        }
    }

    #[test]
    pub fn test_make() {
        let mut ctx = EscrowTestContext::new();

        let lock_time = 0i64; // No lock for this test
        ctx.execute_make(10_000_000, 10_000_000, lock_time);

        // Copy pubkeys to avoid borrow issues
        let vault = ctx.vault;
        let escrow = ctx.escrow;
        let maker = ctx.maker;
        let mint_a = ctx.mint_a;
        let mint_b = ctx.mint_b;

        // Verify vault
        let vault_balance = ctx.get_token_balance(&vault);
        assert_eq!(vault_balance, 10_000_000);

        let vault_account = ctx.program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, mint_a);

        // Verify escrow state
        let escrow_account = ctx.program.get_account(&escrow).unwrap();
        let escrow_data =
            crate::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
        assert_eq!(escrow_data.seed, 123u64);
        assert_eq!(escrow_data.maker, maker);
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, 10_000_000);
        assert_eq!(escrow_data.min_accept_lockin_time, lock_time);
    }

    #[test]
    pub fn test_take() {
        let mut ctx = EscrowTestContext::new();

        let lock_time = 0i64; // No lock for this test
        ctx.execute_make(100_000_000, 50_000_000, lock_time);

        // Create taker
        let taker = Keypair::new();
        ctx.program
            .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .unwrap();
        msg!("Taker: {}\n", taker.pubkey());

        // Create taker's ATAs
        let taker_ata_a =
            associated_token::get_associated_token_address(&taker.pubkey(), &ctx.mint_a);
        msg!("Taker ATA A: {}\n", taker_ata_a);

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut ctx.program, &taker, &ctx.mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        msg!("Taker ATA B: {}\n", taker_ata_b);

        let maker_ata_b = associated_token::get_associated_token_address(&ctx.maker, &ctx.mint_b);
        msg!("Maker ATA B: {}\n", maker_ata_b);

        // Mint tokens to taker
        MintTo::new(
            &mut ctx.program,
            &ctx.payer,
            &ctx.mint_b,
            &taker_ata_b,
            50_000_000,
        )
        .send()
        .unwrap();

        // Execute take instruction
        ctx.execute_take(&taker, taker_ata_a, taker_ata_b, maker_ata_b)
            .unwrap();

        // Verify balances
        assert_eq!(ctx.get_token_balance(&taker_ata_a), 100_000_000);
        assert_eq!(ctx.get_token_balance(&maker_ata_b), 50_000_000);
        assert_eq!(ctx.get_token_balance(&taker_ata_b), 0);

        // Copy pubkeys to avoid borrow issues
        let vault = ctx.vault;
        let escrow = ctx.escrow;

        // Verify accounts closed
        ctx.assert_account_closed(&vault, "Vault");
        ctx.assert_account_closed(&escrow, "Escrow");
    }

    #[test]
    pub fn test_refund() {
        let mut ctx = EscrowTestContext::new();

        let lock_time = 0i64;
        ctx.execute_make(100_000_000, 50_000_000, lock_time);

        let maker_ata_a = ctx.maker_ata_a;
        let balance_before = ctx.get_token_balance(&maker_ata_a);
        msg!("Maker ATA A balance before refund: {}\n", balance_before);

        ctx.execute_refund();

        let balance_after = ctx.get_token_balance(&maker_ata_a);
        assert_eq!(balance_after, balance_before + 100_000_000);
        msg!("Maker ATA A balance after refund: {}\n", balance_after);

        // Copy pubkeys to avoid borrow issues
        let vault = ctx.vault;
        let escrow = ctx.escrow;

        ctx.assert_account_closed(&vault, "Vault");
        ctx.assert_account_closed(&escrow, "Escrow");
    }

    #[test]
    pub fn test_take_with_time_lock_fails_before_unlock() {
        let mut ctx = EscrowTestContext::new();

        // Set lock time to far future (2 hours from epoch ~1970)
        // This ensures current time will always be less than lock time
        let lock_time = i64::MAX - 1000;

        ctx.execute_make(100_000_000, 50_000_000, lock_time);

        // Create taker
        let taker = Keypair::new();
        ctx.program
            .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .unwrap();

        // Create taker's ATAs
        let taker_ata_a =
            associated_token::get_associated_token_address(&taker.pubkey(), &ctx.mint_a);

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut ctx.program, &taker, &ctx.mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();

        let maker_ata_b = associated_token::get_associated_token_address(&ctx.maker, &ctx.mint_b);

        // Mint tokens to taker
        MintTo::new(
            &mut ctx.program,
            &ctx.payer,
            &ctx.mint_b,
            &taker_ata_b,
            50_000_000,
        )
        .send()
        .unwrap();

        // Attempt to take before time lock expires - should fail
        let result = ctx.execute_take(&taker, taker_ata_a, taker_ata_b, maker_ata_b);

        assert!(
            result.is_err(),
            "Take should fail when time lock not elapsed"
        );
        msg!("✓ Take correctly failed before time lock elapsed");
    }

    #[test]
    pub fn test_take_with_time_lock_succeeds_after_unlock() {
        let mut ctx = EscrowTestContext::new();

        // Set lock time to 0 (always in the past, already elapsed)
        let lock_time = 0i64;

        ctx.execute_make(100_000_000, 50_000_000, lock_time);

        // Create taker
        let taker = Keypair::new();
        ctx.program
            .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .unwrap();

        // Create taker's ATAs
        let taker_ata_a =
            associated_token::get_associated_token_address(&taker.pubkey(), &ctx.mint_a);

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut ctx.program, &taker, &ctx.mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();

        let maker_ata_b = associated_token::get_associated_token_address(&ctx.maker, &ctx.mint_b);

        // Mint tokens to taker
        MintTo::new(
            &mut ctx.program,
            &ctx.payer,
            &ctx.mint_b,
            &taker_ata_b,
            50_000_000,
        )
        .send()
        .unwrap();

        // Take after time lock elapsed - should succeed
        ctx.execute_take(&taker, taker_ata_a, taker_ata_b, maker_ata_b)
            .unwrap();

        // Verify balances
        assert_eq!(ctx.get_token_balance(&taker_ata_a), 100_000_000);
        assert_eq!(ctx.get_token_balance(&maker_ata_b), 50_000_000);
        assert_eq!(ctx.get_token_balance(&taker_ata_b), 0);

        msg!("✓ Take succeeded after time lock elapsed");
    }
}
