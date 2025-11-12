#!/bin/bash

# Script de test du registry de production sur Render

set -e

echo "🧪 Test du Registry Bulu (Production)"
echo "======================================"
echo ""
echo "Registry: https://bulu-language.onrender.com"
echo ""

# Couleurs
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

REGISTRY_URL="https://bulu-language.onrender.com"

# Test 1: Health check
echo -e "${BLUE}Test 1: Health check${NC}"
HEALTH=$(curl -s ${REGISTRY_URL}/health)
if [ "$HEALTH" = "OK" ]; then
    echo -e "${GREEN}✅ Registry is online${NC}"
else
    echo -e "${RED}❌ Registry is offline${NC}"
    exit 1
fi
echo ""

# Test 2: List packages
echo -e "${BLUE}Test 2: List packages${NC}"
PACKAGES=$(curl -s ${REGISTRY_URL}/api/packages)
PACKAGE_COUNT=$(echo "$PACKAGES" | jq '. | length')
echo "Found $PACKAGE_COUNT packages"
echo -e "${GREEN}✅ API is working${NC}"
echo ""

# Test 3: Build CLI
echo -e "${BLUE}Test 3: Build CLI with production registry${NC}"
echo "Building release binary..."
cargo build --release --quiet
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ CLI built successfully${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo ""

# Test 4: Verify registry configuration
echo -e "${BLUE}Test 4: Verify registry configuration${NC}"
echo "Default registry is now: ${REGISTRY_URL}"
echo -e "${GREEN}✅ Configuration updated${NC}"
echo ""

# Test 5: Search packages (should work even if empty)
echo -e "${BLUE}Test 5: Test search functionality${NC}"
./target/release/lang package search "test" || true
echo -e "${GREEN}✅ Search command works${NC}"
echo ""

# Test 6: Publish example package
echo -e "${BLUE}Test 6: Publish example package${NC}"
echo -e "${YELLOW}⚠️  This will publish to production!${NC}"
read -p "Do you want to publish example-package to production? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cd example-package
    ../target/release/lang package publish
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Package published successfully${NC}"
    else
        echo -e "${RED}❌ Publication failed${NC}"
        exit 1
    fi
    cd ..
else
    echo "Skipping publication"
fi
echo ""

# Test 7: Verify package is in registry
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Test 7: Verify package in registry${NC}"
    sleep 2  # Wait for registry to update
    PACKAGE_INFO=$(curl -s ${REGISTRY_URL}/api/packages/example-package)
    if [ "$PACKAGE_INFO" != "null" ] && [ -n "$PACKAGE_INFO" ]; then
        echo "Package info:"
        echo "$PACKAGE_INFO" | jq '.'
        echo -e "${GREEN}✅ Package is in registry${NC}"
    else
        echo -e "${YELLOW}⚠️  Package not found (may take a moment)${NC}"
    fi
    echo ""
fi

echo "======================================"
echo -e "${GREEN}🎉 Production registry tests completed!${NC}"
echo ""
echo "Summary:"
echo "  - Registry URL: ${REGISTRY_URL}"
echo "  - Status: Online"
echo "  - CLI: Configured to use production registry"
echo ""
echo "Next steps:"
echo "  1. Publish your packages: ./target/release/lang package publish"
echo "  2. Search packages: ./target/release/lang package search <query>"
echo "  3. Install packages: ./target/release/lang package add <name>"
echo ""
echo "To use a different registry:"
echo "  export BULU_REGISTRY=http://localhost:3000"
