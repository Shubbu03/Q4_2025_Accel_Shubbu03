
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
    TOKEN_2022_PROGRAM_ID,
    getAssociatedTokenAddressSync,
    createInitializeMintInstruction,
    getMintLen,
    ExtensionType,
    createTransferCheckedWithTransferHookInstruction,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    createInitializeTransferHookInstruction,
    createAssociatedTokenAccountInstruction,
    createMintToInstruction,
} from "@solana/spl-token";
import {
    PublicKey,
    SystemProgram,
    Transaction,
    SendTransactionError,
    sendAndConfirmTransaction,
} from "@solana/web3.js";
import { WhitelistTransferHook } from "../target/types/whitelist_transfer_hook";

describe("whitelist-transfer-hook (PDA-based whitelist)", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const wallet = provider.wallet as anchor.Wallet;
    const program = anchor.workspace.whitelistTransferHook as Program<WhitelistTransferHook>;

    const mint2022 = anchor.web3.Keypair.generate();

    const pdaForExtraAccountMetaList = (mint: PublicKey) =>
        PublicKey.findProgramAddressSync(
            [Buffer.from("extra-account-metas"), mint.toBuffer()],
            program.programId
        )[0];

    const pdaForWhitelistEntry = (mint: PublicKey, user: PublicKey) =>
        PublicKey.findProgramAddressSync(
            [Buffer.from("whitelist"), mint.toBuffer(), user.toBuffer()],
            program.programId
        )[0];

    // ATAs
    const sourceTokenAccount = getAssociatedTokenAddressSync(
        mint2022.publicKey,
        wallet.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const recipient = anchor.web3.Keypair.generate();
    const destinationTokenAccount = getAssociatedTokenAddressSync(
        mint2022.publicKey,
        recipient.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    it("Creates mint with TransferHook extension", async () => {
        const extensions = [ExtensionType.TransferHook];
        const mintLen = getMintLen(extensions);
        const lamports = await provider.connection.getMinimumBalanceForRentExemption(mintLen);

        const tx = new Transaction().add(
            SystemProgram.createAccount({
                fromPubkey: wallet.publicKey,
                newAccountPubkey: mint2022.publicKey,
                space: mintLen,
                lamports,
                programId: TOKEN_2022_PROGRAM_ID,
            }),

            createInitializeTransferHookInstruction(
                mint2022.publicKey,
                wallet.publicKey,
                program.programId,
                TOKEN_2022_PROGRAM_ID
            ),
            createInitializeMintInstruction(
                mint2022.publicKey,
                9,
                wallet.publicKey,
                null,
                TOKEN_2022_PROGRAM_ID
            )
        );

        const sig = await sendAndConfirmTransaction(provider.connection, tx, [wallet.payer, mint2022], {
            skipPreflight: true,
            commitment: "finalized",
        });
        console.log("Mint+TransferHook initialized:", sig);
    });

    it("Creates ATAs and mints tokens", async () => {
        const amount = 100n * 10n ** 9n;

        const tx = new Transaction().add(
            createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                sourceTokenAccount,
                wallet.publicKey,
                mint2022.publicKey,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID
            ),
            createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                destinationTokenAccount,
                recipient.publicKey,
                mint2022.publicKey,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID
            ),
            createMintToInstruction(
                mint2022.publicKey,
                sourceTokenAccount,
                wallet.publicKey,
                Number(amount),
                [],
                TOKEN_2022_PROGRAM_ID
            )
        );

        const sig = await sendAndConfirmTransaction(provider.connection, tx, [wallet.payer], {
            skipPreflight: true,
        });
        console.log("ATAs created and tokens minted:", sig);
    });

    it("Initializes ExtraAccountMetaList PDA for the mint", async () => {
        const extraAccountMetaListPDA = pdaForExtraAccountMetaList(mint2022.publicKey);

        const ix = await program.methods
            .initializeTransferHook()
            .accountsPartial({
                payer: wallet.publicKey,
                mint: mint2022.publicKey,
                extraAccountMetaList: extraAccountMetaListPDA,
                systemProgram: SystemProgram.programId,
            })
            .instruction();

        const tx = new Transaction().add(ix);
        const sig = await sendAndConfirmTransaction(provider.connection, tx, [wallet.payer], {
            skipPreflight: true,
            commitment: "confirmed",
        });
        console.log("ExtraAccountMetaList initialized:", extraAccountMetaListPDA.toBase58(), sig);
    });

    it("Fails transfer before whitelisting", async () => {
        const amount = 1n * 10n ** 9n;

        const transferIx = await createTransferCheckedWithTransferHookInstruction(
            provider.connection,
            sourceTokenAccount,
            mint2022.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            amount,
            9,
            [],
            "confirmed",
            TOKEN_2022_PROGRAM_ID
        );

        const tx = new Transaction().add(transferIx);

        let failedAsExpected = false;
        try {
            await sendAndConfirmTransaction(provider.connection, tx, [wallet.payer], {
                skipPreflight: false,
            });
        } catch (e) {
            failedAsExpected = true;
            if (e instanceof SendTransactionError) {
                console.log("Expected failure before whitelist:", e.logs?.find(Boolean) ?? "no logs");
            } else {
                console.log("Expected failure (non-SendTransactionError):", e);
            }
        }
        if (!failedAsExpected) {
            throw new Error("Transfer unexpectedly succeeded before whitelisting");
        }
    });

    it("Adds wallet to mint-specific whitelist (PDA) and transfers successfully", async () => {
        const whitelistPDA = pdaForWhitelistEntry(mint2022.publicKey, wallet.publicKey);

        const addIx = await program.methods
            .addToWhitelist(wallet.publicKey)
            .accountsPartial({
                admin: wallet.publicKey,
                mint: mint2022.publicKey,
                whitelistEntry: whitelistPDA,
                systemProgram: SystemProgram.programId,
            })
            .instruction();

        const addTx = new Transaction().add(addIx);
        await sendAndConfirmTransaction(provider.connection, addTx, [wallet.payer], {
            skipPreflight: true,
        });

        const amount = 1n * 10n ** 9n;
        const transferIx = await createTransferCheckedWithTransferHookInstruction(
            provider.connection,
            sourceTokenAccount,
            mint2022.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            amount,
            9,
            [],
            "confirmed",
            TOKEN_2022_PROGRAM_ID
        );

        const tx = new Transaction().add(transferIx);
        const sig = await sendAndConfirmTransaction(provider.connection, tx, [wallet.payer], {
            skipPreflight: false,
        });
        console.log("Transfer succeeded after whitelist:", sig);
    });

    it("Removes wallet from whitelist and transfer fails again", async () => {
        const whitelistPDA = pdaForWhitelistEntry(mint2022.publicKey, wallet.publicKey);

        const removeIx = await program.methods
            .removeFromWhitelist(wallet.publicKey)
            .accountsPartial({
                admin: wallet.publicKey,
                mint: mint2022.publicKey,
                whitelistEntry: whitelistPDA,
            })
            .instruction();

        const removeTx = new Transaction().add(removeIx);
        await sendAndConfirmTransaction(provider.connection, removeTx, [wallet.payer], {
            skipPreflight: true,
        });

        const amount = 1n * 10n ** 9n;
        const transferIx = await createTransferCheckedWithTransferHookInstruction(
            provider.connection,
            sourceTokenAccount,
            mint2022.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            amount,
            9,
            [],
            "confirmed",
            TOKEN_2022_PROGRAM_ID
        );

        const tx = new Transaction().add(transferIx);

        let failedAsExpected = false;
        try {
            await sendAndConfirmTransaction(provider.connection, tx, [wallet.payer], {
                skipPreflight: false,
            });
        } catch (e) {
            failedAsExpected = true;
            if (e instanceof SendTransactionError) {
                console.log("Expected failure after removal:", e.logs?.find(Boolean) ?? "no logs");
            } else {
                console.log("Expected failure (non-SendTransactionError):", e);
            }
        }
        if (!failedAsExpected) {
            throw new Error("Transfer unexpectedly succeeded after whitelist removal");
        }
    });
});

