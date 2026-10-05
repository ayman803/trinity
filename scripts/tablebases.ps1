# Download the 3-4-5 piece Syzygy endgame tablebases (about 1 GB) into
# C:\Trinity\syzygy. Files already present are skipped, so it can be run
# again after an interruption. Double-click 9-Get-Tablebases.bat.
#
# Several known download locations are tried in turn; the first set that
# lists the files is used.

param(
    [string[]]$Source = @()
)

$candidates = if ($Source.Count -gt 0) { , $Source } else { @(
    , @("https://tablebase.lichess.ovh/tables/standard/3-4-5-wdl/",
        "https://tablebase.lichess.ovh/tables/standard/3-4-5-dtz/")
    , @("http://tablebase.sesse.net/syzygy/3-4-5/")
    , @("https://tablebase.lichess.ovh/tables/standard/3-4-5/")
) }

# File name -> full download address, from the folder listings in $Folders.
function Get-TableList([string[]]$Folders) {
    $list = @{}
    foreach ($f in $Folders) {
        Write-Host "Trying $f"
        try {
            $page = Invoke-WebRequest -Uri $f -UseBasicParsing
        } catch {
            Write-Warn "  not available ($($_.Exception.Message))"
            return @{}
        }
        foreach ($m in [regex]::Matches($page.Content, 'href="(?:[^"]*/)?([KQRBNPvkqrbnp]+\.rtb[wz])"')) {
            $list[$m.Groups[1].Value] = $f + $m.Groups[1].Value
        }
    }
    return $list
}

. (Join-Path $PSScriptRoot "common.ps1")

try {
    Show-DiskSpace
    $dest = Join-Path $script:Root "syzygy"
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    Write-Step "Reading the list of tablebase files"
    $urls = @{}
    foreach ($c in $candidates) {
        $urls = Get-TableList $c
        if ($urls.Count -ge 250) { break }
        if ($urls.Count -gt 0) { Write-Warn "  only $($urls.Count) files listed there; trying the next location" }
    }
    if ($urls.Count -lt 250) { Stop-WithMessage "No download location listed the 290 tablebase files." }
    $names = @($urls.Keys | Sort-Object)
    Write-Host "$($names.Count) files listed."
    $i = 0
    foreach ($n in $names) {
        $i++
        $target = Join-Path $dest $n
        if (Test-Path -LiteralPath $target) { continue }
        Write-Host "[$i/$($names.Count)] $n"
        $tmp = "$target.part"
        Save-Download $urls[$n] $tmp
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
