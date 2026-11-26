# vtl-rs in the browser

The web version runs the same machine as the command line program, compiled to WebAssembly, in a page without a framework or build step:

- `src/lib.rs` exports the machine as a handful of functions with numbers in and out, without a glue generator.
- `www/vtl.mjs` wraps them for JavaScript; `www/app.mjs` is the terminal, the front panel and the settings; `www/style.css` holds the themes as CSS variables; `www/fonts/` the fonts (SIL Open Font License, each with its license).
- `test/` tests the module through `vtl.mjs` in Node; `showcase.mjs` takes the pictures the README shows.

```
web/build.sh                          # the site into _site
python3 -m http.server -d _site       # and look at it on http://localhost:8000
node --test web/test/*.test.mjs       # the tests (after web/build.sh)
```

The workflow `pages.yml` builds the site on every pull request, tests it, takes the pictures and publishes it from `main`. The repository's Settings, Pages, has to name "GitHub Actions" as the source once.

The machine runs in slices of statements per animation frame, so the page stays responsive and a program can be stopped at any time (Ctrl-C, Esc or the STOP switch). Output can be held back to the speed of a serial line; the address lights show the line that runs, the data lights the low byte of the value stored last.
