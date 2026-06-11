use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked;
use crate::constants::ED25519_PROGRAM_ID;
use crate::errors::ArcPayError;

/// Verify backend authorization for an accept_offer instruction.
///
/// The accept is a consent event only — no funds move — so the message carries
/// no amounts. The uuid is the witness id that the backend maps to the accepted
/// counter offers in the database.
///
/// Signed message layout (56 bytes):
///   [0..32]  seller pubkey
///   [32..48] uuid (16 bytes)
///   [48..56] expiry (i64 LE)
pub fn verify_accept_offer_auth(
    instructions_sysvar: &AccountInfo,
    backend_pubkey: &Pubkey,
    seller: &Pubkey,
    uuid: &[u8; 16],
    expiry: i64,
) -> Result<()> {
    let clock = Clock::get()?;
    require!(clock.unix_timestamp < expiry, ArcPayError::AuthorizationExpired);

    let mut ed25519_ix = None;
    let mut i: usize = 0;
    loop {
        match load_instruction_at_checked(i, instructions_sysvar) {
            Ok(ix) if ix.program_id == ED25519_PROGRAM_ID => {
                ed25519_ix = Some(ix);
                break;
            }
            Ok(_) => i += 1,
            Err(_) => break,
        }
    }
    let ix = ed25519_ix.ok_or(error!(ArcPayError::MissingEd25519Instruction))?;
    let data = &ix.data;

    require!(data.len() >= 16, ArcPayError::InvalidAuthorizationSignature);
    require!(data[0] >= 1,     ArcPayError::InvalidAuthorizationSignature);

    let sig_ix_idx = u16::from_le_bytes([data[4],  data[5]]);
    let pk_ix_idx  = u16::from_le_bytes([data[8],  data[9]]);
    let msg_ix_idx = u16::from_le_bytes([data[14], data[15]]);
    require!(
        sig_ix_idx == u16::MAX && pk_ix_idx == u16::MAX && msg_ix_idx == u16::MAX,
        ArcPayError::InvalidAuthorizationSignature
    );

    let pk_offset  = u16::from_le_bytes([data[6],  data[7]])  as usize;
    let msg_offset = u16::from_le_bytes([data[10], data[11]]) as usize;
    let msg_size   = u16::from_le_bytes([data[12], data[13]]) as usize;

    require!(data.len() >= pk_offset  + 32,       ArcPayError::InvalidAuthorizationSignature);
    require!(data.len() >= msg_offset + msg_size,  ArcPayError::InvalidAuthorizationSignature);
    require!(msg_size == 56,                       ArcPayError::InvalidAuthorizationSignature);

    let pk = Pubkey::try_from(&data[pk_offset..pk_offset + 32])
        .map_err(|_| error!(ArcPayError::InvalidAuthorizationSignature))?;
    require_keys_eq!(pk, *backend_pubkey, ArcPayError::InvalidAuthorizationSignature);

    let msg = &data[msg_offset..msg_offset + 56];

    require!(msg[0..32]  == seller.as_ref()[..],      ArcPayError::InvalidAuthorizationSignature);
    require!(msg[32..48] == uuid[..],                 ArcPayError::InvalidAuthorizationSignature);
    require!(msg[48..56] == expiry.to_le_bytes()[..], ArcPayError::InvalidAuthorizationSignature);

    Ok(())
}
