# Idle beacon (owner and advisor, 19 October 2026). Runs in Ayman's own
# Windows account (scheduled task "Trinity Idle Beacon", started at his
# sign-in by the installer) and writes, every 30 seconds, the time of the
# last keyboard or mouse use to C:\TrinityRunner\beacon\last-input.txt.
# The runner account can only read that file: it starts real jobs after 30
# minutes without use and stops them if Ayman comes back. This script reads
# nothing and changes nothing else.

param([string]$RunnerDir = "C:\TrinityRunner")

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class TrinityIdle {
    [StructLayout(LayoutKind.Sequential)]
    struct LastInputInfo { public uint cbSize; public uint dwTime; }
    [DllImport("user32.dll")]
    static extern bool GetLastInputInfo(ref LastInputInfo info);
    // Milliseconds since the last keyboard or mouse input in this session.
    public static uint IdleMs() {
        var info = new LastInputInfo();
        info.cbSize = (uint)Marshal.SizeOf(info);
        if (!GetLastInputInfo(ref info)) { return 0; }
        return unchecked((uint)Environment.TickCount - info.dwTime);
    }
}
'@

$dir = Join-Path $RunnerDir "beacon"
$file = Join-Path $dir "last-input.txt"
$tmp = "$file.tmp"
while ($true) {
    try {
        $now = [DateTime]::UtcNow
        $last = $now.AddMilliseconds(-[double][TrinityIdle]::IdleMs())
        # "<last input> <written>", both UTC; replaced whole so the runner never reads half a line.
        [IO.File]::WriteAllText($tmp, ("{0:o} {1:o}" -f $last, $now))
        Move-Item -LiteralPath $tmp -Destination $file -Force
    } catch { }
    Start-Sleep -Seconds 30
}
