//! Who pressed, and whether they may (D1). The AI may press a person-only
//! action only after the person approves that one press, and never a filing
//! (O4): that stays a person's own act, approved or not.
//!
//! [`authorize_press`] decides and holds no state, so its answer is only as
//! good as what the host gives it:
//!
//! - The host builds the [`Caller`] from the path a press arrived on. It
//!   never takes `kind` or an [`Approval`] from model output.
//! - The host builds the [`PressAction`] from what the app declared, not
//!   from what the caller says the action is.
//! - The host makes one [`OnceId`] per press it asks the person about, and
//!   puts it in [`PressAction::once`] only while that press is open. When
//!   the answer is [`PressedBy::AiApproved`], the host retires the id.
//!   That is what stops the same approval from working twice: the next
//!   press has a different id, or none.
//! - The host hashes the arguments it is about to send, and what the card
//!   shows now, into [`PressAction::args_hash`] and
//!   [`PressAction::shown_hash`]. An [`Approval`] carries the two hashes the
//!   person's yes was given to, so any other values are refused. The args
//!   hash is `sha256:` over the arguments' JCS (RFC 8785) bytes.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::ids::{ActorId, AgentId, ContentHash, OnceId, SafeU64, Target};
use crate::tool::Effect;

/// The person's yes to exactly one press.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    pub target: Target,
    pub action: AgentId,
    pub once: OnceId,
    pub approved_by: ActorId,
    /// The arguments the person saw.
    pub args_hash: ContentHash,
    /// The card the person saw.
    pub shown_hash: ContentHash,
    /// When they said yes, in milliseconds since the Unix epoch.
    pub at: SafeU64,
}

/// Who is pressing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[ts(optional_fields)]
pub enum Caller {
    /// A person, at the app.
    Person { who: ActorId },
    /// The AI. `approval` is the person's yes, when there is one.
    Agent {
        who: ActorId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        approval: Option<Approval>,
    },
    /// A scheduled routine. Nobody is there to approve anything.
    Routine { who: ActorId },
}

/// The press being decided, as the host knows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct PressAction {
    pub target: Target,
    pub id: AgentId,
    pub effect: Effect,
    /// The open press's one-time id. Absent when the host has no open press
    /// for this action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub once: Option<OnceId>,
    /// The arguments about to go out. Absent means no open press.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args_hash: Option<ContentHash>,
    /// What the card shows now. Absent means no open press.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shown_hash: Option<ContentHash>,
}

/// Whose press this was, to show and to log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(tag = "by", rename_all = "snake_case", deny_unknown_fields)]
pub enum PressedBy {
    /// The person pressed it.
    Person { who: ActorId },
    /// The AI pressed it after the person approved this press. The host
    /// retires `once` now.
    AiApproved {
        who: ActorId,
        approved_by: ActorId,
        once: OnceId,
        args_hash: ContentHash,
        shown_hash: ContentHash,
        at: SafeU64,
    },
    /// The AI pressed something that needs no person.
    Ai { who: ActorId },
    /// A routine pressed something that needs no person.
    Routine { who: ActorId },
}

/// A press that may go ahead.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Allowed {
    pub target: Target,
    pub action: AgentId,
    pub effect: Effect,
    pub pressed: PressedBy,
}

/// Why a press may not go ahead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Refused {
    /// The action is missing, malformed, or has an effect nobody knows.
    UnknownAction,
    /// The caller is missing, malformed, or of a kind nobody knows.
    UnknownCaller,
    /// A routine reached for an action that needs a person.
    RoutineNeedsPerson,
    /// The AI has no approval for an action that needs a person.
    ApprovalMissing,
    /// The approval names no person, so the AI can't approve its own press.
    ApprovalNotPerson,
    /// The approval is for another app.
    ApprovalWrongTarget,
    /// The approval is for another action.
    ApprovalWrongAction,
    /// The host has no open press for this action, so no approval can match.
    NoOpenPress,
    /// The approval is for another press of this action, such as a used one.
    ApprovalWrongPress,
    /// The approval is for other arguments.
    ApprovalWrongArgs,
    /// The approval is for another card than the one showing now.
    ApprovalWrongShown,
    /// The AI reached for a filing: no approval covers one.
    FilingNotForAgents,
}

