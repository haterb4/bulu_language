#!/bin/bash
# Test d'intégration complète avec Cloudflare R2

set -e

echo "🧪 Test d'intégration Bulu Registry + Cloudflare R2"
echo "=================================================="
echo ""

# Vérifier que les variables R2 sont définies
if [ -z "$R2_ACCOUNT_ID" ]; then
    echo "❌ R2_ACCOUNT_ID n'est pas défini"
    echo "   Copiez .env.example vers .env et configurez vos credentials R2"
    exit 1
fi

echo "✅ Variables R2 configurées"
echo ""

# Démarrer le registry server en arrière-plan
echo "🚀 Démarrage du registry server..."
cd registry-server
cargo build --release --quiet
./target/release/bulu-registry &
REGISTRY_PID=$!
cd ..

# Attendre que le serveur démarre
sleep 3

# Vérifier que le serveur est démarré
if ! curl -s http://localhost:3000/ > /dev/null; then
    echo "❌ Le registry server n'a pas démarré"
    kill $REGISTRY_PID 2>/dev/null || true
    exit 1
fi

echo "✅ Registry server démarré (PID: $REGISTRY_PID)"
echo ""

# Compiler lang
echo "🔨 Compilation de lang..."
cargo build --release --quiet --bin lang
echo "✅ lang compilé"
echo ""

# Publier le package exemple
echo "📦 Publication du package exemple..."
cd example-package
../target/release/lang publish --verbose

if [ $? -eq 0 ]; then
    echo "✅ Package publié avec succès"
else
    echo "❌ Échec de la publication"
    kill $REGISTRY_PID 2>/dev/null || true
    exit 1
fi
cd ..
echo ""

# Créer un projet de test
echo "🆕 Création d'un projet de test..."
rm -rf test-r2-project
mkdir -p test-r2-project/src

cat > test-r2-project/lang.toml << 'EOF'
[package]
name = "test-r2-project"
version = "0.1.0"
authors = ["Test"]

[dependencies]
example-package = "1.0.0"
EOF

cat > test-r2-project/src/main.bu << 'EOF'
import { Point } from "example-package"

func main() {
    let p = Point.new(10.0, 20.0)
    println("Point créé: (" + p.x.toString() + ", " + p.y.toString() + ")")
}
EOF

echo "✅ Projet de test créé"
echo ""

# Installer les dépendances
echo "📥 Installation des dépendances depuis R2..."
cd test-r2-project
../target/release/lang install --verbose

if [ $? -eq 0 ]; then
    echo "✅ Dépendances installées depuis R2"
else
    echo "❌ Échec de l'installation"
    cd ..
    kill $REGISTRY_PID 2>/dev/null || true
    exit 1
fi
cd ..
echo ""

# Nettoyer
echo "🧹 Nettoyage..."
kill $REGISTRY_PID 2>/dev/null || true
rm -rf test-r2-project

echo ""
echo "✅ Test d'intégration R2 réussi !"
echo ""
echo "Le workflow complet fonctionne :"
echo "  1. ✅ Publication du package"
echo "  2. ✅ Upload du tarball vers Cloudflare R2"
echo "  3. ✅ Téléchargement depuis R2"
echo "  4. ✅ Installation dans un nouveau projet"
