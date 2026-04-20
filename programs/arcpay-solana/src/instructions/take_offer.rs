use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    pub user: Signer<'info>,
}

pub fn handler(_ctx: Context<TakeOffer>) -> Result<()> {
    Ok(())
}
