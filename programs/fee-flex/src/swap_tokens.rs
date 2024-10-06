use anchor_lang::prelude::*; 
 use anchor_spl::{ 
     associated_token::AssociatedToken, 
     token::{self, Mint, Token, TokenAccount, Transfer}, 
 }; 
 use fixed::types::I64F64; 
  
 use crate::{ 
     constants::AUTHORITY_SEED, 
     errors::*, 
     state::{Amm, Pool}, 
 }; 
  
 pub fn swap_exact_tokens_for_tokens( 
     ctx: Context<SwapExactTokensForTokens>, 
     swap_a: bool, 
     input_amount: u64, 
     min_output_amount: u64, 
 ) -> Result<()> { 
     // Prevent depositing assets the depositor does not own 
     let input = if swap_a && input_amount > ctx.accounts.trader_account_a.amount { 
         ctx.accounts.trader_account_a.amount 
     } else if !swap_a && input_amount > ctx.accounts.trader_account_b.amount { 
         ctx.accounts.trader_account_b.amount 
     } else { 
         input_amount 
     }; 


// Apply trading fee, used to compute the output 
let amm = &ctx.accounts.amm; 
let taxed_input = input - input * amm.fee as u64 / 10000; 


let pool_a = &ctx.accounts.pool_account_a; 
 let pool_b = &ctx.accounts.pool_account_b; 
 let output = if swap_a { 
     I64F64::from_num(taxed_input) 
         .checked_mul(I64F64::from_num(pool_b.amount)) 
         .unwrap() 
         .checked_div( 
             I64F64::from_num(pool_a.amount) 
                 .checked_add(I64F64::from_num(taxed_input)) 
                 .unwrap(), 
         ) 
         .unwrap() 
 } else { 
     I64F64::from_num(taxed_input) 
         .checked_mul(I64F64::from_num(pool_a.amount)) 
         .unwrap() 
         .checked_div( 
             I64F64::from_num(pool_b.amount) 
                 .checked_add(I64F64::from_num(taxed_input)) 
                 .unwrap(), 
         ) 
         .unwrap() 
 } 
 .to_num::<u64>(); 


 if output < min_output_amount { 
    return err!(TutorialError::OutputTooSmall); 
} 

 // Compute the invariant before the trade 
 let invariant = pool_a.amount * pool_b.amount;


// Transfer tokens to the pool 
let authority_bump = *ctx.bumps.get("pool_authority").unwrap(); 
let authority_seeds = &[ 
    &ctx.accounts.pool.amm.to_bytes(), 
    &ctx.accounts.mint_a.key().to_bytes(), 
    &ctx.accounts.mint_b.key().to_bytes(), 
    AUTHORITY_SEED.as_bytes(), 
    &[authority_bump], 
]; 
let signer_seeds = &[&authority_seeds[..]]; 
if swap_a { 
    token::transfer( 
        CpiContext::new( 
            ctx.accounts.token_program.to_account_info(), 
            Transfer { 
                from: ctx.accounts.trader_account_a.to_account_info(), 
                to: ctx.accounts.pool_account_a.to_account_info(), 
                authority: ctx.accounts.trader.to_account_info(), 
            }, 
        ), 
        input, 
    )?; 
    token::transfer( 
        CpiContext::new_with_signer( 
            ctx.accounts.token_program.to_account_info(), 
            Transfer { 
                from: ctx.accounts.pool_account_b.to_account_info(), 
                to: ctx.accounts.trader_account_b.to_account_info(), 
                authority: ctx.accounts.pool_authority.to_account_info(), 
            }, 
            signer_seeds, 
        ), 
        output, 
    )?; 
} else { 
    token::transfer( 
        CpiContext::new_with_signer( 
            ctx.accounts.token_program.to_account_info(), 
            Transfer { 
                from: ctx.accounts.pool_account_a.to_account_info(), 
                to: ctx.accounts.trader_account_a.to_account_info(), 
                authority: ctx.accounts.pool_authority.to_account_info(), 
            }, 
            signer_seeds, 
        ), 
        input, 
    )?; 
    token::transfer( 
        CpiContext::new( 
            ctx.accounts.token_program.to_account_info(), 
            Transfer { 
                from: ctx.accounts.trader_account_b.to_account_info(), 
                to: ctx.accounts.pool_account_b.to_account_info(), 
                authority: ctx.accounts.trader.to_account_info(), 
            }, 
        ), 
        output, 
    )?; 
} 

msg!( 
    "Traded {} tokens ({} after fees) for {}", 
    input, 
    taxed_input, 
    output 
); 


// Verify the invariant still holds 
 // Reload accounts because of the CPIs 
 // We tolerate if the new invariant is higher because it means a rounding error for LPs 
 ctx.accounts.pool_account_a.reload()?; 
 ctx.accounts.pool_account_b.reload()?; 
 if invariant > ctx.accounts.pool_account_a.amount * ctx.accounts.pool_account_a.amount { 
     return err!(TutorialError::InvariantViolated); 
 } 
  
 Ok(()) 
