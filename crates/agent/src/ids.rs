//! Validated names and small wire values. Each one parses or refuses, and
//! once built it is known good, so nothing downstream checks it again.
//!
//! Errors never repeat the refused text: a mistaken value could be a TIN or
//! a secret, and errors end up in logs.

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The wire version this crate speaks. It starts at 1 and has nothing to do
/// with gpui-agent's 2 or Mini App Protocol's 1.0.
pub const PROTOCOL_VERSION: u32 = 1;

/// The largest integer JavaScript holds exactly (2^53 - 1).
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// A value refused by a parser. It names the type, never the value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid(pub &'static str);

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a valid {}", self.0)
    }
}

impl std::error::Error for Invalid {}

/// A string type with one validity check. `pattern` says the same as the
/// check in JSON Schema; the vectors hold the two together.
macro_rules! text_type {
    ($(#[$doc:meta])* $name:ident, $check:path, $pattern:expr, $max:expr) => {
        $(#[$doc])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
        #[serde(try_from = "String", into = "String")]
        #[ts(type = "string")]
        pub struct $name(String);

        impl $name {
            pub fn parse(text: &str) -> Result<Self, Invalid> {
                if $check(text) {
                    Ok(Self(text.to_owned()))
                } else {
                    Err(Invalid(stringify!($name)))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = Invalid;

            fn try_from(text: String) -> Result<Self, Invalid> {
                if $check(&text) {
                    Ok(Self(text))
                } else {
                    Err(Invalid(stringify!($name)))
                }
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> String {
                value.0
            }
        }

        impl FromStr for $name {
            type Err = Invalid;

            fn from_str(text: &str) -> Result<Self, Invalid> {
                Self::parse(text)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({:?})", stringify!($name), self.0)
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> Cow<'static, str> {
                stringify!($name).into()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({ "type": "string", "pattern": $pattern, "maxLength": $max })
            }
        }
    };
}

pub(crate) use text_type;

fn lower_alnum(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit()
}

fn lower_hex(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
}

fn is_target(text: &str) -> bool {
    let bytes = text.as_bytes();
    (2..=16).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|b| lower_alnum(*b))
}

fn is_dotted(text: &str) -> bool {
    text.split('.').all(|label| {
        let bytes = label.as_bytes();
        !bytes.is_empty()
            && bytes[0].is_ascii_lowercase()
            && bytes
                .iter()
                .all(|b| lower_alnum(*b) || *b == b'_' || *b == b'-')
    })
}

fn is_agent_id(text: &str) -> bool {
    text.len() <= 128 && is_dotted(text)
}

fn is_actor_id(text: &str) -> bool {
    text.len() <= 64 && is_dotted(text)
}

fn is_once_id(text: &str) -> bool {
    let bytes = text.as_bytes();
    (8..=64).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|b| lower_alnum(*b) || *b == b'-')
}

fn is_content_hash(text: &str) -> bool {
    text.strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(lower_hex))
}

fn is_fingerprint(text: &str) -> bool {
    text.len() == 16 && text.bytes().all(lower_hex)
}

text_type!(
    /// The app a message is for, such as `bot`, `books`, `ebirforms` or
    /// `sheet`. Same grammar as a plugin's `short` in Mini App Protocol 1.0.
    Target,
    is_target,
    "^[a-z][a-z0-9]{1,15}$",
    16
);

text_type!(
    /// A stable id inside an app, such as `ledger.post`: dot-separated
    /// labels of lowercase letters, digits, `_` and `-`, each starting with
    /// a letter. UI nodes, actions, tools and their parameters use it.
    AgentId,
    is_agent_id,
    "^[a-z][a-z0-9_-]*(\\.[a-z][a-z0-9_-]*)*$",
    128
);

text_type!(
    /// Who did something, as a handle such as `person.owner`, `agent.broker`
    /// or `routine.month-end`. It is a handle on purpose: `@`, spaces and
    /// leading digits are refused, so an email or a TIN cannot go here.
    ActorId,
    is_actor_id,
    "^[a-z][a-z0-9_-]*(\\.[a-z][a-z0-9_-]*)*$",
    64
);

text_type!(
    /// The one-time id of a single press. The host makes a fresh one for
    /// each press it asks the person about and retires it after use.
    OnceId,
    is_once_id,
    "^[a-z][a-z0-9-]{7,63}$",
    64
);

text_type!(
    /// `sha256:` then 64 lowercase hex digits.
    ContentHash,
    is_content_hash,
    "^sha256:[0-9a-f]{64}$",
    71
);

text_type!(
    /// A UI node's content fingerprint: 16 lowercase hex digits. gpui-agent
    /// sends it as a u64 number, which JavaScript would round.
    Fingerprint,
    is_fingerprint,
    "^[0-9a-f]{16}$",
    16
);

/// A count, revision or duration that both halves hold exactly: a u64 no
/// larger than 2^53 - 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(try_from = "u64", into = "u64")]
#[ts(type = "number")]
pub struct SafeU64(u64);

impl SafeU64 {
    pub fn get(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for SafeU64 {
    type Error = Invalid;

    fn try_from(value: u64) -> Result<Self, Invalid> {
        if value <= MAX_SAFE_INTEGER {
            Ok(Self(value))
        } else {
            Err(Invalid("SafeU64"))
        }
    }
}

impl From<SafeU64> for u64 {
    fn from(value: SafeU64) -> u64 {
        value.0
    }
}

impl JsonSchema for SafeU64 {
    fn schema_name() -> Cow<'static, str> {
        "SafeU64".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({ "type": "integer", "minimum": 0, "maximum": MAX_SAFE_INTEGER })
    }
}

/// The `v` of a message. Only the version this crate speaks parses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(try_from = "u32", into = "u32")]
#[ts(type = "1")]
pub struct Version(u32);

impl Default for Version {
    fn default() -> Self {
        Self(PROTOCOL_VERSION)
    }
}

impl TryFrom<u32> for Version {
    type Error = Invalid;

    fn try_from(value: u32) -> Result<Self, Invalid> {
        if value == PROTOCOL_VERSION {
            Ok(Self(value))
        } else {
            Err(Invalid("Version"))
        }
    }
}

impl From<Version> for u32 {
    fn from(value: Version) -> u32 {
        value.0
    }
}

impl JsonSchema for Version {
    fn schema_name() -> Cow<'static, str> {
        "Version".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({ "const": PROTOCOL_VERSION })
    }
}
