//! The wire protocol: the envelope, its ops, the response and the auth
//! handshake. Shapes from gpui-agent's protocol, changed in two ways:
//!
//! - Every message carries `target`, the app it is for (D2). gpui-agent used
//!   `target` for the node inside the app; here that field is `node`.
//! - Unknown fields are refused rather than dropped, so a misspelled
//!   `approval` cannot pass as no approval without anyone seeing it.

use std::collections::BTreeMap;
use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::ids::{AgentId, Fingerprint, SafeU64, Target, Version};
use crate::press::Caller;
use crate::tree::{PlatformKind, TreeDiff, UiTree};

fn is_false(value: &bool) -> bool {
    !*value
}

/// How click, type and key reach the app. `semantic` (the default) calls the
/// widget's handler by id; `virtual` synthesizes in-window events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMode {
    #[default]
    Semantic,
    Virtual,
}

impl DeliveryMode {
    fn is_semantic(&self) -> bool {
        matches!(self, Self::Semantic)
    }
}

/// Which keymap a `keybinding` fires in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KeybindingScope {
    Focused,
    Global,
}

/// How `screenshot` captures: the painted window, or scrolled and stitched.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScreenshotMode {
    #[default]
    Viewport,
    Scrolled,
}

impl ScreenshotMode {
    fn is_viewport(&self) -> bool {
        matches!(self, Self::Viewport)
    }
}

/// The `snapshot` answer's form: the JSON tree, or one line per visible node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotFormat {
    Json,
    Compact,
}

/// What `assert` and `wait_until` check about one node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct AssertSpec {
    pub node: AgentId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_viewport: Option<bool>,
}

/// What a message asks the target to do. Ops without fields are written as
/// empty struct variants because serde would ignore extra fields on unit ones.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
#[ts(optional_fields)]
pub enum Op {
    Hello {},
    /// Read the tree. `since` asks for a diff against a revision the client
    /// holds, and `fingerprints` adds an id to fingerprint map.
    Snapshot {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        since: Option<SafeU64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<SnapshotFormat>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fingerprints: Option<bool>,
    },
    Revision {},
    /// Poll the bounded event stream for events after `since`.
    Events {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        since: Option<SafeU64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
    },
    /// `expect_fingerprint`, here and on the other input ops, is the
    /// staleness gate: on a mismatch nothing happens.
    Click {
        node: AgentId,
        #[serde(default, skip_serializing_if = "DeliveryMode::is_semantic")]
        #[ts(as = "Option<DeliveryMode>", optional)]
        delivery: DeliveryMode,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expect_fingerprint: Option<Fingerprint>,
    },
    Type {
        node: AgentId,
        text: String,
        #[serde(default, skip_serializing_if = "DeliveryMode::is_semantic")]
        #[ts(as = "Option<DeliveryMode>", optional)]
        delivery: DeliveryMode,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expect_fingerprint: Option<Fingerprint>,
    },
    SetValue {
        node: AgentId,
        value: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expect_fingerprint: Option<Fingerprint>,
    },
    Key {
        node: AgentId,
        key: String,
        #[serde(default, skip_serializing_if = "DeliveryMode::is_semantic")]
        #[ts(as = "Option<DeliveryMode>", optional)]
        delivery: DeliveryMode,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expect_fingerprint: Option<Fingerprint>,
    },
    /// Fire an action by its binding id.
    Keybinding {
        binding: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        chord: Option<String>,
        scope: KeybindingScope,
        #[serde(default, skip_serializing_if = "is_false")]
        #[ts(as = "Option<bool>", optional)]
        confirm: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        #[ts(as = "Option<bool>", optional)]
        activate: bool,
    },
    Keybindings {},
    Assert(AssertSpec),
    Invoke {
        name: AgentId,
        #[serde(default, skip_serializing_if = "Value::is_null")]
        #[ts(type = "unknown", optional)]
        args: Value,
    },
    /// Wait for the host to be ready.
    Wait {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<SafeU64>,
    },
    /// Poll `until` until it holds or `timeout_ms` passes. gpui-agent flattens
    /// the spec; serde cannot refuse unknown fields through a flatten here.
    WaitUntil {
        timeout_ms: SafeU64,
        until: AssertSpec,
    },
    Shutdown {},
    /// A PNG of the app's surface, written by the host to `path`. `node`
    /// names the scroll view when `mode` is `scrolled`.
    Screenshot {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(default, skip_serializing_if = "ScreenshotMode::is_viewport")]
        #[ts(as = "Option<ScreenshotMode>", optional)]
        mode: ScreenshotMode,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        node: Option<AgentId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_height_px: Option<u32>,
    },
}

