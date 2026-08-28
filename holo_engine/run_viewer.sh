#!/bin/bash
# Install xvfb if needed (usually installed)
# sudo apt-get install -y xvfb

# Run the bevy cinematic app in Xvfb and capture a screenshot if possible
echo "Building Bevy Cinematic Viewer..."
cargo build --release --bin bevy_cinematic --features full

echo "Running Bevy in headless/xvfb mode..."
export DISPLAY=:99
Xvfb :99 -screen 0 1280x720x24 &
XVFB_PID=$!
sleep 2

WGPU_BACKEND=vulkan cargo run --release --bin bevy_cinematic --features full &
APP_PID=$!

# Give it 10 seconds to render the first few frames
echo "Waiting for rendering to start..."
sleep 15

# Capture a screenshot
echo "Capturing screenshot..."
import -window root assets/screenshot_cinematic.png

echo "Done. Killing processes..."
kill $APP_PID
kill $XVFB_PID

echo "Screenshot saved to assets/screenshot_cinematic.png"
