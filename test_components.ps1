#!/usr/bin/env pwsh
# Test script to verify all components are working

Write-Host "🧪 Testing Ratatui Components" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

Write-Host ""
Write-Host "Building all components..." -ForegroundColor Yellow

# Test CLI components
Write-Host "1. Testing Journal CLI..." -ForegroundColor Green
try {
    cargo build --bin journal_cli 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Journal CLI builds successfully" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Journal CLI build failed" -ForegroundColor Red
    }
} catch {
    Write-Host "   ❌ Journal CLI build error: $_" -ForegroundColor Red
}

# Test Enhanced CLI
Write-Host "2. Testing Enhanced Journal CLI..." -ForegroundColor Green
try {
    cargo build --bin journal_cli_enhanced 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Enhanced Journal CLI builds successfully" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Enhanced Journal CLI build failed" -ForegroundColor Red
    }
} catch {
    Write-Host "   ❌ Enhanced Journal CLI build error: $_" -ForegroundColor Red
}

# Test Interactive Form
Write-Host "3. Testing Interactive Form..." -ForegroundColor Green
try {
    cargo build --bin interactive_form 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Interactive Form builds successfully" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Interactive Form build failed" -ForegroundColor Red
    }
} catch {
    Write-Host "   ❌ Interactive Form build error: $_" -ForegroundColor Red
}

# Test Interactive Table
Write-Host "4. Testing Interactive Table..." -ForegroundColor Green
try {
    cargo build --bin interactive_table 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Interactive Table builds successfully" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Interactive Table build failed" -ForegroundColor Red
    }
} catch {
    Write-Host "   ❌ Interactive Table build error: $_" -ForegroundColor Red
}

# Test Charts Demo
Write-Host "5. Testing Charts Demo..." -ForegroundColor Green
try {
    cargo build --bin charts_demo 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Charts Demo builds successfully" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Charts Demo build failed" -ForegroundColor Red
    }
} catch {
    Write-Host "   ❌ Charts Demo build error: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "🎯 Available Commands:" -ForegroundColor Magenta
Write-Host "   cargo run --bin journal_cli authenticate login" -ForegroundColor White
Write-Host "   cargo run --bin journal_cli_enhanced authenticate login" -ForegroundColor White
Write-Host "   cargo run --bin interactive_form" -ForegroundColor White
Write-Host "   cargo run --bin interactive_table" -ForegroundColor White
Write-Host "   cargo run --bin charts_demo" -ForegroundColor White

Write-Host ""
Write-Host "If any component shows build errors, please share the specific error message." -ForegroundColor Yellow
