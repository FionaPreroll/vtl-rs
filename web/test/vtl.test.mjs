// The WebAssembly module through web/www/vtl.mjs, as the page uses it.
// Needs the module built first:
//   cargo build -p vtl-web --release --target wasm32-unknown-unknown
// VTL_WASM names another file.

import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { Status, Vtl, isComment } from '../www/vtl.mjs';
import { examples } from '../www/examples.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const wasm = process.env.VTL_WASM ?? `${root}target/wasm32-unknown-unknown/release/vtl_web.wasm`;
const vtl = await Vtl.load(readFileSync(wasm));

/** Runs until the machine waits; returns what it printed. */
function settle(status) {
  while (status === Status.RUNNING) status = vtl.run(10_000);
  return status;
}

function session(lines) {
  for (const line of lines) settle(vtl.enter(line));
  return vtl.output();
}

test('a direct statement', () => {
  vtl.reset({ seed: 1 });
  assert.equal(session(['?=2+5*2']), '14\nOK\n');
  assert.equal(vtl.free(), 32768 - 264);
});

test('options', () => {
  vtl.reset({ precedence: true, memory: 4096 });
  assert.equal(session(['?=2+5*2', '?=*']), '12\nOK\n4096\nOK\n');
  vtl.reset({ strict: true });
  assert.match(session(['?=6&3']), /^ERROR: BAD OPERATOR/);
});

test('a program with line and key input', () => {
  vtl.reset();
  vtl.load('10 A=?\n20 B=$\n30 ?=A*2\n40 $=B\n');
  assert.equal(settle(vtl.enter('RUN')), Status.LINE);
  assert.equal(vtl.line(), 10);
  assert.equal(settle(vtl.enter('20+1')), Status.KEY);
  assert.equal(settle(vtl.key(90)), Status.READY);
  assert.equal(vtl.output(), '42Z\nOK\n');
  assert.equal(vtl.value(), 90);
  assert.ok(vtl.statements() >= 4);
});

test('text survives the trip in both directions', () => {
  vtl.reset();
  assert.equal(session(['?="GRÜSSE AUS DER ZUKUNFT ✓"']), 'GRÜSSE AUS DER ZUKUNFT ✓\nOK\n');
  session(['10 ?="ÄÖÜ"']);
  assert.equal(vtl.listing(), '10 ?="ÄÖÜ"\n');
  vtl.newProgram();
  assert.equal(vtl.listing(), '');
});

test('stopping a program', () => {
  vtl.reset();
  vtl.load('10 #=20\n20 #=10\n');
  vtl.enter('#=1');
  assert.equal(vtl.run(1000), Status.RUNNING);
  vtl.stop();
  assert.match(vtl.output(), /^STOP AT (10|20)\nOK\n$/);
  assert.equal(vtl.status(), Status.READY);
});

test('loading files', () => {
  vtl.reset();
  const direct = vtl.load('#!/usr/bin/env vtl-rs\n# comment\n) comment\n10 ?=1\n\n#=1\n');
  assert.deepEqual(direct, ['#=1']);
  assert.ok(isComment(') x') && isComment('# x') && !isComment('# = 1'));
});

test('every example runs and is offered on the page', async () => {
  const files = readdirSync(`${root}examples`).filter((f) => f.endsWith('.vtl'));
  assert.deepEqual(examples.map((e) => `${e.file}`).sort(), files.sort());
  for (const name of ['hello', 'primes', 'sieve']) {
    vtl.reset();
    vtl.load(readFileSync(`${root}examples/${name}.vtl`, 'utf8'));
    const expected = readFileSync(`${root}tests/expected/${name}.out`, 'utf8');
    assert.equal(session(['RUN']), `${expected}${expected.endsWith('\n') ? '' : '\n'}OK\n`);
  }
});
