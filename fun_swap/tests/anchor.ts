import BN from "bn.js";
import assert from "assert";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import type { FunSwap } from "../target/types/fun_swap";

describe("fun_swap", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.FunSwap as anchor.Program<FunSwap>;
  
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.FunSwap;

  let partyA = anchor.web3.Keypair.generate();
  let partyB = anchor.web3.Keypair.generate();
  let mintA = null;
  let mintB = null;
  let partyATokenAccount = null;
  let partyBTokenAccount = null;
  let swapAccount = anchor.web3.Keypair.generate();

  const amountTokenA = new anchor.BN(100_000);
  const amountTokenB = new anchor.BN(200_000);
  const deadline = new anchor.BN(Date.now() / 1000 + 86400); // 1 day in seconds
  const gracePeriod = new anchor.BN(3600); // 1 hour in seconds

  it("Initialize mint and token accounts", async () => {
    // Airdrop some SOL to partyA and partyB for transaction fees
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(partyA.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(partyB.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );

    // Create Mints for Party A and Party B
    mintA = await createMint(partyA.publicKey);
    mintB = await createMint(partyB.publicKey);

    // Create Token Accounts for Party A and Party B
    partyATokenAccount = await createTokenAccount(mintA, partyA.publicKey);
    partyBTokenAccount = await createTokenAccount(mintB, partyB.publicKey);

    // Mint tokens to Party A and Party B token accounts
    await mintTokens(mintA, partyATokenAccount, partyA, 1_000_000);
    await mintTokens(mintB, partyBTokenAccount, partyB, 1_000_000);
  });

  it("Initiate Swap", async () => {
    // Call the initiate_swap method
    await program.methods
      .initiateSwap(amountTokenA, amountTokenB, deadline, gracePeriod)
      .accounts({
        swap: swapAccount.publicKey,
        partyA: partyA.publicKey,
        partyB: partyB.publicKey,
        partyATokenAccount: partyATokenAccount,
        partyBTokenAccount: partyBTokenAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([swapAccount, partyA])
      .rpc();

    // Fetch swap account data
    const swapData = await program.account.swap.fetch(swapAccount.publicKey);
    console.log("Swap initiated: ", swapData);

    // Validate swap data
    assert.equal(swapData.partyA.toBase58(), partyA.publicKey.toBase58());
    assert.equal(swapData.partyB.toBase58(), partyB.publicKey.toBase58());
    assert.equal(swapData.amountTokenA.toString(), amountTokenA.toString());
    assert.equal(swapData.amountTokenB.toString(), amountTokenB.toString());
    assert.equal(swapData.isCompleted, false);
  });

  it("Approve Swap", async () => {
    // Call the approve_swap method
    await program.methods
      .approveSwap()
      .accounts({
        swap: swapAccount.publicKey,
        partyA: partyA.publicKey,
        partyB: partyB.publicKey,
        partyATokenAccount: partyATokenAccount,
        partyBTokenAccount: partyBTokenAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([partyA])
      .rpc();

    // Fetch swap account data to verify the swap is completed
    const swapData = await program.account.swap.fetch(swapAccount.publicKey);
    console.log("Swap approved: ", swapData);

    assert.equal(swapData.isCompleted, true);
  });

  it("Expire Swap", async () => {
    // Wait for the swap to expire by simulating time (e.g., using test clock manipulation)
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(partyA.publicKey, anchor.web3.LAMPORTS_PER_SOL)
    );

    // Call the expire_swap method
    await program.methods
      .expireSwap()
      .accounts({
        swap: swapAccount.publicKey,
        partyA: partyA.publicKey,
        partyB: partyB.publicKey,
        partyATokenAccount: partyATokenAccount,
        partyBTokenAccount: partyBTokenAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([partyA])
      .rpc();

    // Fetch swap account data to verify expiration
    const swapData = await program.account.swap.fetch(swapAccount.publicKey);
    console.log("Swap expired: ", swapData);
  });

  it("Extend Deadline", async () => {
    // New deadline set further in the future
    const newDeadline = new anchor.BN(Date.now() / 1000 + 172800); // 2 days in seconds

    // Call the extend_deadline method
    await program.methods
      .extendDeadline(newDeadline)
      .accounts({
        swap: swapAccount.publicKey,
        partyA: partyA.publicKey,
      })
      .signers([partyA])
      .rpc();

    // Fetch swap account data to verify the deadline is extended
    const swapData = await program.account.swap.fetch(swapAccount.publicKey);
    console.log("Deadline extended: ", swapData);

    assert.equal(swapData.deadline.toString(), newDeadline.toString());
  });

  // Utility function for creating a new mint
  async function createMint(mintAuthority) {
    const mint = anchor.web3.Keypair.generate();
    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(mintAuthority, anchor.web3.LAMPORTS_PER_SOL)
    );
    await program.rpc.createMint(mint.publicKey, mintAuthority, {
      accounts: {
        mint: mint.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [mint],
    });
    return mint.publicKey;
  }

  // Utility function for creating a token account
  async function createTokenAccount(mint, owner) {
    const tokenAccount = anchor.web3.Keypair.generate();
    await program.rpc.createTokenAccount(tokenAccount.publicKey, mint, owner, {
      accounts: {
        tokenAccount: tokenAccount.publicKey,
        mint: mint,
        owner: owner,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [tokenAccount],
    });
    return tokenAccount.publicKey;
  }

  // Utility function for minting tokens
  async function mintTokens(mint, tokenAccount, authority, amount) {
    await program.rpc.mintTo(tokenAccount, mint, authority.publicKey, amount, {
      accounts: {
        tokenAccount: tokenAccount,
        mint: mint,
        authority: authority.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      },
      signers: [authority],
    });
  }
});
