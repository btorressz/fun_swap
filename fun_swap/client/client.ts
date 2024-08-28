import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import * as anchor from "@project-serum/anchor"; // Add anchor imports if necessary

// Client
console.log("My address:",import type { FunSwap } from "../target/types/fun_swap";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.FunSwap as anchor.Program<FunSwap>;

 program.provider.publicKey.toString());
const balance = await program.provider.connection.getBalance(program.provider.publicKey);
console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

// Define public keys for parties
const partyA = program.provider.publicKey; // Use the wallet for Party A
const partyB = new web3.Keypair();  // New random keypair for Party B

// Define token mint and token accounts (replace these with actual public keys)
const mintA = new web3.PublicKey("MINT_A_PUBLIC_KEY");
const mintB = new web3.PublicKey("MINT_B_PUBLIC_KEY");
const partyATokenAccount = new web3.PublicKey("PARTY_A_TOKEN_ACCOUNT");
const partyBTokenAccount = new web3.PublicKey("PARTY_B_TOKEN_ACCOUNT");

// Define swap details
const amountTokenA = new anchor.BN(100000); // Example: 100,000 tokens of A
const amountTokenB = new anchor.BN(200000); // Example: 200,000 tokens of B
const deadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400); // 1 day from now
const gracePeriod = new anchor.BN(3600); // 1 hour grace period

// Create a keypair for the swap account
const swapAccount = new web3.Keypair();

console.log("Initiating a swap...");
try {
    const txHash = await program.methods
        .initiateSwap(amountTokenA, amountTokenB, deadline, gracePeriod)
        .accounts({
            swap: swapAccount.publicKey,   // Swap account public key
            partyA: partyA,                // Party A's public key
            partyB: partyB.publicKey,      // Party B's public key
            partyATokenAccount: partyATokenAccount, // Party A's token account
            partyBTokenAccount: partyBTokenAccount, // Party B's token account
            tokenProgram: TOKEN_PROGRAM_ID,         // SPL Token program ID
            systemProgram: web3.SystemProgram.programId, // System Program
            rent: web3.SYSVAR_RENT_PUBKEY           // Rent Sysvar Program
        })
        .signers([swapAccount, partyB])  // Signers include swap account and party B
        .rpc();
    
    console.log(`Swap initiated successfully! Transaction Hash: ${txHash}`);
} catch (err) {
    console.error("Failed to initiate swap:", err);
}

// Fetch and display swap account data
try {
    const swapAccountData = await program.account.swap.fetch(swapAccount.publicKey);
    console.log("Swap account data:", swapAccountData);
} catch (err) {
    console.error("Failed to fetch swap account:", err);
}

// Check updated balance of Party A
const updatedBalance = await program.provider.connection.getBalance(program.provider.publicKey);
console.log(`Updated balance: ${updatedBalance / web3.LAMPORTS_PER_SOL} SOL`);
