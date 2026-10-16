# Checks every PowerShell script of the project before it reaches the PC
# (Coder 2, 16 October 2026). Runs on PowerShell 7 on Linux (the cloud) or
# on Windows: pwsh -NoProfile -File tools/net/Check-Scripts.ps1
# 1. Syntax: every scripts\*.ps1 and tools\net\*.ps1 must parse.
# 2. Names: PowerShell ignores case in variable names, so a script-level
#    variable named like one of common.ps1's shared names ($Root, $Work,
#    $Out, $ExeSuffix, $OnWindows, $CommitIdentity) overwrites it. This
#    broke the first wide-network run (a loop variable $out). Any use of
#    such a name without the script: prefix is reported.
# Exit code: the number of problems (0 = clean).
$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
# Names set by scripts\common.ps1 (every script) and by tools\net\net-common.ps1 (Coder 2's scripts add $script:Out).
$sharedNames = @("root", "work", "exesuffix", "onwindows", "commitidentity", "minfreegb", "fastchessversion", "fastchessurl", "bookname", "bulletrev")
$coder2Names = $sharedNames + @("out")
$problems = 0
foreach ($f in @(Get-ChildItem (Join-Path $repoRoot "scripts") -Filter *.ps1) + @(Get-ChildItem (Join-Path $repoRoot "tools/net") -Filter *.ps1)) {
    $tokens = $null; $errors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile($f.FullName, [ref]$tokens, [ref]$errors)
    if ($errors.Count -gt 0) {
        $problems += $errors.Count
        foreach ($e in $errors) { Write-Host "$($f.Name): syntax: $($e.Message) (line $($e.Extent.StartLineNumber))" -ForegroundColor Red }
    }
    if ($f.Name -in @("common.ps1", "net-common.ps1", "Check-Scripts.ps1")) { continue }   # they define the names, or use none
    $reserved = if ($f.DirectoryName -like "*tools*") { $coder2Names } else { $sharedNames }
    foreach ($t in $tokens) {
        if ($t.Kind -ne "Variable") { continue }
        $name = $t.VariablePath.UserPath
        if ($name -match '^script:') { continue }
        if ($reserved -contains $name.ToLowerInvariant()) {
            $problems++
            Write-Host "$($f.Name): line $($t.Extent.StartLineNumber): `$$name clashes with `$script:$name (PowerShell ignores case)" -ForegroundColor Red
        }
    }
}
if ($problems -eq 0) { Write-Host "All scripts parse; no shared-name clashes." -ForegroundColor Green } else { Write-Host "$problems problem(s)." -ForegroundColor Red }
exit $problems
