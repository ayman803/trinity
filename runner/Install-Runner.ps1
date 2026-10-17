# Install (or reinstall) the Trinity job runner on this machine. Ayman runs
# it as administrator by double-clicking runner\Install-Runner.bat. It:
#  1. creates the standard account trinity-runner (password random, never
#     shown, account hidden from the sign-in screen);
#  2. makes C:\Trinity read-only for that account and removes the write
#     access every signed-in account gets by default on folders under C:\;
#  3. creates C:\TrinityRunner: the installed runner (read-only for the
#     account), its own folders (state, repo), and a secret folder for the
#     GitHub key that only the account and Ayman can read;
#  4. stores the trinity-results key Ayman copies (read from the clipboard,
#     checked with GitHub, never shown);
#  5. lets it log on as a batch job, registers the runner to start at boot, and stops sleep and automatic
#     update restarts.
# Rerunning it is safe: it updates the installed copy and keeps the key.

param(
    [string]$Owner = "",                 # Ayman's account (passed when the script elevates itself)
    [string]$Repo = "C:\Trinity",
    [string]$RunnerDir = "C:\TrinityRunner"
)

$ErrorActionPreference = "Stop"
$Account = "trinity-runner"
$TaskName = "Trinity Runner"

$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    $me = "$env:USERDOMAIN\$env:USERNAME"
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Owner `"$me`""
    exit
}
if (-not $Owner) { $Owner = "$env:USERDOMAIN\$env:USERNAME" }

function Step([string]$Text) { Write-Host ""; Write-Host "==> $Text" -ForegroundColor Cyan }
function Run-Icacls([string[]]$IcaclsArgs) {
    & icacls.exe @IcaclsArgs /C /Q | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "icacls $($IcaclsArgs -join ' ') failed (exit code $LASTEXITCODE)" }
}

# Add or remove an account in the "Log on as a batch job" right, which a
# task that runs whether or not anyone is signed in needs.
function Set-BatchLogon([string]$Sid, [bool]$Allow) {
    $tmp = Join-Path $env:TEMP "trinity-rights"
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    & secedit.exe /export /cfg "$tmp\now.inf" /areas USER_RIGHTS | Out-Null
    $line = (Get-Content -LiteralPath "$tmp\now.inf" | Where-Object { $_ -match '^SeBatchLogonRight\s*=' }) -replace '^SeBatchLogonRight\s*=\s*', ''
    $list = @("$line" -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ -and $_ -ne "*$Sid" })
    if ($Allow) { $list += "*$Sid" }
    Set-Content -LiteralPath "$tmp\new.inf" -Encoding Unicode -Value @("[Unicode]", "Unicode=yes", "[Version]", 'signature="$CHICAGO$"', "Revision=1",
        "[Privilege Rights]", "SeBatchLogonRight = $($list -join ',')")
    & secedit.exe /configure /db "$tmp\new.sdb" /cfg "$tmp\new.inf" /areas USER_RIGHTS /quiet | Out-Null
    $code = $LASTEXITCODE
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    if ($code -ne 0) { throw "secedit could not change the batch logon right (exit code $code)" }
}

