// That nothing moves that should stay: the panel, the terminal, the settings
// and the parts of each while a program runs and prints, the lights flicker,
// the cursor blinks, a switch flips or a setting changes; that the page
// does not shift while it starts; and that it fits narrow phones.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { done, open, setup, type, until, waitLight } from './harness.mjs';

const env = setup();

/** Every part that has to stay put, with where it is. */
const PARTS = [
  '.panel',
  '.panel-top',
  '.group',
  '.led',
  '.switch',
  '.switch .label',
  '.switch .nut',
  '.terminal',
  '.bezel',
  '#screen',
  '.controls',
  '.toolbar button',
  '.toolbar select',
  'fieldset',
  'fieldset select',
  'fieldset input',
];

function boxes(page, parts = PARTS) {
  return page.evaluate(
    (parts) =>
      Object.fromEntries(
        parts.map((selector) => [
          selector,
          [...document.querySelectorAll(selector)]
            .filter((el) => el.offsetParent !== null || el === document.body)
            .map((el) => {
              const r = el.getBoundingClientRect();
              return [r.x, r.y + window.scrollY, r.width, r.height].map((n) => Math.round(n * 2) / 2);
            }),
        ]),
      ),
    parts,
  );
}

/** Fails with the first part that moved or changed its size. */
function same(before, after, what) {
  for (const selector of Object.keys(before)) {
    assert.equal(after[selector].length, before[selector].length, `${what}: the number of ${selector}`);
    before[selector].forEach((box, i) => {
      assert.deepEqual(after[selector][i], box, `${what}: ${selector} #${i} moved (x, y, width, height)`);
    });
  }
}

async function settled(page) {
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(100);
}

test('the page does not shift while it starts, prints and runs', async () => {
  const page = await open(env, { settings: { baud: 300 } });
  // From the first moment on, as the browser counts layout shifts.
  await page.addInitScript(() => {
    window.shifts = [];
    new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        window.shifts.push({ value: entry.value, sources: entry.sources.map((s) => s.node?.className ?? '?') });
      }
    }).observe({ type: 'layout-shift', buffered: true });
  });
  await page.reload();
  await until(page, /^OK$/m, 6000);
  await page.selectOption('#baud', '0');
  await page.selectOption('#example', 'primes.vtl');
  await until(page, /\nOK\n$/, 20_000);
  await page.keyboard.type('?="TYPED"');
  const shifts = await page.evaluate(() => window.shifts);
  const total = shifts.reduce((sum, s) => sum + s.value, 0);
  assert.ok(total < 0.001, `layout shifts: ${JSON.stringify(shifts)}`);
  await done(page, assert);
});

test('nothing moves while a program runs, prints and the lights flicker', async () => {
  const page = await open(env);
  await settled(page);
  const before = await boxes(page);
  await type(page, '10 ?=I', '20 ?=" ";', '30 I=I+1', '40 #=10', 'RUN');
  await waitLight(page, 'RUN');
  for (let i = 0; i < 5; i++) {
    await page.waitForTimeout(150);
    same(before, await boxes(page), `running, look ${i + 1}`);
  }
  // The screen keeps its size, scrolls, and shows the newest line.
  const scroll = await page.$eval('#screen', (el) => ({
    overflow: el.scrollHeight > el.clientHeight,
    bottom: Math.abs(el.scrollHeight - el.clientHeight - el.scrollTop) < 2,
  }));
  assert.deepEqual(scroll, { overflow: true, bottom: true });
  await page.keyboard.press('Escape');
  await waitLight(page, 'RUN', false);
  same(before, await boxes(page), 'stopped');
  await done(page, assert);
});

test('nothing moves while the cursor blinks or keys are typed', async () => {
  const page = await open(env);
  await settled(page);
  const before = await boxes(page);
  const cursor = () => page.$eval('.cursor', (el) => el.getBoundingClientRect().toJSON());
  const first = await cursor();
  // Over more than a blink: the cursor fades, but stays where it is.
  const seen = new Set();
  for (let i = 0; i < 12; i++) {
    seen.add(await page.$eval('.cursor', (el) => getComputedStyle(el).opacity));
    assert.deepEqual(await cursor(), first, 'the cursor moved while it blinked');
    await page.waitForTimeout(100);
  }
  assert.deepEqual([...seen].sort(), ['0', '1'], 'the cursor blinks');
  await page.keyboard.type('?="SOME TEXT"');
  await page.keyboard.press('Backspace');
  same(before, await boxes(page), 'typing');
  await done(page, assert);
});

