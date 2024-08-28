u  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.FunSwap as anchor.Program<FunSwap>;
  
import type { FunSwap } from "../target/types/fun_swap";
se anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_spl::token::{Mint, TokenAccount};
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
    instruction::Instruction,
};
use spl_token::{instruction, state::Account as SplTokenAccount};
use solana_program::program_pack::Pack;
use fun_swap::{self, Swap};
use borsh::BorshDeserialize;

#[tokio::test]
async fn test_initiate_swap() {
    // Set up the program test environment
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "fun_swap",
        program_id,
        processor!(fun_swap::entry),
    );

    program_test.set_compute_max_units(500_000); // Set max compute units to a higher value.

    let mut context = program_test.start_with_context().await;

    println!("Test environment set up successfully");

    // Set up token mints, token accounts, and program accounts
    let party_a = Keypair::new();
    let party_b = Keypair::new();

    println!("Creating and funding accounts for party_a and party_b");
    create_and_fund_account(&mut context, &party_a, 1_000_000_000).await;
    create_and_fund_account(&mut context, &party_b, 1_000_000_000).await;

    println!("Creating mints");
    let mint_a = create_mint(&mut context, &party_a.pubkey(), 6).await;
    let mint_b = create_mint(&mut context, &party_b.pubkey(), 6).await;

    println!("Creating token accounts");
    let party_a_token_account = create_token_account(&mut context, &mint_a, &party_a.pubkey()).await;
    let party_b_token_account = create_token_account(&mut context, &mint_b, &party_b.pubkey()).await;

    println!("Minting tokens");
    mint_tokens(&mut context, &mint_a, &party_a_token_account, &party_a, 1_000_000).await;
    mint_tokens(&mut context, &mint_b, &party_b_token_account, &party_b, 1_000_000).await;

    let swap_account = Keypair::new();

    println!("Preparing initiate_swap instruction");
    let ix_data = fun_swap::instruction::InitiateSwap {
        amount_token_a: 100_000,
        amount_token_b: 200_000,
        deadline: 86400,
        grace_period: 3600,
    };

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(swap_account.pubkey(), true),
            AccountMeta::new(party_a.pubkey(), true),
            AccountMeta::new(party_b.pubkey(), false),
            AccountMeta::new(party_a_token_account, false),
            AccountMeta::new(party_b_token_account, false),
            AccountMeta::new_readonly(anchor_spl::token::ID, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
        ],
        data: ix_data.data(),
    };

    println!("Creating and signing transaction");

    // Include compute budget instruction
    let compute_budget_ix = solana_sdk::compute_budget::ComputeBudgetInstruction::request_units(200_000, 0); // Use 0 instead of 0.001

    let mut transaction = Transaction::new_with_payer(
        &[compute_budget_ix, ix], // Add compute budget instruction to the transaction
        Some(&context.payer.pubkey()),
    );

    transaction.sign(&[&context.payer, &swap_account, &party_a], context.last_blockhash);

    println!("Processing transaction");
    let result = context.banks_client.process_transaction(transaction).await;

    match result {
        Ok(_) => println!("Transaction processed successfully"),
        Err(e) => panic!("Failed to process transaction: {:?}", e),
    }

    println!("Fetching swap account data");
    let swap_account_data = context
        .banks_client
        .get_account(swap_account.pubkey())
        .await
        .expect("Failed to fetch swap account")
        .expect("Swap account not found");

    println!("Deserializing swap data");
    let swap_data: Swap = Swap::try_from_slice(&swap_account_data.data).expect("Failed to deserialize swap data");

    println!("Verifying swap data");
    assert_eq!(swap_data.party_a, party_a.pubkey());
    assert_eq!(swap_data.party_b, party_b.pubkey());
    assert_eq!(swap_data.amount_token_a, 100_000);
    assert_eq!(swap_data.amount_token_b, 200_000);
    assert_eq!(swap_data.is_completed, false);

    println!("Swap initiated successfully with correct data.");
}

// Same utility functions for minting and token account creation

async fn create_and_fund_account(context: &mut ProgramTestContext, account: &Keypair, lamports: u64) {
    let transaction = Transaction::new_signed_with_payer(
        &[solana_sdk::system_instruction::transfer(
            &context.payer.pubkey(),
            &account.pubkey(),
            lamports,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(transaction).await.unwrap_or_else(|e| {
        panic!("Failed to create and fund account: {:?}", e);
    });
}

async fn create_mint(context: &mut ProgramTestContext, mint_authority: &Pubkey, decimals: u8) -> Pubkey {
    let mint = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let rent_lamports = rent.minimum_balance(Mint::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            solana_sdk::system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent_lamports,
                Mint::LEN as u64,
                &anchor_spl::token::ID,
            ),
            spl_token::instruction::initialize_mint(
                &anchor_spl::token::ID,
                &mint.pubkey(),
                mint_authority,
                None,
                decimals,
            ).unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(transaction).await.unwrap_or_else(|e| {
        panic!("Failed to create mint: {:?}", e);
    });

    mint.pubkey()
}

async fn create_token_account(context: &mut ProgramTestContext, mint: &Pubkey, owner: &Pubkey) -> Pubkey {
    let token_account = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let rent_lamports = rent.minimum_balance(SplTokenAccount::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            solana_sdk::system_instruction::create_account(
                &context.payer.pubkey(),
                &token_account.pubkey(),
                rent_lamports,
                SplTokenAccount::LEN as u64,
                &anchor_spl::token::ID,
            ),
            spl_token::instruction::initialize_account(
                &anchor_spl::token::ID,
                &token_account.pubkey(),
                mint,
                owner,
            ).unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &token_account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(transaction).await.unwrap_or_else(|e| {
        panic!("Failed to create token account: {:?}", e);
    });

    token_account.pubkey()
}

async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    token_account: &Pubkey,
    authority: &Keypair,
    amount: u64,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &anchor_spl::token::ID,
            mint,
            token_account,
            &authority.pubkey(),
            &[&authority.pubkey()],
            amount,
        ).unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(transaction).await.unwrap_or_else(|e| {
        panic!("Failed to mint tokens: {:?}", e);
    });
}
