#!/bin/bash

echo "=== ArceOS HashMap Exercise Test ==="
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "$PROJECT_DIR"

ARCHS=("riscv64" "x86_64" "aarch64" "loongarch64")
EXPECTED_LINES=("test_hashmap() OK!" "Memory tests run OK!")

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

    output=$(cargo xtask run --arch="$arch" 2>&1) || true

    all_matched=true
    for expected in "${EXPECTED_LINES[@]}"; do
        if ! echo "$output" | grep -qF "$expected"; then
            echo "  ✗ expected text \"$expected\" not found in output"
            all_matched=false
        fi
    done

    if [ "$all_matched" = true ]; then
        echo "  ✓ all expected output matched"
        passed=$((passed + 1))
    else
        echo "  --- output ---"
        echo "$output"
        echo "  --------------"
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
