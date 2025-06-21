# Layer9 Image Optimization Example

This example demonstrates the comprehensive image optimization capabilities of Layer9, including:

## Features Demonstrated

1. **Responsive Images**
   - Automatic srcset generation
   - Proper sizes attribute usage
   - Art direction with Picture element

2. **Lazy Loading**
   - Intersection Observer-based lazy loading
   - Configurable root margin and threshold
   - Smooth fade-in transitions

3. **Image Formats**
   - Automatic WebP conversion
   - AVIF support (where available)
   - Format negotiation based on browser support

4. **Optimization Techniques**
   - On-the-fly resizing
   - Quality adjustment
   - Blur placeholders
   - Progressive enhancement

5. **Performance Features**
   - Service Worker caching
   - Preloading critical images
   - CDN-friendly URLs
   - Efficient cache headers

## Running the Example

1. Build the WASM module:
   ```bash
   wasm-pack build --target web --out-dir pkg
   ```

2. Start a local server (with image optimization support):
   ```bash
   # If using the Layer9 development server
   cargo run --bin layer9-server --features ssr
   
   # Or use any static file server for client-side only
   python -m http.server 8000
   ```

3. Open http://localhost:8000 in your browser

## Image Optimization Endpoints

The example uses Layer9's image optimization service at `/_layer9/image` with these parameters:

- `src`: Source image URL or path
- `w`: Target width
- `h`: Target height
- `f`: Output format (jpeg, png, webp, avif)
- `q`: Quality (1-100)
- `blur`: Blur radius for placeholders

Example: `/_layer9/image?src=/images/hero.jpg&w=1920&q=90&f=webp`

## Code Structure

- `src/lib.rs`: Main component implementations
  - `ImageGallery`: Main gallery component
  - `QualityDemo`: Interactive quality adjustment
  - Lazy loading setup and service worker registration

- `index.html`: Entry point with critical CSS and preloading

- `sw.js`: Service Worker for offline image caching

## Best Practices Shown

1. **Preload critical images** in the document head
2. **Use appropriate image formats** based on content type
3. **Implement proper lazy loading** for below-the-fold images
4. **Provide fallbacks** for unsupported formats
5. **Cache aggressively** with proper cache headers
6. **Use blur placeholders** for better perceived performance

## Customization

To adapt this example for your needs:

1. Update image paths in `src/lib.rs`
2. Adjust lazy loading configuration in `LazyLoadManager::init()`
3. Modify service worker caching strategies
4. Customize the optimization parameters
5. Add your own image processing transformations