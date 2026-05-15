use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked;
use crate::constants::ED25519_PROGRAM_ID;
use crate::errors::ArcPayError;

/// Verify backend authorization for a buy instruction.
///
/// Signed message layout (104 bytes):
///   [0..32]   buyer pubkey
///   [32..64]  seller pubkey
///   [64..80]  offer_id (UUID bytes)
///   [80..88]  seller_amount (u64 LE)
///   [88..96]  fee_amount (u64 LE)
///   [96..104] expiry (i64 LE)
pub fn verify_buy_auth(
    instructions_sysvar: &AccountInfo,
    backend_pubkey: &Pubkey,
    buyer: &Pubkey,
    seller: &Pubkey,
    offer_id: &[u8; 16],
    seller_amount: u64,
    fee_amount: u64,
    expiry: i64,
) -> Result<()> {
    let clock = Clock::get()?;
    require!(clock.unix_timestamp < expiry, ArcPayError::AuthorizationExpired);

    // Scan for the ed25519 instruction — wallet adapters may prepend compute
    // budget instructions so it is not guaranteed to be at index 0.
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

    // Ed25519 instruction data layout:
    // byte 0        count (u8)
    // byte 1        padding (u8)
    // byte 2-3      signature_offset
    // byte 4-5      signature_instruction_index
    // byte 6-7      public_key_offset
    // byte 8-9      public_key_instruction_index
    // byte 10-11    message_data_offset
    // byte 12-13    message_data_size
    // byte 14-15    message_instruction_index
    // byte 16+      signature(64) | pubkey(32) | message(104)
    require!(data.len() >= 16, ArcPayError::InvalidAuthorizationSignature);
    require!(data[0] >= 1,     ArcPayError::InvalidAuthorizationSignature);

    // Assert all data is inline (0xFFFF sentinel) to prevent an attacker
    // from pointing offsets at bytes in a different instruction they control.
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
    require!(msg_size == 104,                      ArcPayError::InvalidAuthorizationSignature);

    let pk = Pubkey::try_from(&data[pk_offset..pk_offset + 32])
        .map_err(|_| error!(ArcPayError::InvalidAuthorizationSignature))?;
    require_keys_eq!(pk, *backend_pubkey, ArcPayError::InvalidAuthorizationSignature);

    let msg = &data[msg_offset..msg_offset + 104];

    require!(msg[0..32]   == buyer.as_ref()[..],               ArcPayError::InvalidAuthorizationSignature);
    require!(msg[32..64]  == seller.as_ref()[..],              ArcPayError::InvalidAuthorizationSignature);
    require!(msg[64..80]  == offer_id[..],                     ArcPayError::InvalidAuthorizationSignature);
    require!(msg[80..88]  == seller_amount.to_le_bytes()[..],  ArcPayError::InvalidAuthorizationSignature);
    require!(msg[88..96]  == fee_amount.to_le_bytes()[..],     ArcPayError::InvalidAuthorizationSignature);
    require!(msg[96..104] == expiry.to_le_bytes()[..],         ArcPayError::InvalidAuthorizationSignature);

    Ok(())
}
