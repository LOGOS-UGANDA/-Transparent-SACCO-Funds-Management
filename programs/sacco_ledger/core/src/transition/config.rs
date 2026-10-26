//! The singleton registry config shard.

use crate::codec::encode;
use crate::effect::ConfigEffect;
use crate::ProgramConfig;

/// Applies a config effect to the config shard.
#[must_use]
pub fn apply(effect: ConfigEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        ConfigEffect::Init { version } => {
            assert!(pre.is_empty(), "Config account is already initialized");
            Some(encode(&ProgramConfig { version }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::decode;

    #[test]
    fn init_writes_the_version() {
        let post = apply(ConfigEffect::Init { version: 1 }, &[]).expect("writes");
        let decoded: ProgramConfig = decode(&post).expect("config data");
        assert_eq!(decoded.version, 1);
    }

    #[test]
    #[should_panic(expected = "Config account is already initialized")]
    fn init_is_write_once() {
        let pre = encode(&ProgramConfig { version: 1 });
        let _ = apply(ConfigEffect::Init { version: 1 }, &pre);
    }
}
