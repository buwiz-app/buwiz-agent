// Shared types for Buwiz mini apps. The types in generated.ts and the JSON
// Schema come from the Rust crate; the parsers and the press decision are
// the small hand-written half, held to the Rust one by vectors/.

export * from "./generated";
export { parseCtxRef, printCtxRef, type CtxParts } from "./ctx";
export { Invalid, PROTOCOL_VERSION, parseActorId, parseAgentId, parseContentHash, parseFingerprint, parseOnceId, parseSafeU64, parseTarget } from "./ids";
export { authorizePress, needsPerson, type PressResult } from "./press";
export * from "./wire";
