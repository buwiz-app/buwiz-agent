// Generated from crates/agent by buwiz-agent-gen (ts-rs). Do not edit.

/**
 * The app a message is for, such as `bot`, `books`, `ebirforms` or
 * `sheet`. Same grammar as a plugin's `short` in Mini App Protocol 1.0.
 */
export type Target = string;

/**
 * A stable id inside an app, such as `ledger.post`: dot-separated
 * labels of lowercase letters, digits, `_` and `-`, each starting with
 * a letter. UI nodes, actions, tools and their parameters use it.
 */
export type AgentId = string;

/**
 * Who did something, as a handle such as `person.owner`, `agent.broker`
 * or `routine.month-end`. It is a handle on purpose: `@`, spaces and
 * leading digits are refused, so an email or a TIN cannot go here.
 */
export type ActorId = string;

/**
 * The one-time id of a single press. The host makes a fresh one for
 * each press it asks the person about and retires it after use.
 */
export type OnceId = string;

/**
 * `sha256:` then 64 lowercase hex digits.
 */
export type ContentHash = string;

/**
 * A UI node's content fingerprint: 16 lowercase hex digits. gpui-agent
 * sends it as a u64 number, which JavaScript would round.
 */
export type Fingerprint = string;

/**
 * A count, revision or duration that both halves hold exactly: a u64 no
 * larger than 2^53 - 1.
 */
export type SafeU64 = number;

/**
 * The `v` of a message. Only the version this crate speaks parses.
 */
export type Version = 1;

/**
 * `ctx://<kind>/<id>`, such as `ctx://file/sales.xlsx` or
 * `ctx://books/proposal/2551Q-2026Q3`. `kind` is the first segment and
 * `id` is the rest, which may hold more `/`. There is one spelling per
 * ref: no escapes, no query, no fragment, and no `.` or `..` segments,
 * so printing a parsed ref gives the same text.
 */
export type CtxRef = string;

/**
 * Where a piece of context came from. Holds no content, only handles and a
 * hash of it.
 */
export type Provenance = { 
/**
 * Who made it.
 */
made_by: ActorId, 
/**
 * What it was made from. Empty for an original, such as an upload.
 */
from: Array<CtxRef>, 
/**
 * The hash of its content.
 */
content_hash: ContentHash, };

/**
 * Which apps may read a ref.
 */
export type Grant = { ctx: CtxRef, readers: Array<Target>, };

/**
 * What a tool or action does to the world.
 */
export type Effect = "read" | "draft" | "write" | "file";

/**
 * The shape of an input or a result. Amounts are integers (centavos), so
 * there is no decimal kind.
 */
export type ValueKind = "text" | "integer" | "boolean" | "ctx" | "object" | "list";

/**
 * One input a tool needs.
 */
export type Need = { name: AgentId, kind: ValueKind, about?: string, };

/**
 * One input filled in from shared context rather than by the caller. It
 * names where the value comes from (such as `period`), never the value.
 */
export type Fill = { 
/**
 * The need this fills.
 */
need: AgentId, 
/**
 * The context key that supplies it.
 */
from: AgentId, };

/**
 * What a tool gives back.
 */
export type Returns = { kind: ValueKind, about?: string, };

/**
 * One operation. All of `effect`, `needs`, `fills_in` and `returns` are
 * required on the wire: an empty list is a statement, a missing one is not.
 */
export type ToolSpec = { target: Target, name: AgentId, about?: string, effect: Effect, needs: Array<Need>, fills_in: Array<Fill>, returns: Returns, };

/**
 * The person's yes to exactly one press.
 */
export type Approval = { target: Target, action: AgentId, once: OnceId, approved_by: ActorId, 
/**
 * The arguments the person saw.
 */
args_hash: ContentHash, 
/**
 * The card the person saw.
 */
shown_hash: ContentHash, 
/**
 * When they said yes, in milliseconds since the Unix epoch.
 */
at: SafeU64, };

/**
 * Who is pressing.
 */
export type Caller = { "kind": "person", who: ActorId, } | { "kind": "agent", who: ActorId, approval?: Approval, } | { "kind": "routine", who: ActorId, };

/**
 * The press being decided, as the host knows it.
 */
export type PressAction = { target: Target, id: AgentId, effect: Effect, 
/**
 * The open press's one-time id. Absent when the host has no open press
 * for this action.
 */
once?: OnceId, 
/**
 * The arguments about to go out. Absent means no open press.
 */
args_hash?: ContentHash, 
/**
 * What the card shows now. Absent means no open press.
 */
shown_hash?: ContentHash, };

