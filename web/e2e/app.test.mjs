// The page as an app: the About box, full screen, and what makes it
// installable and work offline.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { done, open, ready, setup, type, until } from './harness.mjs';

const env = setup();

test('About shows the project, the build and the fonts with their licences', async () => {
  const page = await open(env);
  for (const opener of ['.toolbar [data-open="about-dialog"]', '.brand', '.footer [data-open="about-dialog"]']) {
    await page.click(opener);
    assert.equal(await page.$eval('#about-dialog', (d) => d.open), true, opener);
    await page.keyboard.press('Escape');
  }
  await page.click('.toolbar [data-open="about-dialog"]');
  const build = await (await page.request.get(`${env.url}build.json`)).json();
  const field = (name) => page.$eval(`[data-build="${name}"]`, (el) => el.textContent);
  await page.waitForFunction(() => document.querySelector('[data-build="commit"]').textContent !== '–');
  assert.equal(await field('version'), build.version);
  assert.match(build.version, /^\d+\.\d+\.\d+$/);
  assert.equal(await field('commit'), build.commit.slice(0, 7));
  assert.equal(
    await page.$eval('[data-build="commit"] a', (a) => a.href),
    `${build.repository}/commit/${build.commit}`,
  );
  assert.equal(await field('branch'), build.branch);
  assert.notEqual(await field('built'), '–');
  assert.equal(await field('run'), build.run);
  // The licences of the fonts are on the site.
  const licences = await page.$$eval('#about-dialog .licence', (all) => all.map((a) => a.href));
  assert.equal(licences.length, 4);
  for (const href of licences) assert.equal((await page.request.get(href)).status(), 200, href);
  await page.click('#about-dialog footer button');
  assert.equal(await page.$eval('#about-dialog', (d) => d.open), false);
  await done(page, assert);
});

test('full screen shows the display alone, or the whole machine', async () => {
  const page = await open(env);
  const visible = async () =>
    Object.fromEntries(
      await Promise.all(['#panel', '#screen', '.controls', '.footer', '#leave-full'].map(async (s) => [s, await page.isVisible(s)])),
    );
  assert.deepEqual(await visible(), { '#panel': true, '#screen': true, '.controls': true, '.footer': true, '#leave-full': false });

  await page.click('#full-display');
  assert.deepEqual(await visible(), { '#panel': false, '#screen': true, '.controls': false, '.footer': false, '#leave-full': true });
  const box = await page.$eval('#screen', (el) => el.getBoundingClientRect().toJSON());
  assert.ok(box.width > 1100 && box.height > 820, `the display fills the window: ${box.width} × ${box.height}`);
  // Typing still works.
  await type(page, '?=6*7');
  await until(page, /^42$/m);
  await page.click('#leave-full');
  assert.equal(await page.$eval('body', (b) => b.className.includes('full')), false);

  await page.click('#full-machine');
  assert.deepEqual(await visible(), { '#panel': true, '#screen': true, '.controls': true, '.footer': false, '#leave-full': true });
  // All of it in the window, the terminal as tall as the rest leaves it.
  const fits = await page.$eval('.controls', (el) => el.getBoundingClientRect().bottom <= innerHeight + 0.5);
  assert.ok(fits, 'the settings are in the window');
  await page.click('#leave-full');
  assert.deepEqual(await visible(), { '#panel': true, '#screen': true, '.controls': true, '.footer': true, '#leave-full': false });
  await done(page, assert);
});

test('the manifest makes it an app with icons, in black bars', async () => {
  const page = await open(env);
  const href = await page.$eval('link[rel="manifest"]', (l) => l.href);
  const manifest = await (await page.request.get(href)).json();
  assert.equal(manifest.display, 'standalone');
  assert.equal(manifest.start_url, './');
  assert.equal(manifest.theme_color, '#000000');
  assert.equal(manifest.background_color, '#000000');
  const sizes = manifest.icons.map((i) => `${i.sizes} ${i.purpose ?? 'any'}`);
  for (const wanted of ['192x192 any', '512x512 any', '512x512 maskable']) assert.ok(sizes.includes(wanted), wanted);
  for (const icon of manifest.icons) {
    const url = new URL(icon.src, href).href;
    if (icon.type !== 'image/png') continue;
    const size = await page.evaluate(async (url) => {
      const image = new Image();
      image.src = url;
      await image.decode();
      return `${image.naturalWidth}x${image.naturalHeight}`;
    }, url);
    assert.equal(size, icon.sizes, icon.src);
  }
  const apple = await page.$eval('link[rel="apple-touch-icon"]', (l) => l.href);
  assert.equal((await page.request.get(apple)).status(), 200);
  assert.equal(await page.$eval('meta[name="apple-mobile-web-app-status-bar-style"]', (m) => m.content), 'black');
  // The bars on a phone take the theme's colour: near black for amber.
  assert.equal(await page.$eval('meta[name="theme-color"]', (m) => m.content), '#161616');
  // No offer to install until the browser makes one.
  assert.equal(await page.isVisible('#install'), false);
  await done(page, assert);
});

test('once loaded, the page works offline', async () => {
  const page = await open(env);
  await page.evaluate(() => navigator.serviceWorker.ready);
  // The service worker takes over this page as well, and caches what it gets.
  await page.reload();
  await ready(page);
  await page.waitForFunction(() => navigator.serviceWorker.controller !== null);
  await page.selectOption('#example', 'hello.vtl');
  await until(page, /^HELLO, WORLD!\nOK$/m);

  await page.context().setOffline(true);
  await page.reload();
  await ready(page);
  await type(page, '?=1+1');
  await until(page, /^2$/m);
  // An example seen before is there too.
  await page.selectOption('#example', 'hello.vtl');
  await until(page, /^HELLO, WORLD!\nOK\n.*\n?$/m);
  await page.context().setOffline(false);
  await done(page, assert);
});
