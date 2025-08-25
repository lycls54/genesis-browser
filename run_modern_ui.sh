#!/bin/bash

# Run Genesis Browser with modern UI
echo "🚀 Starting Genesis Browser with Modern UI"
echo "✨ 144 FPS egui + GPU-accelerated rendering"

# Build with modern UI feature enabled (it's default now)
cargo build --release --bin genesis-browser

# Run the browser
if [ $? -eq 0 ]; then
    echo "🎨 Launching Modern Genesis Browser..."
    ./target/release/genesis-browser
else
    echo "❌ Build failed"
fi