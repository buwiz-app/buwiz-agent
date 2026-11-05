//! The generated halves: TypeScript types from ts-rs and JSON Schema from
//! schemars, both read off the types in this crate.

use schemars::generate::SchemaSettings;
use serde_json::{Map, Value, json};
use ts_rs::{Config, TS};

use crate::*;

/// Where the generated files live, from the repo root.
pub const TYPESCRIPT_PATH: &str = "packages/agent/src/generated.ts";
pub const SCHEMA_PATH: &str = "packages/agent/schema/buwiz-agent.schema.json";

/// Calls `$visit` for every exported type, once.
macro_rules! each_type {
    ($visit:ident!($state:expr)) => {
        $visit!(
            $state,
            Target,
            AgentId,
            ActorId,
            OnceId,
            ContentHash,
            Fingerprint,
            SafeU64,
            Version,
            CtxRef,
            Provenance,
            Grant,
            Effect,
            ValueKind,
            Need,
            Fill,
            Returns,
            ToolSpec,
            Approval,
            Caller,
            PressAction,
            PressedBy,
            Allowed,
            Refused,
            PlatformKind,
            Bounds,
            UiNode,
            UiTree,
            AddedNode,
            ChangedNode,
            TreeDiff,
            DeliveryMode,
            KeybindingScope,
            ScreenshotMode,
            SnapshotFormat,
            AssertSpec,
            Op,
            Envelope,
            HelloAuth,
            HelloInfo,
            Response,
            ChallengeOp,
            Challenge,
            SnapshotResult,
            EventLevel,
            AgentEvent,
            EventPoll,
            KeybindingInfo
        )
    };
}

/// The text of `packages/agent/src/generated.ts`.
pub fn typescript() -> String {
    macro_rules! write_decls {
        ($out:expr, $($ty:ty),* $(,)?) => {
            $(
                let cfg = Config::new();
                if let Some(docs) = <$ty as TS>::docs() {
                    $out.push_str(&docs);
                }
                $out.push_str("export ");
                $out.push_str(&<$ty as TS>::decl(&cfg));
                $out.push_str("\n\n");
            )*
        };
    }
    let mut out =
        String::from("// Generated from crates/agent by buwiz-agent-gen (ts-rs). Do not edit.\n\n");
    each_type!(write_decls!(out));
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}

/// The text of `packages/agent/schema/buwiz-agent.schema.json`.
pub fn schema() -> String {
    macro_rules! add_defs {
        ($generator:expr, $($ty:ty),* $(,)?) => {
            $( $generator.subschema_for::<$ty>(); )*
        };
    }
    let mut generator = SchemaSettings::draft2020_12().into_generator();
    each_type!(add_defs!(generator));
    let defs: Map<String, Value> = generator.take_definitions(true);
    let root = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Buwiz agent types",
        "description": "Generated from crates/agent by buwiz-agent-gen (schemars). Do not edit.",
        "$defs": defs,
    });
    let mut text = serde_json::to_string_pretty(&root).unwrap_or_default();
    text.push('\n');
    text
}
