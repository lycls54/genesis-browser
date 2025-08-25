#!/bin/bash

# Genesis Browser - Ubuntu Dependencies Installation Script
# Run with: bash install-deps-ubuntu.sh

set -euo pipefail

echo "🚀 Genesis Browser Dependencies Installation for Ubuntu"
echo "======================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Initialize trap variables
current_command=""
last_command=""

# Trap errors for better debugging
trap 'last_command=$current_command; current_command=$BASH_COMMAND' DEBUG
trap 'print_error "\"${last_command}\" failed with exit code $?"' ERR

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root (don't use sudo)"
   print_warning "It will ask for sudo when needed"
   exit 1
fi

# Detect Ubuntu version
UBUNTU_VERSION=$(lsb_release -rs 2>/dev/null || echo "unknown")
print_status "Ubuntu version: $UBUNTU_VERSION"

# Check if we have sudo access
if ! sudo -n true 2>/dev/null; then
    print_warning "This script requires sudo access. Please enter your password when prompted."
    sudo -v
fi

# Update package list
print_status "Updating package list..."
sudo apt update

# Install build essentials
print_status "Installing build essentials..."
sudo apt install -y \
    curl \
    wget \
    build-essential \
    pkg-config \
    cmake \
    git \
    gcc \
    g++ \
    m4 \
    autoconf \
    automake \
    libtool

print_success "Build essentials installed"

# Install Python and pip
print_status "Installing Python dependencies..."
sudo apt install -y \
    python3 \
    python3-pip \
    python3-venv \
    python3-dev \
    python3-setuptools \
    python3-wheel

# Ensure pip is up to date
print_status "Updating pip..."
python3 -m pip install --upgrade pip --user || true

print_success "Python dependencies installed"

# Install graphics and audio libraries
print_status "Installing graphics and audio libraries..."
sudo apt install -y \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libxinerama-dev \
    libxkbcommon-dev \
    libgl1-mesa-dev \
    libgles2-mesa-dev \
    libegl1-mesa-dev \
    libvulkan-dev \
    mesa-utils \
    libasound2-dev \
    libpulse-dev \
    libjack-dev \
    libxss-dev \
    libglib2.0-dev \
    libxcomposite-dev \
    libxdamage-dev \
    libxext-dev \
    libxfixes-dev \
    libxrender-dev \
    libxtst-dev

print_success "Graphics and audio libraries installed"

# Install font and rendering libraries
print_status "Installing font and rendering libraries..."
sudo apt install -y \
    libfreetype6-dev \
    libfontconfig1-dev \
    libharfbuzz-dev \
    libpango1.0-dev \
    libcairo2-dev \
    libgdk-pixbuf2.0-dev

print_success "Font and rendering libraries installed"

# Install GStreamer for media support
print_status "Installing GStreamer for media support..."
sudo apt install -y \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-good1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    gstreamer1.0-tools \
    gstreamer1.0-x \
    gstreamer1.0-alsa \
    gstreamer1.0-gl \
    gstreamer1.0-gtk3 \
    gstreamer1.0-pulseaudio

print_success "GStreamer installed"

# Install additional development libraries
print_status "Installing additional development libraries..."
sudo apt install -y \
    libssl-dev \
    libdbus-1-dev \
    libudev-dev \
    libgtk-3-dev \
    libgtk-4-dev \
    libwayland-dev \
    libwayland-client0 \
    libwayland-cursor0 \
    libwayland-egl1 \
    wayland-protocols \
    libxkbcommon-x11-dev \
    libzstd-dev \
    libbz2-dev \
    liblzma-dev

# Try to install WebKit (optional for Servo)
print_status "Installing WebKit (optional)..."
if sudo apt install -y libwebkit2gtk-4.1-dev 2>/dev/null; then
    print_success "WebKit installed"
elif sudo apt install -y libwebkit2gtk-4.0-dev 2>/dev/null; then
    print_success "WebKit 4.0 installed"
else
    print_warning "WebKit not available - skipping (not required for Servo)"
fi

# Try to install AppIndicator (optional)
print_status "Installing AppIndicator (optional)..."
if sudo apt install -y libappindicator3-dev 2>/dev/null; then
    print_success "AppIndicator installed"
else
    print_warning "AppIndicator conflict detected - skipping (not required)"
fi

print_success "Additional libraries installed"

# Check for system Rust and warn if found
if [ -f /usr/bin/rustc ]; then
    SYSTEM_RUST_VERSION=$(/usr/bin/rustc --version 2>/dev/null | cut -d' ' -f2)
    print_warning "System Rust detected: $SYSTEM_RUST_VERSION at /usr/bin/rustc"
    print_status "This may conflict with rustup. Removing system Rust..."
    
    # Remove system Rust packages
    sudo apt remove -y rustc cargo 2>/dev/null || true
    sudo apt autoremove -y 2>/dev/null || true
    print_success "System Rust removed"
fi

# Install Rust via rustup
if [ ! -f "$HOME/.cargo/bin/rustc" ]; then
    print_status "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source ~/.cargo/env
    print_success "Rust installed via rustup"
else
    print_success "Rustup Rust already installed"
    source ~/.cargo/env
    rustc --version
fi

# Ensure we have the required Rust version
if command -v rustc &> /dev/null; then
    source ~/.cargo/env 2>/dev/null || true
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    REQUIRED_VERSION="1.85.0"
    
    if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
        print_warning "Rust version $RUST_VERSION is older than required $REQUIRED_VERSION"
        print_status "Updating Rust toolchain..."
        rustup update stable
        rustup default stable
    fi
    
    # Install additional Rust components
    print_status "Installing Rust components..."
    rustup component add rustfmt clippy rust-src 2>/dev/null || true
fi

# Install UV package manager (if not already installed)
if ! command -v uv &> /dev/null; then
    print_status "Installing UV package manager..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
    print_success "UV package manager installed"
else
    print_success "UV already installed"
    uv --version
fi

# Install ccache for faster rebuilds
print_status "Installing ccache for faster rebuilds..."
if sudo apt install -y ccache; then
    print_success "ccache installed"
    # Configure ccache
    if [ ! -d "$HOME/.ccache" ]; then
        mkdir -p "$HOME/.ccache"
    fi
    ccache --set-config max_size=10G 2>/dev/null || true
    export PATH="/usr/lib/ccache:$PATH"
else
    print_warning "ccache installation failed (optional)"
fi

# Verify installations
print_status "Verifying installations..."

# Check essential tools
commands_to_check=("gcc" "g++" "make" "pkg-config" "cmake" "python3" "pip3")
MISSING_DEPS=0
for cmd in "${commands_to_check[@]}"; do
    if command -v $cmd &> /dev/null; then
        version=$($cmd --version 2>&1 | head -n1 || echo "version unknown")
        print_success "$cmd: $version"
    else
        print_error "$cmd is not available"
        MISSING_DEPS=1
    fi
done

# Source cargo environment
if [ -f ~/.cargo/env ]; then
    source ~/.cargo/env
fi

# Check Rust
if command -v rustc &> /dev/null; then
    print_success "Rust $(rustc --version)"
else
    print_warning "Rust not found in PATH, you may need to restart your shell"
fi

# Check UV
if command -v uv &> /dev/null; then
    print_success "UV $(uv --version)"
else
    print_warning "UV not found in PATH, you may need to restart your shell or run: export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

# Check library availability
print_status "Checking critical libraries..."
LIBRARIES_TO_CHECK=("x11" "gl" "gstreamer-1.0" "freetype2" "fontconfig")
for lib in "${LIBRARIES_TO_CHECK[@]}"; do
    if pkg-config --exists $lib 2>/dev/null; then
        version=$(pkg-config --modversion $lib 2>/dev/null || echo "unknown")
        print_success "$lib: version $version"
    else
        print_warning "$lib: not found (may cause build issues)"
    fi
done

# Create convenience script for environment setup
print_status "Creating environment setup script..."
cat > ~/genesis-browser-env.sh << 'EOF'
#!/bin/bash
# Genesis Browser Environment Setup

# CRITICAL: Load Rust environment FIRST (before system paths)
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Add UV to PATH
if [ -d "$HOME/.local/bin" ]; then
    export PATH="$HOME/.local/bin:$PATH"
fi

# Add ccache to PATH
if [ -d "/usr/lib/ccache" ]; then
    export PATH="/usr/lib/ccache:$PATH"
fi

# Ensure rustup Rust takes precedence over system Rust
export PATH="$HOME/.cargo/bin:$PATH"

# Set build optimizations (use GCC as default compiler)
export CC="gcc"
export CXX="g++"

# Verify correct Rust is being used
RUST_VERSION=$(rustc --version 2>/dev/null | cut -d' ' -f2)
echo "Genesis Browser environment loaded!"
echo "Using Rust: $RUST_VERSION from $(which rustc)"
EOF

chmod +x ~/genesis-browser-env.sh
print_success "Environment setup script created: ~/genesis-browser-env.sh"

if [ $MISSING_DEPS -eq 0 ]; then
    echo ""
    echo "🎉 Installation completed successfully!"
    echo "======================================="
else
    echo ""
    echo "⚠️  Installation completed with warnings"
    echo "======================================="
    print_warning "Some dependencies are missing. The build might fail."
fi

echo ""
echo "Next steps:"
echo "1. Load environment: source ~/genesis-browser-env.sh"
echo "2. Navigate to genesis-browser directory"
echo "3. Run: ./build-genesis.sh"
echo ""
echo "Build options:"
echo "  ./build-genesis.sh --dev        # Fast debug build"
echo "  ./build-genesis.sh              # Optimized release build"
echo "  ./build-genesis.sh --production # Maximum optimization"
echo "  ./build-genesis.sh --help       # Show all options"
echo ""
echo "To start Genesis Browser after building:"
echo "  ./genesis-browser start"
echo ""
print_success "Ready to build Genesis Browser! 🚀"