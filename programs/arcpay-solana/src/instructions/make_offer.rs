use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MakeOffer<'info> {
    pub user: Signer<'info>,
}

pub fn handler(_ctx: Context<MakeOffer>) -> Result<()> {
    Ok(())
}
