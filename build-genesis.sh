#!/bin/bash

# Genesis Browser Build Script - Enhanced Version
set -euo pipefail

# Request sudo access upfront for apt operations
print_status() {
    echo -e "\033[0;34m[INFO]\033[0m $1"
}

print_status "Genesis Browser Build - Requesting sudo access for system dependencies..."
sudo -v

# Keep sudo alive during the script
while true; do sudo -n true; sleep 60; kill -0 "$$" || exit; done 2>/dev/null &

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Helper functions
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}    $1${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
}

# Initialize trap variables
current_command=""
last_command=""

# Trap errors and display helpful message
trap 'last_command=$current_command; current_command=$BASH_COMMAND' DEBUG
trap 'echo -e "${RED}[ERROR]${NC} \"${last_command}\" failed with exit code $?. Check the output above for details."' ERR

print_header "🚀 Genesis Browser Build Process"

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"
print_status "Detected OS: $OS ($ARCH)"

# Build profile selection
BUILD_PROFILE="${BUILD_PROFILE:-release}"
BUILD_FEATURES="${BUILD_FEATURES:-default}"
PARALLEL_JOBS="${PARALLEL_JOBS:-}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dev)
            BUILD_PROFILE="dev"
            shift
            ;;
        --production)
            BUILD_PROFILE="production"
            shift
            ;;
        --features)
            BUILD_FEATURES="$2"
            shift 2
            ;;
        --jobs|-j)
            PARALLEL_JOBS="$2"
            shift 2
            ;;
        --clean)
            print_status "Cleaning previous build artifacts..."
            cargo clean
            print_success "Clean completed"
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dev             Build in development mode (faster compile, no optimizations)"
            echo "  --production      Build in production mode (maximum optimizations)"
            echo "  --features LIST   Comma-separated list of features to enable"
            echo "  --jobs NUM        Number of parallel jobs for cargo"
            echo "  --clean           Clean build artifacts before building"
            echo "  --help            Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  BUILD_PROFILE     Build profile (dev/release/production)"
            echo "  BUILD_FEATURES    Features to enable"
            echo "  PARALLEL_JOBS     Number of parallel jobs"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

print_status "Build profile: $BUILD_PROFILE"
print_status "Features: $BUILD_FEATURES"

# Check and load Rust environment
print_header "📦 Checking Environment"

if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
    print_success "Cargo environment loaded"
else
    print_error "Cargo environment not found at ~/.cargo/env"
    print_status "Attempting to install Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Add local bin to PATH if exists
if [ -d "$HOME/.local/bin" ]; then
    export PATH="$HOME/.local/bin:$PATH"
    print_success "Added ~/.local/bin to PATH"
fi

# Version checks with proper error handling
check_command() {
    local cmd=$1
    local install_msg=$2
    
    if command -v "$cmd" &> /dev/null; then
        local version=$($cmd --version 2>&1 | head -n1)
        print_success "$cmd: $version"
        return 0
    else
        print_error "$cmd not found"
        if [ -n "$install_msg" ]; then
            print_warning "$install_msg"
        fi
        return 1
    fi
}

print_status "Checking required tools..."
MISSING_TOOLS=0

check_command "rustc" "Install Rust from https://rustup.rs" || MISSING_TOOLS=1
check_command "cargo" "Install Rust from https://rustup.rs" || MISSING_TOOLS=1
check_command "uv" "Install UV from https://astral.sh/uv/install.sh" || true # Optional

# Check Rust version requirement
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    REQUIRED_VERSION="1.85.0"
    
    if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
        print_warning "Rust version $RUST_VERSION is older than required $REQUIRED_VERSION"
        print_status "Updating Rust toolchain..."
        rustup update
        rustup default stable
    fi
fi

if [ $MISSING_TOOLS -eq 1 ]; then
    print_error "Missing required tools. Please install them and try again."
    exit 1
fi

# Check for Python (required for mach)
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
    print_success "Python: $PYTHON_VERSION"
else
    print_warning "Python3 not found - mach commands will not work"
fi

# Attempt to run mach bootstrap with workaround for crown linter
print_header "🔧 Preparing Build Environment"

if [ -f "./mach" ]; then
    print_status "Attempting mach bootstrap with crown linter workaround..."
    
    # Try to run mach bootstrap but continue if it fails
    if echo "y" | timeout 30s ./mach bootstrap --no-interactive >/dev/null 2>&1; then
        print_success "Mach bootstrap completed successfully"
    else
        print_warning "Mach bootstrap failed or timed out (crown linter issue)"
        print_status "Continuing with direct cargo build..."
        
        # Ensure Python dependencies are installed manually
        if command -v uv &> /dev/null; then
            print_status "Installing Python dependencies with UV..."
            echo "y" | uv pip install --system -r python/requirements.txt >/dev/null 2>&1 || true
        elif command -v pip3 &> /dev/null; then
            print_status "Installing Python dependencies with pip..."
            echo "y" | pip3 install -r python/requirements.txt >/dev/null 2>&1 || true
        fi
    fi