try {
    Write-Host "Installing the Trinity job runner. Ayman's account: $Owner" -ForegroundColor Green
    if (-not (Test-Path -LiteralPath (Join-Path $Repo "runner\runner.ps1"))) { throw "$Repo\runner\runner.ps1 not found. Update C:\Trinity first (any .bat does it)." }

    Step "Which machine is this?"
    $machine = ""
    while ($machine -notin @("pc", "laptop")) { $machine = (Read-Host "Type pc or laptop and press Enter").Trim().ToLower() }

    Step "Account $Account"
    Add-Type -AssemblyName System.Web
    $password = [System.Web.Security.Membership]::GeneratePassword(32, 6)
    $secure = ConvertTo-SecureString $password -AsPlainText -Force
    if (Get-LocalUser -Name $Account -ErrorAction SilentlyContinue) {
        Set-LocalUser -Name $Account -Password $secure
        Write-Host "Exists; new random password set."
    } else {
        New-LocalUser -Name $Account -Password $secure -PasswordNeverExpires -UserMayNotChangePassword `
            -Description "Runs Trinity jobs (standard account, no admin)" | Out-Null
        Add-LocalGroupMember -Group (Get-LocalGroup -SID "S-1-5-32-545").Name -Member $Account   # Users
        Write-Host "Created (standard account)."
    }
    Set-BatchLogon (Get-LocalUser -Name $Account).SID.Value $true
    Write-Host "Allowed to run as a background task (log on as a batch job)."
    $hide = "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList"
    New-Item -Path $hide -Force | Out-Null
    New-ItemProperty -Path $hide -Name $Account -Value 0 -PropertyType DWord -Force | Out-Null

    Step "Making $Repo read-only for $Account (this can take a few minutes on a big folder)"
    Run-Icacls @($Repo, "/inheritance:d")
    Run-Icacls @($Repo, "/remove:g", "*S-1-5-11")                      # Authenticated Users: drop the default write access
    Run-Icacls @($Repo, "/grant", "*S-1-5-32-545:(OI)(CI)RX")          # Users: read
    Run-Icacls @($Repo, "/grant", "${Owner}:(OI)(CI)F", "/grant", "*S-1-5-32-544:(OI)(CI)F", "/grant", "*S-1-5-18:(OI)(CI)F")
    Run-Icacls @($Repo, "/deny", "${Account}:(OI)(CI)(W,D,DC,WDAC,WO)") # the wall: no write, delete or permission change

    Step "Folder $RunnerDir"
    foreach ($d in @($RunnerDir, "$RunnerDir\state", "$RunnerDir\repo", "$RunnerDir\secret")) { New-Item -ItemType Directory -Force -Path $d | Out-Null }
    Run-Icacls @($RunnerDir, "/inheritance:r", "/grant", "*S-1-5-32-544:(OI)(CI)F", "/grant", "*S-1-5-18:(OI)(CI)F",
        "/grant", "${Owner}:(OI)(CI)M", "/grant", "${Account}:(OI)(CI)RX")
    Run-Icacls @("$RunnerDir\state", "/grant", "${Account}:(OI)(CI)M")
    Run-Icacls @("$RunnerDir\repo", "/grant", "${Account}:(OI)(CI)M")
    Run-Icacls @("$RunnerDir\secret", "/inheritance:r", "/grant", "*S-1-5-32-544:(OI)(CI)F", "/grant", "*S-1-5-18:(OI)(CI)F",
        "/grant", "${Owner}:(OI)(CI)F", "/grant", "${Account}:(OI)(CI)R")
    foreach ($f in @("runner.ps1", "Approve-Scripts.ps1")) { Copy-Item -LiteralPath (Join-Path $Repo "runner\$f") -Destination $RunnerDir -Force }
    Set-Content -LiteralPath "$RunnerDir\machine.txt" -Value $machine
    $ps = "%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
    Set-Content -LiteralPath "$RunnerDir\Approve-Scripts.bat" -Value "@echo off`r`n`"$ps`" -NoProfile -ExecutionPolicy Bypass -File `"$RunnerDir\Approve-Scripts.ps1`"`r`npause"
    Set-Content -LiteralPath "$RunnerDir\Stop-Runner.bat" -Value "@echo off`r`necho stop> `"$RunnerDir\STOP`"`r`necho The runner will start no new job. (A job already running finishes.)`r`npause"
    Set-Content -LiteralPath "$RunnerDir\Start-Runner.bat" -Value "@echo off`r`nif exist `"$RunnerDir\STOP`" del `"$RunnerDir\STOP`"`r`necho The runner will take jobs again within 5 minutes.`r`npause"
    Set-Content -LiteralPath "$RunnerDir\Runner-Status.bat" -Value "@echo off`r`n`"$ps`" -NoProfile -Command `"Get-Content '$RunnerDir\state\runner.log' -Tail 25`"`r`npause"
    $desktop = [Environment]::GetFolderPath("Desktop")
    if ("$env:USERDOMAIN\$env:USERNAME" -eq $Owner -and (Test-Path $desktop)) {
        foreach ($b in @("Approve-Scripts", "Stop-Runner", "Start-Runner", "Runner-Status")) { Copy-Item "$RunnerDir\$b.bat" (Join-Path $desktop "Trinity $b.bat") -Force }
        Write-Host "Shortcuts placed on the desktop: Trinity Approve-Scripts, Trinity Stop-Runner, Trinity Start-Runner, Trinity Runner-Status."
    } else {
        Write-Host "The four .bat shortcuts are in $RunnerDir."
    }

    Step "GitHub key for trinity-results"
    $creds = "$RunnerDir\secret\git-credentials"
    $keep = (Test-Path -LiteralPath $creds) -and ((Read-Host "A key is already stored. Press Enter to keep it, or type N to paste a new one").Trim().ToUpper() -ne "N")
    if (-not $keep) {
        # Read the key from the clipboard: pasting into a hidden prompt proved unreliable.
        $token = ""
        for ($try = 1; $try -le 3 -and -not $token; $try++) {
            Read-Host "In the browser, click the copy button next to the key, then come back here and press Enter" | Out-Null
            $clip = "$(Get-Clipboard -Raw)".Trim()
            if ($clip -match '^(github_pat_|ghp_)[A-Za-z0-9_]{30,}$') { $token = $clip }
            else { Write-Host "The clipboard does not hold a GitHub key (it holds $($clip.Length) characters). Copy it again." -ForegroundColor Yellow }
        }
        if (-not $token) { throw "No GitHub key found on the clipboard." }
        Set-Clipboard -Value " "
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        try {
            $r = Invoke-RestMethod -Uri "https://api.github.com/repos/ayman803/trinity-results" -Headers @{ Authorization = "Bearer $token"; "User-Agent" = "trinity-runner" }
        } catch { throw "GitHub did not accept the key ($($_.Exception.Message))." }
        if ($r.permissions -and $r.permissions.push -eq $false) { throw "The key works but cannot write to trinity-results. Check that Contents is set to Read and write." }
        [IO.File]::WriteAllText($creds, "https://x-access-token:$token@github.com`n")
        $token = $null; $clip = $null
        Write-Host "Checked with GitHub and stored (readable only by $Account and you)."
    }

    Step "Start at boot, no sleep, no automatic restarts"
    # Remember the current settings once, so Uninstall-Runner can put them back.
    $saved = "$RunnerDir\settings-before-install.txt"   # not in state: the runner can write there, and the uninstaller reads this as administrator
    if (-not (Test-Path -LiteralPath $saved)) { try {
        $sleepAc = ((& powercfg.exe /query SCHEME_CURRENT SUB_SLEEP STANDBYIDLE) -match "Current AC Power Setting Index") -replace ".*:\s*", ""
        $hibAc = ((& powercfg.exe /query SCHEME_CURRENT SUB_SLEEP HIBERNATEIDLE) -match "Current AC Power Setting Index") -replace ".*:\s*", ""
        $auOld = (Get-ItemProperty "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Name NoAutoRebootWithLoggedOnUsers -ErrorAction SilentlyContinue).NoAutoRebootWithLoggedOnUsers
        $uxOld = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings" -ErrorAction SilentlyContinue
        Set-Content -LiteralPath $saved -Value @(
            "sleep_ac_seconds=$([Convert]::ToInt32("$sleepAc".Trim(), 16))",
            "hibernate_ac_seconds=$([Convert]::ToInt32("$hibAc".Trim(), 16))",
            "no_auto_reboot=$auOld",
            "active_hours_start=$($uxOld.ActiveHoursStart)",
            "active_hours_end=$($uxOld.ActiveHoursEnd)")
    } catch { Write-Host "Could not read the current sleep settings; the uninstaller will leave them as they are." -ForegroundColor Yellow } }
    $action = New-ScheduledTaskAction -Execute "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe" `
        -Argument "-NoProfile -NonInteractive -ExecutionPolicy Bypass -File `"$RunnerDir\runner.ps1`""
    $trigger = New-ScheduledTaskTrigger -AtStartup
    $settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 5) `
        -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -Priority 4
    Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings `
        -User $Account -Password $password -RunLevel Limited -Force | Out-Null
    $password = $null
    & powercfg.exe /change standby-timeout-ac 0
    & powercfg.exe /change hibernate-timeout-ac 0
    $au = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU"
    New-Item -Path $au -Force | Out-Null
    New-ItemProperty -Path $au -Name "NoAutoRebootWithLoggedOnUsers" -Value 1 -PropertyType DWord -Force | Out-Null
    $ux = "HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings"
    New-Item -Path $ux -Force | Out-Null
    New-ItemProperty -Path $ux -Name "ActiveHoursStart" -Value 6 -PropertyType DWord -Force | Out-Null
    New-ItemProperty -Path $ux -Name "ActiveHoursEnd" -Value 0 -PropertyType DWord -Force | Out-Null
    Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    Start-ScheduledTask -TaskName $TaskName

    Write-Host "Checking that the runner starts (20 seconds)..."
    Start-Sleep -Seconds 20
    $info = Get-ScheduledTaskInfo -TaskName $TaskName
    Write-Host ("Task: {0}; last result 0x{1:X8}" -f (Get-ScheduledTask -TaskName $TaskName).State, $info.LastTaskResult)
    $log = "$RunnerDir\state\runner.log"
    if (Test-Path -LiteralPath $log) { Get-Content -LiteralPath $log -Tail 5 }
    else { throw "The runner did not start (no log). Send Claude a screenshot of this window." }
    Write-Host ""
    Write-Host "INSTALLED on '$machine'. The runner is running as $Account." -ForegroundColor Green
    Write-Host "It runs nothing until you approve the scripts: double-click 'Trinity Approve-Scripts' on the desktop." -ForegroundColor Green
    Write-Host "Keep your own account signed in (locking the screen is fine): Windows only holds back update restarts while someone is signed in." -ForegroundColor Yellow
} catch {
    Write-Host "PROBLEM: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Send Claude a screenshot of this window." -ForegroundColor Yellow
}
Read-Host "Press Enter to close"