describe("whitelist-transfer-hook (using init_token_factory)", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const wallet = provider.wallet as anchor.Wallet;
    const program = anchor.workspace.whitelistTransferHook as Program<WhitelistTransferHook>;

    const mint = anchor.web3.Keypair.generate();

    const pdaForExtraAccountMetaList = (mint: PublicKey) =>
        PublicKey.findProgramAddressSync(
            [Buffer.from("extra-account-metas"), mint.toBuffer()],
            program.programId
        )[0];

    const pdaForWhitelistEntry = (mint: PublicKey, user: PublicKey) =>
        PublicKey.findProgramAddressSync(
            [Buffer.from("whitelist"), mint.toBuffer(), user.toBuffer()],
            program.programId
        )[0];

    const sourceTokenAccount = getAssociatedTokenAddressSync(
        mint.publicKey,
        wallet.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const recipient = anchor.web3.Keypair.generate();
    const destinationTokenAccount = getAssociatedTokenAddressSync(
        mint.publicKey,
        recipient.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    it("Creates mint using init_token_factory instruction", async () => {
        const sig = await program.methods
            .initTokenFactory()
            .accountsPartial({
                user: wallet.publicKey,
                mint: mint.publicKey,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_2022_PROGRAM_ID,
            })
            .signers([mint])
            .rpc({ skipPreflight: true, commitment: "confirmed" });

        console.log("Mint created via init_token_factory:", sig);
        console.log("Mint address:", mint.publicKey.toBase58());
    });

    it("Initializes ExtraAccountMetaList for the new mint", async () => {
        const extraAccountMetaListPDA = pdaForExtraAccountMetaList(mint.publicKey);

        const sig = await program.methods
            .initializeTransferHook()
            .accountsPartial({
                payer: wallet.publicKey,
                mint: mint.publicKey,
                extraAccountMetaList: extraAccountMetaListPDA,
                systemProgram: SystemProgram.programId,
            })
            .rpc({ skipPreflight: true, commitment: "confirmed" });

        console.log("ExtraAccountMetaList initialized:", sig);
    });

    it("Creates ATAs and mints tokens", async () => {
        const amount = 100n * 10n ** 9n;

        const tx = new Transaction().add(
            createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                sourceTokenAccount,
                wallet.publicKey,
                mint.publicKey,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID
            ),
            createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                destinationTokenAccount,
                recipient.publicKey,
                mint.publicKey,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID
            ),
            createMintToInstruction(
                mint.publicKey,
                sourceTokenAccount,
                wallet.publicKey,
                Number(amount),
                [],
                TOKEN_2022_PROGRAM_ID
            )
        );

        const sig = await sendAndConfirmTransaction(provider.connection, tx, [wallet.payer], {
            skipPreflight: true,
        });
        console.log("ATAs created and tokens minted:", sig);
    });

    it("Adds wallet to whitelist and transfers successfully", async () => {
        const whitelistPDA = pdaForWhitelistEntry(mint.publicKey, wallet.publicKey);

        await program.methods
            .addToWhitelist(wallet.publicKey)
            .accountsPartial({
                admin: wallet.publicKey,
                mint: mint.publicKey,
                whitelistEntry: whitelistPDA,
                systemProgram: SystemProgram.programId,
            })
            .rpc({ skipPreflight: true });

        const amount = 1n * 10n ** 9n;
        const transferIx = await createTransferCheckedWithTransferHookInstruction(
            provider.connection,
            sourceTokenAccount,
            mint.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            amount,
            9,
            [],
            "confirmed",
            TOKEN_2022_PROGRAM_ID
        );

        const tx = new Transaction().add(transferIx);
        const sig = await sendAndConfirmTransaction(provider.connection, tx, [wallet.payer], {
            skipPreflight: false,
        });
        console.log("Transfer succeeded with whitelist:", sig);
    });
});