#!/bin/bash

# Journal CLI Demo Script
# This script demonstrates both interface patterns

echo "🦀 Ratatui Advanced UI Demo"
echo "================================"
echo ""
echo "This project showcases TWO different terminal interface patterns:"
echo ""

echo "1. 📊 TUI-Based Interface (Rich Visual Widgets)"
echo "   - Boxes, borders, charts, tables"
echo "   - Arrow key navigation"
echo "   - Multi-pane layouts"
echo ""

echo "2. 🖥️  CLI-Style Interface (Sequential Prompts)"
echo "   - Clean terminal output"
echo "   - Text-based prompts"
echo "   - Like 'npm create vue@latest'"
echo ""

echo "Choose a demo to run:"
echo ""
echo "TUI Examples:"
echo "  [1] Main Menu (showcase all TUI features)"
echo "  [2] Interactive Form (Vue.js style)"
echo "  [3] Charts & Graphs"
echo "  [4] Advanced Table Navigation"
echo "  [5] Dashboard"
echo ""
echo "CLI Examples:"
echo "  [6] Journal CLI (basic sequential prompts)"
echo "  [7] Journal CLI Enhanced (with web service simulation)"
echo ""
echo "  [q] Quit"
echo ""

read -p "Enter your choice [1-7, q]: " choice

case $choice in
    1)
        echo "🚀 Running Main Menu (TUI Interface)..."
        cargo run --bin main_menu
        ;;
    2)
        echo "🚀 Running Interactive Form (TUI Interface)..."
        cargo run --bin interactive_form
        ;;
    3)
        echo "🚀 Running Charts Demo (TUI Interface)..."
        cargo run --bin charts_demo
        ;;
    4)
        echo "🚀 Running Interactive Table (TUI Interface)..."
        cargo run --bin interactive_table
        ;;
    5)
        echo "🚀 Running Dashboard (TUI Interface)..."
        cargo run --bin dashboard
        ;;
    6)
        echo "🚀 Running Journal CLI (CLI Interface)..."
        echo "This demonstrates the sequential prompt style requested by Joseph."
        echo "Try: authenticate login"
        echo ""
        cargo run --bin journal_cli authenticate login
        ;;
    7)
        echo "🚀 Running Journal CLI Enhanced (CLI Interface)..."
        echo "This includes web service simulation and enhanced file generation."
        echo ""
        cargo run --bin journal_cli_enhanced authenticate login
        ;;
    q)
        echo "👋 Goodbye!"
        exit 0
        ;;
    *)
        echo "❌ Invalid choice. Please run the script again."
        exit 1
        ;;
esac

echo ""
echo "✨ Demo completed!"
echo ""
echo "📁 Generated files can be found in:"
echo "   - ./campaigns/ (email campaign files)"
echo "   - ./logs/ (API call logs)"
echo ""
echo "🔧 Both interface patterns are now implemented:"
echo "   ✅ TUI-based (rich visual interface)"
echo "   ✅ CLI-style (sequential prompts)"
