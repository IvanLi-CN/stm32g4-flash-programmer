#!/bin/bash

set -e

echo "🐾 白羽的 CI 测试脚本开始运行..."

echo ""
echo "=== 1. Check 步骤 ==="
echo "测试 host-tool 检查..."
cd host-tool && cargo check && cd ..
echo "✅ host-tool 检查通过"

echo "测试 firmware 检查..."
cd firmware && cargo check --target thumbv7em-none-eabihf && cd ..
echo "✅ firmware 检查通过"

echo "测试 protocol 检查..."
cd protocol && cargo check && cd ..
echo "✅ protocol 检查通过"

echo ""
echo "=== 2. Test 步骤 ==="
echo "测试 host-tool 测试..."
cd host-tool && cargo test && cd ..
echo "✅ host-tool 测试通过"

echo "测试 protocol 测试..."
cd protocol && cargo test && cd ..
echo "✅ protocol 测试通过"

echo ""
echo "=== 3. Lints 步骤 ==="
echo "测试 host-tool fmt..."
cd host-tool && cargo fmt --all -- --check && cd ..
echo "✅ host-tool 格式检查通过"

echo "测试 firmware fmt..."
cd firmware && cargo fmt --all -- --check && cd ..
echo "✅ firmware 格式检查通过"

echo "测试 protocol fmt..."
cd protocol && cargo fmt --all -- --check && cd ..
echo "✅ protocol 格式检查通过"

echo "测试 host-tool clippy..."
cd host-tool && cargo clippy -- -D warnings && cd ..
echo "✅ host-tool Clippy 检查通过"

echo "测试 firmware clippy..."
cd firmware && cargo clippy --target thumbv7em-none-eabihf -- -D warnings && cd ..
echo "✅ firmware Clippy 检查通过"

echo "测试 protocol clippy..."
cd protocol && cargo clippy -- -D warnings && cd ..
echo "✅ protocol Clippy 检查通过"

echo ""
echo "=== 4. Build 步骤 ==="
echo "测试 host-tool 构建..."
cd host-tool && cargo build && cd ..
echo "✅ host-tool 构建通过"

echo "测试 firmware 构建..."
cd firmware && cargo build --target thumbv7em-none-eabihf && cd ..
echo "✅ firmware 构建通过"

echo "测试 protocol 构建..."
cd protocol && cargo build && cd ..
echo "✅ protocol 构建通过"

echo ""
echo "🎉 所有 CI 步骤都通过了！白羽成功修复了 CI 问题！"
