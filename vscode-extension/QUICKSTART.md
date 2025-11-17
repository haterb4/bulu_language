# Quick Start - Extension Bulu VS Code

## Installation en 5 Minutes

### 1. Prérequis
```bash
# Vérifier Node.js
node --version  # Doit être >= 18

# Vérifier npm
npm --version

# Installer le serveur LSP Bulu
cd ..  # Retour au répertoire racine
cargo install --path . --bin bulu_lsp
```

### 2. Build de l'Extension
```bash
cd vscode-extension
npm install
npm run compile
```

### 3. Test en Mode Dev
```bash
# Dans VS Code, ouvrir le dossier vscode-extension
# Puis appuyer sur F5
```

### 4. Créer le Package
```bash
./scripts/build.sh
# Ou manuellement :
npx vsce package
```

### 5. Installer Localement
```bash
code --install-extension bulu-language-*.vsix
```

## Test Rapide

### Créer un fichier test
```bash
cat > test.bu << 'EOF'
func main() {
    println("Hello, Bulu!")
}
EOF
```

### Ouvrir dans VS Code
```bash
code test.bu
```

### Tester les Fonctionnalités

1. **Coloration** : Le code doit être coloré
2. **Snippets** : Taper `func` puis Tab
3. **Complétion** : Taper `pr` puis Ctrl+Space
4. **Hover** : Survoler `println`

## Commandes Utiles

```bash
# Compiler
npm run compile

# Watch mode
npm run watch

# Lint
npm run lint

# Package
npm run package

# Installer
code --install-extension *.vsix

# Désinstaller
code --uninstall-extension bulu-lang.bulu-language
```

## Dépannage Express

### LSP ne démarre pas
```bash
which bulu_lsp
# Si vide, réinstaller :
cargo install --path .. --bin bulu_lsp --force
```

### Erreur de compilation TypeScript
```bash
rm -rf node_modules out
npm install
npm run compile
```

### Extension ne se charge pas
1. Vérifier les logs : `Ctrl+Shift+P` → "Developer: Show Logs"
2. Recharger : `Ctrl+Shift+P` → "Reload Window"

## Prochaines Étapes

- 📖 Lire [README.md](README.md) pour la documentation complète
- 🇫🇷 Voir [GUIDE_RAPIDE.md](GUIDE_RAPIDE.md) en français
- 🚀 Consulter [INSTALLATION.md](INSTALLATION.md) pour la publication

Bon développement ! 🎉
