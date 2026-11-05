// Parsers for the wire shapes, the TypeScript half of serde in the Rust
// crate. Each takes JSON as parsed and returns the value as the Rust half
// would print it, or throws `Invalid`. Like the Rust half, they refuse
// unknown fields, drop nulls from optional fields, and leave out defaults.

import { parseCtxRef } from "./ctx";
import type {
  AddedNode,
  Allowed,
  Approval,
  Caller,
  Challenge,
  CtxRef,
  Envelope,
  EventPoll,
  Grant,
  HelloInfo,
  KeybindingInfo,
  Op,
  PressAction,
  PressedBy,
  Provenance,
  Response,
  SnapshotResult,
  ToolSpec,
  TreeDiff,
  UiNode,
  UiTree,
} from "./generated";
import { Invalid, PROTOCOL_VERSION, parseActorId, parseAgentId, parseContentHash, parseFingerprint, parseOnceId, parseSafeU64, parseTarget } from "./ids";

type Parser<T = unknown> = (value: unknown) => T;

interface Field {
  parse: Parser;
  /** Left out of the output when it holds this value's default. */
  isDefault?: (value: unknown) => boolean;
  required?: boolean;
  /** Whether JSON null counts as absent, as it does for a Rust `Option`. */
  nullable?: boolean;
}
type Fields = Record<string, Field>;

const required = (parse: Parser): Field => ({ parse, required: true });
const optional = (parse: Parser): Field => ({ parse, nullable: true });
const orDefault = (parse: Parser, isDefault: (value: unknown) => boolean): Field => ({ parse, isDefault });
const orValue = (parse: Parser, value: unknown): Field => orDefault(parse, (seen) => seen === value);
const orEmpty = (parse: Parser): Field => orDefault(parse, (seen) => (seen as unknown[]).length === 0);

const text: Parser<string> = (value) => {
  if (typeof value === "string") return value;
  throw new Invalid("string");
};

const bool: Parser<boolean> = (value) => {
  if (typeof value === "boolean") return value;
  throw new Invalid("boolean");
};

const integer =
  (min: number, max: number): Parser<number> =>
  (value) => {
    if (typeof value === "number" && Number.isInteger(value) && value >= min && value <= max) return value;
    throw new Invalid("integer");
  };

const u32 = integer(0, 2 ** 32 - 1);
const i32 = integer(-(2 ** 31), 2 ** 31 - 1);
const anything: Parser = (value) => value;

const version: Parser<number> = (value) => {
  if (value === PROTOCOL_VERSION) return value;
  throw new Invalid("Version");
};

const ctxRef: Parser<CtxRef> = (value) => {
  parseCtxRef(value);
  return value as CtxRef;
};

const oneOf =
  (what: string, ...values: string[]): Parser<string> =>
  (value) => {
    if (typeof value === "string" && values.includes(value)) return value;
    throw new Invalid(what);
  };

const list =
  <T>(item: Parser<T>): Parser<T[]> =>
  (value) => {
    if (Array.isArray(value)) return value.map((entry) => item(entry));
    throw new Invalid("list");
  };

const isObject = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);

const mapOf =
  (key: Parser<string>, item: Parser): Parser<Record<string, unknown>> =>
  (value) => {
    if (!isObject(value)) throw new Invalid("map");
    return Object.fromEntries(Object.entries(value).map(([name, entry]) => [key(name), item(entry)]));
  };

const object =
  <T>(what: string, fields: Fields): Parser<T> =>
  (value) => {
    if (!isObject(value)) throw new Invalid(what);
    for (const name of Object.keys(value)) {
      if (!Object.hasOwn(fields, name)) throw new Invalid(what);
    }
    const out: Record<string, unknown> = {};
    for (const [name, field] of Object.entries(fields)) {
      const seen = Object.hasOwn(value, name) ? value[name] : undefined;
      if (seen === undefined || (seen === null && field.nullable)) {
        if (field.required) throw new Invalid(what);
        continue;
      }
      const parsed = field.parse(seen);
      if (!field.isDefault?.(parsed)) out[name] = parsed;
    }
    return out as T;
  };

