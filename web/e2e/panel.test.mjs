// The front panel: its lights while a program runs and waits, and its
// switches.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { done, lights, open, screen, setup, type, until, waitLight, watchLights } from './harness.mjs';

const env = setup();

const LOOP = ['10 A=A+1', '20 #=10'];

test('the lights flicker while a program runs and settle when it stops', async () => {
  const page = await open(env);
  await type(page, ...LOOP, 'RUN');
  await waitLight(page, 'RUN');
  // HLTA's afterglow fades first.
  await waitLight(page, 'HLTA', false);
  const seen = await watchLights(page, 40);
  const min = (label) => Math.min(...seen[label]);
  const max = (label) => Math.max(...seen[label]);
  const distinct = (label) => new Set(seen[label]).size;

  assert.ok(min('RUN') > 0.9, 'RUN is lit all the time');
  assert.equal(max('HLTA'), 0, 'HLTA is dark');
  // Lines 10 (1010) and 20 (10100) take turns: A1 and A3 are lit for one,
  // A2 and A4 for the other, so each glows half bright; A0 and A5 up stay dark.
  for (const label of ['A1', 'A2', 'A3', 'A4']) {
    assert.ok(min(label) > 0.1 && max(label) < 0.95, `${label} glows dimmed: ${seen[label]}`);
  }
  for (const label of ['A0', 'A5', 'A15']) assert.equal(max(label), 0, `${label} is dark`);
  // A counts up, so its low bits change all the time.
  assert.ok(distinct('D0') + distinct('D1') + distinct('D2') > 3, 'the data lights change');

  // The lit layer shows the brightness the lights were given.
  const shown = await page.$eval('.led-cell:first-child .led', (led) => ({
    b: Number(led.style.getPropertyValue('--b')),
    opacity: Number(getComputedStyle(led, '::after').opacity),
  }));
  assert.ok(Math.abs(shown.b - shown.opacity) < 0.01, `opacity ${shown.opacity} for --b ${shown.b}`);

  await page.keyboard.press('Escape');
  await until(page, /^STOP AT/m);
  await waitLight(page, 'RUN', false);
  await waitLight(page, 'HLTA', true);
  const still = await watchLights(page, 10);
  for (const label of ['A1', 'A2', 'A3', 'A4', 'D0']) {
    assert.equal(new Set(still[label]).size, 1, `${label} is steady once the program stopped`);
  }
  await done(page, assert);
});

test('the address lights show the line that waits for input', async () => {
  const page = await open(env);
  await type(page, '10 A=?', 'RUN');
  await waitLight(page, 'INP');
  // A waiting machine shows its line steadily: 10 is 1010.
  await page.waitForTimeout(300);
  const on = await lights(page);
  assert.deepEqual(
    ['A3', 'A2', 'A1', 'A0'].map((label) => on[label]),
    [1, 0, 1, 0],
  );
  assert.equal(on.RUN, 0);
  await done(page, assert);
});

test('the data lights show the value stored last', async () => {
  const page = await open(env);
  await type(page, 'A=165');
  await until(page, /^OK\nA=165\nOK$/m);
  await page.waitForTimeout(300);
  const on = await lights(page);
  // 165 is 10100101.
  assert.deepEqual(
    ['D7', 'D6', 'D5', 'D4', 'D3', 'D2', 'D1', 'D0'].map((label) => on[label]),
    [1, 0, 1, 0, 0, 1, 0, 1],
  );
  await done(page, assert);
});

test('the OUT light is lit while output waits for the line', async () => {
  const page = await open(env, { settings: { baud: 110 } });
  await waitLight(page, 'OUT');
  await waitLight(page, 'OUT', false, 15_000);
  await done(page, assert);
});

/** Flips a switch with the mouse, as a person would. */
async function flip(page, action) {
  await page.click(`.switch[data-action="${action}"]`);
}

test('the switches run, list, stop, clear and reset', async () => {
  const page = await open(env);
  await type(page, '10 ?="SWITCHED"');
  await flip(page, 'run');
  await until(page, /^SWITCHED\nOK$/m);
  await flip(page, 'list');
  await until(page, /^LIST\n10 \?="SWITCHED"\nOK$/m);

  await type(page, ...LOOP);
  await flip(page, 'run');
  await waitLight(page, 'RUN');
  await flip(page, 'stop');
  await until(page, /^STOP AT (10|20)\nOK$/m);
  await waitLight(page, 'RUN', false);

  await flip(page, 'clear');
  assert.equal((await screen(page)).trim(), '');

  await flip(page, 'reset');
  await until(page, /BYTES FREE/);
  await type(page, '0');
  await until(page, /^0\nOK$/m);
  // Keys still go to the terminal after a switch.
  await type(page, '?=7');
  await until(page, /^7$/m);
  await done(page, assert);
});

test('a switch flips down while pressed and springs back', async () => {
  const page = await open(env);
  const lever = page.locator('.switch[data-action="list"] .lever');
  const transform = () => lever.evaluate((el) => getComputedStyle(el).transform);
  assert.equal(await transform(), 'none');
  await page.locator('.switch[data-action="list"]').hover();
  await page.mouse.down();
  await page.waitForTimeout(250);
  assert.equal(await transform(), 'matrix(1, 0, 0, -1, 0, 0)');
  await page.mouse.up();
  await page.waitForTimeout(450);
  assert.equal(await transform(), 'none');
  await done(page, assert);
});

test('the brand on the panel opens the About box', async () => {
  const page = await open(env);
  await page.click('.brand');
  assert.equal(await page.locator('#about-dialog').evaluate((d) => d.open), true);
  await done(page, assert);
});
