// What the end-to-end tests share: the site as built (web/build.sh) on a
// local server, Chromium through Playwright, and helpers that read the
// terminal and the front panel as a person sees them.
//   web/build.sh _site && (cd web && npm run e2e)
// SITE names another build; CHROMIUM a Chromium to use instead of
// Playwright's own.

import { createReadStream, existsSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, join, resolve } from 'node:path';
import { after, before } from 'node:test';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

export const root = fileURLToPath(new URL('../../', import.meta.url));

const types = {
  '.html': 'text/html',
  '.css': 'text/css',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.json': 'application/json',
  '.webmanifest': 'application/manifest+json',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.woff2': 'font/woff2',
};

/** Serves a directory on a free port of 127.0.0.1; resolves to its URL. */
export async function serve(dir) {
  const site = resolve(dir);
  const server = createServer((request, response) => {
    const path = new URL(request.url, 'http://localhost').pathname;
    let file = join(site, decodeURIComponent(path));
    if (existsSync(file) && statSync(file).isDirectory()) file = join(file, 'index.html');
    if (!file.startsWith(site) || !existsSync(file)) {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, { 'content-type': types[extname(file)] ?? 'text/plain' });
    createReadStream(file).pipe(response);
  });
  await new Promise((ready) => server.listen(0, '127.0.0.1', ready));
  return { url: `http://127.0.0.1:${server.address().port}/`, close: () => server.close() };
}

export function launch() {
  return chromium.launch(process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {});
}

/**
 * The site and a browser for the tests of one file: `env.url` and
 * `env.browser` once `before` ran.
 */
export function setup() {
  const env = {};
  before(async () => {
    const site = process.env.SITE ?? join(root, '_site');
    if (!existsSync(join(site, 'vtl_web.wasm'))) throw new Error(`no site in ${site}: run web/build.sh first`);
    env.server = await serve(site);
    env.url = env.server.url;
    env.browser = await launch();
  });
  after(async () => {
    await env.browser?.close();
    env.server?.close();
  });
  return env;
}

/**
 * A page of the site in a context of its own, started and greeted.
 * `settings` are stored before the page first loads (and kept when it
 * reloads); by default output comes at once rather than at 300 baud. Errors
 * of the page land in `page.errors`.
 */
export async function open(env, { settings = { baud: 0 }, context = {}, hash = '' } = {}) {
  const ctx = await env.browser.newContext({ viewport: { width: 1200, height: 900 }, ...context });
  if (settings) {
    await ctx.addInitScript((stored) => {
      if (!localStorage.getItem('vtl-rs:settings')) localStorage.setItem('vtl-rs:settings', stored);
    }, JSON.stringify(settings));
  }
  const page = await ctx.newPage();
  page.errors = [];
  page.on('pageerror', (error) => page.errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') page.errors.push(message.text());
  });
  await page.goto(env.url + hash);
  await ready(page);
  return page;
}

/** Waits until the machine greeted. */
export async function ready(page) {
  await page.locator('#screen').getByText('BYTES FREE').waitFor();
}

/** Closes the page's context, after a check that the page had no errors. */
export async function done(page, assert) {
  const { errors } = page;
  await page.context().close();
  assert.deepEqual(errors, [], 'errors in the page');
}

/** The terminal's text, line by line, as typed (not as CSS shows it). */
export function screen(page) {
  return page.$eval('#screen', (el) => [...el.children].map((line) => line.textContent).join('\n'));
}

/** Waits until the terminal shows text that matches. */
export async function until(page, pattern, timeout = 10_000) {
  try {
    await page.waitForFunction(
      ([source, flags]) => {
        const text = [...document.getElementById('screen').children].map((line) => line.textContent).join('\n');
        return new RegExp(source, flags).test(text);
      },
      [pattern.source, pattern.flags],
      { timeout, polling: 'raf' },
    );
  } catch (error) {
    throw new Error(`the terminal never showed ${pattern}; it shows:\n${await screen(page)}`, { cause: error });
  }
}

/** Types lines on the keyboard, each ended with Return. */
export async function type(page, ...lines) {
  for (const line of lines) {
    await page.keyboard.type(line);
    await page.keyboard.press('Enter');
  }
}

/** How often the terminal shows a line that is exactly `text`. */
export async function count(page, text) {
  return (await screen(page)).split('\n').filter((line) => line === text).length;
}

/** The brightness (0 to 1) of the front panel's lights, by their labels. */
export function lights(page) {
  return page.$$eval('.led-cell', (cells) =>
    Object.fromEntries(
      cells.map((cell) => [
        cell.querySelector('.led-label').textContent,
        Number(cell.querySelector('.led').style.getPropertyValue('--b') || 0),
      ]),
    ),
  );
}

/**
 * The lights frame by frame for `frames` animation frames: an object of
 * arrays by label.
 */
export function watchLights(page, frames) {
  return page.evaluate(
    (frames) =>
      new Promise((resolve) => {
        const cells = [...document.querySelectorAll('.led-cell')];
        const seen = {};
        let left = frames;
        const look = () => {
          for (const cell of cells) {
            const label = cell.querySelector('.led-label').textContent;
            const b = Number(cell.querySelector('.led').style.getPropertyValue('--b') || 0);
            (seen[label] ??= []).push(b);
          }
          if (--left > 0) requestAnimationFrame(look);
          else resolve(seen);
        };
        requestAnimationFrame(look);
      }),
    frames,
  );
}

/** Waits until a light is (`on`) or is not lit. */
export async function waitLight(page, label, on = true, timeout = 5000) {
  await page.waitForFunction(
    ([label, on]) => {
      const cell = [...document.querySelectorAll('.led-cell')].find(
        (c) => c.querySelector('.led-label').textContent === label,
      );
      const b = Number(cell.querySelector('.led').style.getPropertyValue('--b') || 0);
      return on ? b > 0.5 : b === 0;
    },
    [label, on],
    { timeout, polling: 'raf' },
  );
}
