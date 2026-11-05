// Context refs, the TypeScript half of crates/agent/src/ctx.rs. On the wire
// a ref is its text; parsing gives the two parts, and printing the parts
// gives the same text back.

import type { CtxRef } from "./generated";
import { Invalid } from "./ids";

const SCHEME = "ctx://";
const KIND = /^[a-z][a-z0-9_-]{0,31}$/;
const SEGMENT = /^[A-Za-z0-9._-]+$/;

export interface CtxParts {
  kind: string;
  id: string;
}

const isId = (id: string) => id.split("/").every((segment) => segment !== "." && segment !== ".." && SEGMENT.test(segment));

/** The kind and id of a ref, or `Invalid`. */
export const parseCtxRef = (value: unknown): CtxParts => {
  if (typeof value !== "string" || value.length > 256 || !value.startsWith(SCHEME)) throw new Invalid("CtxRef");
  const rest = value.slice(SCHEME.length);
  const slash = rest.indexOf("/");
  const kind = rest.slice(0, Math.max(slash, 0));
  const id = rest.slice(slash + 1);
  if (slash < 0 || !KIND.test(kind) || !isId(id)) throw new Invalid("CtxRef");
  return { kind, id };
};

/** The text of a ref from its parts, or `Invalid`. */
export const printCtxRef = ({ kind, id }: CtxParts): CtxRef => {
  if (!KIND.test(kind)) throw new Invalid("CtxRef");
  const text = `${SCHEME}${kind}/${id}`;
  parseCtxRef(text);
  return text;
};
