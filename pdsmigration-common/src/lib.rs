use bsky_sdk::api::types::string::Did;
use multibase::Base::Base58Btc;
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};

mod activate_account;
mod agent;
mod create_account;
mod deactivate_account;
mod errors;
mod export_all_blobs;
mod export_blobs;
mod export_pds;
mod import_pds;
mod migrate_plc;
mod migrate_preferences;
mod missing_blobs;
mod request_token;
mod service_auth;
mod upload_blobs;

pub use activate_account::*;
pub use agent::*;
pub use create_account::*;
pub use deactivate_account::*;
pub use errors::*;
pub use export_all_blobs::*;
pub use export_blobs::*;
pub use export_pds::*;
pub use import_pds::*;
pub use migrate_plc::*;
pub use migrate_preferences::*;
pub use missing_blobs::*;
pub use request_token::*;
pub use service_auth::*;
pub use upload_blobs::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetRepoRequest {
    pub did: Did,
    pub token: String,
}

pub fn multicodec_wrap(bytes: Vec<u8>) -> Vec<u8> {
    let mut buf = [0u8; 3];
    unsigned_varint::encode::u16(0xe7, &mut buf);
    let mut v: Vec<u8> = Vec::new();
    for b in &buf {
        v.push(*b);
        // varint uses first bit to indicate another byte follows, stop if not the case
        if *b <= 127 {
            break;
        }
    }
    v.extend(bytes);
    v
}

pub fn public_key_to_did_key(public_key: PublicKey) -> String {
    let pk_compact = public_key.serialize();
    let pk_wrapped = multicodec_wrap(pk_compact.to_vec());
    let pk_multibase = multibase::encode(Base58Btc, pk_wrapped.as_slice());
    let public_key_str = format!("did:key:{pk_multibase}");
    public_key_str
}
