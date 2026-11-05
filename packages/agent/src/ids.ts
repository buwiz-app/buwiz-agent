// Validated names, the TypeScript half of crates/agent/src/ids.rs. Errors
// never repeat the refused value: it could be a TIN or a secret, and errors
// end up in logs.

import type { ActorId, AgentId, ContentHash, Fingerprint, OnceId, SafeU64, Target } from "./generated";

export const PROTOCOL_VERSION = 1;

/** A value refused by a parser. It names the type, never the value. */
export class Invalid extends Error {
  constructor(what: string) {
    super(`not a valid ${what}`);
    this.name = "Invalid";
  }
}

const textType =
  <T extends string>(what: string, pattern: RegExp, max: number) =>
  (value: unknown): T => {
    if (typeof value === "string" && value.length <= max && pattern.test(value)) return value as T;
    throw new Invalid(what);
  };

const DOTTED = /^[a-z][a-z0-9_-]*(\.[a-z][a-z0-9_-]*)*$/;

export const parseTarget = textType<Target>("Target", /^[a-z][a-z0-9]{1,15}$/, 16);
export const parseAgentId = textType<AgentId>("AgentId", DOTTED, 128);
export const parseActorId = textType<ActorId>("ActorId", DOTTED, 64);
export const parseOnceId = textType<OnceId>("OnceId", /^[a-z][a-z0-9-]{7,63}$/, 64);
export const parseContentHash = textType<ContentHash>("ContentHash", /^sha256:[0-9a-f]{64}$/, 71);
export const parseFingerprint = textType<Fingerprint>("Fingerprint", /^[0-9a-f]{16}$/, 16);

export const parseSafeU64 = (value: unknown): SafeU64 => {
  if (typeof value === "number" && Number.isSafeInteger(value) && value >= 0) return value;
  throw new Invalid("SafeU64");
};
