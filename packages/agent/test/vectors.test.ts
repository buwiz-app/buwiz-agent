// The shared vectors (vectors/). crates/agent/tests/vectors.rs runs the same
// files and must reach the same answers. Each message is also held against
// the generated JSON Schema, so all three of Rust, TypeScript and the schema
// are checked by one set of cases.

import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import Ajv2020 from "ajv/dist/2020";
import * as agent from "../src";
import schema from "../schema/buwiz-agent.schema.json";

const VECTORS = join(import.meta.dir, "../../../vectors");
const read = (name: string) => JSON.parse(readFileSync(join(VECTORS, name), "utf8"));

const ajv = new Ajv2020({ strict: false, validateFormats: false });
ajv.addSchema(schema, "agent");
const schemaAccepts = (type: string, value: unknown): boolean => {
  const validate = ajv.getSchema(`agent#/$defs/${type}`);
  if (validate === undefined) throw new Error(`no schema for ${type}`);
  return validate(value) as boolean;
};

const parsers: Record<string, (value: unknown) => unknown> = {
  Target: agent.parseTarget,
  AgentId: agent.parseAgentId,
  ActorId: agent.parseActorId,
  OnceId: agent.parseOnceId,
  ContentHash: agent.parseContentHash,
  Fingerprint: agent.parseFingerprint,
  SafeU64: agent.parseSafeU64,
  Envelope: agent.parseEnvelope,
  Op: agent.parseOp,
  Response: agent.parseResponse,
  Challenge: agent.parseChallenge,
  HelloInfo: agent.parseHelloInfo,
  UiNode: agent.parseUiNode,
  UiTree: agent.parseUiTree,
  TreeDiff: agent.parseTreeDiff,
  SnapshotResult: agent.parseSnapshotResult,
  EventPoll: agent.parseEventPoll,
  KeybindingInfo: agent.parseKeybindingInfo,
  Provenance: agent.parseProvenance,
  Grant: agent.parseGrant,
  ToolSpec: agent.parseToolSpec,
  Approval: agent.parseApproval,
  Caller: agent.parseCaller,
  PressAction: agent.parsePressAction,
  PressedBy: agent.parsePressedBy,
  Allowed: agent.parseAllowed,
};

const parserFor = (type: string) => {
  const parse = parsers[type];
  if (parse === undefined) throw new Error(`no parser for ${type}`);
  return parse;
};

test("every name is accepted or refused", () => {
  const { cases } = read("ids.json") as { cases: { type: string; input: unknown; valid: boolean }[] };
  expect(cases.length).toBeGreaterThanOrEqual(50);
  for (const v of cases) {
    const label = `${v.type} ${JSON.stringify(v.input)}`;
    if (v.valid) {
      expect(parserFor(v.type)(v.input), label).toBe(v.input as never);
    } else {
      expect(() => parserFor(v.type)(v.input), label).toThrow(agent.Invalid);
    }
    expect(schemaAccepts(v.type, v.input), `schema: ${label}`).toBe(v.valid);
  }
});

test("every ctx ref parses and prints or is refused", () => {
  const { valid, invalid, invalid_parts } = read("ctx.json") as {
    valid: { input: string; kind: string; id: string }[];
    invalid: unknown[];
    invalid_parts: agent.CtxParts[];
  };
  expect(valid.length).toBeGreaterThanOrEqual(3);
  expect(invalid.length).toBeGreaterThanOrEqual(15);
  for (const v of valid) {
    expect(agent.parseCtxRef(v.input), v.input).toEqual({ kind: v.kind, id: v.id });
    expect(agent.printCtxRef({ kind: v.kind, id: v.id })).toBe(v.input);
    expect(schemaAccepts("CtxRef", v.input), `schema: ${v.input}`).toBe(true);
  }
  for (const v of invalid) {
    expect(() => agent.parseCtxRef(v), JSON.stringify(v)).toThrow(agent.Invalid);
    expect(schemaAccepts("CtxRef", v), `schema: ${JSON.stringify(v)}`).toBe(false);
  }
  for (const v of invalid_parts) {
    expect(() => agent.printCtxRef(v), JSON.stringify(v)).toThrow(agent.Invalid);
  }
});

test("every message prints back or is refused", () => {
  const { cases } = read("messages.json") as {
    cases: { name: string; type: string; input: unknown; output?: unknown; valid: boolean; schema_allows?: boolean }[];
  };
  expect(cases.length).toBeGreaterThanOrEqual(60);
  for (const v of cases) {
    if (v.valid) {
      expect(parserFor(v.type)(v.input), v.name).toEqual((v.output ?? v.input) as never);
    } else {
      expect(() => parserFor(v.type)(v.input), v.name).toThrow(agent.Invalid);
    }
    expect(schemaAccepts(v.type, v.input), `schema: ${v.name}`).toBe(v.valid || v.schema_allows === true);
  }
});

test("every press is decided the same way", () => {
  const { cases } = read("press.json") as {
    cases: { name: string; action?: unknown; caller?: unknown; expect: { allowed?: unknown; refused?: string } }[];
  };
  expect(cases.length).toBeGreaterThanOrEqual(30);
  for (const v of cases) {
    const got = agent.authorizePress(v.action, v.caller);
    expect(got.ok ? { allowed: got.allowed } : { refused: got.refused }, v.name).toEqual(v.expect as never);
  }
});

test("a refused value is not repeated in the error", () => {
  expect(() => agent.parseTarget("000-000-000")).toThrow("not a valid Target");
  expect(() => agent.parseActorId("someone@example.invalid")).toThrow("not a valid ActorId");
});
