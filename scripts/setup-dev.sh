#!/bin/bash
# Development environment setup script
# Generated on 2025-09-10

set -euo pipefail

echo "🚀 Setting up development environment for dbfast..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust found: $(rustc --version)"

# Install required Rust components
echo "📦 Installing Rust components..."
rustup component add rustfmt clippy rust-src rust-analyzer

# Install cargo tools
echo "🔧 Installing cargo tools..."
cargo install --locked \
    cargo-nextest \
    cargo-audit \
    cargo-deny \
    cargo-tarpaulin \
    cargo-outdated \
    cargo-watch \
    cargo-edit

# Install pre-commit if Python is available
if command -v pip &> /dev/null; then
    echo "🪝 Installing pre-commit..."
    pip install pre-commit
    pre-commit install
    echo "✅ Pre-commit hooks installed"
else
    echo "⚠️  Python/pip not found. Skipping pre-commit installation."
    echo "   Install Python and run: pip install pre-commit && pre-commit install"
fi

# Verify installation
echo "🔍 Verifying installation..."
cargo --version
cargo nextest --version
cargo audit --version
cargo deny --version

# Run initial checks
echo "🧪 Running initial checks..."
cargo check --all-targets --all-features
cargo test --all-features

echo "🎉 Development environment setup complete!"
echo ""
echo "📚 Quick start:"
echo "  make help          # Show available commands"
echo "  make dev           # Run development workflow"
echo "  make quality       # Run all quality checks"
echo "  cargo watch -x check -x test  # Watch for changes"
echo ""
echo "💡 Tip: Run 'make install' to install additional tools"