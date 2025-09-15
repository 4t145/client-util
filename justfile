fix:
    cargo fmt --all
    git add .
    cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged