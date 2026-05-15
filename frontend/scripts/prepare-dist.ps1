# Prepares frontend distribution assets for the Tauri build.
$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$dist = Join-Path $root "dist"
$tesseractDist = Join-Path $root "node_modules\tesseract.js\dist"

New-Item -ItemType Directory -Force -Path $dist | Out-Null
Copy-Item -LiteralPath (Join-Path $root "index.html") -Destination (Join-Path $dist "index.html") -Force
Copy-Item -LiteralPath (Join-Path $root "styles.css") -Destination (Join-Path $dist "styles.css") -Force
Copy-Item -LiteralPath (Join-Path $root "..\icons\icon.png") -Destination (Join-Path $dist "icon.png") -Force

$tesseractBrowser = Join-Path $tesseractDist "tesseract.min.js"
if (Test-Path -LiteralPath $tesseractBrowser) {
  Copy-Item -LiteralPath $tesseractBrowser -Destination (Join-Path $dist "tesseract.min.js") -Force
}
