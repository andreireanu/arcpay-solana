//! Backend authorization: verifies the ed25519 signatures the backend co-signs
//! for every privileged action.
//!
//! Each gated entry point prepends a native Ed25519-program instruction
//! carrying the backend's signature over a canonical little-endian message.
//! These helpers locate that instruction in the Instructions sysvar, assert the
//! signature/pubkey/message are inline (the offset fields must be the `0xFFFF`
//! sentinel, so an attacker can't point them at bytes in another instruction),
//! check the signer key matches `Config.backend_pubkey`, validate the expiry,
//! and compare the signed message field-by-field against the instruction's
//! arguments. Per-flow message layouts are documented on each `verify_*` fn.

pub mod auth_accept_offer;
pub mod auth_buy;
pub mod auth_offer;
