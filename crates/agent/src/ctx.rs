//! Context refs (D3): the broker keeps shared context, and apps hold a
//! handle, `ctx://<kind>/<id>`, instead of the bytes.

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::{ActorId, ContentHash, Invalid, Target, text_type};

const SCHEME: &str = "ctx://";

fn is_kind(text: &str) -> bool {
    let bytes = text.as_bytes();
    (1..=32).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_' || *b == b'-')
}

fn is_id(text: &str) -> bool {
    !text.is_empty()
        && text.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        })
}

fn is_ctx_ref(text: &str) -> bool {
    text.len() <= 256
        && text
            .strip_prefix(SCHEME)
            .and_then(|rest| rest.split_once('/'))
            .is_some_and(|(kind, id)| is_kind(kind) && is_id(id))
}

text_type!(
    /// `ctx://<kind>/<id>`, such as `ctx://file/sales.xlsx` or
    /// `ctx://books/proposal/2551Q-2026Q3`. `kind` is the first segment and
    /// `id` is the rest, which may hold more `/`. There is one spelling per
    /// ref: no escapes, no query, no fragment, and no `.` or `..` segments,
    /// so printing a parsed ref gives the same text.
    CtxRef,
    is_ctx_ref,
    "^ctx://[a-z][a-z0-9_-]{0,31}/(?!\\.{1,2}(/|$))[A-Za-z0-9._-]+(/(?!\\.{1,2}(/|$))[A-Za-z0-9._-]+)*$",
    256
);

impl CtxRef {
    pub fn new(kind: &str, id: &str) -> Result<Self, Invalid> {
        if !is_kind(kind) {
            return Err(Invalid("CtxRef"));
        }
        Self::parse(&format!("{SCHEME}{kind}/{id}"))
    }

    pub fn kind(&self) -> &str {
        self.split().0
    }

    pub fn id(&self) -> &str {
        self.split().1
    }

    fn split(&self) -> (&str, &str) {
        // Parsing proved the scheme and the first `/` are there.
        self.as_str()[SCHEME.len()..]
            .split_once('/')
            .unwrap_or_default()
    }
}

/// Where a piece of context came from. Holds no content, only handles and a
/// hash of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Who made it.
    pub made_by: ActorId,
    /// What it was made from. Empty for an original, such as an upload.
    pub from: Vec<CtxRef>,
    /// The hash of its content.
    pub content_hash: ContentHash,
}

/// Which apps may read a ref.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    pub ctx: CtxRef,
    pub readers: Vec<Target>,
}
