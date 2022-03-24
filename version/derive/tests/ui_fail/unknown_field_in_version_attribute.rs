use iroha_version_derive::{declare_versioned, version};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

declare_versioned!(VersionedMessage 1..2);

#[version(n = 1, versioned = "VersionedMessage", derive = "Clone")]
#[derive(Debug, Clone, Decode, Encode, Deserialize, Serialize)]
struct Message;

pub fn main() {}