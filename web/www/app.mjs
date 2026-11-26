// The page: a terminal for the VTL-2 machine in vtl_web.wasm, with a front
// panel, themes, fonts and the speed of a serial line.

import { examples } from './examples.mjs';
import { Status, Vtl } from './vtl.mjs';

const $ = (id) => document.getElementById(id);
const screen = $('screen');
const keys = $('keys');

const STORE = 'vtl-rs:';
const MAX_LINES = 3000;
/** Milliseconds of work per frame for the machine. */
const FRAME_MS = 10;
/** Looks at the front panel's lines per frame while a program runs. */
const SAMPLES = 16;
/** How much of an LED's glow is left after a frame once it went out. */
const AFTERGLOW = 0.8;

// ---------------------------------------------------------------- settings

const defaults = {
  theme: 'green',
  font: 'auto',
  size: 20,
  baud: 0,
  scanlines: true,
  glow: true,
  flicker: false,
  curve: true,
  upper: false,
  'show-panel': true,
  strict: false,
  precedence: false,
  memory: 32768,
};

function loadSettings() {
  try {
    return { ...defaults, ...JSON.parse(localStorage.getItem(`${STORE}settings`) ?? '{}') };
  } catch {
    return { ...defaults };
  }
}

const settings = loadSettings();

function saveSettings() {
  try {
    localStorage.setItem(`${STORE}settings`, JSON.stringify(settings));
  } catch {
    // Private windows may refuse; the settings then last for this visit.
  }
}

function applySettings() {
  const body = document.body;
  body.dataset.theme = settings.theme;
  body.dataset.font = settings.font;
  body.style.setProperty('--size', `${settings.size}px`);
  for (const fx of ['scanlines', 'glow', 'flicker', 'curve', 'upper']) {
    body.classList.toggle(`fx-${fx}`, settings[fx]);
  }
  $('panel').hidden = !settings['show-panel'];
  cps = settings.baud > 0 ? settings.baud / 10 : 0;
  requestAnimationFrame(() => {
    screen.scrollTop = screen.scrollHeight;
  });
}

/** The Teletype printed capitals only, and so does "Upper case only". */
function upper() {
  return settings.upper || settings.theme === 'teletype';
}

function bindSettings() {
  for (const [name, value] of Object.entries(settings)) {
    const input = $(name);
    if (!input) continue;
    if (input.type === 'checkbox') input.checked = value;
    else input.value = String(value);
    input.addEventListener(input.type === 'range' ? 'input' : 'change', () => {
      const raw = input.type === 'checkbox' ? input.checked : input.value;
      settings[name] = typeof defaults[name] === 'number' ? Number(raw) : raw;
      saveSettings();
      applySettings();
      if (['strict', 'precedence', 'memory'].includes(name)) rebuild();
      focus();
    });
  }
}

// ---------------------------------------------------------------- the screen

let line = newLine();
let typed = '';
let cursor = document.createElement('span');
cursor.className = 'cursor';
let typedNode = document.createElement('span');
typedNode.className = 'typed';

function newLine() {
  const div = document.createElement('div');
  div.className = 'line';
  div.append(document.createTextNode(''));
  screen.append(div);
  while (screen.childElementCount > MAX_LINES) screen.firstElementChild.remove();
  return div;
}

function placeCursor() {
  typedNode.textContent = typed;
  line.append(typedNode, cursor);
}

function clearScreen() {
  screen.replaceChildren();
  line = newLine();
  placeCursor();
}

function bell() {
  screen.classList.remove('bell');
  void screen.offsetWidth;
  screen.classList.add('bell');
}

