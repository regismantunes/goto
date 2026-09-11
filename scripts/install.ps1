param(
    [string]$Repository = $(if ($env:GOTO_REPOSITORY) { $env:GOTO_REPOSITORY } else { '@GOTO_REPOSITORY@' }),
    [string]$Version = $(if ($env:GOTO_VERSION) { $env:GOTO_VERSION } else { 'latest' }),
    [string]$InstallDir = $(if ($env:GOTO_INSTALL_DIR) { $env:GOTO_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\goto' })
)

$ErrorActionPreference = 'Stop'

if ($Repository.Contains('@GOTO_' + 'REPOSITORY@')) {
    throw 'Set GOTO_REPOSITORY to owner/repository when running the source installer.'
}
if (-not [Environment]::Is64BitOperatingSystem) {
    throw 'The prebuilt Windows release requires a 64-bit operating system.'
}

$asset = 'goto-x86_64-pc-windows-msvc.zip'
if ($Version -eq 'latest') {
    $releaseUrl = "https://github.com/$Repository/releases/latest/download"
}
else {
    $tag = if ($Version.StartsWith('v')) { $Version } else { "v$Version" }
    $releaseUrl = "https://github.com/$Repository/releases/download/$tag"
}

$temporaryDir = Join-Path ([System.IO.Path]::GetTempPath()) ("goto-install-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $temporaryDir | Out-Null

try {
    $archive = Join-Path $temporaryDir $asset
    $checksums = Join-Path $temporaryDir 'checksums.txt'
    Write-Host "Downloading $asset..."
    Invoke-WebRequest "$releaseUrl/$asset" -OutFile $archive
    Invoke-WebRequest "$releaseUrl/checksums.txt" -OutFile $checksums

    $checksumLine = Get-Content $checksums | Where-Object { $_ -match "\s+$([regex]::Escape($asset))$" } | Select-Object -First 1
    if (-not $checksumLine) { throw "$asset is missing from checksums.txt." }
    $expected = ($checksumLine -split '\s+')[0].ToLowerInvariant()
    $actual = (Get-FileHash -Algorithm SHA256 $archive).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw "Checksum verification failed for $asset." }

    Expand-Archive -LiteralPath $archive -DestinationPath $temporaryDir -Force
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $temporaryDir 'goto.exe') -Destination $InstallDir -Force
    Copy-Item -LiteralPath (Join-Path $temporaryDir 'goto-shell.cmd') -Destination $InstallDir -Force
    Copy-Item -LiteralPath (Join-Path $temporaryDir 'goto-init.cmd') -Destination $InstallDir -Force

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $pathEntries = @($userPath -split ';' | Where-Object { $_ })
    if ($pathEntries -notcontains $InstallDir) {
        $newPath = (@($pathEntries) + $InstallDir) -join ';'
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    }
    if (($env:PATH -split ';') -notcontains $InstallDir) {
        $env:PATH = "$InstallDir;$env:PATH"
    }

    $gotoExecutable = [System.IO.Path]::GetFullPath((Join-Path $InstallDir 'goto.exe'))
    $escapedExecutable = $gotoExecutable.Replace("'", "''")
    $profileLine = "Invoke-Expression (& '$escapedExecutable' init powershell | Out-String)"
    $legacyProfileLine = 'Invoke-Expression (& goto.exe init powershell | Out-String)'
    $profileDirectory = Split-Path -Parent $PROFILE
    if ($profileDirectory) {
        New-Item -ItemType Directory -Path $profileDirectory -Force | Out-Null
    }
    if (-not (Test-Path -LiteralPath $PROFILE)) {
        New-Item -ItemType File -Path $PROFILE -Force | Out-Null
    }

    $profileLines = @(Get-Content -LiteralPath $PROFILE)
    if (($profileLines -notcontains $profileLine) -and ($profileLines -notcontains $legacyProfileLine)) {
        Add-Content -LiteralPath $PROFILE -Value $profileLine
    }

    Write-Host "Installed goto in $InstallDir."
    Write-Host "Enabled goto in PowerShell through $PROFILE."
    Write-Host 'Open a new PowerShell session to use goto directory shortcuts.'
    Write-Host 'Activate it in CMD with:'
    Write-Host "  call `"$InstallDir\goto-init.cmd`""
}
finally {
    $resolvedTemp = [System.IO.Path]::GetFullPath($temporaryDir)
    $resolvedBase = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
    if ($resolvedTemp.StartsWith($resolvedBase, [System.StringComparison]::OrdinalIgnoreCase) -and
        (Split-Path -Leaf $resolvedTemp).StartsWith('goto-install-')) {
        Remove-Item -LiteralPath $resolvedTemp -Recurse -Force -ErrorAction SilentlyContinue
    }
}
