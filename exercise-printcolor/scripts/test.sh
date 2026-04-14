#!/bin/bash

echo "=== ArceOS Printcolor Exercise Test ==="
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "$PROJECT_DIR"

ARCHS=("riscv64" "x86_64" "aarch64" "loongarch64")
EXPECTED_TEXT="Hello, Arceos!"

passed=0
failed=0
skipped=0
failed_archs=""

for arch in "${ARCHS[@]}"; do
    echo "Testing architecture: $arch"

    qemu_cmd="qemu-system-$arch"
    if ! command -v "$qemu_cmd" &> /dev/null; then
        echo "  ⊘ skipped ($qemu_cmd not found)"
        skipped=$((skipped + 1))
        echo ""
        continue
    fi

    output=$(AX_LOG=warn cargo xtask run --arch="$arch" 2>&1) || true

    tail_output=$(echo "$output" | tail -n 10)

    if ! echo "$tail_output" | grep -q "$EXPECTED_TEXT"; then
        echo "  ✗ expected text \"$EXPECTED_TEXT\" not found in output"
        echo "  --- last 10 lines ---"
        echo "$tail_output"
        echo "  ---------------------"
        failed=$((failed + 1))
        failed_archs="$failed_archs $arch"
        echo ""
        continue
    fi

    # Check the last 10 lines for any ANSI color-setting sequence
    # (e.g. \x1b[32m, \x1b[1;31m), excluding bare resets (\x1b[0m, \x1b[m).
    if echo "$tail_output" | grep -qP '\x1b\[[0-9;]*[1-9][0-9;]*m'; then
        echo "  ✓ colored output detected"
        passed=$((passed + 1))
    else
        echo "  ✗ no ANSI color codes found in output"
        echo "  Hint: use ANSI escape sequences like \"\\x1b[32m\" for colored text."
        echo "  --- last 10 lines (cat -v) ---"
        echo "$tail_output" | cat -v
        echo "  ------------------------------"
        failed=$((failed + 1))
        failed_archs="$failed_archs $arch"
    fi
    echo ""
done

echo "=================SUMMARY================="
echo "  Passed:  $passed / ${#ARCHS[@]}"
echo "  Failed:  $failed / ${#ARCHS[@]}"
echo "  Skipped: $skipped / ${#ARCHS[@]}"
echo "========================================="

if [ "$((passed + failed))" -eq 0 ]; then
    echo "Error: no QEMU emulators found, cannot run any tests"
    exit 1
fi

if [ "$failed" -gt 0 ]; then
    echo "Failed architectures:$failed_archs"
    exit 1
else
    echo "✓ All exercise tests passed!"
fi
