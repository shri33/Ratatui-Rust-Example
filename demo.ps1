# Journal CLI Demo Script (PowerShell)
# This script demonstrates both interface patterns

Write-Host "🦀 Ratatui Advanced UI Demo" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This project showcases TWO different terminal interface patterns:" -ForegroundColor White
Write-Host ""

Write-Host "1. 📊 TUI-Based Interface (Rich Visual Widgets)" -ForegroundColor Green
Write-Host "   - Boxes, borders, charts, tables" -ForegroundColor Gray
Write-Host "   - Arrow key navigation" -ForegroundColor Gray
Write-Host "   - Multi-pane layouts" -ForegroundColor Gray
Write-Host ""

Write-Host "2. 🖥️  CLI-Style Interface (Sequential Prompts)" -ForegroundColor Yellow
Write-Host "   - Clean terminal output" -ForegroundColor Gray
Write-Host "   - Text-based prompts" -ForegroundColor Gray
Write-Host "   - Like 'npm create vue@latest'" -ForegroundColor Gray
Write-Host ""

Write-Host "Choose a demo to run:" -ForegroundColor White
Write-Host ""
Write-Host "TUI Examples:" -ForegroundColor Green
Write-Host "  [1] Main Menu (showcase all TUI features)" -ForegroundColor Gray
Write-Host "  [2] Interactive Form (Vue.js style)" -ForegroundColor Gray
Write-Host "  [3] Charts & Graphs" -ForegroundColor Gray
Write-Host "  [4] Advanced Table Navigation" -ForegroundColor Gray
Write-Host "  [5] Dashboard" -ForegroundColor Gray
Write-Host ""
Write-Host "CLI Examples:" -ForegroundColor Yellow
Write-Host "  [6] Journal CLI (basic sequential prompts)" -ForegroundColor Gray
Write-Host "  [7] Journal CLI Enhanced (with web service simulation)" -ForegroundColor Gray
Write-Host ""
Write-Host "  [q] Quit" -ForegroundColor Red
Write-Host ""

$choice = Read-Host "Enter your choice [1-7, q]"

switch ($choice) {
    "1" {
        Write-Host "🚀 Running Main Menu (TUI Interface)..." -ForegroundColor Cyan
        cargo run --bin main_menu
    }
    "2" {
        Write-Host "🚀 Running Interactive Form (TUI Interface)..." -ForegroundColor Cyan
        cargo run --bin interactive_form
    }
    "3" {
        Write-Host "🚀 Running Charts Demo (TUI Interface)..." -ForegroundColor Cyan
        cargo run --bin charts_demo
    }
    "4" {
        Write-Host "🚀 Running Interactive Table (TUI Interface)..." -ForegroundColor Cyan
        cargo run --bin interactive_table
    }
    "5" {
        Write-Host "🚀 Running Dashboard (TUI Interface)..." -ForegroundColor Cyan
        cargo run --bin dashboard
    }
    "6" {
        Write-Host "🚀 Running Journal CLI (CLI Interface)..." -ForegroundColor Cyan
        Write-Host "This demonstrates the sequential prompt style requested by Joseph." -ForegroundColor Yellow
        Write-Host "Example usage: authenticate login" -ForegroundColor Gray
        Write-Host ""
        cargo run --bin journal_cli authenticate login
    }
    "7" {
        Write-Host "🚀 Running Journal CLI Enhanced (CLI Interface)..." -ForegroundColor Cyan
        Write-Host "This includes web service simulation and enhanced file generation." -ForegroundColor Yellow
        Write-Host ""
        cargo run --bin journal_cli_enhanced authenticate login
    }
    "q" {
        Write-Host "👋 Goodbye!" -ForegroundColor Green
        exit 0
    }
    default {
        Write-Host "❌ Invalid choice. Please run the script again." -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "✨ Demo completed!" -ForegroundColor Green
Write-Host ""
Write-Host "📁 Generated files can be found in:" -ForegroundColor Cyan
Write-Host "   - ./campaigns/ (email campaign files)" -ForegroundColor Gray
Write-Host "   - ./logs/ (API call logs)" -ForegroundColor Gray
Write-Host ""
Write-Host "🔧 Both interface patterns are now implemented:" -ForegroundColor Cyan
Write-Host "   ✅ TUI-based (rich visual interface)" -ForegroundColor Green
Write-Host "   ✅ CLI-style (sequential prompts)" -ForegroundColor Green
