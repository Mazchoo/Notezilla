# Trunk post_build hook. Skip debug; optimize the staged wasm-bindgen module in release.
if ($env:TRUNK_PROFILE -ne "release") {
    exit 0
}

$staging = $env:TRUNK_STAGING_DIR
if (-not $staging -or -not (Test-Path $staging)) {
    Write-Error "TRUNK_STAGING_DIR is not set or does not exist."
    exit 1
}

$wasm = Get-ChildItem -Path $staging -Filter "*_bg.wasm" -ErrorAction SilentlyContinue
if (-not $wasm) {
    Write-Error "No *_bg.wasm in staging directory: $staging"
    exit 1
}

$wasmOpt = Get-Command wasm-opt -ErrorAction SilentlyContinue
if (-not $wasmOpt) {
    Write-Error "wasm-opt is not on PATH. Install Binaryen and retry."
    exit 1
}

foreach ($file in $wasm) {
    & wasm-opt `
        $file.FullName `
        -O4 `
        --converge `
        --closed-world `
        --zero-filled-memory `
        --inline-functions-with-loops `
        --traps-never-happen `
        --strip-debug `
        --strip-producers `
        --enable-bulk-memory `
        --enable-sign-ext `
        --enable-mutable-globals `
        --enable-nontrapping-float-to-int `
        --enable-multivalue `
        --enable-reference-types `
        --enable-simd `
        --enable-extended-const `
        -o $file.FullName
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
