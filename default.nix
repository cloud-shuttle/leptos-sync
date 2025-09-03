{ pkgs ? import <nixpkgs> {} }:

let
  nodeVersion = "20";
  
  # Use nightly Rust for Leptos 0.8 compatibility
  rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
    extensions = [ "rust-src" "rust-analyzer" ];
    targets = [ "wasm32-unknown-unknown" ];
  };
  
  nodejs = pkgs."nodejs_${nodeVersion}";
  
  wasmTools = with pkgs; [
    wasm-pack
  ];
  
  devTools = with pkgs; [
    cargo-audit
    cargo-tarpaulin
    cargo-deny
    sqlx-cli
    postgresql
    redis
    pkg-config
    openssl
    nodePackages.pnpm
  ];
  
in pkgs.mkShell {
  buildInputs = [
    rustToolchain
    nodejs
    pkgs.pkg-config
    pkgs.openssl
    pkgs.postgresql
    pkgs.redis
  ] ++ wasmTools ++ devTools;
  
  shellHook = ''
    echo "🚀 Leptos-Sync Development Environment"
    echo "Rust: $(rustc --version)"
    echo "Node: $(node --version)"
    echo ""
    echo "Available commands:"
    echo "  make build    - Build all crates"
    echo "  make test     - Run all tests"
    echo "  make wasm     - Build WASM packages"
    echo "  make docs     - Generate documentation"
    echo ""
    echo "Docker services:"
    echo "  docker-compose up -d  - Start PostgreSQL & Redis"
    echo "  docker-compose down   - Stop services"
    echo ""
    echo "Nix environment loaded successfully! 🎉"
  '';
  
  # Environment variables
  RUST_BACKTRACE = "1";
  RUST_LOG = "leptos_sync=debug";
  DATABASE_URL = "postgresql://user:pass@localhost:5432/leptos_sync_dev";
  REDIS_URL = "redis://localhost:6379/0";
  
  # Ensure Rust tools are in PATH
  nativeBuildInputs = [ pkgs.makeWrapper ];
  
  postShellHook = ''
    # Add Rust tools to PATH
    export PATH="$PWD/.cargo/bin:$PATH"
    
    # Ensure WASM target is available
    rustup target add wasm32-unknown-unknown 2>/dev/null || true
  '';
}