/** Puts printed text on the screen at once. */
function print(text) {
  let node = line.firstChild;
  let pending = '';
  const flush = () => {
    if (pending) node.appendData(pending);
    pending = '';
  };
  for (const ch of text) {
    if (ch === '\n') {
      flush();
      line = newLine();
      node = line.firstChild;
    } else if (ch === '\x07') {
      bell();
    } else if (ch === '\x0c') {
      flush();
      clearScreen();
      node = line.firstChild;
    } else if (ch === '\b') {
      flush();
      node.deleteData(Math.max(0, node.length - 1), 1);
    } else if (ch >= ' ' || ch === '\t') {
      pending += ch;
    }
  }
  flush();
  placeCursor();
  screen.scrollTop = screen.scrollHeight;
}

// What waits to be printed at the speed of the line.
let queue = '';
let cps = 0;
let credit = 0;
let lastPump = performance.now();

function output(text) {
  if (!text) return;
  if (cps === 0 && !queue) print(text);
  else queue += text;
}

/** What the user typed: on the screen at once, unless output still waits. */
function echo(text) {
  if (queue) queue += text;
  else print(text);
}

function pump(now) {
  const elapsed = Math.min(now - lastPump, 250);
  lastPump = now;
  if (!queue) {
    credit = 0;
    return;
  }
  if (cps === 0) {
    print(queue);
    queue = '';
    return;
  }
  credit += (elapsed / 1000) * cps;
  const count = Math.floor(credit);
  if (count === 0) return;
  credit -= count;
  const chars = Array.from(queue.slice(0, count * 2)).slice(0, count).join('');
  queue = queue.slice(chars.length);
  print(chars);
}

// ---------------------------------------------------------------- the machine

/** @type {Vtl} */
let vtl;
let status = Status.READY;
/** What the running program waited for last: keys typed then go to $. */
let lastWait = Status.READY;
let budget = 20_000;
/** Lines waiting to be entered: pasted text and the rest of a loaded file. */
let pendingLines = [];
const typedLines = [];
let historyAt = 0;

function machineOptions() {
  return { memory: settings.memory, strict: settings.strict, precedence: settings.precedence };
}

/** A new machine with the current options, keeping the program. */
function rebuild() {
  const program = vtl.listing();
  vtl.stop();
  vtl.output();
  vtl.reset(machineOptions());
  vtl.load(program);
  output(vtl.output());
  status = vtl.status();
  output(`MACHINE SET: ${settings.strict ? 'STRICT VTL-2' : 'VTL-2 WITH VTL02 EXTENSIONS'}, ${settings.precedence ? 'PRECEDENCE' : 'LEFT TO RIGHT'}, * = ${settings.memory}\nOK\n`);
}

function greeting() {
  output(`VTL-RS - VERY TINY LANGUAGE\n${vtl.free()} BYTES FREE. TYPE HELP FOR HELP.\nOK\n`);
}

function enter(text) {
  status = vtl.enter(text);
  output(vtl.output());
}

/** A line the user typed (or pasted, or that came from a file). */
function submit(text) {
  echo(`${text}\n`);
  if (status === Status.READY && hostCommand(text)) return;
  if (text.trim()) {
    typedLines.push(text);
    historyAt = typedLines.length;
  }
  enter(text);
}

/** The commands of the page rather than of the machine. */
function hostCommand(text) {
  const match = /^\s*([A-Za-z]+)\s*(.*)$/.exec(text);
  if (!match || match[2].startsWith('=')) return false;
  const word = match[1].toUpperCase();
  const arg = match[2].trim().replace(/^"|"$/g, '').toUpperCase();
  const key = `${STORE}program:${arg}`;
  switch (word) {
    case 'HELP':
      $('help-dialog').showModal();
      output('OK\n');
      return true;
    case 'CLS':
      clearScreen();
      output('OK\n');
      return true;
    case 'DIR': {
      const names = Object.keys(localStorage)
        .filter((k) => k.startsWith(`${STORE}program:`))
        .map((k) => k.slice(`${STORE}program:`.length))
        .sort();
      output(names.length ? `${names.join('\n')}\nOK\n` : 'NO PROGRAMS SAVED IN THIS BROWSER\nOK\n');
      return true;
    }
    case 'SAVE':
      if (!arg) output('SAVE NEEDS A NAME\nOK\n');
      else {
        try {
          localStorage.setItem(key, vtl.listing());
          output('OK\n');
        } catch {
          output('CANNOT SAVE IN THIS BROWSER\nOK\n');
        }
      }
      return true;
    case 'LOAD': {
      const program = arg ? localStorage.getItem(key) : null;
      if (program === null) output(arg ? `NO PROGRAM ${arg}. TRY DIR\nOK\n` : 'LOAD NEEDS A NAME\nOK\n');
      else loadProgram(program, false);
      return true;
    }
    default:
      return false;
  }
}

