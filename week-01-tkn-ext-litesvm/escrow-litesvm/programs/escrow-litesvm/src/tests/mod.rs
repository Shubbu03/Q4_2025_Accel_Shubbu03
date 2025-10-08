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

    #[test]
    pub fn test_make() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (mut program, payer) = setup();

        // Get the maker's public key from the payer keypair
        let maker = payer.pubkey();

        // Create two mints (Mint A and Mint B) with 6 decimal places and the maker as the authority
        let mint_a = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint A: {:?}\n", mint_a);

        let mint_b = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint B: {}\n", mint_b);

        // Create the maker's associated token account for Mint A
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&maker)
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1_000_000_000)
            .send()
            .unwrap();

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                deposit: 10_000_000,
                seed: 123u64,
                receive: 10_000_000,
            }
            .data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify the vault account and escrow account data after the "Make" instruction
        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10_000_000);
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, mint_a);

        let escrow_account = program.get_account(&escrow).unwrap();
        let escrow_data =
            crate::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
        assert_eq!(escrow_data.seed, 123u64);
        assert_eq!(escrow_data.maker, maker);
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, 10_000_000);
    }

    #[test]
    pub fn test_take() {
        let (mut program, payer) = setup();

        let maker = payer.pubkey();

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

        // Create maker's ATA for mint_a
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&maker)
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // Derive escrow and vault PDAs
        let seed = 123u64;
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        let associated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1000 tokens of mint_a to maker
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        // Execute make instruction to create escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker,
                mint_a,
                mint_b,
                maker_ata_a,
                escrow,
                vault,
                associated_token_program,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                deposit: 100_000_000,
                seed,
                receive: 50_000_000,
            }
            .data(),
        };

        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let transaction = Transaction::new(&[&payer], message, program.latest_blockhash());
        program.send_transaction(transaction).unwrap();
        msg!("Make transaction successful\n");

        // Create taker
        let taker = Keypair::new();
        program
            .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .unwrap();
        msg!("Taker: {}\n", taker.pubkey());

        // Create taker's ATAs for both mints
        let taker_ata_a = associated_token::get_associated_token_address(&taker.pubkey(), &mint_a);
        msg!("Taker ATA A: {}\n", taker_ata_a);

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        msg!("Taker ATA B: {}\n", taker_ata_b);

        // Maker's ATA for mint_b (will be created by take instruction)
        let maker_ata_b = associated_token::get_associated_token_address(&maker, &mint_b);
        msg!("Maker ATA B: {}\n", maker_ata_b);

        // Mint 50 tokens of mint_b to taker
        MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 50_000_000)
            .send()
            .unwrap();

        // Execute take instruction
        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Take {
                taker: taker.pubkey(),
                maker,
                mint_a,
                mint_b,
                taker_ata_a,
                taker_ata_b,
                maker_ata_b,
                escrow,
                vault,
                associated_token_program,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Take {}.data(),
        };

        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let transaction = Transaction::new(&[&taker], message, program.latest_blockhash());
        let tx = program.send_transaction(transaction).unwrap();

        msg!("\n\nTake transaction successful");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify taker received mint_a tokens
        let taker_ata_a_account = program.get_account(&taker_ata_a).unwrap();
        let taker_ata_a_data =
            spl_token::state::Account::unpack(&taker_ata_a_account.data).unwrap();
        assert_eq!(taker_ata_a_data.amount, 100_000_000);
        assert_eq!(taker_ata_a_data.owner, taker.pubkey());

        // Verify maker received mint_b tokens
        let maker_ata_b_account = program.get_account(&maker_ata_b).unwrap();
        let maker_ata_b_data =
            spl_token::state::Account::unpack(&maker_ata_b_account.data).unwrap();
        assert_eq!(maker_ata_b_data.amount, 50_000_000);
        assert_eq!(maker_ata_b_data.owner, maker);

        // Verify taker spent mint_b tokens
        let taker_ata_b_account = program.get_account(&taker_ata_b).unwrap();
        let taker_ata_b_data =
            spl_token::state::Account::unpack(&taker_ata_b_account.data).unwrap();
        assert_eq!(taker_ata_b_data.amount, 0);

        // Verify vault is closed (lamports should be 0)
        let vault_after = program.get_account(&vault);
        if let Some(vault_account) = vault_after {
            assert_eq!(
                vault_account.lamports, 0,
                "Vault should have 0 lamports after closure"
            );
        }

        // Verify escrow is closed (lamports should be 0)
        let escrow_after = program.get_account(&escrow);
        if let Some(escrow_account) = escrow_after {
            assert_eq!(
                escrow_account.lamports, 0,
                "Escrow should have 0 lamports after closure"
            );
        }
    }

    #[test]
    pub fn test_refund() {
        // first we'll do make ix, then we do refund

        let (mut program, payer) = setup();

        let maker = payer.pubkey();

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

        // Create maker's ATA for mint_a
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&maker)
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // Derive escrow and vault PDAs
        let seed = 123u64;
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        let associated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1000 tokens of mint_a to maker
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        // Execute make instruction to create escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker,
                mint_a,
                mint_b,
                maker_ata_a,
                escrow,
                vault,
                associated_token_program,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                deposit: 100_000_000,
                seed,
                receive: 50_000_000,
            }
            .data(),
        };

        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let transaction = Transaction::new(&[&payer], message, program.latest_blockhash());
        program.send_transaction(transaction).unwrap();
        msg!("Make transaction successful\n");

        // Get maker's token balance before refund
        let maker_ata_a_before = program.get_account(&maker_ata_a).unwrap();
        let maker_balance_before =
            spl_token::state::Account::unpack(&maker_ata_a_before.data).unwrap();
        msg!(
            "Maker ATA A balance before refund: {}\n",
            maker_balance_before.amount
        );

        // Execute refund instruction
        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Refund {
                maker,
                mint_a,
                maker_ata_a,
                escrow,
                vault,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Refund {}.data(),
        };

        let message = Message::new(&[refund_ix], Some(&payer.pubkey()));
        let transaction = Transaction::new(&[&payer], message, program.latest_blockhash());
        let tx = program.send_transaction(transaction).unwrap();

        msg!("\n\nRefund transaction successful");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify maker received tokens back
        let maker_ata_a_after = program.get_account(&maker_ata_a).unwrap();
        let maker_balance_after =
            spl_token::state::Account::unpack(&maker_ata_a_after.data).unwrap();
        assert_eq!(
            maker_balance_after.amount,
            maker_balance_before.amount + 100_000_000
        );
        msg!(
            "Maker ATA A balance after refund: {}\n",
            maker_balance_after.amount
        );

        // Verify vault is closed (lamports should be 0)
        let vault_after = program.get_account(&vault);
        if let Some(vault_account) = vault_after {
            assert_eq!(
                vault_account.lamports, 0,
                "Vault should have 0 lamports after closure"
            );
        }

        // Verify escrow is closed (lamports should be 0)
        let escrow_after = program.get_account(&escrow);
        if let Some(escrow_account) = escrow_after {
            assert_eq!(
                escrow_account.lamports, 0,
                "Escrow should have 0 lamports after closure"
            );
        }
    }
}
