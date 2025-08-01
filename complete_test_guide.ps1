#!/usr/bin/env pwsh

Write-Host "🎯 INTERACTIVE FORM TROUBLESHOOTING GUIDE" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""

Write-Host "🚀 TWO VERSIONS TO TEST:" -ForegroundColor Green
Write-Host ""

Write-Host "1. 🧪 SIMPLE DEBUG VERSION (Start here):" -ForegroundColor Yellow
Write-Host "   cargo run --bin simple_form_test" -ForegroundColor White
Write-Host "   • Minimal features, easier to debug" -ForegroundColor Gray
Write-Host "   • Shows debug info at bottom" -ForegroundColor Gray
Write-Host "   • Clear visual feedback" -ForegroundColor Gray
Write-Host ""

Write-Host "2. 🎨 FULL FEATURED VERSION:" -ForegroundColor Yellow  
Write-Host "   cargo run --bin interactive_form" -ForegroundColor White
Write-Host "   • All features: validation, charts, history" -ForegroundColor Gray
Write-Host "   • More complex but prettier" -ForegroundColor Gray
Write-Host ""

Write-Host "📋 STEP-BY-STEP TEST PROCEDURE:" -ForegroundColor Magenta
Write-Host ""
Write-Host "Step 1: Check Navigation" -ForegroundColor Green
Write-Host "  • Press Tab - should move between Name → Email → Selection → Name" -ForegroundColor White
Write-Host "  • Press Up/Down arrows - should also move between fields" -ForegroundColor White
Write-Host "  • Watch the field borders change color" -ForegroundColor White
Write-Host ""

Write-Host "Step 2: Test Text Input" -ForegroundColor Green
Write-Host "  • Navigate to Name field" -ForegroundColor White
Write-Host "  • Press Enter (should see 'Editing Name' in title)" -ForegroundColor White
Write-Host "  • Type characters - should appear in the field" -ForegroundColor White
Write-Host "  • Press Enter or Esc to stop editing" -ForegroundColor White
Write-Host ""

Write-Host "Step 3: Test Email Input" -ForegroundColor Green
Write-Host "  • Navigate to Email field" -ForegroundColor White
Write-Host "  • Press Enter to start editing" -ForegroundColor White
Write-Host "  • Type an email address" -ForegroundColor White
Write-Host "  • Press Enter or Esc to stop editing" -ForegroundColor White
Write-Host ""

Write-Host "Step 4: Test Selection" -ForegroundColor Green
Write-Host "  • Navigate to Selection field" -ForegroundColor White
Write-Host "  • Use Left/Right arrows to change Yes/No/Maybe/Other" -ForegroundColor White
Write-Host "  • Should see highlighted option change" -ForegroundColor White
Write-Host ""

Write-Host "Step 5: Exit" -ForegroundColor Green
Write-Host "  • Press Q or Esc to quit" -ForegroundColor White
Write-Host ""

Write-Host "❗ IF KEYS DON'T WORK:" -ForegroundColor Red
Write-Host "  • Try different terminal (Windows Terminal, PowerShell, CMD)" -ForegroundColor White
Write-Host "  • Check if your terminal supports raw mode" -ForegroundColor White
Write-Host "  • Report which keys work and which don't" -ForegroundColor White
Write-Host ""

Write-Host "✨ WHAT SHOULD HAPPEN:" -ForegroundColor Cyan
Write-Host "  • Borders change color when field is selected" -ForegroundColor White
Write-Host "  • Title shows current mode and field" -ForegroundColor White
Write-Host "  • Characters appear when typing" -ForegroundColor White
Write-Host "  • Cursor visible in editing mode" -ForegroundColor White
Write-Host ""

Write-Host "🎮 Start testing: cargo run --bin simple_form_test" -ForegroundColor Green