/** Replaces the program; lines without a number are entered after it. */
function loadProgram(text, list = true) {
  vtl.stop();
  vtl.output();
  vtl.newProgram();
  const direct = vtl.load(text);
  output(vtl.output());
  status = vtl.status();
  if (list) pendingLines.push('0');
  else output('OK\n');
  pendingLines.push(...direct);
}

function stop() {
  const dropped = queue.length > 0 || pendingLines.length > 0;
  queue = '';
  pendingLines = [];
  if (status === Status.READY) {
    // Nothing runs: only the printing stops.
    if (dropped) print(`${line.firstChild.length ? '\n' : ''}STOP\nOK\n`);
    return;
  }
  vtl.stop();
  output(vtl.output());
  status = vtl.status();
}

/** Whether keys go to the program's $ now rather than into a line. */
function wantsKeys() {
  return status === Status.KEY || (status === Status.RUNNING && lastWait === Status.KEY);
}

function key(code) {
  status = vtl.key(code);
}

function tick(now) {
  if (status === Status.KEY || status === Status.LINE) lastWait = status;
  else if (status === Status.READY) lastWait = Status.READY;
  if (status === Status.RUNNING && queue.length < 2048) {
    // In slices, with a look at the lines after each, so that the front
    // panel shows what happened during the frame and not only its end.
    const start = performance.now();
    const slice = Math.max(1, Math.round(budget / SAMPLES));
    for (let i = 0; i < SAMPLES && status === Status.RUNNING; i++) {
      status = vtl.run(slice);
      sample(true);
    }
    const spent = performance.now() - start;
    budget = Math.max(1000, Math.min(5_000_000, Math.round(budget * (spent > FRAME_MS ? 0.7 : 1.3))));
    output(vtl.output());
  }
  if ((status === Status.READY || status === Status.LINE) && pendingLines.length && !queue) {
    submit(pendingLines.shift());
  }
  if (status === Status.KEY && typed) {
    // Typed ahead while the program ran: the keys go to $ one by one.
    const [first] = typed;
    typed = typed.slice(first.length);
    placeCursor();
    echo(first);
    if (first.codePointAt(0) < 256) key(first.codePointAt(0));
  }
  pump(now);
  lights();
  requestAnimationFrame(tick);
}

// ---------------------------------------------------------------- the panel

function ledRow(container, labels) {
  return labels.map((label) => {
    const cell = document.createElement('span');
    cell.className = 'led-cell';
    const led = document.createElement('span');
    led.className = 'led';
    const text = document.createElement('span');
    text.className = 'led-label';
    text.textContent = label;
    cell.append(led, text);
    container.append(cell);
    return led;
  });
}

const statusLeds = ledRow($('status-leds'), ['RUN', 'INP', 'OUT', 'HLTA']);
const addressLeds = ledRow(
  $('address-leds'),
  Array.from({ length: 16 }, (_, i) => `A${15 - i}`),
);
const dataLeds = ledRow(
  $('data-leds'),
  Array.from({ length: 8 }, (_, i) => `D${7 - i}`),
);
// Per LED (status, address, data): how often it was on in this frame's
// samples, and how bright it glows.
const leds = [...statusLeds, ...addressLeds, ...dataLeds];
const hits = new Float32Array(leds.length);
const glow = new Float32Array(leds.length);
const shown = new Float32Array(leds.length);
let samples = 0;

