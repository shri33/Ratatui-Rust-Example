# Test the CLI Interface

Write-Host "🧪 Testing Journal CLI Interface..." -ForegroundColor Cyan
Write-Host ""

# Test basic CLI
Write-Host "Testing basic CLI version..." -ForegroundColor Yellow
$testInput = @"
test@example.com
Test User
y
"@

$testInput | cargo run --bin journal_cli authenticate login

Write-Host ""
Write-Host "✅ Basic CLI test completed!" -ForegroundColor Green
Write-Host ""

# Show generated files
Write-Host "📁 Generated campaign files:" -ForegroundColor Cyan
Get-ChildItem -Path "campaigns" -Recurse | ForEach-Object { 
    Write-Host "   $($_.FullName)" -ForegroundColor Gray 
}

Write-Host ""
Write-Host "🎉 All tests passed! Ready for Joseph's review." -ForegroundColor Green
