#!/usr/bin/env node
// 3.1.1 + 3.2.4 — pins the viewer's JS fold to the Rust engine via the
// committed golden fixtures, and checks the scrub path's speed budget on a
// synthetic 100k-event trace. Run: node viewer/test-fold.mjs
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(here, "memlens.html"), "utf8");

// Extract the pure fold between its markers and evaluate it.
const m = html.match(/MEMLENS_FOLD_BEGIN =+[\s\S]*?\*\/([\s\S]*?)\/\* =+ MEMLENS_FOLD_END/);
if (!m) throw new Error("fold markers not found in memlens.html");
const fold = new Function(`${m[1]}; return { parseTrace, replayTo, liveBytes };`)();

let failures = 0;
const check = (cond, msg) => { if (!cond) { failures++; console.error("FAIL:", msg); } };

// ---- Golden conformance (bit-for-bit against the Rust engine) ----
const fixDir = join(here, "..", "crates", "memlens-replay", "fixtures");
const trace = fold.parseTrace(readFileSync(join(fixDir, "basic.jsonl"), "utf8"));
const expected = JSON.parse(readFileSync(join(fixDir, "basic.expected.json"), "utf8"));

for (const [tStr, snap] of Object.entries(expected)) {
  const t = Number(tStr);
  check(fold.liveBytes(trace.events, t) === snap.live_bytes, `live_bytes(${t})`);
  const live = fold.replayTo(trace.events, t);
  const keys = [...live.keys()].map((a) => "0x" + a.toString(16)).sort();
  const expKeys = Object.keys(snap.live).sort();
  check(JSON.stringify(keys) === JSON.stringify(expKeys), `live set keys at t=${t}: ${keys} vs ${expKeys}`);
  for (const [addrHex, exp] of Object.entries(snap.live)) {
    const a = live.get(BigInt(addrHex));
    check(a !== undefined, `missing ${addrHex} at t=${t}`);
    if (!a) continue;
    check(a.size === exp.size, `size of ${addrHex} at t=${t}`);
    check(a.align === exp.align, `align of ${addrHex} at t=${t}`);
    check(a.bornSeq === exp.born_seq, `born_seq of ${addrHex} at t=${t}`);
    check(JSON.stringify(a.scopes) === JSON.stringify(exp.scopes), `scopes of ${addrHex} at t=${t}`);
    check("0x" + a.lineageRoot.toString(16) === exp.lineage_root, `lineage_root of ${addrHex} at t=${t}`);
  }
}
console.log(`goldens: ${Object.keys(expected).length} snapshots checked`);

// ---- R13 speed budget: 100k-event synthetic trace ----
// (Index-build + live-set query; canvas paint is verified by the human in
// the operations bolt — node has no canvas.)
const lines = [];
let seq = 0; const liveAddrs = [];
for (let i = 0; i < 100_000; i++) {
  seq++;
  if (liveAddrs.length && i % 3 === 2) {
    const addr = liveAddrs.pop();
    lines.push(`{"event_id":"p-${seq}","ts":"2026-07-05T00:00:00Z","session_id":"perf","spec_id":null,"actor":"memlens","event_type":"memlens.dealloc","schema_version":1,"payload":{"addr":"${addr}","size":64,"align":8,"seq":${seq}}}`);
  } else {
    const addr = "0x" + (0x10000 + i * 16).toString(16);
    liveAddrs.push(addr);
    lines.push(`{"event_id":"p-${seq}","ts":"2026-07-05T00:00:00Z","session_id":"perf","spec_id":null,"actor":"memlens","event_type":"memlens.alloc","schema_version":1,"payload":{"addr":"${addr}","size":64,"align":8,"seq":${seq}}}`);
  }
}
const perfTrace = fold.parseTrace(lines.join("\n"));
check(perfTrace.events.length === 100_000, "perf trace size");

// Mirror the viewer's scrub path: lifetimes built once, then filtered per t.
const lifetimes = [];
const open = new Map();
for (const e of perfTrace.events) {
  if (e.k === "A") { const lt = { bornSeq: e.seq, deathSeq: Infinity, size: e.size }; lifetimes.push(lt); open.set(e.addr, lt); }
  else if (e.k === "D") { const lt = open.get(e.addr); if (lt) lt.deathSeq = e.seq; open.delete(e.addr); }
}
const t0 = performance.now();
let scrubs = 0;
for (let t = 0; t <= 100_000; t += 5_000) { // 21 scrub positions
  lifetimes.filter((l) => l.bornSeq <= t && t < l.deathSeq).length;
  scrubs++;
}
const perStep = (performance.now() - t0) / scrubs;
console.log(`scrub: ${perStep.toFixed(2)} ms/step over ${lifetimes.length} lifetimes`);
check(perStep < 100, `R13: scrub step ${perStep.toFixed(1)}ms exceeds 100ms budget`);

if (failures) { console.error(`\n${failures} failure(s)`); process.exit(1); }
console.log("all fold checks green ✓");
