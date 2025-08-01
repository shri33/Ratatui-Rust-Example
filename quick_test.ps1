#!/usr/bin/env pwsh

Write-Host "🚀 Quick Component Test" -ForegroundColor Cyan
Write-Host ""

# Test CLI first (safest)
Write-Host "Testing Journal CLI (will exit automatically after usage display)..." -ForegroundColor Green
cargo run --bin journal_cli

Write-Host ""
Write-Host "✅ If you see 'Usage: cargo run --bin journal_cli authenticate login' above, the CLI is working!" -ForegroundColor Green
Write-Host ""
Write-Host "To test the full flow, run:" -ForegroundColor Yellow
Write-Host "  cargo run --bin journal_cli authenticate login" -ForegroundColor White
Write-Host ""
Write-Host "Other working components:" -ForegroundColor Cyan
Write-Host "  cargo run --bin interactive_form" -ForegroundColor White
Write-Host "  cargo run --bin interactive_table" -ForegroundColor White  
Write-Host "  cargo run --bin charts_demo" -ForegroundColor White
