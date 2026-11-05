//! The shared vectors (`vectors/`). `packages/agent/test/vectors.test.ts`
//! runs the same files and must reach the same answers:
//!
//! - `ids.json`: valid and invalid names;
//! - `ctx.json`: context refs, parsed and printed;
//! - `messages.json`: every wire shape, accepted and printed back, or refused;
//! - `press.json`: `authorize_press`, allowed or refused and why.

use buwiz_agent::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, from_value, json, to_value};

const IDS: &str = include_str!("../../../vectors/ids.json");
const CTX: &str = include_str!("../../../vectors/ctx.json");
const MESSAGES: &str = include_str!("../../../vectors/messages.json");
const PRESS: &str = include_str!("../../../vectors/press.json");

fn cases(text: &str) -> Vec<Value> {
    let all: Value = serde_json::from_str(text).unwrap();
    all["cases"].as_array().unwrap().clone()
}

fn accepts<T: DeserializeOwned>(input: &Value) -> bool {
    from_value::<T>(input.clone()).is_ok()
}

fn round<T: DeserializeOwned + Serialize>(input: &Value) -> Option<Value> {
    let parsed: T = from_value(input.clone()).ok()?;
    Some(to_value(parsed).unwrap())
}

fn tool(input: &Value) -> Option<Value> {
    let parsed: ToolSpec = from_value(input.clone()).ok()?;
    parsed.check().ok()?;
    Some(to_value(parsed).unwrap())
}

fn print(kind: &str, input: &Value) -> Option<Value> {
    match kind {
        "Envelope" => round::<Envelope>(input),
        "Op" => round::<Op>(input),
        "Response" => round::<Response>(input),
        "Challenge" => round::<Challenge>(input),
        "HelloInfo" => round::<HelloInfo>(input),
        "UiNode" => round::<UiNode>(input),
        "UiTree" => round::<UiTree>(input),
        "TreeDiff" => round::<TreeDiff>(input),
        "SnapshotResult" => round::<SnapshotResult>(input),
        "EventPoll" => round::<EventPoll>(input),
        "KeybindingInfo" => round::<KeybindingInfo>(input),
        "Provenance" => round::<Provenance>(input),
        "Grant" => round::<Grant>(input),
        "ToolSpec" => tool(input),
        "Approval" => round::<Approval>(input),
        "Caller" => round::<Caller>(input),
        "PressAction" => round::<PressAction>(input),
        "PressedBy" => round::<PressedBy>(input),
        "Allowed" => round::<Allowed>(input),
        other => panic!("no parser for {other}"),
    }
}

#[test]
fn every_name_is_accepted_or_refused() {
    let all = cases(IDS);
    assert!(all.len() >= 50);
    for v in all {
        let input = &v["input"];
        let ok = match v["type"].as_str().unwrap() {
            "Target" => accepts::<Target>(input),
            "AgentId" => accepts::<AgentId>(input),
            "ActorId" => accepts::<ActorId>(input),
            "OnceId" => accepts::<OnceId>(input),
            "ContentHash" => accepts::<ContentHash>(input),
            "Fingerprint" => accepts::<Fingerprint>(input),
            "SafeU64" => accepts::<SafeU64>(input),
            other => panic!("no parser for {other}"),
        };
        assert_eq!(ok, v["valid"] == true, "{v}");
    }
}

#[test]
fn every_ctx_ref_parses_and_prints_or_is_refused() {
    let all: Value = serde_json::from_str(CTX).unwrap();
    let valid = all["valid"].as_array().unwrap();
    let invalid = all["invalid"].as_array().unwrap();
    assert!(valid.len() >= 3 && invalid.len() >= 15);
    for v in valid {
        let text = v["input"].as_str().unwrap();
        let (kind, id) = (v["kind"].as_str().unwrap(), v["id"].as_str().unwrap());
        let parsed = CtxRef::parse(text).unwrap();
        assert_eq!((parsed.kind(), parsed.id()), (kind, id), "{text}");
        assert_eq!(parsed.to_string(), text);
        assert_eq!(CtxRef::new(kind, id).unwrap().as_str(), text);
        assert_eq!(to_value(&parsed).unwrap(), v["input"]);
    }
    for v in invalid {
        assert!(!accepts::<CtxRef>(v), "{v}");
    }
    for v in all["invalid_parts"].as_array().unwrap() {
        let made = CtxRef::new(v["kind"].as_str().unwrap(), v["id"].as_str().unwrap());
        assert_eq!(made, Err(Invalid("CtxRef")), "{v}");
    }
}

#[test]
fn every_message_prints_back_or_is_refused() {
    let all = cases(MESSAGES);
    assert!(all.len() >= 60);
    for v in all {
        let name = v["name"].as_str().unwrap();
        let printed = print(v["type"].as_str().unwrap(), &v["input"]);
        if v["valid"] == true {
            let want = v.get("output").unwrap_or(&v["input"]);
            assert_eq!(printed.as_ref(), Some(want), "{name}");
        } else {
            assert_eq!(printed, None, "{name}");
        }
    }
}

#[test]
fn every_press_is_decided_the_same_way() {
    let all = cases(PRESS);
    assert!(all.len() >= 30);
    for v in all {
        let name = v["name"].as_str().unwrap();
        let null = Value::Null;
        let action = v.get("action").unwrap_or(&null);
        let caller = v.get("caller").unwrap_or(&null);
        let got = match authorize_press_wire(action, caller) {
            Ok(allowed) => json!({ "allowed": allowed }),
            Err(refused) => json!({ "refused": refused.code() }),
        };
        assert_eq!(got, v["expect"], "{name}");
    }
}
