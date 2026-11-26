// The VTL-2 machine of vtl-rs as WebAssembly (web/src/lib.rs), wrapped for
// JavaScript. Works in the browser and in Node (web/test).

/** What the machine waits for, as `run` and `enter` return it. */
export const Status = Object.freeze({ READY: 0, RUNNING: 1, LINE: 2, KEY: 3 });

export class Vtl {
  /** @param {WebAssembly.Instance} instance */
  constructor(instance) {
    this.x = instance.exports;
    this.encoder = new TextEncoder();
    this.decoder = new TextDecoder();
  }

  /** Loads the module from bytes, a Response or a promise of one. */
  static async load(source) {
    const resolved = await source;
    const { instance } =
      typeof Response !== 'undefined' && resolved instanceof Response
        ? await WebAssembly.instantiateStreaming(resolved, {}).catch(async () =>
            WebAssembly.instantiate(await resolved.clone().arrayBuffer(), {}),
          )
        : await WebAssembly.instantiate(resolved, {});
    return new Vtl(instance);
  }

  /**
   * A new machine with an empty program.
   * @param {{memory?: number, strict?: boolean, precedence?: boolean, seed?: number}} options
   */
  reset({ memory = 32768, strict = false, precedence = false, seed } = {}) {
    const flags = (strict ? 1 : 0) | (precedence ? 2 : 0);
    const s = seed ?? Math.floor(Math.random() * 0x1_0000_0000);
    this.x.vtl_init(memory, flags, s >>> 0);
    this.decoder = new TextDecoder();
  }

  /** Enters a line: a program line, a statement, a command or input. */
  enter(line) {
    const bytes = this.encoder.encode(line);
    const ptr = this.x.vtl_buffer(bytes.length);
    new Uint8Array(this.x.memory.buffer, ptr, bytes.length).set(bytes);
    return this.x.vtl_enter(bytes.length);
  }

  /** A key press for `$` (13 for Return). */
  key(code) {
    return this.x.vtl_key(code);
  }

  /** Runs up to `budget` statements. */
  run(budget) {
    return this.x.vtl_run(budget);
  }

  status() {
    return this.x.vtl_status();
  }

  stop() {
    this.x.vtl_stop();
  }

  newProgram() {
    this.x.vtl_new();
  }

  /** What the machine printed since the last call. */
  output() {
    const len = this.x.vtl_output_len();
    if (len === 0) return '';
    const bytes = new Uint8Array(this.x.memory.buffer, this.x.vtl_output(), len);
    const text = this.decoder.decode(bytes, { stream: true });
    this.x.vtl_output_clear();
    return text;
  }

  /** The program as text, as `0` lists it. */
  listing() {
    const len = this.x.vtl_listing();
    const bytes = new Uint8Array(this.x.memory.buffer, this.x.vtl_text(), len);
    return new TextDecoder().decode(bytes);
  }

  /** The line that runs (0 if none), for the address lights. */
  line() {
    return this.x.vtl_line();
  }

  /** The value stored last, for the data lights. */
  value() {
    return this.x.vtl_value();
  }

  free() {
    return this.x.vtl_free();
  }

  statements() {
    return this.x.vtl_statements();
  }

  /**
   * Enters the lines of a program file as the command line program does:
   * numbered lines are stored, comments (`)` or `#` without `=`) skipped,
   * and the other lines returned, to be run in order by the caller.
   */
  load(text) {
    const direct = [];
    for (const raw of text.split(/\r?\n/)) {
      const line = raw.trim();
      if (line === '' || isComment(line)) continue;
      if (/^\d/.test(line)) this.enter(line);
      else direct.push(line);
    }
    return direct;
  }
}

/** Whether a line of a file is a comment: `)` or `#` that is no `#=`. */
export function isComment(line) {
  return line.startsWith(')') || (line.startsWith('#') && !/^#\s*=/.test(line));
}
