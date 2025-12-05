#!/usr/bin/env bash
#
# Flux Framework - Dependency Checker & Installer
# Verifies and installs all required build dependencies
#
# Usage:
#   ./scripts/check_dependencies.sh          # Check only (no installation)
#   ./scripts/check_dependencies.sh --install # Check and install missing deps
#

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
INSTALL_MODE=false
MISSING_DEPS=()
WARNINGS=()

# Parse arguments
for arg in "$@"; do
    case $arg in
        --install|-i)
            INSTALL_MODE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --install, -i    Install missing dependencies"
            echo "  --help, -h       Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0               # Check dependencies only"
            echo "  $0 --install     # Check and install missing dependencies"
            exit 0
            ;;
        *)
            ;;
    esac
done

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Flux Framework - Dependency Checker${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Function to print status
print_status() {
    local status=$1
    local message=$2
    case $status in
        ok)
            echo -e "${GREEN}✓${NC} $message"
            ;;
        fail)
            echo -e "${RED}✗${NC} $message"
            ;;
        warn)
            echo -e "${YELLOW}⚠${NC} $message"
            ;;
        info)
            echo -e "${BLUE}ℹ${NC} $message"
            ;;
    esac
}

# Detect OS and distribution
detect_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS_NAME=$ID
        OS_VERSION=$VERSION_ID
        OS_PRETTY=$PRETTY_NAME
    else
        print_status fail "Cannot detect OS. /etc/os-release not found."
        exit 1
    fi
}

# Get package manager
get_package_manager() {
    case $OS_NAME in
        ubuntu|debian|linuxmint|pop)
            PKG_MANAGER="apt"
            PKG_UPDATE="apt-get update"
            PKG_INSTALL="apt-get install -y"
            BUILD_ESSENTIAL="build-essential"
            ;;
        rhel|centos|rocky|almalinux|fedora)
            if command -v dnf &> /dev/null; then
                PKG_MANAGER="dnf"
                PKG_INSTALL="dnf install -y"
                PKG_UPDATE="dnf check-update || true"
            else
                PKG_MANAGER="yum"
                PKG_INSTALL="yum install -y"
                PKG_UPDATE="yum check-update || true"
            fi
            BUILD_ESSENTIAL="gcc gcc-c++ make"
            ;;
        arch|manjaro)
            PKG_MANAGER="pacman"
            PKG_INSTALL="pacman -S --noconfirm"
            PKG_UPDATE="pacman -Sy"
            BUILD_ESSENTIAL="base-devel"
            ;;
        alpine)
            PKG_MANAGER="apk"
            PKG_INSTALL="apk add"
            PKG_UPDATE="apk update"
            BUILD_ESSENTIAL="build-base"
            ;;
        *)
            print_status fail "Unsupported distribution: $OS_NAME"
            exit 1
            ;;
    esac
}

# Check if running as root (needed for installation)
check_root() {
    if [[ $INSTALL_MODE == true ]] && [[ $EUID -ne 0 ]]; then
        print_status fail "Installation requires root privileges. Please run with sudo:"
        echo -e "  ${YELLOW}sudo $0 --install${NC}"
        exit 1
    fi
}

# Check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Check Rust installation
check_rust() {
    echo -e "\n${BLUE}Checking Rust...${NC}"

    if command_exists rustc && command_exists cargo; then
        local rust_version=$(rustc --version | awk '{print $2}')
        print_status ok "Rust is installed (version $rust_version)"

        # Check if version meets minimum requirement (1.77+)
        local major=$(echo $rust_version | cut -d. -f1)
        local minor=$(echo $rust_version | cut -d. -f2)

        if [[ $major -eq 1 ]] && [[ $minor -lt 77 ]]; then
            print_status warn "Rust version $rust_version is older than recommended (1.77+)"
            WARNINGS+=("Consider updating Rust: rustup update")
        fi

        return 0
    else
        print_status fail "Rust is not installed"
        MISSING_DEPS+=("rust")
        return 1
    fi
}

# Check C compiler and linker
check_cc() {
    echo -e "\n${BLUE}Checking C compiler and linker...${NC}"

    local has_compiler=false

    if command_exists gcc; then
        local gcc_version=$(gcc --version | head -n1)
        print_status ok "GCC is installed ($gcc_version)"
        has_compiler=true
    elif command_exists clang; then
        local clang_version=$(clang --version | head -n1)
        print_status ok "Clang is installed ($clang_version)"
        has_compiler=true
    else
        print_status fail "No C compiler found (gcc or clang required)"
        MISSING_DEPS+=("build-essential")
        has_compiler=false
    fi

    if command_exists cc; then
        print_status ok "C compiler linker (cc) is available"
    else
        if [[ $has_compiler == false ]]; then
            print_status fail "C compiler linker (cc) not found"
            MISSING_DEPS+=("build-essential")
        fi
    fi

    return 0
}

# Check make
check_make() {
    echo -e "\n${BLUE}Checking build tools...${NC}"

    if command_exists make; then
        local make_version=$(make --version | head -n1)
        print_status ok "Make is installed ($make_version)"
    else
        print_status fail "Make is not installed"
        MISSING_DEPS+=("build-essential")
    fi
}

# Check pkg-config
check_pkg_config() {
    if command_exists pkg-config; then
        print_status ok "pkg-config is installed"
    else
        print_status warn "pkg-config is not installed (may be needed for some dependencies)"
        WARNINGS+=("Consider installing pkg-config")
    fi
}

