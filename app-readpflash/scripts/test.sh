#!/bin/bash
set -e

echo "=== ArceOS Helloworld Test Script ==="
echo ""

# 定义要测试的架构列表
ARCHS=("riscv64" "x86_64" "aarch64" "loongarch64")

# Check if required tools are installed
check_tools() {
    echo "[1/7] Checking required tools..."
    
    if ! command -v cargo &> /dev/null; then
        echo "Error: cargo is not installed"
        exit 1
    fi
    
    if ! command -v rust-objcopy &> /dev/null; then
        echo "Warning: cargo-binutils not installed, installing..."
        cargo install cargo-binutils
    fi
    
    echo "✓ All required tools are available"
    echo ""
}

# 为指定架构设置配置文件
setup_config() {
    local arch=$1
    cp configs/${arch}.toml .axconfig.toml
}

# Format check for all architectures
check_format() {
    echo "[2/7] Checking code format for all architectures..."
    
    for arch in "${ARCHS[@]}"; do
        echo "  Checking format for $arch..."
        setup_config "$arch"
        cargo fmt -- --check
    done
    
    echo "✓ Code format check passed for all architectures"
    echo ""
}

# Clippy lint check for all architectures
check_clippy() {
    echo "[3/7] Running clippy lint checks for all architectures..."
    
    local arch_info=(
        "riscv64" "riscv64gc-unknown-none-elf"
        "x86_64" "x86_64-unknown-none"
        "aarch64" "aarch64-unknown-none-softfloat"
        "loongarch64" "loongarch64-unknown-none"
    )
    
    for ((i=0; i<${#arch_info[@]}; i+=2)); do
        local arch=${arch_info[$i]}
        local target=${arch_info[$i+1]}
        
        echo "  Running clippy for $arch..."
        setup_config "$arch"
        cargo clippy --target "$target" -- -D warnings
    done
    
    echo "✓ Clippy check passed for all architectures"
    echo ""
}

# Basic build check (no default features to avoid platform-specific issues)
check_build() {
    echo "[4/7] Checking basic build (no default features)..."
    # 使用默认架构进行基本构建检查
    setup_config "riscv64"
    cargo check --no-default-features
    echo "✓ Basic build check passed"
    echo ""
}

# Run tests for each architecture
run_arch_tests() {
    echo "[5/7] Running architecture-specific tests..."
    
    local qemu_ok=true
    
    for arch in "${ARCHS[@]}"; do
        echo ""
        echo "Testing architecture: $arch"
        
        # Check if QEMU is available
        qemu_cmd="qemu-system-$arch"
        if ! command -v "$qemu_cmd" &> /dev/null; then
            echo "Warning: $qemu_cmd not found, skipping run test for $arch"
            qemu_ok=false
            continue
        fi
        
        # Build and run
        if cargo xtask run --arch="$arch" 2>&1 | grep -q "Got pflash magic: PFLA"; then
            echo "✓ $arch test passed"
        else
            echo "Error: $arch test failed"
            exit 1
        fi
    done
    
    if [ "$qemu_ok" = true ]; then
        echo ""
        echo "✓ All architecture tests passed"
    fi
    echo ""
}

# Publish dry-run check
# Publish dry-run check by architecture
check_publish() {
    echo "[6/7] Checking publish readiness..."
    
    local archs=("riscv64" "x86_64" "aarch64" "loongarch64")
    local targets=("riscv64gc-unknown-none-elf" "x86_64-unknown-none" "aarch64-unknown-none-softfloat" "loongarch64-unknown-none")
    
    for i in "${!archs[@]}"; do
        local arch="${archs[$i]}"
        local target="${targets[$i]}"
        
        echo ""
        echo "Checking publish for architecture: $arch"
        
        # Install config file for the architecture
        cp "configs/${arch}.toml" ".axconfig.toml"
        
        if cargo publish --dry-run --allow-dirty --target="$target" --registry crates-io; then
            echo "✓ $arch publish check passed"
        else
            echo "Error: $arch publish check failed"
            rm -f .axconfig.toml
            exit 1
        fi
    done
    
    rm -f .axconfig.toml
    echo ""
    echo "✓ All architecture publish checks passed"
    echo ""
}


# Summary
print_summary() {
    echo "[7/7] Test Summary"
    echo "=================="
    echo "✓ All checks passed successfully!"
    echo ""
    echo "The following checks were performed:"
    echo "  1. Code format check (cargo fmt) for all architectures"
    echo "  2. Lint check (cargo clippy) for all architectures"
    echo "  3. Basic build check (cargo check)"
    echo "  4. Architecture tests (${ARCHS[*]})"
    echo "  5. Publish readiness check (cargo publish --dry-run)"
    echo ""
}

# Main execution
main() {
    local skip_qemu=${SKIP_QEMU:-false}
    
    check_tools
    check_format
    check_clippy
    check_build
    
    if [ "$skip_qemu" = "true" ]; then
        echo "[5/7] Skipping architecture tests (SKIP_QEMU=true)"
        echo ""
    else
        run_arch_tests
    fi
    
    check_publish
    print_summary
}

# Run main function
main "$@"