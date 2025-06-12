#!/usr/bin/env python3
"""
Simple HTTP server for serving the Layer9 Todo App
"""

import http.server
import socketserver
import os
import sys

# Configuration
PORT = 8082
DIRECTORY = ".."  # Serve from the parent directory (todo-app)

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)
    
    def end_headers(self):
        # Add CORS headers for WASM
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()
    
    def guess_type(self, path):
        mimetype = super().guess_type(path)
        # Ensure .wasm files are served with correct MIME type
        if path.endswith('.wasm'):
            return 'application/wasm'
        return mimetype

def main():
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    
    with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
        print(f"🚀 Server running at http://localhost:{PORT}")
        print(f"📁 Serving directory: {os.path.abspath(DIRECTORY)}")
        print("Press Ctrl+C to stop...")
        
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n👋 Server stopped.")
            sys.exit(0)

if __name__ == "__main__":
    main()