// The terminal: typing, the keys, input to a program, stopping, pasting and
// the speed of the line.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { count, done, open, screen, setup, type, until, waitLight } from './harness.mjs';

const env = setup();

test('a statement runs as it is typed', async () => {
  const page = await open(env);
  await type(page, '?=2+5*2');
  await until(page, /^14$/m);
  await done(page, assert);
});

test('a program is stored, listed and run', async () => {
  const page = await open(env);
  await type(page, '10 I=1', '20 ?=I', '30 I=I+1', '40 #=I<4*20', '0');
  await until(page, /^40 #=I<4\*20$/m);
  await type(page, 'RUN');
  // ?= prints a number without a newline.
  await until(page, /^RUN\n123\nOK$/m);
  await done(page, assert);
});

test('Backspace takes back a character, the arrows bring back lines', async () => {
  const page = await open(env);
  await page.keyboard.type('?=12');
  await page.keyboard.press('Backspace');
  await type(page, '3');
  await until(page, /^13$/m);
  await page.keyboard.press('ArrowUp');
  await page.keyboard.press('Enter');
  await until(page, /^\?=13\n13\nOK\n\?=13\n13\nOK$/m);
  assert.equal(await count(page, '13'), 2);
  await done(page, assert);
});

test('? reads a number from a line', async () => {
  const page = await open(env);
  await type(page, '10 A=?', '20 ?="TWICE: ";', '30 ?=A*2', 'RUN');
  await waitLight(page, 'INP');
  await type(page, '21');
  await until(page, /^TWICE: 42$/m);
  await done(page, assert);
});

test('$ reads a single key, without Return', async () => {
  const page = await open(env);
  await type(page, '10 A=$', '20 ?=""', '30 ?=A', 'RUN');
  await waitLight(page, 'INP');
  await page.keyboard.type('Z');
  await until(page, /^Z\n90\nOK$/m);
  await done(page, assert);
});

test('$ gets every key, however fast they come', async () => {
  const page = await open(env);
  await type(page, '10 A=$', '20 B=$', '30 C=$', '40 ?=""', '50 ?=A+B+C', 'RUN');
  await waitLight(page, 'INP');
  // Three keys within one frame.
  await page.keyboard.type('ABC');
  await until(page, /^ABC\n198\nOK$/m);
  await done(page, assert);
});

test('Esc and Ctrl-C stop a program', async () => {
  const page = await open(env);
  await type(page, '10 A=A+1', '20 #=10', 'RUN');
  await waitLight(page, 'RUN');
  await page.keyboard.press('Escape');
  await until(page, /^STOP AT (10|20)\nOK$/m);
  await type(page, 'RUN');
  await waitLight(page, 'RUN');
  await page.keyboard.press('Control+c');
  await page.waitForFunction(() => document.getElementById('screen').textContent.split('STOP AT').length === 3);
  await waitLight(page, 'RUN', false);
  await done(page, assert);
});

test('Ctrl-L and CLS clear the screen', async () => {
  const page = await open(env);
  await type(page, '?=1');
  await until(page, /^1$/m);
  await page.keyboard.press('Control+l');
  assert.equal((await screen(page)).trim(), '');
  await type(page, '?=2');
  await until(page, /^2$/m);
  await type(page, 'CLS');
  await until(page, /^OK$/m);
  assert.doesNotMatch(await screen(page), /\?=2/);
  await done(page, assert);
});

test('an error names the line and the column', async () => {
  const page = await open(env);
  await type(page, '10 ?=(', 'RUN');
  await until(page, /ERROR IN LINE 10: MISSING VALUE AT COLUMN 4\nOK$/m);
  await done(page, assert);
});

test('pasted lines are entered one after the other', async () => {
  const page = await open(env);
  await page.evaluate(() => {
    const data = new DataTransfer();
    data.setData('text/plain', '10 ?="PASTED"\r\n20 ?=6*7\nRUN\n');
    document.getElementById('keys').dispatchEvent(new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true }));
  });
  await until(page, /^PASTED\n42\nOK$/m);
  await done(page, assert);
});

test('keyboards that send text only (on phones) type too', async () => {
  const page = await open(env);
  await page.evaluate(() => {
    const keys = document.getElementById('keys');
    keys.value = '?=3*3\n';
    keys.dispatchEvent(new InputEvent('input', { bubbles: true }));
  });
  await until(page, /^9$/m);
  await done(page, assert);
});

test('output comes at the speed of the line, and Esc cuts it short', async () => {
  const page = await open(env, { settings: { baud: 300 } });
  // The greeting itself comes at 30 characters a second.
  await until(page, /^OK$/m);
  const line = 'X'.repeat(60);
  const start = Date.now();
  await type(page, `?="${line}"`);
  await page.waitForTimeout(700);
  const early = (await screen(page)).match(/^X+$/m)?.[0].length ?? 0;
  assert.ok(early > 0 && early < 40, `after 0.7 s ${early} of 60 characters`);
  await until(page, new RegExp(`^${line}$`, 'm'));
  const seconds = (Date.now() - start) / 1000;
  assert.ok(seconds >= 1.6 && seconds < 5, `60 characters at 300 baud took ${seconds} s`);

  await type(page, `?="${'Y'.repeat(200)}"`);
  await page.waitForTimeout(500);
  await page.keyboard.press('Escape');
  await until(page, /^STOP\nOK$/m);
  const printed = (await screen(page)).match(/^Y+$/m)?.[0].length ?? 0;
  assert.ok(printed < 200, `${printed} of 200 characters printed after Esc`);
  await done(page, assert);
});