/**
 * Whose press this was, to show and to log.
 */
export type PressedBy = { "by": "person", who: ActorId, } | { "by": "ai_approved", who: ActorId, approved_by: ActorId, once: OnceId, args_hash: ContentHash, shown_hash: ContentHash, at: SafeU64, } | { "by": "ai", who: ActorId, } | { "by": "routine", who: ActorId, };

/**
 * A press that may go ahead.
 */
export type Allowed = { target: Target, action: AgentId, effect: Effect, pressed: PressedBy, };

/**
 * Why a press may not go ahead.
 */
export type Refused = "unknown_action" | "unknown_caller" | "routine_needs_person" | "approval_missing" | "approval_not_person" | "approval_wrong_target" | "approval_wrong_action" | "no_open_press" | "approval_wrong_press" | "approval_wrong_args" | "approval_wrong_shown" | "filing_not_for_agents";

/**
 * Which runtime is answering.
 */
export type PlatformKind = "desktop" | "headless" | "web" | "mobile";

/**
 * Bounds in whole logical pixels. gpui-agent uses f32; this crate has no
 * floats, so a host rounds. Hosts that cannot measure send zeros.
 */
export type Bounds = { x: number, y: number, w: number, h: number, };

/**
 * One node. `visible` is left out when true, and `states` and `children`
 * when empty.
 */
export type UiNode = { id: AgentId, role: string, name: string, value?: string, checked?: boolean, enabled: boolean, focused: boolean, visible?: boolean, bounds: Bounds, states?: Array<string>, children?: Array<UiNode>, };

/**
 * One app's tree at one revision.
 */
export type UiTree = { target: Target, platform: PlatformKind, ready: boolean, nodes: Array<UiNode>, 
/**
 * Absent when the host tracks no revision: rebuild, do not cache.
 */
rev?: SafeU64, };

/**
 * A node in the new revision that the old one lacks. `parent` is the id it
 * attaches under; absent means top level. gpui-agent flattens `node`, but
 * serde cannot refuse a flattened struct's unknown fields.
 */
export type AddedNode = { node: UiNode, parent?: AgentId, };

/**
 * A node in both revisions with at least one differing field. `fields`
 * holds only the new values.
 */
export type ChangedNode = { id: AgentId, fields: Record<string, unknown>, };

/**
 * What a client at one revision needs to reach another.
 */
export type TreeDiff = { added: Array<AddedNode>, removed: Array<AgentId>, changed: Array<ChangedNode>, };

/**
 * How click, type and key reach the app. `semantic` (the default) calls the
 * widget's handler by id; `virtual` synthesizes in-window events.
 */
export type DeliveryMode = "semantic" | "virtual";

/**
 * Which keymap a `keybinding` fires in.
 */
export type KeybindingScope = "focused" | "global";

/**
 * How `screenshot` captures: the painted window, or scrolled and stitched.
 */
export type ScreenshotMode = "viewport" | "scrolled";

/**
 * The `snapshot` answer's form: the JSON tree, or one line per visible node.
 */
export type SnapshotFormat = "json" | "compact";

/**
 * What `assert` and `wait_until` check about one node.
 */
export type AssertSpec = { node: AgentId, name?: string, value?: string, role?: string, checked?: boolean, exists?: boolean, visible?: boolean, in_viewport?: boolean, };

/**
 * What a message asks the target to do. Ops without fields are written as
 * empty struct variants because serde would ignore extra fields on unit ones.
 */
export type Op = { "op": "hello", } | { "op": "snapshot", since?: SafeU64, format?: SnapshotFormat, fingerprints?: boolean, } | { "op": "revision", } | { "op": "events", since?: SafeU64, limit?: number, } | { "op": "click", node: AgentId, delivery?: DeliveryMode, expect_fingerprint?: Fingerprint, } | { "op": "type", node: AgentId, text: string, delivery?: DeliveryMode, expect_fingerprint?: Fingerprint, } | { "op": "set_value", node: AgentId, value: string, expect_fingerprint?: Fingerprint, } | { "op": "key", node: AgentId, key: string, delivery?: DeliveryMode, expect_fingerprint?: Fingerprint, } | { "op": "keybinding", binding: string, chord?: string, scope: KeybindingScope, confirm?: boolean, activate?: boolean, } | { "op": "keybindings", } | { "op": "assert" } & AssertSpec | { "op": "invoke", name: AgentId, args?: unknown, } | { "op": "wait", timeout_ms?: SafeU64, } | { "op": "wait_until", timeout_ms: SafeU64, until: AssertSpec, } | { "op": "shutdown", } | { "op": "screenshot", path?: string, mode?: ScreenshotMode, node?: AgentId, max_height_px?: number, };

