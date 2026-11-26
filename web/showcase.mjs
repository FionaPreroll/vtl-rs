// Pictures of the web version for the README, taken from the site as built
// (web/build.sh) and written into its screenshots/ folder, so that they are
// published with it and never committed. The workflow pages.yml runs it.
//   node web/showcase.mjs _site
// CHROMIUM names a Chromium to use instead of Playwright's own.

import { createReadStream, existsSync, mkdirSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, join, resolve } from 'node:path';
import { chromium } from 'playwright';

const site = resolve(process.argv[2] ?? '_site');
const types = {
  '.html': 'text/html',
  '.css': 'text/css',
  '.mjs': 'text/javascript',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
  '.woff2': 'font/woff2',
};

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
const base = `http://127.0.0.1:${server.address().port}/`;

const shots = [
  { name: 'green', theme: 'green', example: 'primes.vtl' },
  { name: 'teletype', theme: 'teletype', example: 'subroutine.vtl' },
  { name: 'amber', theme: 'amber', example: 'sieve.vtl' },
  { name: 'blue', theme: 'blue', example: 'fibonacci.vtl' },
  { name: 'phone', theme: 'green', example: 'hello.vtl', phone: true },
];

const out = join(site, 'screenshots');
mkdirSync(out, { recursive: true });
const browser = await chromium.launch(process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {});
let failed = false;
for (const shot of shots) {
  const context = await browser.newContext(
    shot.phone
      ? { viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true }
      : { viewport: { width: 1180, height: 900 }, deviceScaleFactor: 2 },
  );
  await context.addInitScript((theme) => {
    localStorage.setItem('vtl-rs:settings', JSON.stringify({ theme }));
  }, shot.theme);
  const page = await context.newPage();
  page.on('pageerror', (error) => {
    console.error(`${shot.name}: ${error.message}`);
    failed = true;
  });
  await page.goto(base);
  await page.locator('#screen').getByText('BYTES FREE').waitFor();
  await page.selectOption('#example', shot.example);
  await page.locator('#screen .line', { hasText: /^OK$/ }).nth(2).waitFor();
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: join(out, `${shot.name}.png`) });
  console.log(`screenshots/${shot.name}.png`);
  await context.close();
}
await browser.close();
server.close();
process.exit(failed ? 1 : 0);
