//! The semantic UI tree an app publishes and the diffs between revisions.
//! Shapes from gpui-agent, with whole-pixel bounds and validated ids.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::ids::{AgentId, SafeU64, Target};

/// Well-known role strings. The wire keeps a string so apps can add their own.
pub mod role {
    pub const WINDOW: &str = "window";
    pub const PAGE: &str = "page";
    pub const NAVIGATION: &str = "navigation";
    pub const BUTTON: &str = "button";
    pub const TEXTBOX: &str = "textbox";
    pub const LIST: &str = "list";
    pub const LIST_ITEM: &str = "listitem";
    pub const SCROLL: &str = "scroll";
    pub const CHECKBOX: &str = "checkbox";
    pub const NOTE: &str = "note";
    pub const STATUS: &str = "status";
    pub const DIALOG: &str = "dialog";
}

/// Which runtime is answering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlatformKind {
    Desktop,
    Headless,
    Web,
    Mobile,
}

/// Bounds in whole logical pixels. gpui-agent uses f32; this crate has no
/// floats, so a host rounds. Hosts that cannot measure send zeros.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

fn default_visible() -> bool {
    true
}

fn is_visible(value: &bool) -> bool {
    *value
}

/// One node. `visible` is left out when true, and `states` and `children`
/// when empty.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct UiNode {
    pub id: AgentId,
    pub role: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    pub enabled: bool,
    pub focused: bool,
    #[serde(default = "default_visible", skip_serializing_if = "is_visible")]
    #[ts(as = "Option<bool>", optional)]
    pub visible: bool,
    pub bounds: Bounds,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub states: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<UiNode>>", optional)]
    pub children: Vec<UiNode>,
}

/// One app's tree at one revision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct UiTree {
    pub target: Target,
    pub platform: PlatformKind,
    pub ready: bool,
    pub nodes: Vec<UiNode>,
    /// Absent when the host tracks no revision: rebuild, do not cache.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<SafeU64>,
}

/// A node in the new revision that the old one lacks. `parent` is the id it
/// attaches under; absent means top level. gpui-agent flattens `node`, but
/// serde cannot refuse a flattened struct's unknown fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct AddedNode {
    pub node: UiNode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<AgentId>,
}

/// A node in both revisions with at least one differing field. `fields`
/// holds only the new values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangedNode {
    pub id: AgentId,
    #[ts(type = "Record<string, unknown>")]
    pub fields: BTreeMap<String, Value>,
}

/// What a client at one revision needs to reach another.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TreeDiff {
    pub added: Vec<AddedNode>,
    pub removed: Vec<AgentId>,
    pub changed: Vec<ChangedNode>,
}
