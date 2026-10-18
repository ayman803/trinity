# Harmless test job for the runner: reports who it runs as, what it can
# see, and whether the tools and the graphics card work for this account.
# Writes nothing outside the runner's own folders.

$ErrorActionPreference = "Continue"
Write-Host "HELLO from $env:USERNAME on $env:COMPUTERNAME at $(Get-Date -Format 'yyyy-MM-dd HH:mm')"
Write-Host "Working folder: $(Get-Location)"
$drive = [IO.DriveInfo]::new([IO.Path]::GetPathRoot((Get-Location).Path))
Write-Host "Free disk: $([Math]::Floor($drive.AvailableFreeSpace / 1GB)) GB"
foreach ($tool in @("git", "cargo", "rustc")) {
    $cmd = Get-Command $tool -ErrorAction SilentlyContinue
    if ($cmd) { Write-Host "$tool : $((& $tool --version 2>&1) -join ' ')" } else { Write-Host "$tool : NOT FOUND" }
}
$smi = Get-Command nvidia-smi -ErrorAction SilentlyContinue
if ($smi) { Write-Host "GPU: $((& nvidia-smi -L 2>&1) -join ' ')" } else { Write-Host "GPU: nvidia-smi not found" }
Write-Host "CUDA_PATH: $env:CUDA_PATH"
# The runner account must NOT be able to write to Ayman's folder, including
# subfolders that keep their own permissions.
$writable = 0
foreach ($dir in @("C:\Trinity", "C:\Trinity\sprt", "C:\Trinity\sprt\opponents", "C:\Trinity\sprt\results", "C:\Trinity\data")) {
    if (-not (Test-Path -LiteralPath $dir)) { continue }
    $probe = Join-Path $dir "runner-write-test.txt"
    try {
        Set-Content -LiteralPath $probe -Value "test" -ErrorAction Stop
        Remove-Item -LiteralPath $probe -ErrorAction SilentlyContinue
        Write-Host "WRITE CHECK: $dir is WRITABLE for this account (should not be!)"
        $writable++
    } catch {
        Write-Host "WRITE CHECK: $dir is read-only for this account (good)"
    }
}
if ($writable -gt 0) { exit 2 }
foreach ($l in @("data", "checkpoints", "syzygy")) {
    Write-Host "link $l : $(if (Test-Path $l) { 'readable' } else { 'missing' })"
}
exit 0
