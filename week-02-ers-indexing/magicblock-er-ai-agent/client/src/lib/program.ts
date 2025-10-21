import { Program, AnchorProvider } from '@coral-xyz/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import idl from '../types/magicblock_er_ai_agent.json';

export const PROGRAM_ID = new PublicKey("53GFYSJPbrYcaqD3o54z5WCWcCM8WGqixgUjc4nsw2tY");

export const useProgram = () => {
    const { wallet, publicKey } = useWallet();

    if (!wallet || !publicKey) return null;

    const connection = new Connection("https://api.devnet.solana.com");

    const anchorWallet = {
        publicKey: publicKey,
        signTransaction: async (tx: any) => {
            if (!(wallet.adapter as any)?.signTransaction) {
                throw new Error('Wallet does not support signTransaction');
            }
            return await (wallet.adapter as any).signTransaction(tx);
        },
        signAllTransactions: async (txs: any[]) => {
            if (!(wallet.adapter as any)?.signAllTransactions) {
                throw new Error('Wallet does not support signAllTransactions');
            }
            return await (wallet.adapter as any).signAllTransactions(txs);
        },
    };

    const provider = new AnchorProvider(connection, anchorWallet as any, {
        preflightCommitment: 'confirmed',
        commitment: 'confirmed',
    });

    return new Program(
        idl as any,
        provider
    );
};

export const deriveAnalysisPDA = (userPubkey: PublicKey) => {
    return PublicKey.findProgramAddressSync(
        [Buffer.from("analysis"), userPubkey.toBuffer()],
        PROGRAM_ID
    );
};

export const deriveAgentPDA = () => {
    return PublicKey.findProgramAddressSync(
        [Buffer.from("agent")],
        PROGRAM_ID
    );
};

export const ORACLE_PROGRAM_ID = new PublicKey("LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab");

export const deriveOracleCounterPDA = () => {
    return PublicKey.findProgramAddressSync(
        [Buffer.from("counter")],
        ORACLE_PROGRAM_ID
    );
};

export const deriveOracleLlmContextPDA = async (connection: any, counterValue?: number) => {
    let actualCounterValue = counterValue;

    if (actualCounterValue === undefined) {
        const [oracleCounterPDA] = deriveOracleCounterPDA();
        try {
            const counterAccount = await connection.getAccountInfo(oracleCounterPDA);
            if (counterAccount) {
                actualCounterValue = counterAccount.data.readUInt32LE(8);
            } else {
                actualCounterValue = 20;
            }
        } catch (error) {
            console.log('Error fetching counter, using 20:', error);
            actualCounterValue = 20;
        }
    }

    const counterBytes = Buffer.alloc(4);
    counterBytes.writeUInt32LE(actualCounterValue!, 0);
    return PublicKey.findProgramAddressSync(
        [Buffer.from("test-context"), counterBytes],
        ORACLE_PROGRAM_ID
    );
};

// Derive the Interaction PDA for the oracle program
// Based on MagicBlock repository: [Interaction::seed(), payer.key().as_ref(), context_account.key().as_ref()]
export const deriveOracleInteractionPDA = (payer: PublicKey, contextAccount: PublicKey) => {
    return PublicKey.findProgramAddressSync(
        [Buffer.from("interaction"), payer.toBuffer(), contextAccount.toBuffer()],
        ORACLE_PROGRAM_ID
    );
};