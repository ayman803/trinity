# Download the 3-4-5 piece Syzygy endgame tablebases (about 1 GB) from the
# Lichess tablebase server into C:\Trinity\syzygy. Files already present
# are skipped, so it can be run again after an interruption.
# Double-click 9-Get-Tablebases.bat.

param(
    [string]$Source = "https://tablebase.lichess.ovh/tables/standard/3-4-5/"
)

. (Join-Path $PSScriptRoot "common.ps1")

try {
    Show-DiskSpace
    $dest = Join-Path $script:Root "syzygy"
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    Write-Step "Reading the list of tablebase files from $Source"
    $page = Invoke-WebRequest -Uri $Source -UseBasicParsing
    $names = @([regex]::Matches($page.Content, 'href="([KQRBNPvkqrbnp]+\.rtb[wz])"') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)
    if ($names.Count -lt 100) { Stop-WithMessage "Expected about 290 files on $Source but found $($names.Count)." }
    Write-Host "$($names.Count) files listed."
    $i = 0
    foreach ($n in $names) {
        $i++
        $target = Join-Path $dest $n
        if (Test-Path -LiteralPath $target) { continue }
        Write-Host "[$i/$($names.Count)] $n"
        $tmp = "$target.part"
        Save-Download ($Source + $n) $tmp
        Move-Item -LiteralPath $tmp -Destination $target -Force
    }
    $count = @(Get-ChildItem -LiteralPath $dest -Filter "*.rtb?").Count
    $sizeMB = [Math]::Round(((Get-ChildItem -LiteralPath $dest -Filter "*.rtb?" | Measure-Object Length -Sum).Sum) / 1MB)
    Write-Host ""
    Write-Good "Done: $count tablebase files ($sizeMB MB) in $dest"
    Write-Host "Calibration (5-Calibrate.bat) now gives these tables to every engine."
} catch {
    Write-UnexpectedError $_
    exit 1
}
