use anchor_lang::prelude::*; 
 use anchor_spl::{ 
     associated_token::AssociatedToken, 
     token::{self, Mint, MintTo, Token, TokenAccount, Transfer}, 
 }; 
 use fixed::types::I64F64; 
 use fixed_sqrt::FixedSqrt; 
  
 use crate::{ 
     constants::{AUTHORITY_SEED, LIQUIDITY_SEED, MINIMUM_LIQUIDITY}, 
     errors::TutorialError, 
     state::Pool, 
 }; 
  
 pub fn deposit_liquidity( 
     ctx: Context<DepositLiquidity>, 
     amount_a: u64, 
     amount_b: u64, 
 ) -> Result<()> { 
     // Prevent depositing assets the depositor does not own 
     let mut amount_a = if amount_a > ctx.accounts.depositor_account_a.amount { 
         ctx.accounts.depositor_account_a.amount 
     } else { 
         amount_a 
     }; 
     let mut amount_b = if amount_b > ctx.accounts.depositor_account_b.amount { 
         ctx.accounts.depositor_account_b.amount 
     } else { 
         amount_b 
     }; 

    // Making sure they are provided in the same proportion as existing liquidity 
 let pool_a = &ctx.accounts.pool_account_a; 
 let pool_b = &ctx.accounts.pool_account_b; 
 // Defining pool creation like this allows attackers to frontrun pool creation with bad ratios 
 let pool_creation = pool_a.amount == 0 && pool_b.amount == 0; 
 (amount_a, amount_b) = if pool_creation { 
     // Add as is if there is no liquidity 
     (amount_a, amount_b) 
 } else { 
     let ratio = I64F64::from_num(pool_a.amount) 
         .checked_mul(I64F64::from_num(pool_b.amount)) 
         .unwrap(); 
     if pool_a.amount > pool_b.amount { 
         ( 
             I64F64::from_num(amount_b) 
                 .checked_mul(ratio) 
                 .unwrap() 
                 .to_num::<u64>(), 
             amount_b, 
         ) 
     } else { 
         ( 
             amount_a, 
             I64F64::from_num(amount_a) 
                 .checked_div(ratio) 
                 .unwrap() 
                 .to_num::<u64>(), 
         ) 
     } 
 }; 

 
// Computing the amount of liquidity about to be deposited 
let mut liquidity = I64F64::from_num(amount_a) 
.checked_mul(I64F64::from_num(amount_b)) 
.unwrap() 
.sqrt() 
.to_num::<u64>(); 

// Lock some minimum liquidity on the first deposit 
if pool_creation { 
if liquidity < MINIMUM_LIQUIDITY { 
    return err!(TutorialError::DepositTooSmall); 
} 

liquidity -= MINIMUM_LIQUIDITY; 
} 


// Transfer tokens to the pool 
token::transfer( 
    CpiContext::new( 
        ctx.accounts.token_program.to_account_info(), 
        Transfer { 
            from: ctx.accounts.depositor_account_a.to_account_info(), 
            to: ctx.accounts.pool_account_a.to_account_info(), 
            authority: ctx.accounts.depositor.to_account_info(), 
        }, 
    ), 
    amount_a, 
)?; 
token::transfer( 
    CpiContext::new( 
        ctx.accounts.token_program.to_account_info(), 
        Transfer { 
            from: ctx.accounts.depositor_account_b.to_account_info(), 
            to: ctx.accounts.pool_account_b.to_account_info(), 
            authority: ctx.accounts.depositor.to_account_info(), 
        }, 
    ), 
    amount_b, 
)?; 



// Mint the liquidity to user 
let authority_bump = *ctx.bumps.get("pool_authority").unwrap(); 
let authority_seeds = &[ 
    &ctx.accounts.pool.amm.to_bytes(), 
    &ctx.accounts.mint_a.key().to_bytes(), 
    &ctx.accounts.mint_b.key().to_bytes(), 
    AUTHORITY_SEED.as_bytes(), 
    &[authority_bump], 
]; 
let signer_seeds = &[&authority_seeds[..]]; 
token::mint_to( 
    CpiContext::new_with_signer( 
        ctx.accounts.token_program.to_account_info(), 
        MintTo { 
            mint: ctx.accounts.mint_liquidity.to_account_info(), 
            to: ctx.accounts.depositor_account_liquidity.to_account_info(), 
            authority: ctx.accounts.pool_authority.to_account_info(), 
        }, 
        signer_seeds, 
    ), 
    liquidity, 
)?; 