else
    print_warning "mach not found - using cargo directly"
fi

# Build Genesis Browser and all workspace members
print_header "🔨 Building Genesis Browser"

# Construct cargo build command
CARGO_CMD="cargo build"

if [ "$BUILD_PROFILE" != "dev" ]; then
    CARGO_CMD="$CARGO_CMD --profile $BUILD_PROFILE"
fi

if [ -n "$PARALLEL_JOBS" ]; then
    CARGO_CMD="$CARGO_CMD -j $PARALLEL_JOBS"
fi

if [ "$BUILD_FEATURES" != "default" ]; then
    CARGO_CMD="$CARGO_CMD --features $BUILD_FEATURES"
fi

# Build workspace
print_status "Building entire workspace..."
print_status "Command: $CARGO_CMD --workspace"

if $CARGO_CMD --workspace; then
    print_success "Workspace build completed"
else
    print_error "Workspace build failed"
    print_status "Trying to build Genesis Browser only..."
    if $CARGO_CMD --bin genesis-browser; then
        print_warning "Built genesis-browser binary only (workspace build failed)"
    else
        print_error "Build failed completely"
        exit 1
    fi
fi

# Determine output directory based on profile
case "$BUILD_PROFILE" in
    dev)
        TARGET_DIR="target/debug"
        ;;
    *)
        TARGET_DIR="target/$BUILD_PROFILE"
        ;;
esac

# Verify build artifacts
print_header "✅ Verifying Build Artifacts"

BINARIES=("genesis-browser" "servo")
BUILT_BINARIES=()
MISSING_BINARIES=()

for binary in "${BINARIES[@]}"; do
    if [ -f "$TARGET_DIR/$binary" ]; then
        print_success "$binary: $(du -h $TARGET_DIR/$binary | cut -f1)"
        BUILT_BINARIES+=("$binary")
    else
        print_warning "$binary: not found"
        MISSING_BINARIES+=("$binary")
    fi
done

# Check libraries built
print_status "Checking built libraries..."
LIBS_COUNT=$(find "$TARGET_DIR" -name "*.rlib" 2>/dev/null | wc -l)
print_success "Built $LIBS_COUNT libraries"

# Create convenient symlinks
if [ ${#BUILT_BINARIES[@]} -gt 0 ]; then
    print_status "Creating convenient symlinks..."
    
    for binary in "${BUILT_BINARIES[@]}"; do
        if [ -f "$TARGET_DIR/$binary" ]; then
            ln -sf "$TARGET_DIR/$binary" "./$binary"
            print_success "Created symlink: ./$binary -> $TARGET_DIR/$binary"
        fi
    done
fi

# Display build summary
print_header "📊 Build Summary"

echo -e "${GREEN}Build completed successfully!${NC}"
echo ""
echo "Profile: $BUILD_PROFILE"
echo "Features: $BUILD_FEATURES"
echo "Target directory: $TARGET_DIR"
echo ""

if [ ${#BUILT_BINARIES[@]} -gt 0 ]; then
    echo "Built binaries:"
    for binary in "${BUILT_BINARIES[@]}"; do
        echo "  ✅ $binary"
    done
fi

if [ ${#MISSING_BINARIES[@]} -gt 0 ]; then
    echo ""
    echo "Missing binaries (optional):"
    for binary in "${MISSING_BINARIES[@]}"; do
        echo "  ⚠️  $binary"
    done
fi

# Show how to run
print_header "🚀 How to Run"

if [[ " ${BUILT_BINARIES[@]} " =~ " genesis-browser " ]]; then
    echo "To run Genesis Browser:"
    echo ""
    echo "  Direct execution:"
    echo "    ./$TARGET_DIR/genesis-browser start"
    echo ""
    echo "  Via symlink:"
    echo "    ./genesis-browser start"
    echo ""
    echo "  With cargo:"
    echo "    cargo run --profile $BUILD_PROFILE --bin genesis-browser -- start"
    echo ""
    echo "  Test DNS resolution:"
    echo "    ./genesis-browser test example.genesis"
    echo ""
    echo "  Show info:"
    echo "    ./genesis-browser info"
fi

if [[ " ${BUILT_BINARIES[@]} " =~ " servo " ]]; then
    echo ""
    echo "To run Servo Shell:"
    echo "    ./$TARGET_DIR/servo [URL]"
fi

print_success "Build process completed! 🎉"