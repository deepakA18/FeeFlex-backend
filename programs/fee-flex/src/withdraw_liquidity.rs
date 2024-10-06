use anchor_lang::prelude::*; 
 use anchor_spl::{ 
     associated_token::AssociatedToken, 
     token::{self, Burn, Mint, Token, TokenAccount, Transfer}, 
 }; 
 use fixed::types::I64F64; 
  
 use crate::{ 
     constants::{AUTHORITY_SEED, LIQUIDITY_SEED, MINIMUM_LIQUIDITY}, 
     state::{Amm, Pool}, 
 }; 

pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> { 
    let authority_bump = *ctx.bumps.get("pool_authority").unwrap(); 
    let authority_seeds = &[ 
     &ctx.accounts.pool.amm.to_bytes(), 
     &ctx.accounts.mint_a.key().to_bytes(), 
     &ctx.accounts.mint_b.key().to_bytes(), 
     AUTHORITY_SEED.as_bytes(), 
     &[authority_bump], 
 ]; 
 let signer_seeds = &[&authority_seeds[..]]; 

// Transfer tokens from the pool 
let amount_a = I64F64::from_num(amount) 
.checked_mul(I64F64::from_num(ctx.accounts.pool_account_a.amount)) 
.unwrap() 
.checked_div(I64F64::from_num( 
    ctx.accounts.mint_liquidity.supply + MINIMUM_LIQUIDITY, 
)) 
.unwrap() 
.floor() 
.to_num::<u64>(); 
token::transfer( 
CpiContext::new_with_signer( 
    ctx.accounts.token_program.to_account_info(), 
    Transfer { 
        from: ctx.accounts.pool_account_a.to_account_info(), 
        to: ctx.accounts.depositor_account_a.to_account_info(), 
        authority: ctx.accounts.pool_authority.to_account_info(), 
    }, 
    signer_seeds, 
), 
amount_a, 
)?; 


let amount_b = I64F64::from_num(amount) 
.checked_mul(I64F64::from_num(ctx.accounts.pool_account_b.amount)) 
.unwrap() 
.checked_div(I64F64::from_num( 
    ctx.accounts.mint_liquidity.supply + MINIMUM_LIQUIDITY, 
)) 
.unwrap() 
.floor() 
.to_num::<u64>(); 
token::transfer( 
CpiContext::new_with_signer( 
    ctx.accounts.token_program.to_account_info(), 
    Transfer { 
        from: ctx.accounts.pool_account_b.to_account_info(), 
        to: ctx.accounts.depositor_account_b.to_account_info(), 
        authority: ctx.accounts.pool_authority.to_account_info(), 
    }, 
    signer_seeds, 
), 
amount_b, 
)?;


// Burn the liquidity tokens 
 // It will fail if the amount is invalid 
 token::burn( 
    CpiContext::new( 
        ctx.accounts.token_program.to_account_info(), 
        Burn { 
            mint: ctx.accounts.mint_liquidity.to_account_info(), 
            from: ctx.accounts.depositor_account_liquidity.to_account_info(), 
            authority: ctx.accounts.depositor.to_account_info(), 
        }, 
    ), 
    amount, 
)?; 
 
Ok(()) 