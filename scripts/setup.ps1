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
        } elseif (Test-Command "winget") {
            Write-Host "Installing 'Build Tools for Visual Studio 2022' (C++ workload)."
            Write-Host "Windows will ask for permission - click Yes. This takes 5-15 minutes."
            $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
            & winget install --id Microsoft.VisualStudio.2022.BuildTools -e --accept-package-agreements --accept-source-agreements `
                --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended" | Out-Host
            $ErrorActionPreference = $old
            if (-not (Test-MsvcInstalled)) {
                Stop-WithMessage "The C++ build tools did not install. Install them by hand from https://visualstudio.microsoft.com/visual-cpp-build-tools/ (tick 'Desktop development with C++'), then run this again."
            }
            Write-Good "C++ build tools installed."
        } else {
            Stop-WithMessage "Please install the C++ build tools from https://visualstudio.microsoft.com/visual-cpp-build-tools/ (tick 'Desktop development with C++'), then run this again."
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
