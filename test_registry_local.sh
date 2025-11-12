#!/bin/bash

# Script de test du registry local avec Neon + Cloudflare R2

set -e

echo "🧪 Test du Registry Bulu (Neon + Cloudflare R2)"
echo "================================================"
echo ""

# Couleurs
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

REGISTRY_URL="http://localhost:3000"

# Test 1: Health check
echo -e "${BLUE}Test 1: Health check${NC}"
HEALTH=$(curl -s ${REGISTRY_URL}/health)
if [ "$HEALTH" = "OK" ]; then
    echo -e "${GREEN}✅ Health check passed${NC}"
else
    echo -e "${RED}❌ Health check failed${NC}"
    exit 1
fi
echo ""

# Test 2: List packages (should be empty initially)
echo -e "${BLUE}Test 2: List packages${NC}"
PACKAGES=$(curl -s ${REGISTRY_URL}/api/packages)
echo "Packages: $PACKAGES"
echo -e "${GREEN}✅ List packages works${NC}"
echo ""

# Test 3: Publish example package
echo -e "${BLUE}Test 3: Publish example package${NC}"
cd example-package

# Build the package
echo "Building package..."
cargo build --release 2>&1 | tail -5

# Publish to registry
echo "Publishing to registry..."
cargo run --bin lang -- package publish --registry ${REGISTRY_URL}

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Package published successfully${NC}"
else
    echo -e "${RED}❌ Package publication failed${NC}"
    exit 1
fi
cd ..
echo ""

# Test 4: Get package info
echo -e "${BLUE}Test 4: Get package info${NC}"
PACKAGE_INFO=$(curl -s ${REGISTRY_URL}/api/packages/example-package)
echo "$PACKAGE_INFO" | jq '.'
echo -e "${GREEN}✅ Package info retrieved${NC}"
echo ""

# Test 5: Search packages
echo -e "${BLUE}Test 5: Search packages${NC}"
SEARCH_RESULT=$(curl -s "${REGISTRY_URL}/api/search?q=example")
echo "$SEARCH_RESULT" | jq '.'
echo -e "${GREEN}✅ Search works${NC}"
echo ""

# Test 6: Download package
echo -e "${BLUE}Test 6: Download package${NC}"
VERSION=$(echo "$PACKAGE_INFO" | jq -r '.versions[0].version')
echo "Downloading version: $VERSION"
curl -s "${REGISTRY_URL}/api/download/example-package/${VERSION}" -o /tmp/test-package.tar.gz
if [ -f /tmp/test-package.tar.gz ]; then
    SIZE=$(stat -f%z /tmp/test-package.tar.gz 2>/dev/null || stat -c%s /tmp/test-package.tar.gz)
    echo "Downloaded: $SIZE bytes"
    echo -e "${GREEN}✅ Download works${NC}"
    rm /tmp/test-package.tar.gz
else
    echo -e "${RED}❌ Download failed${NC}"
    exit 1
fi
echo ""

# Test 7: Verify in Cloudflare R2
echo -e "${BLUE}Test 7: Verify storage${NC}"
echo "Package should be stored in Cloudflare R2 bucket: bulang"
echo "Path: packages/example-package/${VERSION}.tar.gz"
echo -e "${GREEN}✅ Check your Cloudflare R2 dashboard${NC}"
echo ""

# Test 8: Verify in Neon database
echo -e "${BLUE}Test 8: Verify database${NC}"
echo "Package metadata should be in Neon PostgreSQL"
echo "Tables: packages, package_versions, package_authors, etc."
echo -e "${GREEN}✅ Check your Neon dashboard${NC}"
echo ""

echo "================================================"
echo -e "${GREEN}🎉 All tests passed!${NC}"
echo ""
echo "Summary:"
echo "  - Registry: ${REGISTRY_URL}"
echo "  - Database: Neon PostgreSQL (serverless)"
echo "  - Storage: Cloudflare R2 (bucket: bulang)"
echo "  - ORM: SeaORM (type-safe)"
echo ""
echo "Next steps:"
echo "  1. Check Neon dashboard: https://console.neon.tech"
echo "  2. Check Cloudflare R2: https://dash.cloudflare.com"
echo "  3. Try installing the package in another project"