impl Refused {
    /// The wire name, which the vectors compare.
    pub fn code(self) -> &'static str {
        match self {
            Self::UnknownAction => "unknown_action",
            Self::UnknownCaller => "unknown_caller",
            Self::RoutineNeedsPerson => "routine_needs_person",
            Self::ApprovalMissing => "approval_missing",
            Self::ApprovalNotPerson => "approval_not_person",
            Self::ApprovalWrongTarget => "approval_wrong_target",
            Self::ApprovalWrongAction => "approval_wrong_action",
            Self::NoOpenPress => "no_open_press",
            Self::ApprovalWrongPress => "approval_wrong_press",
            Self::ApprovalWrongArgs => "approval_wrong_args",
            Self::ApprovalWrongShown => "approval_wrong_shown",
            Self::FilingNotForAgents => "filing_not_for_agents",
        }
    }
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for Refused {}

/// Decide one press. `None` means the action or caller could not be
/// established, and that is a refusal, as is anything else not shown to be
/// allowed.
pub fn authorize_press(
    action: Option<&PressAction>,
    caller: Option<&Caller>,
) -> Result<Allowed, Refused> {
    let action = action.ok_or(Refused::UnknownAction)?;
    let caller = caller.ok_or(Refused::UnknownCaller)?;
    let pressed = match caller {
        Caller::Person { who } if !is_person(who) => return Err(Refused::UnknownCaller),
        Caller::Person { who } => PressedBy::Person { who: who.clone() },
        Caller::Agent { .. } if action.effect == Effect::File => {
            return Err(Refused::FilingNotForAgents);
        }
        Caller::Agent { who, .. } if !action.effect.needs_person() => {
            PressedBy::Ai { who: who.clone() }
        }
        Caller::Routine { who } if !action.effect.needs_person() => {
            PressedBy::Routine { who: who.clone() }
        }
        Caller::Routine { .. } => return Err(Refused::RoutineNeedsPerson),
        Caller::Agent { who, approval } => {
            let approval = approval.as_ref().ok_or(Refused::ApprovalMissing)?;
            if !is_person(&approval.approved_by) {
                return Err(Refused::ApprovalNotPerson);
            }
            if approval.target != action.target {
                return Err(Refused::ApprovalWrongTarget);
            }
            if approval.action != action.id {
                return Err(Refused::ApprovalWrongAction);
            }
            let once = action.once.as_ref().ok_or(Refused::NoOpenPress)?;
            if approval.once != *once {
                return Err(Refused::ApprovalWrongPress);
            }
            let args_hash = action.args_hash.as_ref().ok_or(Refused::NoOpenPress)?;
            if approval.args_hash != *args_hash {
                return Err(Refused::ApprovalWrongArgs);
            }
            let shown_hash = action.shown_hash.as_ref().ok_or(Refused::NoOpenPress)?;
            if approval.shown_hash != *shown_hash {
                return Err(Refused::ApprovalWrongShown);
            }
            PressedBy::AiApproved {
                who: who.clone(),
                approved_by: approval.approved_by.clone(),
                once: once.clone(),
                args_hash: args_hash.clone(),
                shown_hash: shown_hash.clone(),
                at: approval.at,
            }
        }
    };
    Ok(Allowed {
        target: action.target.clone(),
        action: action.id.clone(),
        effect: action.effect,
        pressed,
    })
}

/// Whether a handle names a person, such as `person.owner`.
fn is_person(who: &ActorId) -> bool {
    who.as_str().starts_with("person.")
}

/// [`authorize_press`] on JSON as it arrived: whatever does not parse counts
/// as missing, so it is refused.
pub fn authorize_press_wire(action: &Value, caller: &Value) -> Result<Allowed, Refused> {
    let action = serde_json::from_value::<PressAction>(action.clone()).ok();
    let caller = serde_json::from_value::<Caller>(caller.clone()).ok();
    authorize_press(action.as_ref(), caller.as_ref())
}
