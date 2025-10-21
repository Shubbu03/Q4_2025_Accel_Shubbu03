'use client';

import React, { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import {
    useProgram,
    deriveAgentPDA,
    deriveAnalysisPDA,
    deriveOracleCounterPDA,
    deriveOracleLlmContextPDA,
    ORACLE_PROGRAM_ID
} from '../lib/program';

const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');

interface AnalysisResult {
    analysis: string;
    timestamp: number;
}

const fetchUserData = async (connection: any, userPubkey: PublicKey): Promise<string> => {
    try {
        const accountInfo = await connection.getAccountInfo(userPubkey);
        const tokenAccounts = await connection.getTokenAccountsByOwner(userPubkey, {
            programId: TOKEN_PROGRAM_ID
        });
        const signatures = await connection.getSignaturesForAddress(userPubkey, {
            limit: 10
        });

        const transactionDetails = signatures.slice(0, 5).map((sig: any) => ({
            signature: sig.signature,
            slot: sig.slot,
            blockTime: sig.blockTime,
            err: sig.err
        }));

        const tokenBalances = await Promise.all(
            tokenAccounts.value.slice(0, 3).map(async (tokenAccount: any) => {
                try {
                    if (!tokenAccount.account.data.parsed || !tokenAccount.account.data.parsed.info) {
                        return null;
                    }

                    const balance = await connection.getTokenAccountBalance(tokenAccount.pubkey);
                    return {
                        mint: tokenAccount.account.data.parsed.info.mint,
                        amount: balance.value.amount,
                        decimals: balance.value.decimals
                    };
                } catch (error) {
                    return null;
                }
            })
        ).then(results => results.filter(result => result !== null));

        const userData = {
            pubkey: userPubkey.toString(),
            accountInfo: accountInfo ? {
                lamports: accountInfo.lamports,
                owner: accountInfo.owner.toString(),
                executable: accountInfo.executable,
                rentEpoch: accountInfo.rentEpoch
            } : null,
            tokenAccounts: tokenAccounts.value.slice(0, 3).map((account: any) => {
                if (!account.account.data.parsed || !account.account.data.parsed.info) {
                    return {
                        mint: 'unknown',
                        amount: '0'
                    };
                }

                return {
                    mint: account.account.data.parsed.info.mint,
                    amount: account.account.data.parsed.info.tokenAmount?.amount || '0'
                };
            }),
            recentTransactions: transactionDetails,
            tokenBalances: tokenBalances
        };

        const userDataString = JSON.stringify(userData);

        if (userDataString.length > 1000) {
            const minimalData = {
                pubkey: userData.pubkey,
                lamports: userData.accountInfo?.lamports || 0,
                tokenCount: userData.tokenAccounts.length,
                transactionCount: userData.recentTransactions.length,
                recentTxns: userData.recentTransactions.slice(0, 3).map((tx: any) => ({
                    sig: tx.signature.substring(0, 20) + '...',
                    slot: tx.slot,
                    time: tx.blockTime
                }))
            };
            return JSON.stringify(minimalData);
        }

        return userDataString;
    } catch (error) {
        throw error;
    }
};

export default function TransactionAnalyzer() {
    const { connected, publicKey } = useWallet();
    const program = useProgram();
    const [inputPubkey, setInputPubkey] = useState('EcgxCCyx5YrFTN6WeQ9ioX6CGZVgWsbyXxzNSAZDzdVT');
    const [isLoading, setIsLoading] = useState(false);
    const [result, setResult] = useState<AnalysisResult | null>(null);
    const [error, setError] = useState<string | null>(null);

    const handleAnalyze = async () => {
        if (!inputPubkey.trim()) return;

        try {
            new PublicKey(inputPubkey);
        } catch {
            setError('Invalid Solana public key format');
            return;
        }

        if (!program) {
            setError('Program not initialized. Please connect your wallet.');
            return;
        }

        if (isLoading) return;
        setIsLoading(true);
        setError(null);
        setResult(null);

        try {
            const userPubkey = new PublicKey(inputPubkey);
            const userDataString = await fetchUserData(program.provider.connection, userPubkey);

            const [agentPDA] = deriveAgentPDA();
            const [oracleCounterPDA] = deriveOracleCounterPDA();

            let llmContextPubkey: PublicKey;

            try {
                const accountInfo = await program.provider.connection.getAccountInfo(agentPDA);
                if (accountInfo) {
                    const contextBytes = accountInfo.data.slice(8, 40);
                    llmContextPubkey = new PublicKey(contextBytes);
                } else {
                    throw new Error('Agent account not found');
                }
            } catch (err) {
                const [llmContextPDA] = await deriveOracleLlmContextPDA(program.provider.connection);

                await program.methods.initialize().accounts({
                    payer: publicKey!,
                    agent: agentPDA,
                    llmContext: llmContextPDA,
                    counter: oracleCounterPDA,
                    systemProgram: SystemProgram.programId,
                    oracleProgram: ORACLE_PROGRAM_ID,
                }).rpc();

                llmContextPubkey = llmContextPDA;
            }

            // Step 2: Analyze user

            const counterInfo = await program.provider.connection.getAccountInfo(oracleCounterPDA);
            let counterValue = 0;
            if (counterInfo) {
                try {
                    counterValue = counterInfo.data.readUInt32LE(8);
                } catch (_) {
                    counterValue = 0;
                }
            }

            const expectedInteractionPDA = "2dyNkK7KMSSUmUn3rUzGEHJSXJDGmzxQBZst31yFKF9Q";

            let resolvedContext = llmContextPubkey;
            let resolvedInteraction: PublicKey | null = null;

            for (let i = 0; i <= counterValue; i++) {
                const idx = Buffer.alloc(4);
                idx.writeUInt32LE(i, 0);
                const [candidateCtx] = PublicKey.findProgramAddressSync(
                    [Buffer.from("test-context"), idx],
                    ORACLE_PROGRAM_ID
                );
                const [candidateIxn] = PublicKey.findProgramAddressSync(
                    [Buffer.from("interaction"), publicKey!.toBuffer(), candidateCtx.toBuffer()],
                    ORACLE_PROGRAM_ID
                );
                if (candidateIxn.toString() === expectedInteractionPDA) {
                    resolvedContext = candidateCtx;
                    resolvedInteraction = candidateIxn;
                    break;
                }
            }

            const interactionPDA = resolvedInteraction ?? PublicKey.findProgramAddressSync(
                [Buffer.from("interaction"), publicKey!.toBuffer(), resolvedContext.toBuffer()],
                ORACLE_PROGRAM_ID
            )[0];

            try {
                const [analysisPDA] = deriveAnalysisPDA(userPubkey);

                const txSignature = await program.methods.analyzeUser(userPubkey, userDataString).accounts({
                    payer: publicKey!,
                    interaction: interactionPDA,
                    agent: agentPDA,
                    contextAccount: resolvedContext,
                    analysisResult: analysisPDA,
                    oracleProgram: ORACLE_PROGRAM_ID,
                    systemProgram: SystemProgram.programId,
                }).rpc();

                console.log('Transaction signature:', txSignature);
                console.log('View transaction logs at: https://explorer.solana.com/tx/' + txSignature + '?cluster=devnet');

            } catch (e: any) {
                const msg = String(e?.message ?? e);
                if (!msg.includes('already been processed')) {
                    throw e;
                }
            }

            const analysisResult = await pollForAnalysis(program, userPubkey);

            setResult({
                analysis: analysisResult.analysis,
                timestamp: analysisResult.timestamp
            });
        } catch (err) {
            console.error('Analysis error:', err);
            setError('Failed to analyze wallet. Please try again.');
        } finally {
            setIsLoading(false);
        }
    };

    const pollForAnalysis = async (program: any, userPubkey: PublicKey) => {
        const maxAttempts = 60;
        let attempts = 0;

        while (attempts < maxAttempts) {
            try {
                const [analysisPDA] = deriveAnalysisPDA(userPubkey);
                const result = await program.methods.getAnalysis().accountsPartial({
                    analysisResult: analysisPDA,
                    userPubkey: userPubkey,
                }).view();

                if (result && result.length > 0) {
                    return { analysis: result, timestamp: Date.now() };
                }
            } catch (err: any) {
                // Continue polling on error
            }

            await new Promise(resolve => setTimeout(resolve, 2000));
            attempts++;
        }

        throw new Error('Analysis timeout - oracle service may not be running');
    };

    const handleKeyPress = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') {
            handleAnalyze();
        }
    };

    if (!connected) {
        return (
            <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
                <div className="text-center space-y-6 p-8">
                    <div className="space-y-2">
                        <h1 className="text-4xl font-bold text-gray-900 dark:text-white">
                            Transaction Analyzer
                        </h1>
                        <p className="text-lg text-gray-600 dark:text-gray-300">
                            Analyze Solana wallet activity with AI
                        </p>
                    </div>
                    <WalletMultiButton className="!bg-indigo-600 hover:!bg-indigo-700 !rounded-lg !px-6 !py-3 !text-white !font-medium" />
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 p-4">
            <div className="max-w-4xl mx-auto pt-8">
                <div className="text-center mb-8">
                    <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        Transaction Analyzer
                    </h1>
                    <p className="text-gray-600 dark:text-gray-300">
                        Enter any Solana public key to analyze recent activity
                    </p>
                </div>

                <div className="flex justify-center mb-8">
                    <WalletMultiButton className="!bg-indigo-600 hover:!bg-indigo-700 !rounded-lg !px-6 !py-3 !text-white !font-medium" />
                </div>

                <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 mb-6">
                    <div className="space-y-4">
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            Enter any pubkey to know more:
                        </label>
                        <div className="flex gap-3">
                            <input
                                type="text"
                                value={inputPubkey}
                                onChange={(e) => setInputPubkey(e.target.value)}
                                onKeyPress={handleKeyPress}
                                placeholder="e.g., 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"
                                className="flex-1 px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent dark:bg-gray-700 dark:text-white placeholder-gray-500"
                                disabled={isLoading}
                            />
                            <button
                                onClick={handleAnalyze}
                                disabled={isLoading || !inputPubkey.trim()}
                                className="px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed font-medium transition-colors"
                            >
                                Analyze
                            </button>
                        </div>
                    </div>
                </div>

                {isLoading && (
                    <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 text-center">
                        <div className="space-y-4">
                            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"></div>
                            <div className="space-y-2">
                                <p className="text-lg font-medium text-gray-900 dark:text-white">
                                    Thinking...
                                </p>
                                <p className="text-sm text-gray-600 dark:text-gray-300">
                                    Searching the blockchain for transaction data
                                </p>
                            </div>
                        </div>
                    </div>
                )}

                {error && (
                    <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-6">
                        <div className="flex items-center space-x-3">
                            <div className="flex-shrink-0">
                                <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                    <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                                </svg>
                            </div>
                            <p className="text-red-800 dark:text-red-200 font-medium">{error}</p>
                        </div>
                    </div>
                )}

                {result && (
                    <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6">
                        <div className="space-y-4">
                            <div className="flex items-center justify-between">
                                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                                    Analysis Result
                                </h3>
                                <span className="text-sm text-gray-500 dark:text-gray-400">
                                    {new Date(result.timestamp).toLocaleString()}
                                </span>
                            </div>
                            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                <p className="text-gray-800 dark:text-gray-200 leading-relaxed">
                                    {result.analysis}
                                </p>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}