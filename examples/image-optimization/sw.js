// Service Worker for image caching
const CACHE_NAME = 'layer9-images-v1';
const IMAGE_CACHE_NAME = 'layer9-optimized-images-v1';

// Cache static assets on install
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll([
                '/',
                '/index.html',
                '/pkg/image_optimization_example.js',
                '/pkg/image_optimization_example_bg.wasm',
            ]);
        })
    );
});

// Clean up old caches on activate
self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys().then((cacheNames) => {
            return Promise.all(
                cacheNames
                    .filter((name) => name !== CACHE_NAME && name !== IMAGE_CACHE_NAME)
                    .map((name) => caches.delete(name))
            );
        })
    );
});

// Intercept fetch requests
self.addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);
    
    // Handle image optimization requests
    if (url.pathname.startsWith('/_layer9/image')) {
        event.respondWith(handleOptimizedImage(event.request));
        return;
    }
    
    // Handle regular image requests
    if (isImageRequest(event.request)) {
        event.respondWith(handleImage(event.request));
        return;
    }
    
    // Handle other requests with network-first strategy
    event.respondWith(
        fetch(event.request).catch(() => {
            return caches.match(event.request);
        })
    );
});

// Check if request is for an image
function isImageRequest(request) {
    const url = new URL(request.url);
    const imageExtensions = ['.jpg', '.jpeg', '.png', '.webp', '.avif', '.gif', '.svg'];
    return imageExtensions.some(ext => url.pathname.endsWith(ext));
}

// Handle optimized image requests with caching
async function handleOptimizedImage(request) {
    const cache = await caches.open(IMAGE_CACHE_NAME);
    
    // Try cache first
    const cachedResponse = await cache.match(request);
    if (cachedResponse) {
        return cachedResponse;
    }
    
    // Fetch from network
    try {
        const networkResponse = await fetch(request);
        
        // Cache successful responses
        if (networkResponse.ok) {
            cache.put(request, networkResponse.clone());
        }
        
        return networkResponse;
    } catch (error) {
        // Return placeholder image on error
        return new Response(
            '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="#f0f0f0"/><text x="50" y="50" text-anchor="middle" dy=".3em" fill="#999">Image unavailable</text></svg>',
            {
                headers: {
                    'Content-Type': 'image/svg+xml',
                    'Cache-Control': 'no-cache'
                }
            }
        );
    }
}

// Handle regular image requests
async function handleImage(request) {
    const cache = await caches.open(IMAGE_CACHE_NAME);
    
    // Try cache first for images
    const cachedResponse = await cache.match(request);
    if (cachedResponse) {
        // Return cached version and update in background
        fetchAndCache(request, cache);
        return cachedResponse;
    }
    
    // Fetch from network
    return fetchAndCache(request, cache);
}

// Fetch and cache helper
async function fetchAndCache(request, cache) {
    try {
        const response = await fetch(request);
        
        if (response.ok) {
            cache.put(request, response.clone());
        }
        
        return response;
    } catch (error) {
        // Try to return from cache as fallback
        const cachedResponse = await cache.match(request);
        if (cachedResponse) {
            return cachedResponse;
        }
        
        // Return error response
        return new Response('Network error', {
            status: 408,
            headers: { 'Content-Type': 'text/plain' }
        });
    }
}