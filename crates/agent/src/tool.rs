//! Operations, from the Tool Mapping Protocol: before anything runs, a tool
//! has said what it does to the world, what it needs, what it fills in from
//! context, and what it returns. Anything not stated is not mapped.

use std::collections::BTreeSet;
use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::{AgentId, Target};

/// What a tool or action does to the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Looks, changes nothing.
    Read,
    /// Makes something a person can still review and throw away.
    Draft,
    /// Changes the record: post, void, close, reopen.
    Write,
    /// Sends something outside, such as a return to the BIR.
    File,
}

impl Effect {
    /// Whether this effect is the person's to do. The match has no catch-all,
    /// so a new effect cannot compile without an answer here.
    pub fn needs_person(self) -> bool {
        match self {
            Self::Read | Self::Draft => false,
            Self::Write | Self::File => true,
        }
    }
}

/// The shape of an input or a result. Amounts are integers (centavos), so
/// there is no decimal kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueKind {
    Text,
    Integer,
    Boolean,
    /// A `ctx://` ref.
    Ctx,
    Object,
    List,
}

/// One input a tool needs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Need {
    pub name: AgentId,
    pub kind: ValueKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
}

/// One input filled in from shared context rather than by the caller. It
/// names where the value comes from (such as `period`), never the value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Fill {
    /// The need this fills.
    pub need: AgentId,
    /// The context key that supplies it.
    pub from: AgentId,
}

/// What a tool gives back.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Returns {
    pub kind: ValueKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
}

/// One operation. All of `effect`, `needs`, `fills_in` and `returns` are
/// required on the wire: an empty list is a statement, a missing one is not.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct ToolSpec {
    pub target: Target,
    pub name: AgentId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    pub effect: Effect,
    pub needs: Vec<Need>,
    pub fills_in: Vec<Fill>,
    pub returns: Returns,
}

/// A tool whose parts disagree with each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolError {
    /// Two `needs` share a name.
    DuplicateNeed,
    /// Two `fills_in` fill the same need.
    DuplicateFill,
    /// A `fills_in` names something the tool does not need.
    FillWithoutNeed,
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::DuplicateNeed => "two needs share a name",
            Self::DuplicateFill => "two fills_in fill the same need",
            Self::FillWithoutNeed => "a fills_in names no need",
        })
    }
}

impl std::error::Error for ToolError {}

impl ToolSpec {
    /// The checks a type cannot state. Call this on any tool read from the
    /// wire, before trusting it.
    pub fn check(&self) -> Result<(), ToolError> {
        let mut needs = BTreeSet::new();
        for need in &self.needs {
            if !needs.insert(&need.name) {
                return Err(ToolError::DuplicateNeed);
            }
        }
        let mut filled = BTreeSet::new();
        for fill in &self.fills_in {
            if !needs.contains(&fill.need) {
                return Err(ToolError::FillWithoutNeed);
            }
            if !filled.insert(&fill.need) {
                return Err(ToolError::DuplicateFill);
            }
        }
        Ok(())
    }
}
