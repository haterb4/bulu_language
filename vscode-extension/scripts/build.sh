#!/bin/bash
# Script de build complet pour l'extension Bulu VS Code

set -e

echo "🔨 Building Bulu VS Code Extension..."

# Vérifier que nous sommes dans le bon répertoire
if [ ! -f "package.json" ]; then
    echo "❌ Error: package.json not found. Run this script from vscode-extension directory."
    exit 1
fi

# Nettoyer les anciens builds
echo "🧹 Cleaning old builds..."
rm -rf out/
rm -f *.vsix

# Installer les dépendances
echo "📦 Installing dependencies..."
npm install

# Compiler TypeScript
echo "⚙️  Compiling TypeScript..."
npm run compile

# Vérifier que le serveur LSP est disponible
echo "🔍 Checking for bulu_lsp..."
if ! command -v bulu_lsp &> /dev/null; then
    echo "⚠️  Warning: bulu_lsp not found in PATH"
    echo "   The extension will work but LSP features require bulu_lsp to be installed"
    echo "   Install with: cargo install --path .. --bin bulu_lsp"
else
    echo "✅ bulu_lsp found: $(which bulu_lsp)"
fi

# Créer le package VSIX
echo "📦 Creating VSIX package..."
npx vsce package

# Trouver le fichier VSIX créé
VSIX_FILE=$(ls -t *.vsix | head -1)

if [ -f "$VSIX_FILE" ]; then
    echo ""
    echo "✅ Build successful!"
    echo "📦 Package created: $VSIX_FILE"
    echo ""
    echo "To install locally:"
    echo "  code --install-extension $VSIX_FILE"
    echo ""
    echo "To publish:"
    echo "  npx vsce publish"
else
    echo "❌ Error: VSIX package not created"
    exit 1
fi
