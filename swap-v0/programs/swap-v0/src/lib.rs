use anchor_lang::prelude::*;

declare_id!("DGQDm3ApWihv9ugagD7SySKzdTpqdAEaW72mqR1xhzNp");

#[program]
pub mod swap_v0 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
