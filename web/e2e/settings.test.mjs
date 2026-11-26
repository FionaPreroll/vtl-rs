// The settings: the 1977 defaults, themes, fonts, effects, the machine's
// options, and that they last.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { done, open, ready, screen, setup, type, until } from './harness.mjs';

const env = setup();

const value = (page, id) => page.$eval(`#${id}`, (el) => (el.type === 'checkbox' ? el.checked : el.value));
const body = (page) =>
  page.evaluate(() => ({ theme: document.body.dataset.theme, font: document.body.dataset.font, classes: [...document.body.classList] }));

test('the first visit is 1977: amber phosphor at 300 baud', async () => {
  const page = await open(env, { settings: null });
  assert.equal((await body(page)).theme, 'amber');
  assert.equal(await value(page, 'theme'), 'amber');
  assert.equal(await value(page, 'baud'), '300');
  assert.equal(await value(page, 'memory'), '32768');
  assert.equal(await value(page, 'blink'), true);
  assert.deepEqual(
    (await body(page)).classes.sort(),
    ['fx-blink', 'fx-curve', 'fx-glow', 'fx-scanlines'],
  );
  // The greeting comes at 30 characters a second, so it is not done yet.
  assert.doesNotMatch(await screen(page), /^OK$/m);
  await until(page, /^OK$/m, 6000);
  await done(page, assert);
});

const THEMES = ['teletype', 'green', 'amber', 'white', 'blue', 'dark', 'light'];

test('every theme has its own screen, and the bars of the phone follow it', async () => {
  const page = await open(env);
  const looks = new Map();
  for (const theme of THEMES) {
    await page.selectOption('#theme', theme);
    const look = await page.evaluate(() => ({
      theme: document.body.dataset.theme,
      screen: getComputedStyle(document.getElementById('screen')).backgroundColor,
      ink: getComputedStyle(document.getElementById('screen')).color,
      bar: document.querySelector('meta[name="theme-color"]').content,
      desk: getComputedStyle(document.body).getPropertyValue('--desk').trim(),
    }));
    assert.equal(look.theme, theme);
    assert.equal(look.bar, look.desk, `${theme}: the bars take the desk's colour`);
    looks.set(theme, `${look.screen} ${look.ink}`);
  }
  assert.equal(new Set(looks.values()).size, THEMES.length, 'no two themes look alike');
  await done(page, assert);
});

test('the Teletype and "Capitals only" type in capitals', async () => {
  const page = await open(env);
  const typed = () => page.$eval('.typed', (el) => el.textContent);
  await page.keyboard.type('?="abc"');
  assert.equal(await typed(), '?="abc"');
  await page.keyboard.press('Escape');
  for (let i = 0; i < 7; i++) await page.keyboard.press('Backspace');
  await page.check('#upper');
  await page.keyboard.type('?="abc"');
  assert.equal(await typed(), '?="ABC"');
  for (let i = 0; i < 7; i++) await page.keyboard.press('Backspace');
  await page.uncheck('#upper');
  await page.selectOption('#theme', 'teletype');
  await page.keyboard.type('?="abc"');
  await page.keyboard.press('Enter');
  await until(page, /^ABC$/m);
  await done(page, assert);
});

const FONTS = {
  vt323: 'VT323',
  courier: 'Courier Prime',
  sharetech: 'Share Tech Mono',
  plex: 'IBM Plex Mono',
  system: 'ui-monospace',
};

test('every font is used and loads', async () => {
  const page = await open(env);
  for (const [option, family] of Object.entries(FONTS)) {
    await page.selectOption('#font', option);
    const used = await page.$eval('#screen', (el) => getComputedStyle(el).fontFamily);
    assert.ok(used.startsWith(family) || used.startsWith(`"${family}"`), `${option}: ${used}`);
    if (option !== 'system') {
      const loaded = await page.evaluate((family) => document.fonts.load(`20px "${family}"`).then((faces) => faces.length), family);
      assert.equal(loaded, 1, `${family} loads`);
    }
  }
  await done(page, assert);
});

test('the size slider sets the size of the text', async () => {
  const page = await open(env, { settings: { baud: 0, theme: 'dark' } });
  const size = () => page.$eval('#screen', (el) => parseFloat(getComputedStyle(el).fontSize));
  const before = await size();
  await page.$eval('#size', (el) => {
    el.value = '30';
    el.dispatchEvent(new Event('input', { bubbles: true }));
  });
  assert.ok((await size()) > before * 1.4, `${before} px to ${await size()} px`);
  await done(page, assert);
});

