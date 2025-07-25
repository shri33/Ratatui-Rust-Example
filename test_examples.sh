#!/bin/bash
# test_examples.sh

echo "Testing all examples..."

examples=("animation_ascii" "ascii_art" "dashboard" "emoji_picker" "hyperlinks" "image_viewer" "table_example" "text_input" "video_player")

for example in "${examples[@]}"; do
    echo "Testing $example..."
    cargo check --example $example
    if [ $? -eq 0 ]; then
        echo "✅ $example - OK"
    else
        echo "❌ $example - ERROR"
    fi
done

echo "Done testing examples."