/// One request: which app it is for, who is asking, and the op, whose fields
/// sit beside these on the wire.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[schemars(extend("unevaluatedProperties" = false))]
#[ts(optional_fields)]
pub struct Envelope {
    pub v: Version,
    pub id: String,
    pub target: Target,
    /// Set by the host from how the message arrived. Absent means nobody
    /// established a caller, and every press is refused.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caller: Option<Caller>,
    /// The Bot whose turn this is, set by the host. It only narrows: a host
    /// refuses a screen another Bot's chat opened. Absent in a plain chat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<String>,
    /// HMAC-SHA256(token, session nonce) as lowercase hex. Never the raw token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
    #[serde(flatten)]
    pub op: Op,
}

impl Envelope {
    pub fn new(id: impl Into<String>, target: Target, op: Op) -> Self {
        Self {
            v: Version::default(),
            id: id.into(),
            target,
            caller: None,
            bot: None,
            auth: None,
            op,
        }
    }
}

impl fmt::Debug for Envelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Envelope")
            .field("v", &self.v)
            .field("id", &self.id)
            .field("target", &self.target)
            .field("caller", &self.caller)
            .field("bot", &self.bot)
            .field("auth", &self.auth.as_ref().map(|_| "<redacted>"))
            .field("op", &self.op)
            .finish()
    }
}

/// Whether the host needs `auth` on every request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HelloAuth {
    #[default]
    None,
    Required,
}

/// The `hello` answer. `targets` lists the apps this host routes to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct HelloInfo {
    pub protocol: Version,
    pub targets: Vec<Target>,
    pub platform: PlatformKind,
    pub os: String,
    pub ready: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<DeliveryMode>>", optional)]
    pub deliveries: Vec<DeliveryMode>,
    pub auth: HelloAuth,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub features: Vec<String>,
}

/// One answer. `id` matches the request and `target` is the app that answered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Response {
    pub v: Version,
    pub id: String,
    pub target: Target,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hello: Option<HelloInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<UiTree>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "unknown", optional)]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprints: Option<BTreeMap<AgentId, Fingerprint>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<SafeU64>,
}

impl Response {
    pub fn ok(id: impl Into<String>, target: Target) -> Self {
        Self {
            v: Version::default(),
            id: id.into(),
            target,
            ok: true,
            error: None,
            hello: None,
            tree: None,
            result: None,
            fingerprints: None,
            rev: None,
        }
    }

    pub fn err(id: impl Into<String>, target: Target, error: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(error.into()),
            ..Self::ok(id, target)
        }
    }
}

/// The tag of a challenge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeOp {
    #[default]
    Challenge,
}

/// The first line a host with a token writes on a new connection. The client
/// answers by putting HMAC-SHA256(token, nonce) in `auth` on each request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Challenge {
    pub v: Version,
    /// The app whose host issued the challenge.
    pub target: Target,
    pub op: ChallengeOp,
    /// 32 random bytes as lowercase hex.
    pub nonce: String,
}

/// `result` of a `snapshot` with `since` or `format: "compact"`. One of:
/// nothing changed; a diff; the full tree is in `tree` because the revision
/// is too old; or the compact text.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct SnapshotResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unchanged: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff: Option<TreeDiff>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact: Option<String>,
}

/// Severity of one event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// One host event. `seq` is the only ordering: events carry no clock.
/// `source` is the emitting component (gpui-agent's `target`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentEvent {
    pub seq: SafeU64,
    pub level: EventLevel,
    pub source: String,
    pub message: String,
}

/// `result` of `events`. `lagged` means events were lost to the ring's limit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EventPoll {
    pub events: Vec<AgentEvent>,
    pub latest: SafeU64,
    pub lagged: bool,
}

/// One entry of the `keybindings` result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KeybindingInfo {
    pub id: String,
    pub chord: String,
    pub scope: KeybindingScope,
    pub dangerous: bool,
}
