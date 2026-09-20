#!/usr/bin/env python3
"""
UniversCraft HoloEngine — Local Web Demo Server
Serves the interactive WebGPU demo and WGSL shader files.

Usage:
    python serve_demo.py
    # Then open http://localhost:8080 in Chrome/Edge (WebGPU required)
"""

import http.server
import socketserver
import os
import sys
import webbrowser
from pathlib import Path

PORT = 8080
BASE_DIR = Path(__file__).parent

class HoloEngineHandler(http.server.SimpleHTTPRequestHandler):
    """Custom handler that serves demo files and maps /shaders/ to assets/shaders/"""

    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(BASE_DIR), **kwargs)

    def translate_path(self, path):
        """Route /shaders/* to assets/shaders/* and / to public/demo/index.html"""
        # Strip query string
        path = path.split('?')[0].split('#')[0]

        if path == '/' or path == '':
            return str(BASE_DIR / 'public' / 'demo' / 'index.html')
        elif path.startswith('/shaders/'):
            shader_name = path[len('/shaders/'):]
            return str(BASE_DIR / 'assets' / 'shaders' / shader_name)
        elif path.startswith('/output/'):
            # Serve pre-rendered images from public/output/
            img_name = path[len('/output/'):]
            return str(BASE_DIR / 'public' / 'output' / img_name)
        elif path.startswith('/public/'):
            return str(BASE_DIR / path.lstrip('/'))
        elif path.startswith('/rendered/'):
            # Serve rendered images
            img_name = path[len('/rendered/'):]
            return str(BASE_DIR / 'public' / 'rendered' / img_name)
        else:
            # Try serving from public/demo/
            demo_path = BASE_DIR / 'public' / 'demo' / path.lstrip('/')
            if demo_path.exists():
                return str(demo_path)
            # Fall back to base directory
            return str(BASE_DIR / path.lstrip('/'))

    def end_headers(self):
        """Add CORS and caching headers"""
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Cache-Control', 'no-cache')
        # Required for WebGPU SharedArrayBuffer
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        super().end_headers()

    def guess_type(self, path):
        """Add WGSL MIME type"""
        if path.endswith('.wgsl'):
            return 'text/plain'
        return super().guess_type(path)

    def log_message(self, format, *args):
        """Colorized logging"""
        status = args[1] if len(args) > 1 else ''
        if '200' in str(status):
            color = '\033[92m'  # Green
        elif '404' in str(status):
            color = '\033[91m'  # Red
        else:
            color = '\033[93m'  # Yellow
        reset = '\033[0m'
        sys.stderr.write(f"{color}[HoloEngine] {format % args}{reset}\n")


def main():
    print("""
    ==============================================================
    |                                                              |
    |   UniversCraft HoloEngine -- Demo Server                     |
    |                                                              |
    |   Topological Game Engine - Zero-Copy GPU Pipeline            |
    |                                                              |
    ==============================================================
    """)

    # Verify required files exist
    demo_file = BASE_DIR / 'public' / 'demo' / 'index.html'
    shader_dir = BASE_DIR / 'assets' / 'shaders'

    if not demo_file.exists():
        print(f"  [ERROR] Demo file not found: {demo_file}")
        print(f"     Run the build first to generate the demo.")
        sys.exit(1)

    shader_count = len(list(shader_dir.glob('scene_*.wgsl')))
    print(f"  [OK] Demo file: {demo_file}")
    print(f"  [OK] WGSL Shaders: {shader_count} scene shaders found")
    print(f"  [>>] Server: http://localhost:{PORT}")
    print(f"  [i]  WebGPU required: Use Chrome 113+ or Edge 113+")
    print(f"  [x]  Press Ctrl+C to stop\n")

    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.ThreadingTCPServer(("", PORT), HoloEngineHandler) as httpd:
        httpd.daemon_threads = True
        # Auto-open browser
        try:
            webbrowser.open(f"http://localhost:{PORT}")
        except Exception:
            pass

        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n  Server stopped.")
            httpd.server_close()


if __name__ == '__main__':
    main()
