$ErrorActionPreference = 'Stop'

$steps = @(
    { cargo check }
    { cargo fmt --all --check }
    { cargo build --release }
    { cargo test --all --release }
    { cargo clippy --all-targets --all-features -- -D warnings }
)

foreach ($step in $steps) {
    & $step

    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}