test('the effects switch on and off', async () => {
  const page = await open(env);
  for (const fx of ['scanlines', 'glow', 'curve', 'flicker', 'blink', 'upper']) {
    const on = await value(page, fx);
    await page.click(`#${fx}`);
    assert.equal((await body(page)).classes.includes(`fx-${fx}`), !on, fx);
  }
  // What each does to the screen, with all of them on.
  for (const fx of ['scanlines', 'glow', 'curve', 'flicker', 'blink']) await page.check(`#${fx}`);
  const on = await page.evaluate(() => {
    const screen = getComputedStyle(document.getElementById('screen'));
    const glass = getComputedStyle(document.querySelector('.glass'));
    return {
      glow: screen.textShadow,
      scanlines: glass.backgroundImage,
      curve: screen.borderRadius,
      flicker: screen.animationName,
      blink: getComputedStyle(document.querySelector('.cursor')).animationName,
    };
  });
  assert.notEqual(on.glow, 'none');
  assert.match(on.scanlines, /repeating-linear-gradient/);
  assert.equal(on.flicker, 'flicker');
  assert.equal(on.blink, 'blink');
  for (const fx of ['scanlines', 'glow', 'curve', 'flicker', 'blink']) await page.uncheck(`#${fx}`);
  const off = await page.evaluate(() => ({
    glow: getComputedStyle(document.getElementById('screen')).textShadow,
    scanlines: getComputedStyle(document.querySelector('.glass')).backgroundImage,
    curve: getComputedStyle(document.getElementById('screen')).borderRadius,
    flicker: getComputedStyle(document.getElementById('screen')).animationName,
    blink: getComputedStyle(document.querySelector('.cursor')).animationName,
  }));
  assert.deepEqual(
    { glow: off.glow, scanlines: off.scanlines, flicker: off.flicker, blink: off.blink },
    { glow: 'none', scanlines: 'none', flicker: 'none', blink: 'none' },
  );
  assert.notEqual(off.curve, on.curve);
  await done(page, assert);
});

test('the front panel can be put away', async () => {
  const page = await open(env);
  await page.uncheck('#show-panel');
  assert.equal(await page.isVisible('#panel'), false);
  await page.check('#show-panel');
  assert.equal(await page.isVisible('#panel'), true);
  await done(page, assert);
});

test('Strict VTL-2 knows no VTL02 extensions', async () => {
  const page = await open(env);
  await type(page, '10 ?="KEPT"', '?=6&3');
  await until(page, /^\?=6&3\n2\nOK$/m);
  await page.check('#strict');
  await until(page, /^MACHINE SET: STRICT VTL-2, LEFT TO RIGHT, \* = 32768\nOK$/m);
  await type(page, '?=6&3');
  await until(page, /^ERROR: BAD OPERATOR AT COLUMN 4$/m);
  // The program stays.
  await type(page, 'RUN');
  await until(page, /^KEPT\nOK$/m);
  await done(page, assert);
});

test('Operator precedence puts * before +', async () => {
  const page = await open(env);
  await type(page, '?=2+3*4');
  await until(page, /^\?=2\+3\*4\n20\nOK$/m);
  await page.check('#precedence');
  await until(page, /PRECEDENCE, \* = 32768\nOK$/m);
  await type(page, '?=2+3*4');
  await until(page, /^\?=2\+3\*4\n14\nOK$/m);
  await done(page, assert);
});

test('every size of memory, up to all 64 KB VTL-2 can address', async () => {
  const page = await open(env);
  const sizes = await page.$$eval('#memory option', (options) => options.map((o) => Number(o.value)));
  assert.equal(Math.max(...sizes), 65535);
  assert.ok(sizes.length >= 12);
  for (const size of sizes) {
    await page.selectOption('#memory', String(size));
    await until(page, new RegExp(`\\* = ${size}\\nOK$`, 'm'));
    await type(page, `?=*`);
    await until(page, new RegExp(`^\\?=\\*\\n${size}\\nOK$`, 'm'));
  }
  await done(page, assert);
});

test('the settings last', async () => {
  const page = await open(env, { settings: null });
  await page.selectOption('#theme', 'green');
  await page.selectOption('#font', 'plex');
  await page.selectOption('#baud', '110');
  await page.selectOption('#memory', '65535');
  await page.uncheck('#blink');
  await page.reload();
  await ready(page);
  assert.equal((await body(page)).theme, 'green');
  assert.equal((await body(page)).font, 'plex');
  assert.equal(await value(page, 'baud'), '110');
  assert.equal(await value(page, 'memory'), '65535');
  assert.equal(await value(page, 'blink'), false);
  assert.equal(await page.$eval('.cursor', (el) => getComputedStyle(el).animationName), 'none');
  await done(page, assert);
});