/** Notes which LEDs are on now. */
function sample(running = status === Status.RUNNING) {
  const waiting = status === Status.LINE || status === Status.KEY;
  const state = [running, waiting, queue.length > 0, status === Status.READY && !queue];
  state.forEach((on, i) => {
    if (on) hits[i] += 1;
  });
  const line = vtl.line();
  for (let bit = 0; bit < 16; bit++) if ((line >> (15 - bit)) & 1) hits[4 + bit] += 1;
  const value = vtl.value() & 0xff;
  for (let bit = 0; bit < 8; bit++) if ((value >> (7 - bit)) & 1) hits[20 + bit] += 1;
  samples += 1;
}

function lights() {
  if (samples === 0) sample();
  for (let i = 0; i < leds.length; i++) {
    glow[i] = Math.max(hits[i] / samples, glow[i] * AFTERGLOW);
    const b = glow[i] < 0.03 ? 0 : Math.round(glow[i] * 40) / 40;
    if (b !== shown[i]) leds[i].style.setProperty('--b', String(b));
    shown[i] = b;
    hits[i] = 0;
  }
  samples = 0;
}

// ---------------------------------------------------------------- keyboard

function focus() {
  keys.focus({ preventScroll: true });
}

function typeChar(ch) {
  const c = upper() ? ch.toUpperCase() : ch;
  const code = c.codePointAt(0);
  if (wantsKeys() && !typed && code < 256) {
    // A key for $: the machine takes it at once and the terminal echoes it.
    echo(c);
    key(code);
    return;
  }
  typed += c;
  placeCursor();
}

function onKey(event) {
  if (event.isComposing) return;
  const ctrl = event.ctrlKey || event.metaKey;
  if ((ctrl && event.key.toLowerCase() === 'c' && !window.getSelection()?.toString()) || event.key === 'Escape') {
    event.preventDefault();
    stop();
    return;
  }
  if (ctrl && event.key.toLowerCase() === 'l') {
    event.preventDefault();
    clearScreen();
    return;
  }
  if (ctrl || event.altKey) return;
  switch (event.key) {
    case 'Enter':
      event.preventDefault();
      if (wantsKeys() && !typed) {
        echo('\n');
        key(13);
      } else {
        const text = typed;
        typed = '';
        placeCursor();
        submit(text);
      }
      return;
    case 'Backspace':
      event.preventDefault();
      typed = typed.slice(0, -1);
      placeCursor();
      return;
    case 'ArrowUp':
    case 'ArrowDown':
      event.preventDefault();
      if (!typedLines.length) return;
      historyAt = Math.max(0, Math.min(typedLines.length, historyAt + (event.key === 'ArrowUp' ? -1 : 1)));
      typed = typedLines[historyAt] ?? '';
      placeCursor();
      return;
    default:
      if (event.key.length === 1) {
        event.preventDefault();
        typeChar(event.key);
      }
  }
}

function onInput() {
  // Keyboards that send no key codes (on phones) end up here.
  const text = keys.value;
  keys.value = '';
  for (const ch of text) {
    if (ch === '\n') onKey({ key: 'Enter', preventDefault() {} });
    else typeChar(ch);
  }
}

function onPaste(event) {
  const text = event.clipboardData?.getData('text') ?? '';
  if (!text) return;
  event.preventDefault();
  const lines = text.replace(/\r\n?/g, '\n').split('\n');
  const last = lines.pop();
  if (lines.length === 0) {
    for (const ch of last) typeChar(ch);
    return;
  }
  lines[0] = typed + lines[0];
  typed = '';
  pendingLines.push(...lines);
  for (const ch of last) typeChar(ch);
}

// ---------------------------------------------------------------- programs

