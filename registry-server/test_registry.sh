#!/bin/bash

# Script de test pour le registry Bulu

BASE_URL="http://localhost:3000"

echo "🧪 Test du Registry Bulu"
echo "======================="
echo ""

# Test 1: Root endpoint
echo "1️⃣  Test GET /"
curl -s "$BASE_URL/" | jq .
echo ""

# Test 2: Liste des packages (vide au début)
echo "2️⃣  Test GET /api/packages"
curl -s "$BASE_URL/api/packages" | jq .
echo ""

# Test 3: Publier un package de test
echo "3️⃣  Test POST /api/publish"

# Créer un tarball de test (vide pour simplifier)
TARBALL=$(echo "test content" | base64)

curl -s -X POST "$BASE_URL/api/publish" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"test-package\",
    \"version\": \"1.0.0\",
    \"description\": \"Un package de test\",
    \"authors\": [\"Test Author <test@example.com>\"],
    \"license\": \"MIT\",
    \"repository\": \"https://github.com/test/test-package\",
    \"keywords\": [\"test\", \"example\"],
    \"dependencies\": {},
    \"tarball\": \"$TARBALL\"
  }" | jq .
echo ""

# Test 4: Lister les packages (devrait contenir test-package)
echo "4️⃣  Test GET /api/packages (après publication)"
curl -s "$BASE_URL/api/packages" | jq .
echo ""

# Test 5: Obtenir les infos du package
echo "5️⃣  Test GET /api/packages/test-package"
curl -s "$BASE_URL/api/packages/test-package" | jq .
echo ""

# Test 6: Obtenir les versions
echo "6️⃣  Test GET /api/packages/test-package/versions"
curl -s "$BASE_URL/api/packages/test-package/versions" | jq .
echo ""

# Test 7: Obtenir une version spécifique
echo "7️⃣  Test GET /api/packages/test-package/1.0.0"
curl -s "$BASE_URL/api/packages/test-package/1.0.0" | jq .
echo ""

# Test 8: Rechercher
echo "8️⃣  Test GET /api/search?q=test"
curl -s "$BASE_URL/api/search?q=test" | jq .
echo ""

# Test 9: Télécharger
echo "9️⃣  Test GET /api/download/test-package/1.0.0"
curl -s "$BASE_URL/api/download/test-package/1.0.0" | base64
echo ""

echo "✅ Tests terminés!"
