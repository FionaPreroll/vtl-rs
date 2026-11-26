// Pictures of the web version for the README, taken from the site as built
// (web/build.sh) and written into its screenshots/ folder, so that they are
// published with it and never committed. The workflow pages.yml runs it.
//   node web/showcase.mjs _site
// CHROMIUM names a Chromium to use instead of Playwright's own.

import { mkdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { launch, serve } from './e2e/harness.mjs';

const site = resolve(process.argv[2] ?? '_site');
const server = await serve(site);
const base = server.url;

const shots = [
  { name: 'green', theme: 'green', example: 'primes.vtl' },
  { name: 'teletype', theme: 'teletype', example: 'subroutine.vtl' },
  { name: 'amber', theme: 'amber', example: 'sieve.vtl' },
  { name: 'blue', theme: 'blue', example: 'fibonacci.vtl' },
  { name: 'phone', theme: 'green', example: 'hello.vtl', phone: true },
];

const out = join(site, 'screenshots');
mkdirSync(out, { recursive: true });
const browser = await launch();
let failed = false;
for (const shot of shots) {
  const context = await browser.newContext(
    shot.phone
      ? { viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true }
      : { viewport: { width: 1180, height: 900 }, deviceScaleFactor: 2 },
  );
  await context.addInitScript((theme) => {
    localStorage.setItem('vtl-rs:settings', JSON.stringify({ theme, baud: 0 }));
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
