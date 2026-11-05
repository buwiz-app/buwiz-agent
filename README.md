# Buwiz Agent

The types Buwiz mini apps, the Bot and the broker share: which app a message is for, what a context handle looks
like, what a tool does, and who pressed a button. Rust is the source. The TypeScript types and the JSON Schema are
generated from it, and shared vectors prove the two halves agree.

This repo is types and pure functions only. It has no transport, hub or context store.

## What is here

| Path                              | What                                                                  |
| --------------------------------- | --------------------------------------------------------------------- |
| `crates/agent/`                   | The source, in Rust (crate `buwiz-agent`)                             |
| `crates/agent/src/bin/gen.rs`     | `buwiz-agent-gen`, which writes the generated files                   |
| `packages/agent/src/generated.ts` | Generated TypeScript types                                            |
| `packages/agent/schema/`          | Generated JSON Schema, one `$defs` entry per type                     |
| `packages/agent/src/`             | The hand-written TypeScript half: parsers and `authorizePress`        |
| `vectors/`                        | Cases the Rust parsers, the TypeScript parsers and the schema all run |
| `tools/check-all.sh`              | Every gate, both halves                                               |

## The types

**UI targets.** `Target` names an app (`bot`, `books`, `ebirforms`, `sheet`) with the same grammar as a plugin's
`short`. `AgentId` names a thing inside an app (`ledger.post`). `Envelope` is one request: `v`, `id`, `target`, an
optional `caller` and `auth`, and the `Op`, whose fields sit beside these on the wire. `Response`, `Challenge` and
`UiTree` carry `target` too. The ops, the tree, the diffs and the auth handshake come from gpui-agent, with these
changes:

- `target` means the app, so the node inside it is `node`, and an event's emitter is `source`.
- Fingerprints are 16 hex digits, not u64 numbers, which JavaScript would round.
- Revisions, sequence numbers and durations are `SafeU64`, at most 2^53 - 1.
- Bounds are whole pixels, because this repo has no floats.
- Unknown fields are refused. So `wait_until` holds its check in `until`, `AddedNode` holds its node in `node`, and
  ops with no fields are empty struct variants.
- `token` is gone: gpui-agent refuses it on the wire anyway, and `auth` is the only credential.

**Context refs.** `CtxRef` parses and prints `ctx://<kind>/<id>`. `kind` is the first segment and `id` is the rest:

```
ctx://file/sales.xlsx                  kind file, id sales.xlsx
ctx://books/batch/b7                   kind books, id batch/b7
ctx://books/proposal/2551Q-2026Q3      kind books, id proposal/2551Q-2026Q3
```

`Provenance` says who made a piece of context, from which refs, and its content hash. `Grant` says which apps may
read a ref.

**Operations.** `ToolSpec` is one operation from the Tool Mapping Protocol: its `effect` (`read`, `draft`, `write`,
`file`), what it `needs`, what it `fills_in` from context, and what it `returns`. All four are required. Run
`ToolSpec::check` on any tool read from the wire (`parseToolSpec` does this for you).

**Caller and approval.** `Caller` says person, agent or routine, and who. `authorize_press(action, caller)` is the
one decision. `write` and `file` effects need a person; `read` and `draft` do not.

| Caller  | `read`, `draft`    | `write`, `file`                                               |
| ------- | ------------------ | ------------------------------------------------------------- |
| person  | allowed, `person`  | allowed, `person`                                             |
| agent   | allowed, `ai`      | allowed as `ai_approved` with a matching approval, or refused |
| routine | allowed, `routine` | refused, `routine_needs_person`                               |

An `Approval` matches when a person gave it and its `target`, `action` and `once` all equal the press. Anything else
is refused: `unknown_action`, `unknown_caller`, `approval_missing`, `approval_not_person`, `approval_wrong_target`, `approval_wrong_action`,
`no_open_press`, `approval_wrong_press`. `Allowed.pressed` says whose press it was, to show and to log.

The function holds no state, so three things are the host's job:

- Build the `Caller` from the path the press arrived on. Never take `kind` or an `Approval` from model output.
- Build the `PressAction` from what the app declared.
- Make one `once` id per press you ask the person about, put it in `PressAction.once` only while that press is open,
  and retire it when the answer is `ai_approved`. That is what stops an approval from working twice.

## Generating

```
cargo run -p buwiz-agent --bin buwiz-agent-gen            # write the files
cargo run -p buwiz-agent --bin buwiz-agent-gen -- --check # fail if they are stale
```

`ts-rs` writes the TypeScript and `schemars` writes the JSON Schema, each from derives on the Rust types and each
reading the same serde attributes. Both are pinned exactly in `crates/agent/Cargo.toml`, because a new version can
change the generated text.

## Vectors

| File            | Cases                                                                  |
| --------------- | ---------------------------------------------------------------------- |
| `ids.json`      | Valid and invalid targets, ids, one-time ids, hashes and safe integers |
| `ctx.json`      | Context refs parsed and printed, and refused                           |
| `messages.json` | Every op and wire shape printed back, or refused                       |
| `press.json`    | `authorize_press`: what is allowed, as whom, and each refusal          |

`crates/agent/tests/vectors.rs` and `packages/agent/test/vectors.test.ts` run the same files. The TypeScript tests
also hold each case against the generated schema, except `press.json`. A case marked `schema_allows` is refused by
both parsers though JSON Schema cannot say so (for example, a `fills_in` naming a need the tool lacks).

## License

Apache-2.0. See `LICENSE` and `NOTICE`.