/** A union told apart by `tag`. */
const tagged =
  <T>(what: string, tag: string, variants: Record<string, Fields>): Parser<T> =>
  (value) => {
    if (!isObject(value)) throw new Invalid(what);
    const name = value[tag];
    if (typeof name !== "string" || !Object.hasOwn(variants, name)) throw new Invalid(what);
    return object<T>(what, { [tag]: required(anything), ...variants[name] })(value);
  };

const target = required(parseTarget);
const effect = oneOf("Effect", "read", "draft", "write", "file");
const valueKind = oneOf("ValueKind", "text", "integer", "boolean", "ctx", "object", "list");
const platform = oneOf("PlatformKind", "desktop", "headless", "web", "mobile");
const delivery = orValue(oneOf("DeliveryMode", "semantic", "virtual"), "semantic");
const keybindingScope = oneOf("KeybindingScope", "focused", "global");

export const parseProvenance = object<Provenance>("Provenance", {
  made_by: required(parseActorId),
  from: required(list(ctxRef)),
  content_hash: required(parseContentHash),
});

export const parseGrant = object<Grant>("Grant", { ctx: required(ctxRef), readers: required(list(parseTarget)) });

const need = object("Need", { name: required(parseAgentId), kind: required(valueKind), about: optional(text) });
const fill = object("Fill", { need: required(parseAgentId), from: required(parseAgentId) });
const returns = object("Returns", { kind: required(valueKind), about: optional(text) });

const toolShape = object<ToolSpec>("ToolSpec", {
  target,
  name: required(parseAgentId),
  about: optional(text),
  effect: required(effect),
  needs: required(list(need)),
  fills_in: required(list(fill)),
  returns: required(returns),
});

/** A tool, with the checks a type cannot state. */
export const parseToolSpec: Parser<ToolSpec> = (value) => {
  const tool = toolShape(value);
  const needs = new Set(tool.needs.map((entry) => entry.name));
  const filled = new Set(tool.fills_in.map((entry) => entry.need));
  if (needs.size !== tool.needs.length || filled.size !== tool.fills_in.length) throw new Invalid("ToolSpec");
  if (tool.fills_in.some((entry) => !needs.has(entry.need))) throw new Invalid("ToolSpec");
  return tool;
};

export const parseApproval = object<Approval>("Approval", {
  target,
  action: required(parseAgentId),
  once: required(parseOnceId),
  approved_by: required(parseActorId),
  args_hash: required(parseContentHash),
  shown_hash: required(parseContentHash),
  at: required(parseSafeU64),
});

export const parseCaller = tagged<Caller>("Caller", "kind", {
  person: { who: required(parseActorId) },
  agent: { who: required(parseActorId), approval: optional(parseApproval) },
  routine: { who: required(parseActorId) },
});

export const parsePressAction = object<PressAction>("PressAction", {
  target,
  id: required(parseAgentId),
  effect: required(effect),
  once: optional(parseOnceId),
  args_hash: optional(parseContentHash),
  shown_hash: optional(parseContentHash),
});

export const parsePressedBy = tagged<PressedBy>("PressedBy", "by", {
  person: { who: required(parseActorId) },
  ai_approved: {
    who: required(parseActorId),
    approved_by: required(parseActorId),
    once: required(parseOnceId),
    args_hash: required(parseContentHash),
    shown_hash: required(parseContentHash),
    at: required(parseSafeU64),
  },
  ai: { who: required(parseActorId) },
  routine: { who: required(parseActorId) },
});

export const parseAllowed = object<Allowed>("Allowed", {
  target,
  action: required(parseAgentId),
  effect: required(effect),
  pressed: required(parsePressedBy),
});

const bounds = object("Bounds", { x: required(i32), y: required(i32), w: required(i32), h: required(i32) });

export const parseUiNode: Parser<UiNode> = object<UiNode>("UiNode", {
  id: required(parseAgentId),
  role: required(text),
  name: required(text),
  value: optional(text),
  checked: optional(bool),
  enabled: required(bool),
  focused: required(bool),
  visible: orValue(bool, true),
  bounds: required(bounds),
  states: orEmpty(list(text)),
  children: orEmpty(list((value) => parseUiNode(value))),
});

export const parseUiTree = object<UiTree>("UiTree", {
  target,
  platform: required(platform),
  ready: required(bool),
  nodes: required(list(parseUiNode)),
  rev: optional(parseSafeU64),
});

