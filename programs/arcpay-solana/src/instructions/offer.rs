use crate::auth::auth_offer::verify_offer_auth;
use crate::errors::ArcPayError;
use crate::state::{Config, OfferCreated, OfferRecord, SellerVault};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_lang::system_program;

#[derive(Accounts)]
#[instruction(uuid: [u8; 16])]
pub struct Offer<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: seller identity is verified via backend-signed ed25519 message
    pub seller: SystemAccount<'info>,

    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        init_if_needed,
        payer = buyer,
        space = 8 + SellerVault::INIT_SPACE,
        seeds = [b"vault", seller.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, SellerVault>,

    #[account(
        init,
        payer = buyer,
        space = 8 + OfferRecord::INIT_SPACE,
        seeds = [b"offer", uuid.as_ref()],
        bump,
    )]
    pub offer_record: Account<'info, OfferRecord>,

    /// CHECK: instructions sysvar, used to verify the prepended ed25519 instruction
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Offer>, uuid: [u8; 16], amount: u64, expiry: i64) -> Result<()> {
    require!(amount > 0, ArcPayError::InvalidAmount);

    verify_offer_auth(
        &ctx.accounts.instructions_sysvar,
        &ctx.accounts.config.backend_pubkey,
        &ctx.accounts.buyer.key(),
        &uuid,
        amount,
        expiry,
    )?;

    let vault = &mut ctx.accounts.vault;
    vault.seller = ctx.accounts.seller.key();
    vault.bump = ctx.bumps.vault;

    let record = &mut ctx.accounts.offer_record;
    record.buyer = ctx.accounts.buyer.key();
    record.seller = ctx.accounts.seller.key();
    record.amount = amount;
    record.bump = ctx.bumps.offer_record;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    emit!(OfferCreated {
        uuid,
        buyer: ctx.accounts.buyer.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
