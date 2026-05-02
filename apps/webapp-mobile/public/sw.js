const CACHE_NAME = 'novatuner-app-shell-v2'
const CORE_URLS = [
  '/',
  '/index.html',
  '/manifest.webmanifest',
  '/favicon.svg',
  '/icons/favicon-16.png',
  '/icons/favicon-32.png',
  '/icons/apple-touch-icon.png',
  '/icons/icon-192.png',
  '/icons/icon-512.png',
  '/icons/icon-maskable-192.png',
  '/icons/icon-maskable-512.png',
]

function toSameOriginPath(rawUrl) {
  const url = new URL(rawUrl, self.location.origin)
  if (url.origin !== self.location.origin) {
    return null
  }
  return `${url.pathname}${url.search}`
}

function extractPathsFromText(text, pattern) {
  const matches = new Set()
  for (const match of text.matchAll(pattern)) {
    const path = toSameOriginPath(match[1] ?? match[0])
    if (path) {
      matches.add(path)
    }
  }
  return matches
}

async function discoverPrecacheUrls() {
  const discovered = new Set(CORE_URLS)

  const indexResponse = await fetch('/index.html', { cache: 'no-store' })
  if (indexResponse.ok) {
    const indexHtml = await indexResponse.text()
    for (const path of extractPathsFromText(
      indexHtml,
      /(?:href|src)=["']([^"'#?]+\.(?:js|css|png|svg|webmanifest))["']/g,
    )) {
      discovered.add(path)
    }
  }

  const manifestResponse = await fetch('/manifest.webmanifest', { cache: 'no-store' })
  if (manifestResponse.ok) {
    const manifest = await manifestResponse.json()
    for (const icon of manifest.icons ?? []) {
      const path = toSameOriginPath(icon.src)
      if (path) {
        discovered.add(path)
      }
    }
  }

  const scriptAssets = [...discovered].filter((path) => path.endsWith('.js'))
  for (const assetPath of scriptAssets) {
    const assetResponse = await fetch(assetPath, { cache: 'no-store' })
    if (!assetResponse.ok) {
      continue
    }

    const source = await assetResponse.text()
    for (const path of extractPathsFromText(
      source,
      /\/assets\/[^"'`\s)]+\.(?:js|css|wasm|png|svg|json)/g,
    )) {
      discovered.add(path)
    }
    for (const path of extractPathsFromText(
      source,
      /\/icons\/[^"'`\s)]+\.(?:png|svg)/g,
    )) {
      discovered.add(path)
    }
  }

  return [...discovered]
}

async function precacheShell() {
  const cache = await caches.open(CACHE_NAME)
  const urls = await discoverPrecacheUrls()

  await Promise.all(
    urls.map(async (url) => {
      try {
        const response = await fetch(url, { cache: 'no-store' })
        if (response.ok) {
          await cache.put(url, response)
        }
      } catch {
        // Ignore individual precache failures and keep the rest of the shell installable.
      }
    }),
  )
}

async function cacheFirst(request) {
  const cache = await caches.open(CACHE_NAME)
  const cached = await cache.match(request)
  if (cached) {
    return cached
  }

  const response = await fetch(request)
  if (response.ok) {
    await cache.put(request, response.clone())
  }
  return response
}

async function networkFirst(request) {
  const cache = await caches.open(CACHE_NAME)
  try {
    const response = await fetch(request)
    if (response.ok) {
      await cache.put(request, response.clone())
    }
    return response
  } catch {
    const cached = await cache.match(request)
    if (cached) {
      return cached
    }

    return (await cache.match('/index.html')) ?? (await cache.match('/'))
  }
}

self.addEventListener('install', (event) => {
  event.waitUntil(precacheShell())
  self.skipWaiting()
})

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      const keys = await caches.keys()
      await Promise.all(keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key)))
      await self.clients.claim()
    })(),
  )
})

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') {
    return
  }

  const url = new URL(event.request.url)
  if (url.origin !== self.location.origin) {
    return
  }

  if (event.request.mode === 'navigate') {
    event.respondWith(networkFirst(event.request))
    return
  }

  const isStaticAsset =
    ['script', 'style', 'image', 'font', 'worker'].includes(event.request.destination) ||
    url.pathname.endsWith('.wasm') ||
    url.pathname.endsWith('.webmanifest')

  if (isStaticAsset) {
    event.respondWith(cacheFirst(event.request))
  }
})