const addedNode = object<AddedNode>("AddedNode", { node: required(parseUiNode), parent: optional(parseAgentId) });
const changedNode = object("ChangedNode", { id: required(parseAgentId), fields: required(mapOf(text, anything)) });

export const parseTreeDiff = object<TreeDiff>("TreeDiff", {
  added: required(list(addedNode)),
  removed: required(list(parseAgentId)),
  changed: required(list(changedNode)),
});

const ASSERT_FIELDS: Fields = {
  node: required(parseAgentId),
  name: optional(text),
  value: optional(text),
  role: optional(text),
  checked: optional(bool),
  exists: optional(bool),
  visible: optional(bool),
  in_viewport: optional(bool),
};

const INPUT_FIELDS: Fields = { node: required(parseAgentId), expect_fingerprint: optional(parseFingerprint) };

const OPS: Record<string, Fields> = {
  hello: {},
  snapshot: { since: optional(parseSafeU64), format: optional(oneOf("SnapshotFormat", "json", "compact")), fingerprints: optional(bool) },
  revision: {},
  events: { since: optional(parseSafeU64), limit: optional(u32) },
  click: { ...INPUT_FIELDS, delivery },
  type: { ...INPUT_FIELDS, text: required(text), delivery },
  set_value: { ...INPUT_FIELDS, value: required(text) },
  key: { ...INPUT_FIELDS, key: required(text), delivery },
  keybinding: {
    binding: required(text),
    chord: optional(text),
    scope: required(keybindingScope),
    confirm: orValue(bool, false),
    activate: orValue(bool, false),
  },
  keybindings: {},
  assert: ASSERT_FIELDS,
  invoke: { name: required(parseAgentId), args: optional(anything) },
  wait: { timeout_ms: optional(parseSafeU64) },
  wait_until: { timeout_ms: required(parseSafeU64), until: required(object("AssertSpec", ASSERT_FIELDS)) },
  shutdown: {},
  screenshot: {
    path: optional(text),
    mode: orValue(oneOf("ScreenshotMode", "viewport", "scrolled"), "viewport"),
    node: optional(parseAgentId),
    max_height_px: optional(u32),
  },
};

const ENVELOPE_FIELDS: Fields = { v: required(version), id: required(text), target, caller: optional(parseCaller), bot: optional(text), auth: optional(text) };

export const parseOp = tagged<Op>("Op", "op", OPS);

export const parseEnvelope = tagged<Envelope>(
  "Envelope",
  "op",
  Object.fromEntries(Object.entries(OPS).map(([name, fields]) => [name, { ...ENVELOPE_FIELDS, ...fields }])),
);

export const parseHelloInfo = object<HelloInfo>("HelloInfo", {
  protocol: required(version),
  targets: required(list(parseTarget)),
  platform: required(platform),
  os: required(text),
  ready: required(bool),
  deliveries: orEmpty(list(oneOf("DeliveryMode", "semantic", "virtual"))),
  auth: required(oneOf("HelloAuth", "none", "required")),
  features: orEmpty(list(text)),
});

export const parseResponse = object<Response>("Response", {
  v: required(version),
  id: required(text),
  target,
  ok: required(bool),
  error: optional(text),
  hello: optional(parseHelloInfo),
  tree: optional(parseUiTree),
  result: optional(anything),
  fingerprints: optional(mapOf(parseAgentId, parseFingerprint)),
  rev: optional(parseSafeU64),
});

export const parseChallenge = object<Challenge>("Challenge", {
  v: required(version),
  target,
  op: required(oneOf("ChallengeOp", "challenge")),
  nonce: required(text),
});

export const parseSnapshotResult = object<SnapshotResult>("SnapshotResult", {
  unchanged: optional(bool),
  diff: optional(parseTreeDiff),
  full: optional(bool),
  compact: optional(text),
});

const agentEvent = object("AgentEvent", {
  seq: required(parseSafeU64),
  level: required(oneOf("EventLevel", "trace", "debug", "info", "warn", "error")),
  source: required(text),
  message: required(text),
});

export const parseEventPoll = object<EventPoll>("EventPoll", {
  events: required(list(agentEvent)),
  latest: required(parseSafeU64),
  lagged: required(bool),
});

export const parseKeybindingInfo = object<KeybindingInfo>("KeybindingInfo", {
  id: required(text),
  chord: required(text),
  scope: required(keybindingScope),
  dangerous: required(bool),
});
