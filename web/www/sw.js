// The service worker that makes the page an app that works offline: every
// file of the page comes from the network while there is one, and from the
// cache when there is none. web/build.sh writes the build into VERSION, so
// that each build has its own cache and drops the older ones.

const VERSION = 'dev';
const CACHE = `vtl-rs-${VERSION}`;
const SHELL = [
  './',
  'index.html',
  'style.css',
  'app.mjs',
  'vtl.mjs',
  'examples.mjs',
  'vtl_web.wasm',
  'manifest.webmanifest',
  'favicon.svg',
  'icons/icon-192.png',
  'icons/icon-512.png',
  'fonts/VT323-Regular.woff2',
  'fonts/CourierPrime-Regular.woff2',
  'fonts/ShareTechMono-Regular.woff2',
  'fonts/IBMPlexMono-Regular.woff2',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => cache.addAll(SHELL))
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((key) => key.startsWith('vtl-rs-') && key !== CACHE).map((key) => caches.delete(key))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);
  const scope = new URL(self.registration.scope);
  // Only the page's own files; the Rust documentation and the pictures stay
  // online only.
  if (request.method !== 'GET' || url.origin !== scope.origin || !url.pathname.startsWith(scope.pathname)) return;
  const path = url.pathname.slice(scope.pathname.length);
  if (path.startsWith('api/') || path.startsWith('screenshots/')) return;
  event.respondWith(
    fetch(request)
      .then((response) => {
        if (response.ok && response.type === 'basic') {
          const copy = response.clone();
          event.waitUntil(caches.open(CACHE).then((cache) => cache.put(request, copy)));
        }
        return response;
      })
      .catch(async () => (await caches.match(request, { ignoreSearch: true })) ?? Response.error()),
  );
});
