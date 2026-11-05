//! What the vectors cannot show: the typed API and what stays out of logs.

use buwiz_agent::*;

fn target(name: &str) -> Target {
    Target::parse(name).unwrap()
}

#[test]
fn a_missing_action_or_caller_is_refused() {
    let caller = Caller::Person {
        who: ActorId::parse("person.owner").unwrap(),
    };
    let action = PressAction {
        target: target("books"),
        id: AgentId::parse("ledger.post").unwrap(),
        effect: Effect::Write,
        once: None,
    };
    assert_eq!(
        authorize_press(None, Some(&caller)),
        Err(Refused::UnknownAction)
    );
    assert_eq!(
        authorize_press(Some(&action), None),
        Err(Refused::UnknownCaller)
    );
    assert_eq!(authorize_press(None, None), Err(Refused::UnknownAction));
}

#[test]
fn every_refusal_prints_its_wire_name() {
    for refused in [
        Refused::UnknownAction,
        Refused::UnknownCaller,
        Refused::RoutineNeedsPerson,
        Refused::ApprovalMissing,
        Refused::ApprovalWrongTarget,
        Refused::ApprovalWrongAction,
        Refused::NoOpenPress,
        Refused::ApprovalWrongPress,
    ] {
        assert_eq!(serde_json::to_value(refused).unwrap(), refused.code());
    }
}

#[test]
fn an_envelope_never_prints_its_auth() {
    let mut envelope = Envelope::new("1", target("bot"), Op::Hello {});
    envelope.auth = Some("fake-mac-value".into());
    assert!(!format!("{envelope:?}").contains("fake-mac-value"));
}

#[test]
fn a_refused_value_is_not_repeated_in_the_error() {
    let error = Target::parse("000-000-000").unwrap_err();
    assert_eq!(error.to_string(), "not a valid Target");
    let error = serde_json::from_str::<ActorId>("\"someone@example.invalid\"").unwrap_err();
    assert!(!error.to_string().contains("someone"));
}

#[test]
fn a_response_says_who_answered() {
    let response = Response::err("7", target("books"), "not found");
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["target"], "books");
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"], "not found");
}