test('a switch moves only its lever', async () => {
  const page = await open(env);
  await settled(page);
  const before = await boxes(page);
  for (const action of ['list', 'clear']) {
    await page.locator(`.switch[data-action="${action}"]`).hover();
    await page.mouse.down();
    await page.waitForTimeout(200);
    same(before, await boxes(page), `${action} held down`);
    await page.mouse.up();
  }
  await done(page, assert);
});

test('settings change the screen, not where things are', async () => {
  const page = await open(env);
  await settled(page);
  const before = await boxes(page);
  const steps = [
    ['#theme', 'green'],
    ['#theme', 'white'],
    ['#theme', 'blue'],
    ['#theme', 'teletype'],
    ['#theme', 'dark'],
    ['#theme', 'light'],
    ['#theme', 'amber'],
    ['#font', 'courier'],
    ['#font', 'plex'],
    ['#font', 'auto'],
    ['#baud', '110'],
    ['#baud', '0'],
    ['#memory', '65535'],
  ];
  for (const [select, option] of steps) {
    await page.selectOption(select, option);
    await settled(page);
    const now = await boxes(page);
    if (option === 'teletype') {
      // The Teletype's paper sits deeper in its machine than a CRT's glass,
      // in a terminal of the same size.
      const paper = now['#screen'][0];
      assert.ok(paper[2] < before['#screen'][0][2], 'the paper is narrower than the glass');
      now['#screen'] = before['#screen'];
    }
    same(before, now, `${select} ${option}`);
  }
  for (const fx of ['scanlines', 'glow', 'curve', 'flicker', 'blink', 'upper', 'strict', 'precedence']) {
    await page.click(`#${fx}`);
    same(before, await boxes(page), `#${fx} switched`);
    await page.click(`#${fx}`);
  }
  await page.$eval('#size', (el) => {
    el.value = '28';
    el.dispatchEvent(new Event('input', { bubbles: true }));
  });
  same(before, await boxes(page), 'a larger text');
  await done(page, assert);
});

test('the boxes open over the page without moving it', async () => {
  const page = await open(env);
  await settled(page);
  const before = await boxes(page);
  for (const id of ['help-dialog', 'about-dialog']) {
    await page.click(`.toolbar [data-open="${id}"]`);
    await page.waitForTimeout(150);
    same(before, await boxes(page), `${id} open`);
    const inside = await page.$eval(`#${id}`, (d) => {
      const r = d.getBoundingClientRect();
      return r.left >= 0 && r.top >= 0 && r.right <= innerWidth && r.bottom <= innerHeight;
    });
    assert.ok(inside, `${id} fits the window`);
    await page.keyboard.press('Escape');
  }
  await done(page, assert);
});

test('an idle machine does not flicker', async () => {
  // Flicker and the blinking cursor off, the rest as it comes: two pictures
  // apart in time are the same.
  const page = await open(env, { settings: { baud: 0, blink: false, flicker: false } });
  await settled(page);
  await page.waitForTimeout(500);
  const a = await page.screenshot();
  await page.waitForTimeout(700);
  const b = await page.screenshot();
  assert.ok(a.equals(b), 'the page changed while nothing happened');
  await done(page, assert);
});

for (const [name, viewport] of [
  ['a phone', { width: 390, height: 844 }],
  ['a small phone', { width: 320, height: 640 }],
  ['a tablet', { width: 820, height: 1180 }],
  ['a wide screen', { width: 1920, height: 1080 }],
]) {
  test(`the page fits ${name}`, async () => {
    const page = await open(env, { context: { viewport, isMobile: viewport.width < 800, hasTouch: viewport.width < 800 } });
    await settled(page);
    const fit = await page.evaluate(() => {
      const width = document.documentElement.clientWidth;
      const outside = [...document.querySelectorAll('.machine *')]
        .filter((el) => el.offsetParent !== null && !el.closest('dialog'))
        .filter((el) => {
          const r = el.getBoundingClientRect();
          return r.width > 0 && (r.left < -0.5 || r.right > width + 0.5);
        })
        .map((el) => el.className || el.tagName);
      return { scroll: document.documentElement.scrollWidth - width, outside };
    });
    assert.deepEqual(fit, { scroll: 0, outside: [] });
    // The switches are large enough to hit with a finger.
    const smallest = await page.$$eval('.switch', (all) => Math.min(...all.map((s) => Math.min(s.offsetWidth, s.offsetHeight))));
    assert.ok(smallest >= 40, `a switch is ${smallest} px`);
    await done(page, assert);
  });
}
