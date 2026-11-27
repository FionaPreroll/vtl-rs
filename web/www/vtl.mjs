// The VTL-2 machine of vtl-rs as WebAssembly (web/src/lib.rs), wrapped for
// JavaScript. Works in the browser and in Node (web/test).

/** What the machine waits for, as `run` and `enter` return it. */
export const Status = Object.freeze({ READY: 0, RUNNING: 1, LINE: 2, KEY: 3 });

export class Vtl {
  /**
   * @param {WebAssembly.Instance} instance
   * @param {WebAssembly.Module} [module] to start the machine anew after it crashed
   */
  constructor(instance, module) {
    this.module = module;
    this.x = instance.exports;
    this.encoder = new TextEncoder();
    this.decoder = new TextDecoder();
    this.options = {};
    /** Set when the module trapped and was started anew; the page reports it. */
    this.crashed = false;
    this.crashes = 0;
  }

  /** Loads the module from bytes, a Response or a promise of one. */
  static async load(source) {
    const resolved = await source;
    const { instance, module } =
      typeof Response !== 'undefined' && resolved instanceof Response
        ? await WebAssembly.instantiateStreaming(resolved, {}).catch(async () =>
            WebAssembly.instantiate(await resolved.clone().arrayBuffer(), {}),
          )
        : await WebAssembly.instantiate(resolved, {});
    return new Vtl(instance, module);
  }

  /**
   * Calls the module. A trap (a bug, or the stack running out) leaves it
   * unusable, so the machine is started anew with an empty program and
   * `crashed` set, and the call answers READY: the page goes on working.
   */
  call(name, ...args) {
    try {
      return this.x[name](...args);
    } catch (error) {
      if (!(error instanceof WebAssembly.RuntimeError) || !this.module) throw error;
      this.x = new WebAssembly.Instance(this.module, {}).exports;
      this.reset(this.options);
      this.crashed = true;
      this.crashes += 1;
      return 0;
    }
  }

  /**
   * A new machine with an empty program.
   * @param {{memory?: number, strict?: boolean, precedence?: boolean, seed?: number}} options
   */
  reset(options = {}) {
    const { memory = 32768, strict = false, precedence = false, seed } = options;
    this.options = options;
    const flags = (strict ? 1 : 0) | (precedence ? 2 : 0);
    const s = seed ?? Math.floor(Math.random() * 0x1_0000_0000);
    this.x.vtl_init(memory, flags, s >>> 0);
    this.decoder = new TextDecoder();
  }

  /** Enters a line: a program line, a statement, a command or input. */
  enter(line) {
    const bytes = this.encoder.encode(line);
    const crashes = this.crashes;
    const ptr = this.call('vtl_buffer', bytes.length);
    if (this.crashes !== crashes) return 0;
    new Uint8Array(this.x.memory.buffer, ptr, bytes.length).set(bytes);
    return this.call('vtl_enter', bytes.length);
  }

  /** A key press for `$` (13 for Return). */
  key(code) {
    return this.call('vtl_key', code);
  }

  /** Runs up to `budget` statements. */
  run(budget) {
    return this.call('vtl_run', budget);
  }

  status() {
    return this.call('vtl_status');
  }

  stop() {
    this.call('vtl_stop');
  }

  newProgram() {
    this.call('vtl_new');
  }

  /** How many bytes the machine printed and `output` did not take yet. */
  outputLength() {
    return this.call('vtl_output_len');
  }

  /** What the machine printed since the last call. */
  output() {
    const len = this.call('vtl_output_len');
    if (len === 0) return '';
    const bytes = new Uint8Array(this.x.memory.buffer, this.x.vtl_output(), len);
    const text = this.decoder.decode(bytes, { stream: true });
    this.x.vtl_output_clear();
    return text;
  }

  /** The program as text, as `0` lists it. */
  listing() {
    const len = this.call('vtl_listing');
    const bytes = new Uint8Array(this.x.memory.buffer, this.x.vtl_text(), len);
    return new TextDecoder().decode(bytes);
  }

  /** The line that runs (0 if none), for the address lights. */
  line() {
    return this.call('vtl_line');
  }

  /** The value stored last, for the data lights. */
  value() {
    return this.call('vtl_value');
  }

  free() {
    return this.call('vtl_free');
  }

  statements() {
    return this.call('vtl_statements');
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