/**
 * One request: which app it is for, who is asking, and the op, whose fields
 * sit beside these on the wire.
 */
export type Envelope = { v: Version, id: string, target: Target, 
/**
 * Set by the host from how the message arrived. Absent means nobody
 * established a caller, and every press is refused.
 */
caller?: Caller, 
/**
 * The Bot whose turn this is, set by the host. It only narrows: a host
 * refuses a screen another Bot's chat opened. Absent in a plain chat.
 */
bot?: string, 
/**
 * HMAC-SHA256(token, session nonce) as lowercase hex. Never the raw token.
 */
auth?: string, } & ({ "op": "hello", } | { "op": "snapshot", since?: SafeU64, format?: SnapshotFormat, fingerprints?: boolean, } | { "op": "revision", } | { "op": "events", since?: SafeU64, limit?: number, } | { "op": "click", node: AgentId, delivery?: DeliveryMode, expect_fingerprint?: Fingerprint, } | { "op": "type", node: AgentId, text: string, delivery?: DeliveryMode, expect_fingerprint?: Fingerprint, } | { "op": "set_value", node: AgentId, value: string, expect_fingerprint?: Fingerprint, } | { "op": "key", node: AgentId, key: string, delivery?: DeliveryMode, expect_fingerprint?: Fingerprint, } | { "op": "keybinding", binding: string, chord?: string, scope: KeybindingScope, confirm?: boolean, activate?: boolean, } | { "op": "keybindings", } | { "op": "assert" } & AssertSpec | { "op": "invoke", name: AgentId, args?: unknown, } | { "op": "wait", timeout_ms?: SafeU64, } | { "op": "wait_until", timeout_ms: SafeU64, until: AssertSpec, } | { "op": "shutdown", } | { "op": "screenshot", path?: string, mode?: ScreenshotMode, node?: AgentId, max_height_px?: number, });

/**
 * Whether the host needs `auth` on every request.
 */
export type HelloAuth = "none" | "required";

/**
 * The `hello` answer. `targets` lists the apps this host routes to.
 */
export type HelloInfo = { protocol: Version, targets: Array<Target>, platform: PlatformKind, os: string, ready: boolean, deliveries?: Array<DeliveryMode>, auth: HelloAuth, features?: Array<string>, };

/**
 * One answer. `id` matches the request and `target` is the app that answered.
 */
export type Response = { v: Version, id: string, target: Target, ok: boolean, error?: string, hello?: HelloInfo, tree?: UiTree, result?: unknown, fingerprints?: { [key in AgentId]: Fingerprint }, rev?: SafeU64, };

/**
 * The tag of a challenge.
 */
export type ChallengeOp = "challenge";

/**
 * The first line a host with a token writes on a new connection. The client
 * answers by putting HMAC-SHA256(token, nonce) in `auth` on each request.
 */
export type Challenge = { v: Version, 
/**
 * The app whose host issued the challenge.
 */
target: Target, op: ChallengeOp, 
/**
 * 32 random bytes as lowercase hex.
 */
nonce: string, };

/**
 * `result` of a `snapshot` with `since` or `format: "compact"`. One of:
 * nothing changed; a diff; the full tree is in `tree` because the revision
 * is too old; or the compact text.
 */
export type SnapshotResult = { unchanged?: boolean, diff?: TreeDiff, full?: boolean, compact?: string, };

/**
 * Severity of one event.
 */
export type EventLevel = "trace" | "debug" | "info" | "warn" | "error";

/**
 * One host event. `seq` is the only ordering: events carry no clock.
 * `source` is the emitting component (gpui-agent's `target`).
 */
export type AgentEvent = { seq: SafeU64, level: EventLevel, source: string, message: string, };

/**
 * `result` of `events`. `lagged` means events were lost to the ring's limit.
 */
export type EventPoll = { events: Array<AgentEvent>, latest: SafeU64, lagged: boolean, };

/**
 * One entry of the `keybindings` result.
 */
export type KeybindingInfo = { id: string, chord: string, scope: KeybindingScope, dangerous: boolean, };
