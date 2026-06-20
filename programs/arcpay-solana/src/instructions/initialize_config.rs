use crate::errors::ArcPayError;
use crate::state::Config;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + Config::INIT_SPACE,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, Config>,

    /// This program's account, verified to point at `program_data`. Together
    /// with the constraint below it restricts initialization to the program's
    /// upgrade authority (the deployer), closing the deploy→init front-running
    /// window where anyone could otherwise become admin.
    #[account(constraint = program.programdata_address()? == Some(program_data.key()))]
    pub program: Program<'info, crate::program::ArcpaySolana>,

    #[account(
        constraint = program_data.upgrade_authority_address == Some(admin.key())
            @ ArcPayError::Unauthorized,
    )]
    pub program_data: Account<'info, ProgramData>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeConfig>, backend_pubkey: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.backend_pubkey = backend_pubkey;
    config.bump = ctx.bumps.config;
    Ok(())
}
