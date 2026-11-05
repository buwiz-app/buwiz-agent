//! Shared types for Buwiz mini apps. This crate is the source: the
//! TypeScript types and JSON Schema in `packages/agent` are generated from
//! it, and `vectors/` proves the two halves agree.
//!
//! - [`Envelope`], [`Op`], [`Response`], [`UiTree`]: the UI wire protocol,
//!   every message naming its [`Target`] app.
//! - [`CtxRef`], [`Provenance`], [`Grant`]: context handles.
//! - [`ToolSpec`], [`Effect`]: what each operation does and needs.
//! - [`Caller`], [`Approval`], [`authorize_press`]: who pressed, and whether
//!   they may.

pub mod ctx;
pub mod generate;
pub mod ids;
pub mod message;
pub mod press;
pub mod tool;
pub mod tree;

pub use ctx::{CtxRef, Grant, Provenance};
pub use ids::{
    ActorId, AgentId, ContentHash, Fingerprint, Invalid, MAX_SAFE_INTEGER, OnceId,
    PROTOCOL_VERSION, SafeU64, Target, Version,
};
pub use message::{
    AgentEvent, AssertSpec, Challenge, ChallengeOp, DeliveryMode, Envelope, EventLevel, EventPoll,
    HelloAuth, HelloInfo, KeybindingInfo, KeybindingScope, Op, Response, ScreenshotMode,
    SnapshotFormat, SnapshotResult,
};
pub use press::{
    Allowed, Approval, Caller, PressAction, PressedBy, Refused, authorize_press,
    authorize_press_wire,
};
pub use tool::{Effect, Fill, Need, Returns, ToolError, ToolSpec, ValueKind};
pub use tree::{AddedNode, Bounds, ChangedNode, PlatformKind, TreeDiff, UiNode, UiTree};
