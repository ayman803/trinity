# One-time setup: installs Rust (and the Microsoft C++ build tools it needs
# on Windows), builds Trinity, runs its self-tests and downloads the test
# tools. Safe to run again at any time. Double-click 1-Setup.bat.

. (Join-Path $PSScriptRoot "common.ps1")

function Test-MsvcInstalled {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) { return $false }
    $path = & $vswhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    return [bool]$path
}

try {
    Write-Step "Checking Git"
    if (-not (Test-Command "git")) {
        Stop-WithMessage "Git is missing. Install it from https://git-scm.com/download/win (all default options), then run this again."
    }
    Write-Good "Git is installed."

    if ($script:OnWindows) {
        Write-Step "Checking the Microsoft C++ build tools (Rust needs them on Windows)"
        if (Test-MsvcInstalled) {
            Write-Good "C++ build tools are installed."
        } else {
            # Use Microsoft's official installer directly (winget is often
            # missing or not on PATH on Windows 10).
            Write-Host "Installing 'Build Tools for Visual Studio 2022' (C++ part only)."
            Write-Host "Windows will ask for permission - click Yes. A progress window appears; this takes 5-20 minutes."
            $installer = Join-Path $env:TEMP "vs_BuildTools.exe"
            Save-Download "https://aka.ms/vs/17/release/vs_BuildTools.exe" $installer
            $p = Start-Process -FilePath $installer -Wait -PassThru -ArgumentList @(
                "--passive", "--wait", "--norestart",
                "--add", "Microsoft.VisualStudio.Workload.VCTools", "--includeRecommended")
            # 0 = done, 3010 = done but Windows wants a restart.
            if (($p.ExitCode -ne 0 -and $p.ExitCode -ne 3010) -or -not (Test-MsvcInstalled)) {
                Stop-WithMessage ("The C++ build tools did not install (code $($p.ExitCode)). Install them by hand from " +
                    "https://visualstudio.microsoft.com/visual-cpp-build-tools/ (tick 'Desktop development with C++'), then run this again.")
            }
            if ($p.ExitCode -eq 3010) {
                Write-Warn "The build tools are installed, but Windows needs a restart. Restart the PC, then double-click 1-Setup.bat again."
                throw "stopped"
            }
            Write-Good "C++ build tools installed."
        }
    }

    Write-Step "Checking Rust"
    if (-not (Test-Command "cargo")) {
        if (-not $script:OnWindows) { Stop-WithMessage "Install Rust from https://rustup.rs" }
        $installer = Join-Path $env:TEMP "rustup-init.exe"
        Save-Download "https://win.rustup.rs/x86_64" $installer
        Write-Host "Installing Rust (a few minutes)..."
        Invoke-Native $installer @("-y", "--default-toolchain", "stable", "--profile", "minimal")
        $env:PATH = (Join-Path $HOME ".cargo\bin") + ";" + $env:PATH
    } else {
        Invoke-Native rustup @("update", "stable")
    }
    Write-Good ("Rust is installed: " + (Get-NativeOutput rustc @("--version")))

    Write-Step "Building Trinity and running its self-tests"
    Invoke-Native cargo @("test", "--release", "--manifest-path", (Join-Path $script:Root "Cargo.toml"))
    $exe = Build-Trinity "HEAD"
    $bench = Get-NativeOutput $exe @("bench")
    Write-Good "Trinity works. Bench: $bench"

    if ($script:OnWindows) {
        [void](Get-Fastchess)
        [void](Get-OpeningBook)
    }

    Write-Host ""
    Write-Good "Setup finished successfully."
    Write-Host "Next: double-click 2-Run-SPRT-Test.bat when Claude asks you to test a change."
} catch {
    if ($_.Exception.Message -ne "stopped") { Write-Host "Unexpected error: $($_.Exception.Message)" -ForegroundColor Red }
    exit 1
}
