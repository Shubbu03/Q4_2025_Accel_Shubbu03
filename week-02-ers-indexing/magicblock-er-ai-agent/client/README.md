# Transaction Analyzer Frontend

A minimal, responsive Next.js frontend for analyzing Solana wallet transaction activity using AI.

## Features

- **Wallet Connection**: Connect with Phantom, Solflare, and other Solana wallets
- **Pubkey Analysis**: Enter any Solana public key to analyze recent activity
- **AI-Powered Insights**: Get human-readable summaries of wallet behavior
- **Responsive Design**: Clean, minimal interface that works on all devices
- **Loading States**: Smooth user experience with proper loading indicators

## Tech Stack

- **Next.js 15** - React framework
- **Tailwind CSS** - Styling
- **Solana Wallet Adapter** - Wallet integration
- **TypeScript** - Type safety

## Getting Started

1. Install dependencies:
```bash
npm install
```

2. Run the development server:
```bash
npm run dev
```

3. Open [http://localhost:3000](http://localhost:3000) in your browser

## Usage

1. **Connect Wallet**: Click the wallet connect button and select your preferred wallet
2. **Enter Pubkey**: Input any Solana public key in the text field
3. **Analyze**: Click "Analyze" or press Enter to start the analysis
4. **View Results**: See the AI-generated analysis of the wallet's recent activity

## Design Principles

- **Minimal**: Clean, uncluttered interface focused on functionality
- **Responsive**: Works seamlessly on desktop, tablet, and mobile
- **Accessible**: Proper contrast, keyboard navigation, and screen reader support
- **Fast**: Optimized for quick loading and smooth interactions

## Integration

This frontend integrates with the Solana smart contract for transaction analysis. The actual analysis is performed by the oracle service, which:

1. Fetches recent transaction data from Solana RPC
2. Processes the data with AI (OpenAI)
3. Returns human-readable insights about wallet behavior

## Future Enhancements

- Real-time analysis updates
- Historical analysis trends
- Export analysis results
- Multiple analysis types (DeFi, NFT, Gaming)