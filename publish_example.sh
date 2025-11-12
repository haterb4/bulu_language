#!/bin/bash

# Script pour publier le package example-package sur le registry local

echo "📦 Publication de math-utils sur le registry local"
echo "=================================================="
echo ""

# Créer un tarball du package
echo "1️⃣  Création du tarball..."
cd example-package
tar czf ../math-utils-1.0.0.tar.gz src/ lang.toml README.md
cd ..

# Encoder en base64
echo "2️⃣  Encodage en base64..."
TARBALL=$(base64 -w 0 math-utils-1.0.0.tar.gz)

# Publier sur le registry
echo "3️⃣  Publication sur http://localhost:3000..."
curl -s -X POST http://localhost:3000/api/publish \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"math-utils\",
    \"version\": \"1.0.0\",
    \"description\": \"Utilitaires mathématiques pour Bulu\",
    \"authors\": [\"Bulu Team <team@bulu-lang.org>\"],
    \"license\": \"MIT\",
    \"repository\": \"https://github.com/bulu-lang/math-utils\",
    \"keywords\": [\"math\", \"utils\", \"geometry\"],
    \"dependencies\": {},
    \"tarball\": \"$TARBALL\"
  }" | jq .

echo ""
echo "4️⃣  Vérification..."
curl -s http://localhost:3000/api/packages | jq .

echo ""
echo "✅ Publication terminée!"
echo ""
echo "Pour utiliser ce package:"
echo "  lang add math-utils --registry http://localhost:3000"
