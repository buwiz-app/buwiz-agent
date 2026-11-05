// Who pressed, and whether they may: the TypeScript half of
// crates/agent/src/press.rs, where the rules for the host are written down.

import type { Allowed, Caller, Effect, PressAction, PressedBy, Refused } from "./generated";
import { parseCaller, parsePressAction } from "./wire";

export type PressResult = { ok: true; allowed: Allowed } | { ok: false; refused: Refused };

/** Whether an effect is the person's to do. */
export const needsPerson = (effect: Effect): boolean => {
  switch (effect) {
    case "read":
    case "draft":
      return false;
    case "write":
    case "file":
      return true;
  }
};

const attempt = <T>(parse: (value: unknown) => T, value: unknown): T | undefined => {
  try {
    return parse(value);
  } catch {
    return undefined;
  }
};

const refused = (reason: Refused): PressResult => ({ ok: false, refused: reason });

/** Whether a handle names a person, such as `person.owner`. */
const isPerson = (who: string): boolean => who.startsWith("person.");

const decide = (action: PressAction, caller: Caller): PressedBy | Refused => {
  const { who } = caller;
  if (caller.kind === "person") return isPerson(who) ? { by: "person", who } : "unknown_caller";
  if (caller.kind === "agent" && action.effect === "file") return "filing_not_for_agents";
  if (!needsPerson(action.effect)) return caller.kind === "agent" ? { by: "ai", who } : { by: "routine", who };
  if (caller.kind === "routine") return "routine_needs_person";
  const { approval } = caller;
  if (approval === undefined) return "approval_missing";
  if (!isPerson(approval.approved_by)) return "approval_not_person";
  if (approval.target !== action.target) return "approval_wrong_target";
  if (approval.action !== action.id) return "approval_wrong_action";
  if (action.once === undefined) return "no_open_press";
  if (approval.once !== action.once) return "approval_wrong_press";
  if (action.args_hash === undefined) return "no_open_press";
  if (approval.args_hash !== action.args_hash) return "approval_wrong_args";
  if (action.shown_hash === undefined) return "no_open_press";
  if (approval.shown_hash !== action.shown_hash) return "approval_wrong_shown";
  const { args_hash, shown_hash, at, approved_by } = approval;
  return { by: "ai_approved", who, approved_by, once: action.once, args_hash, shown_hash, at };
};

/**
 * Decide one press. Both arguments are JSON as it arrived: whatever does not
 * parse counts as missing, and missing is a refusal.
 */
export const authorizePress = (actionValue: unknown, callerValue: unknown): PressResult => {
  const action = attempt(parsePressAction, actionValue);
  if (action === undefined) return refused("unknown_action");
  const caller = attempt(parseCaller, callerValue);
  if (caller === undefined) return refused("unknown_caller");
  const pressed = decide(action, caller);
  if (typeof pressed === "string") return refused(pressed);
  return { ok: true, allowed: { target: action.target, action: action.id, effect: action.effect, pressed } };
};