# Check OpenSSL development headers
check_openssl() {
    echo -e "\n${BLUE}Checking OpenSSL development headers...${NC}"

    case $OS_NAME in
        ubuntu|debian|linuxmint|pop)
            if dpkg-query -W -f='${Status}' libssl-dev 2>/dev/null | grep -q "install ok installed"; then
                print_status ok "OpenSSL development headers are installed"
            else
                print_status fail "OpenSSL development headers not found"
                MISSING_DEPS+=("libssl-dev")
            fi
            ;;
        rhel|centos|rocky|almalinux|fedora)
            if rpm -q openssl-devel &> /dev/null; then
                print_status ok "OpenSSL development headers are installed"
            else
                print_status fail "OpenSSL development headers not found"
                MISSING_DEPS+=("openssl-devel")
            fi
            ;;
        arch|manjaro)
            if pacman -Q openssl &> /dev/null; then
                print_status ok "OpenSSL is installed"
            else
                print_status warn "OpenSSL may not be installed"
                WARNINGS+=("Consider installing openssl")
            fi
            ;;
        alpine)
            if apk info openssl-dev &> /dev/null; then
                print_status ok "OpenSSL development headers are installed"
            else
                print_status fail "OpenSSL development headers not found"
                MISSING_DEPS+=("openssl-dev")
            fi
            ;;
    esac
}

# Check git
check_git() {
    echo -e "\n${BLUE}Checking version control...${NC}"

    if command_exists git; then
        local git_version=$(git --version)
        print_status ok "Git is installed ($git_version)"
    else
        print_status warn "Git is not installed (needed for cloning repository)"
        WARNINGS+=("Consider installing git")
    fi
}

# Install missing dependencies
install_dependencies() {
    if [[ ${#MISSING_DEPS[@]} -eq 0 ]]; then
        return 0
    fi

    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}Installing missing dependencies...${NC}"
    echo -e "${BLUE}========================================${NC}\n"

    # Update package manager cache
    print_status info "Updating package manager cache..."
    eval "sudo $PKG_UPDATE" || true

    # Install Rust if needed
    if [[ " ${MISSING_DEPS[@]} " =~ " rust " ]]; then
        echo -e "\n${YELLOW}Installing Rust...${NC}"
        print_status info "This will install Rust via rustup"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

        # Source the cargo environment
        if [[ -f "$HOME/.cargo/env" ]]; then
            source "$HOME/.cargo/env"
        fi

        print_status ok "Rust installed successfully"
    fi

    # Build list of packages to install
    local packages=()
    for dep in "${MISSING_DEPS[@]}"; do
        case $dep in
            rust)
                # Already handled above
                ;;
            build-essential)
                packages+=($BUILD_ESSENTIAL)
                ;;
            libssl-dev|openssl-devel|openssl-dev)
                packages+=($dep)
                ;;
            *)
                packages+=($dep)
                ;;
        esac
    done

    # Install system packages
    if [[ ${#packages[@]} -gt 0 ]]; then
        echo -e "\n${YELLOW}Installing system packages: ${packages[*]}${NC}"
        eval "sudo $PKG_INSTALL ${packages[*]}"
        print_status ok "System packages installed successfully"
    fi
}

# Main execution
main() {
    detect_os
    get_package_manager
    check_root

    print_status info "Detected OS: $OS_PRETTY"
    print_status info "Package Manager: $PKG_MANAGER"
    echo ""

    # Run all checks
    check_rust
    check_cc
    check_make
    check_pkg_config
    check_openssl
    check_git

    # Summary
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}Summary${NC}"
    echo -e "${BLUE}========================================${NC}\n"

    if [[ ${#MISSING_DEPS[@]} -eq 0 ]]; then
        print_status ok "All required dependencies are installed!"
        echo ""
        print_status info "You can now build Flux Framework:"
        echo -e "  ${GREEN}cargo build --release${NC}"
        echo ""
    else
        print_status fail "Missing dependencies: ${MISSING_DEPS[*]}"
        echo ""

        if [[ $INSTALL_MODE == true ]]; then
            install_dependencies
            echo ""
            print_status ok "Dependencies installed! Please restart your terminal and run:"
            echo -e "  ${GREEN}cargo build --release${NC}"
            echo ""
        else
            print_status info "To install missing dependencies, run:"
            echo -e "  ${YELLOW}sudo $0 --install${NC}"
            echo ""

            # Manual installation instructions
            echo -e "${BLUE}Or install manually:${NC}"
            case $OS_NAME in
                ubuntu|debian|linuxmint|pop)
                    echo -e "  ${YELLOW}sudo apt-get update${NC}"
                    if [[ " ${MISSING_DEPS[@]} " =~ " rust " ]]; then
                        echo -e "  ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
                    fi
                    local apt_packages=()
                    for dep in "${MISSING_DEPS[@]}"; do
                        [[ $dep != "rust" ]] && apt_packages+=($dep)
                    done
                    [[ ${#apt_packages[@]} -gt 0 ]] && echo -e "  ${YELLOW}sudo apt-get install -y ${apt_packages[*]}${NC}"
                    ;;
                rhel|centos|rocky|almalinux|fedora)
                    if [[ " ${MISSING_DEPS[@]} " =~ " rust " ]]; then
                        echo -e "  ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
                    fi
                    local rpm_packages=()
                    for dep in "${MISSING_DEPS[@]}"; do
                        [[ $dep != "rust" ]] && rpm_packages+=($dep)
                    done
                    [[ ${#rpm_packages[@]} -gt 0 ]] && echo -e "  ${YELLOW}sudo $PKG_INSTALL ${rpm_packages[*]}${NC}"
                    ;;
            esac
            echo ""
            exit 1
        fi
    fi

    # Display warnings
    if [[ ${#WARNINGS[@]} -gt 0 ]]; then
        echo -e "${YELLOW}Warnings:${NC}"
        for warning in "${WARNINGS[@]}"; do
            print_status warn "$warning"
        done
        echo ""
    fi
}

# Run main function
main
