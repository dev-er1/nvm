echo "    Checking that CI passes..."
./../ci/ci.ps1;

echo "    Cleaning..."
cargo clean;
cargo build --release
Copy-Item ../../target/release/nvm.exe ../../.