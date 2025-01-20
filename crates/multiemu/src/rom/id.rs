use data_encoding::HEXLOWER_PERMISSIVE;
use native_db::ToKey;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{fmt::Display, io::Read, str::FromStr};

#[derive(
    Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
/// Sha-1 of rom
pub struct RomId([u8; 20]);

impl RomId {
    pub const fn new(data: [u8; 20]) -> Self {
        Self(data)
    }

    pub fn from_read(data: &mut impl Read) -> Self {
        let mut hasher = Sha1::new();
        std::io::copy(data, &mut hasher).unwrap();
        Self(hasher.finalize().into())
    }
}

impl ToKey for RomId {
    fn to_key(&self) -> native_db::Key {
        native_db::Key::new(self.0.to_vec())
    }

    fn key_names() -> Vec<String> {
        vec!["romid".to_string()]
    }
}

impl AsRef<[u8]> for RomId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 20]> for RomId {
    fn from(value: [u8; 20]) -> Self {
        Self(value)
    }
}

impl Display for RomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", HEXLOWER_PERMISSIVE.encode(&self.0))
    }
}

impl FromStr for RomId {
    type Err = data_encoding::DecodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = HEXLOWER_PERMISSIVE.decode(s.as_bytes())?;
        Ok(Self(bytes.try_into().unwrap()))
    }
}
