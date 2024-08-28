use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer, Token};

declare_id!("FhjkoC2mdDHwE58iJuQmjvxGDCeYUHRFnZ1hphq1eWrE");

#[program]
mod fun_swap {
    use super::*;

    // Initiate a swap with a grace period and deadline
    pub fn initiate_swap(
        ctx: Context<InitiateSwap>,
        amount_token_a: u64,
        amount_token_b: u64,
        deadline: i64,
        grace_period: i64,  
    ) -> Result<()> {
        let swap = &mut ctx.accounts.swap;
        swap.party_a = *ctx.accounts.party_a.key;
        swap.party_b = *ctx.accounts.party_b.key;
        swap.amount_token_a = amount_token_a;
        swap.amount_token_b = amount_token_b;
        swap.deadline = deadline;
        swap.grace_period = grace_period;
        swap.is_completed = false;

        emit!(SwapInitiated {
            party_a: ctx.accounts.party_a.key(),
            party_b: ctx.accounts.party_b.key(),
            amount_token_a,
            amount_token_b,
            deadline,
        });

        Ok(())
    }

    // Approve the swap
    pub fn approve_swap(ctx: Context<ApproveSwap>) -> Result<()> {
        let swap = &mut ctx.accounts.swap;
        require!(
            Clock::get()?.unix_timestamp < swap.deadline,
            SwapError::SwapExpired
        );

        // Transfer tokens from Party A to Party B
        let cpi_ctx_a_to_b = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.party_a_token_account.to_account_info(),
                to: ctx.accounts.party_b_token_account.to_account_info(),
                authority: ctx.accounts.party_a.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_a_to_b, swap.amount_token_a)?;

        // Transfer tokens from Party B to Party A
        let cpi_ctx_b_to_a = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.party_b_token_account.to_account_info(),
                to: ctx.accounts.party_a_token_account.to_account_info(),
                authority: ctx.accounts.party_b.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_b_to_a, swap.amount_token_b)?;

        swap.is_completed = true;
        emit!(SwapCompleted {
            party_a: ctx.accounts.party_a.key(),
            party_b: ctx.accounts.party_b.key(),
        });

        Ok(())
    }

    // Expire the swap after the deadline + grace period
    pub fn expire_swap(ctx: Context<ExpireSwap>) -> Result<()> {
        let swap = &mut ctx.accounts.swap;
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            current_time >= swap.deadline + swap.grace_period,
            SwapError::SwapNotExpired
        );

        // Transfer back the tokens to their owners after expiration
        let cpi_ctx_a_recover = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.party_a_token_account.to_account_info(),
                to: ctx.accounts.party_a.to_account_info(),
                authority: ctx.accounts.party_a.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_a_recover, swap.amount_token_a)?;

        let cpi_ctx_b_recover = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.party_b_token_account.to_account_info(),
                to: ctx.accounts.party_b.to_account_info(),
                authority: ctx.accounts.party_b.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_b_recover, swap.amount_token_b)?;

        emit!(SwapExpired {
            party_a: ctx.accounts.party_a.key(),
            party_b: ctx.accounts.party_b.key(),
        });

        Ok(())
    }

    // Extend the swap deadline
    pub fn extend_deadline(ctx: Context<ExtendDeadline>, new_deadline: i64) -> Result<()> {
        let swap = &mut ctx.accounts.swap;

        require!(
            new_deadline > swap.deadline,
            SwapError::InvalidDeadline
        );

        swap.deadline = new_deadline;

        emit!(DeadlineExtended { new_deadline });

        Ok(())
    }
}

// Account structs

#[derive(Accounts)]
pub struct InitiateSwap<'info> {
    #[account(init, payer = party_a, space = 8 + Swap::LEN)]
    pub swap: Account<'info, Swap>,
    #[account(mut)]
    pub party_a: Signer<'info>,
    #[account(mut)]
    pub party_b: AccountInfo<'info>,
    #[account(mut)]
    pub party_a_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub party_b_token_account: Account<'info, TokenAccount>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ApproveSwap<'info> {
    #[account(mut, has_one = party_a, has_one = party_b)]
    pub swap: Account<'info, Swap>,
    #[account(mut)]
    pub party_a: Signer<'info>,
    #[account(mut)]
    pub party_b: AccountInfo<'info>,
    #[account(mut)]
    pub party_a_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub party_b_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ExpireSwap<'info> {
    #[account(mut, has_one = party_a, has_one = party_b)]
    pub swap: Account<'info, Swap>,
    #[account(mut)]
    pub party_a: AccountInfo<'info>,
    #[account(mut)]
    pub party_b: AccountInfo<'info>,
    #[account(mut)]
    pub party_a_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub party_b_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ExtendDeadline<'info> {
    #[account(mut, has_one = party_a)]
    pub swap: Account<'info, Swap>,
    pub party_a: Signer<'info>,
}

// Data struct for storing swap details
#[account]
pub struct Swap {
    pub party_a: Pubkey,
    pub party_b: Pubkey,
    pub amount_token_a: u64,
    pub amount_token_b: u64,
    pub deadline: i64,
    pub grace_period: i64, 
    pub is_completed: bool,
}

impl Swap {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 1; 
}

// Error handling
#[error_code]
pub enum SwapError {
    #[msg("The swap has already been completed.")]
    SwapAlreadyCompleted,
    #[msg("The swap has expired.")]
    SwapExpired,
    #[msg("The swap is not expired yet.")]
    SwapNotExpired,
    #[msg("The new deadline must be greater than the current deadline.")]
    InvalidDeadline,
}

// Event declarations
#[event]
pub struct SwapInitiated {
    pub party_a: Pubkey,
    pub party_b: Pubkey,
    pub amount_token_a: u64,
    pub amount_token_b: u64,
    pub deadline: i64,
}

#[event]
pub struct SwapCompleted {
    pub party_a: Pubkey,
    pub party_b: Pubkey,
}

#[event]
pub struct SwapExpired {
    pub party_a: Pubkey,
    pub party_b: Pubkey,
}

#[event]
pub struct DeadlineExtended {
    pub new_deadline: i64,
}
