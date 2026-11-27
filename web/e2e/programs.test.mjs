// Programs: the examples, files, links, programs saved in the browser and
// the help.

import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { examples } from '../www/examples.mjs';
import { done, open, ready, root, screen, setup, type, until, waitLight } from './harness.mjs';

const env = setup();

for (const { file, title } of examples) {
  const name = file.replace(/\.vtl$/, '');
  const expected = readFileSync(join(root, 'tests/expected', `${name}.out`), 'utf8');
  const input = join(root, 'tests/expected', `${name}.in`);
  test(`the example "${title}" runs as on the command line`, async () => {
    // The factorials are those of the manual, with 1 KB of memory.
    const page = await open(env, { settings: { baud: 0, memory: name === 'factorials' ? 1024 : 32768 } });
    await page.selectOption('#example', file);
    await until(page, new RegExp(`^LOAD ${name.toUpperCase()}$`, 'm'));
    if (name === 'guess') {
      // A random number to guess: it asks.
      await waitLight(page, 'INP');
    } else if (existsSync(input)) {
      const lines = readFileSync(input, 'utf8').split('\n').filter(Boolean);
      for (const line of lines) {
        await waitLight(page, 'INP');
        await type(page, line);
      }
      await until(page, /\nOK\n$/);
      // As on the command line, but the terminal shows what was typed.
      const text = await screen(page);
      let output = text.slice(text.lastIndexOf('RUN\n') + 4).replace(/OK\n$/, '');
      for (const line of lines) output = output.replace(`${line}\n`, '');
      assert.equal(output.trimEnd(), expected.trimEnd());
    } else {
      await until(page, /^RUN\n/m, 5000);
      await until(page, /\nOK\n$/, 30_000);
      const text = await screen(page);
      const output = text.slice(text.lastIndexOf('RUN\n') + 4);
      // Then the prompt's OK, on a line of its own.
      assert.equal(output.replace(/OK\n$/, '').trimEnd(), expected.trimEnd());
    }
    await done(page, assert);
  });
}

test('a file opens: its lines are stored and listed', async () => {
  const page = await open(env);
  await page.click('#open').catch(() => {});
  await page.setInputFiles('#file', {
    name: 'mine.vtl',
    mimeType: 'text/plain',
    buffer: Buffer.from(') A COMMENT\n10 ?="FROM A FILE"\n20 ?=1+1\n'),
  });
  await until(page, /^LOAD MINE\.VTL$/m);
  await until(page, /^10 \?="FROM A FILE"\n20 \?=1\+1\nOK$/m);
  await type(page, 'RUN');
  await until(page, /^FROM A FILE\n2\nOK$/m);
  await done(page, assert);
});

test('Save hands over the program as a file', async () => {
  const page = await open(env);
  await type(page, '20 ?=2', '10 ?=1');
  const [download] = await Promise.all([page.waitForEvent('download'), page.click('#download')]);
  assert.equal(download.suggestedFilename(), 'program.vtl');
  const text = readFileSync(await download.path(), 'utf8');
  assert.equal(text, '10 ?=1\n20 ?=2\n');
  await done(page, assert);
});

test('Copy link copies a link that brings the program back', async () => {
  const page = await open(env, { context: { permissions: ['clipboard-read', 'clipboard-write'] } });
  await type(page, '10 ?="SHARED ÄÖÜ"');
  await page.click('#share');
  await until(page, /^LINK TO THIS PROGRAM COPIED\nOK$/m);
  const link = await page.evaluate(() => navigator.clipboard.readText());
  assert.equal(link, page.url());
  assert.match(link, /#program=[\w-]+$/);

  const other = await open(env, { hash: new URL(link).hash });
  await until(other, /^10 \?="SHARED ÄÖÜ"\nOK$/m);
  await type(other, 'RUN');
  await until(other, /^SHARED ÄÖÜ\nOK$/m);
  await done(other, assert);
  await done(page, assert);
});

test('A link stores its numbered lines and runs nothing, SAVE least of all', async () => {
  const page = await open(env);
  await type(page, '10 ?="MINE"', 'SAVE GAME');
  await until(page, /^SAVE GAME\nOK$/m);
  const text = '10 ?="FROM A LINK"\nSAVE GAME\nRUN\n?=12345\n';
  const hash = `#program=${Buffer.from(text).toString('base64url')}`;
  // In the same browser, where GAME is kept.
  const other = page;
  await other.goto(env.url + hash);
  await other.reload();
  await ready(other);
  await until(other, /^10 \?="FROM A LINK"\nOK$/m);
  await other.waitForTimeout(300);
  const shown = await screen(other);
  assert.doesNotMatch(shown, /SAVE GAME|12345|^FROM A LINK$/m);
  const kept = await other.evaluate(() => localStorage.getItem('vtl-rs:program:GAME'));
  assert.equal(kept, '10 ?="MINE"\n');
  await done(page, assert);
});

test('A line without end is cut into lines the page can take', async () => {
  const page = await open(env);
  await type(page, '10 ?="0123456789";', '20 #=10', 'RUN');
  await page.waitForTimeout(1000);
  await page.keyboard.press('Escape');
  await until(page, /^STOP AT \d+\nOK$/m);
  const longest = await page.$eval('#screen', (el) => Math.max(...[...el.children].map((l) => l.textContent.length)));
  assert.ok(longest <= 4096 + 20, `a line of ${longest} characters`);
  await done(page, assert);
});

test('SAVE, DIR and LOAD keep programs in the browser', async () => {
  const page = await open(env);
  await type(page, 'DIR');
  await until(page, /^NO PROGRAMS SAVED IN THIS BROWSER\nOK$/m);
  await type(page, '10 ?="KEPT"', 'SAVE KEEP', 'NEW', '0');
  await until(page, /^0\nOK$/m);
  await type(page, 'DIR');
  await until(page, /^DIR\nKEEP\nOK$/m);
  await type(page, 'LOAD keep', 'RUN');
  await until(page, /^KEPT\nOK$/m);
  await type(page, 'LOAD NOTHING');
  await until(page, /^NO PROGRAM NOTHING\. TRY DIR\nOK$/m);
  await done(page, assert);
});

test('Help and HELP open the short guide, which closes again', async () => {
  const page = await open(env);
  const open_ = () => page.$eval('#help-dialog', (d) => d.open);
  await page.click('.toolbar [data-open="help-dialog"]');
  assert.equal(await open_(), true);
  await page.click('#help-dialog footer button');
  assert.equal(await open_(), false);
  // The keys go back to the terminal.
  await page.waitForFunction(() => document.activeElement?.id === 'keys');
  await type(page, 'HELP');
  assert.equal(await open_(), true);
  await page.click('#help-dialog .close');
  assert.equal(await open_(), false);
  await page.waitForFunction(() => document.activeElement?.id === 'keys');
  await type(page, '?=5');
  await until(page, /^5$/m);
  await done(page, assert);
});
