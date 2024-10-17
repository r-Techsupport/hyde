<#
    Script builds the Hyde frontend and backend, then moves builds to a created .\target folder

    Use optional argument -CopyHydeData <Path> to copy the hyde data from a specified directory.
#>

param (
    [String]$C
)

$FrontendFolder = ".\frontend"
$FrontendBuildFolder = "${FrontendFolder}\build"

$BackendFolder = ".\backend"
$BackendBuildExe = "${BackendFolder}\target\release\hyde-backend.exe"

$BuildFolder = ".\target"

function Check-Target {
    if (Test-Path -Path $BuildFolder) {
        Write-Host "Clearing target folder for rebuild"
        Remove-Item "${BuildFolder}\*" -Recurse -Force
        return
    }
    New-Item -Path . -Name $BuildFolder -ItemType "directory"
}

function Build-Frontend {
    Push-Location $FrontendFolder
    npm i
    npm run build
    Pop-Location
}

function Build-Backend {
    Push-Location $BackendFolder
    cargo build --release
    Pop-Location
}

function Copy-Builds {
    New-Item -Path "${BuildFolder}\web" -ItemType Directory -ErrorAction SilentlyContinue
    Copy-Item -Path "${FrontendBuildFolder}\*" -Destination "${BuildFolder}\web" -Force -Recurse
    Copy-Item -Path $BackendBuildExe -Destination "${BuildFolder}\hyde.exe" -Force
}

function Copy-HydeData {
    Param (
        [Parameter(Mandatory=$true)]
        [String] $HydeDataFolder
    )
    Copy-Item -Path $HydeDataFolder -Destination $BuildFolder -Recurse -Force
}

function Main {
    Check-Target
    Build-Frontend
    Build-Backend
    Copy-Builds
}

if ($C.Length -gt 0) {
    if (-not (Test-Path $C)) {
        Write-Host "${C} path does not exist. Cannot copy."
        return
    }
    Main
    Copy-HydeData $C
    return
}

Main