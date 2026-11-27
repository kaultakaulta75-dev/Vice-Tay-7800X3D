# Sums up a run's frame lines (scripts\play.ps1 -More @{VICTORY_LOG_FILE=...}),
# so that two runs with one setting changed can be compared.
#   scripts\fps-report.ps1 D:\CLAUDE\victory.log            (the last run in the file)
#   scripts\fps-report.ps1 D:\CLAUDE\victory.log -All       (every run in the file)
# Only world frames count: over 1,000 draws a frame and fewer than 10
# pipelines still being made (the first minutes of a new translation are left out).
param(
    [Parameter(Mandatory)][string]$Log,
    [switch]$All,
    [int]$MinDraws = 1000
)
if (-not (Test-Path $Log)) { Write-Host "No $Log."; exit 1 }

$frame = 'host: ([\d.]+) fps, ([\d.]+) ms/frame \(([\d.]+) preparing.*?([\d.]+) on the GPU; ([\d.]+) copying back.*?(\d+) draws/frame, dropped (\d+) without a host pipeline \((\d+) being made\)'
$runs = New-Object System.Collections.Generic.List[object]
$current = $null
foreach ($line in [System.IO.File]::ReadLines((Resolve-Path $Log).Path)) {
    if ($line.Contains('GPU timeout detection')) {
        $current = [pscustomobject]@{ Start = $line.Substring(0, [Math]::Min(19, $line.Length)); Rows = New-Object System.Collections.Generic.List[double[]] }
        $runs.Add($current)
        continue
    }
    if ($null -eq $current) { continue }
    $m = [regex]::Match($line, $frame)
    if (-not $m.Success) { continue }
    $v = 1..8 | ForEach-Object { [double]::Parse($m.Groups[$_].Value, [Globalization.CultureInfo]::InvariantCulture) }
    if ($v[5] -gt $MinDraws -and $v[7] -lt 10) { $current.Rows.Add([double[]]$v) }
}

function Median([double[]]$xs) {
    if ($xs.Count -eq 0) { return [double]::NaN }
    $s = $xs | Sort-Object
    $n = $s.Count
    if ($n % 2) { return $s[($n - 1) / 2] } else { return ($s[$n / 2 - 1] + $s[$n / 2]) / 2 }
}
function Pct([double[]]$xs, [double]$p) {
    if ($xs.Count -eq 0) { return [double]::NaN }
    $s = $xs | Sort-Object
    return $s[[Math]::Min($s.Count - 1, [Math]::Floor($p * $s.Count))]
}

$chosen = if ($All) { $runs } else { @($runs | Select-Object -Last 1) }
foreach ($run in $chosen) {
    $r = $run.Rows
    if ($r.Count -eq 0) { Write-Host "$($run.Start): no world frames yet"; continue }
    $col = { param($i) [double[]]($r | ForEach-Object { $_[$i] }) }
    $fps = & $col 0
    '{0}  {1} lines  fps {2:N1} (p10 {3:N1})  ms {4:N1}  CPU preparing {5:N1}  GPU {6:N1}  copying back {7:N1}  draws {8:N0}  dropped {9:N0}' -f `
        $run.Start, $r.Count, (Median $fps), (Pct $fps 0.1), (Median (& $col 1)), (Median (& $col 2)),
        (Median (& $col 3)), (Median (& $col 4)), (Median (& $col 5)), (Median (& $col 6))
}