function encodeProgram(text) {
  const bytes = new TextEncoder().encode(text);
  let binary = '';
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function decodeProgram(code) {
  const binary = atob(code.replace(/-/g, '+').replace(/_/g, '/'));
  return new TextDecoder().decode(Uint8Array.from(binary, (c) => c.charCodeAt(0)));
}

function bindPrograms() {
  const select = $('example');
  for (const { file, title } of examples) {
    const option = document.createElement('option');
    option.value = file;
    option.textContent = title;
    select.append(option);
  }
  select.addEventListener('change', async () => {
    const file = select.value;
    select.value = '';
    if (!file) return;
    try {
      const response = await fetch(`examples/${file}`);
      if (!response.ok) throw new Error(response.statusText);
      output(`LOAD ${file.replace(/\.vtl$/, '').toUpperCase()}\n`);
      loadProgram(await response.text());
      pendingLines.push('RUN');
    } catch (error) {
      output(`CANNOT LOAD ${file}: ${error.message}\nOK\n`);
    }
    focus();
  });
  $('open').addEventListener('click', () => $('file').click());
  $('file').addEventListener('change', async () => {
    const file = $('file').files?.[0];
    if (!file) return;
    output(`LOAD ${file.name.toUpperCase()}\n`);
    loadProgram(await file.text());
    $('file').value = '';
    focus();
  });
  $('download').addEventListener('click', () => {
    const blob = new Blob([vtl.listing()], { type: 'text/plain' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = 'program.vtl';
    link.click();
    URL.revokeObjectURL(link.href);
    focus();
  });
  $('share').addEventListener('click', async () => {
    const url = new URL(location.href);
    url.hash = `program=${encodeProgram(vtl.listing())}`;
    window.history.replaceState(null, '', url);
    try {
      await navigator.clipboard.writeText(url.href);
      output('LINK TO THIS PROGRAM COPIED\nOK\n');
    } catch {
      output('THE LINK TO THIS PROGRAM IS IN THE ADDRESS BAR\nOK\n');
    }
    focus();
  });
  $('help').addEventListener('click', () => $('help-dialog').showModal());
  $('help-dialog').addEventListener('close', focus);
}

function bindSwitches() {
  for (const button of document.querySelectorAll('.switch')) {
    button.addEventListener('click', () => {
      button.classList.add('flipped');
      setTimeout(() => button.classList.remove('flipped'), 180);
      switch (button.dataset.action) {
        case 'stop':
          stop();
          break;
        case 'run':
          if (status !== Status.READY) stop();
          submit('RUN');
          break;
        case 'list':
          if (status === Status.READY) submit('LIST');
          break;
        case 'clear':
          clearScreen();
          break;
        case 'reset':
          stop();
          vtl.reset(machineOptions());
          status = vtl.status();
          clearScreen();
          greeting();
          break;
      }
      focus();
    });
  }
}

// ---------------------------------------------------------------- start

async function start() {
  applySettings();
  bindSettings();
  placeCursor();
  vtl = await Vtl.load(fetch('vtl_web.wasm'));
  vtl.reset(machineOptions());
  greeting();
  const shared = /^#program=([\w-]+)$/.exec(location.hash);
  if (shared) {
    try {
      loadProgram(decodeProgram(shared[1]));
    } catch {
      output('THE LINK HOLDS NO PROGRAM\nOK\n');
    }
  }
  bindPrograms();
  bindSwitches();
  keys.addEventListener('keydown', onKey);
  keys.addEventListener('input', onInput);
  keys.addEventListener('paste', onPaste);
  $('terminal').addEventListener('click', () => {
    if (!window.getSelection()?.toString()) focus();
  });
  setInterval(() => {
    $('speed').textContent = `${Math.round(vtl.statements()).toLocaleString('en')} statements run`;
  }, 1000);
  focus();
  requestAnimationFrame(tick);
}

start().catch((error) => {
  print(`THE MACHINE DID NOT START: ${error.message}\n`);
});
