$ErrorActionPreference = 'Stop'

if (-not $env:CARGO_DIST_VERSION) {
    throw 'CARGO_DIST_VERSION is required'
}

irm "https://github.com/axodotdev/cargo-dist/releases/download/v$($env:CARGO_DIST_VERSION)/cargo-dist-installer.ps1" | iex
