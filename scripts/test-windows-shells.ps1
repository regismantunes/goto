$ErrorActionPreference = 'Stop'

$projectDir = Split-Path -Parent $PSScriptRoot
$binaryDir = Join-Path $projectDir 'target\debug'
$env:PATH = "$binaryDir;$env:PATH"
$testRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("goto-shell-test-" + [guid]::NewGuid())
$target = Join-Path $testRoot 'folder with spaces'
$env:GOTO_CONFIG = Join-Path $testRoot 'workplaces.json'

try {
    Copy-Item -LiteralPath (Join-Path $projectDir 'shell\goto-shell.cmd') -Destination $binaryDir -Force
    Copy-Item -LiteralPath (Join-Path $projectDir 'shell\goto-init.cmd') -Destination $binaryDir -Force
    New-Item -ItemType Directory -Path $target | Out-Null
    & goto.exe -s $target --name ci | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Could not create the test workplace.' }

    Invoke-Expression (& goto.exe init powershell | Out-String)
    goto ci
    if ((Get-Location).Path -ne $target) { throw 'PowerShell integration did not change directory.' }

    Set-Location $projectDir
    $cmdInit = & goto.exe init cmd
    if ($LASTEXITCODE -ne 0 -or $cmdInit -notmatch '^doskey goto=') {
        throw 'CMD initialization did not produce a doskey macro.'
    }
    $staticCmdInit = Get-Content -Raw (Join-Path $binaryDir 'goto-init.cmd')
    if ($staticCmdInit -notmatch 'doskey goto=') {
        throw 'The static CMD initialization script is invalid.'
    }
    $cmdOutput = & cmd.exe /d /c "where goto-shell.cmd && call goto-shell.cmd ci && cd"
    if ($LASTEXITCODE -ne 0 -or $cmdOutput[-1] -ne $target) {
        throw "CMD integration did not change directory. Exit: $LASTEXITCODE. Output: $($cmdOutput -join ' | ')"
    }
}
finally {
    Set-Location $projectDir
    $resolvedTemp = [System.IO.Path]::GetFullPath($testRoot)
    $resolvedBase = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
    if ($resolvedTemp.StartsWith($resolvedBase, [System.StringComparison]::OrdinalIgnoreCase) -and
        (Split-Path -Leaf $resolvedTemp).StartsWith('goto-shell-test-')) {
        Remove-Item -LiteralPath $resolvedTemp -Recurse -Force -ErrorAction SilentlyContinue
    }
}
