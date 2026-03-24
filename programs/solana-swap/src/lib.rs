use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    TokenInterface, TokenAccount, Mint, TransferChecked, transfer_checked,
};

declare_id!("6PG4hM92zEiRPDbK7UdshndJBpSu1rGX7kRxzZFFJXPN");

#[program]
pub mod solana_swap {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.token_a_account = ctx.accounts.token_a_account.key();
        pool.token_b_account = ctx.accounts.token_b_account.key();
        pool.fee_numerator = fee_numerator;
        pool.fee_denominator = fee_denominator;
        pool.bump = ctx.bumps.pool;
        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.pool_token_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.mint_a.to_account_info(),
                },
            ),
            amount_a,
            ctx.accounts.mint_a.decimals,
        )?;

        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.pool_token_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.mint_b.to_account_info(),
                },
            ),
            amount_b,
            ctx.accounts.mint_b.decimals,
        )?;

        msg!("Liquidity added: {} of Token A and {} of Token B", amount_a, amount_b);
        Ok(())
    }

    pub fn swap_a_to_b(
        ctx: Context<Swap>,
        amount_in: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let reserve_a = ctx.accounts.pool_token_a.amount;
        let reserve_b = ctx.accounts.pool_token_b.amount;

        let fee = amount_in
            .checked_mul(pool.fee_numerator).unwrap()
            .checked_div(pool.fee_denominator).unwrap();
        let amount_in_after_fee = amount_in.checked_sub(fee).unwrap();

        let amount_out = reserve_b
            .checked_mul(amount_in_after_fee).unwrap()
            .checked_div(reserve_a.checked_add(amount_in_after_fee).unwrap())
            .unwrap();
        require!(amount_out > 0, SwapError::InsufficientOutputAmount);

        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.pool_token_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.mint_a.to_account_info(),
                },
            ),
            amount_in,
            ctx.accounts.mint_a.decimals,
        )?;

        let bump = pool.bump;
        let seeds = &[b"pool".as_ref(), &[bump]];
        let signer = &[&seeds[..]];

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.pool_token_b.to_account_info(),
                    to: ctx.accounts.user_token_b.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                    mint: ctx.accounts.mint_b.to_account_info(),
                },
                signer,
            ),
            amount_out,
            ctx.accounts.mint_b.decimals,
        )?;

        msg!("Swapped {} of Token A for {} of Token B", amount_in, amount_out);
        Ok(())
    }

    pub fn swap_b_to_a(
        ctx: Context<Swap>,
        amount_in: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let reserve_a = ctx.accounts.pool_token_a.amount;
        let reserve_b = ctx.accounts.pool_token_b.amount;

        let fee = amount_in
            .checked_mul(pool.fee_numerator).unwrap()
            .checked_div(pool.fee_denominator).unwrap();
        let amount_in_after_fee = amount_in.checked_sub(fee).unwrap();

        let amount_out = reserve_a
            .checked_mul(amount_in_after_fee).unwrap()
            .checked_div(reserve_b.checked_add(amount_in_after_fee).unwrap())
            .unwrap();
        require!(amount_out > 0, SwapError::InsufficientOutputAmount);

        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.pool_token_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.mint_b.to_account_info(),
                },
            ),
            amount_in,
            ctx.accounts.mint_b.decimals,
        )?;

        let bump = pool.bump;
        let seeds = &[b"pool".as_ref(), &[bump]];
        let signer = &[&seeds[..]];

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.pool_token_a.to_account_info(),
                    to: ctx.accounts.user_token_a.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                    mint: ctx.accounts.mint_a.to_account_info(),
                },
                signer,
            ),
            amount_out,
            ctx.accounts.mint_a.decimals,
        )?;

        msg!("Swapped {} of Token B for {} of Token A", amount_in, amount_out);
        Ok(())
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        share_numerator: u64,
        share_denominator: u64,
    ) -> Result<()> {
        require!(share_numerator > 0 && share_denominator > 0, SwapError::InvalidShare);
        require!(share_numerator <= share_denominator, SwapError::InvalidShare);

        let reserve_a = ctx.accounts.pool_token_a.amount;
        let reserve_b = ctx.accounts.pool_token_b.amount;

        let amount_a = reserve_a
            .checked_mul(share_numerator).unwrap()
            .checked_div(share_denominator).unwrap();
        let amount_b = reserve_b
            .checked_mul(share_numerator).unwrap()
            .checked_div(share_denominator).unwrap();

        require!(amount_a > 0 && amount_b > 0, SwapError::InsufficientOutputAmount);

        let bump = ctx.accounts.pool.bump;
        let seeds = &[b"pool".as_ref(), &[bump]];
        let signer = &[&seeds[..]];

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.pool_token_a.to_account_info(),
                    to: ctx.accounts.user_token_a.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                    mint: ctx.accounts.mint_a.to_account_info(),
                },
                signer,
            ),
            amount_a,
            ctx.accounts.mint_a.decimals,
        )?;

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.pool_token_b.to_account_info(),
                    to: ctx.accounts.user_token_b.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                    mint: ctx.accounts.mint_b.to_account_info(),
                },
                signer,
            ),
            amount_b,
            ctx.accounts.mint_b.decimals,
        )?;

        msg!("Removed liquidity: {} of Token A and {} of Token B", amount_a, amount_b);
        Ok(())
    }
}

#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub token_a_account: Pubkey,
    pub token_b_account: Pubkey,
    pub fee_numerator: u64,
    pub fee_denominator: u64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 1,
        seeds = [b"pool".as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_a_account: InterfaceAccount<'info, TokenAccount>,
    pub token_b_account: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut, seeds = [b"pool".as_ref()], bump = pool.bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = pool.token_a_account)]
    pub pool_token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = pool.token_b_account)]
    pub pool_token_b: InterfaceAccount<'info, TokenAccount>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut, seeds = [b"pool".as_ref()], bump = pool.bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = pool.token_a_account)]
    pub pool_token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = pool.token_b_account)]
    pub pool_token_b: InterfaceAccount<'info, TokenAccount>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut, seeds = [b"pool".as_ref()], bump = pool.bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut, address = pool.authority)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub user_token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = pool.token_a_account)]
    pub pool_token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = pool.token_b_account)]
    pub pool_token_b: InterfaceAccount<'info, TokenAccount>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[error_code]
pub enum SwapError {
    #[msg("Output amount is too low")]
    InsufficientOutputAmount,
    #[msg("Share must be between 0 and 1 (numerator <= denominator)")]
    InvalidShare,
}