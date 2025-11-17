# Guide d'Installation et de Publication - Extension Bulu VS Code

## Installation pour le Développement

### Prérequis
- Node.js 18+ et npm
- VS Code 1.75+
- Le serveur LSP Bulu installé

### Étapes

1. **Installer les dépendances**
```bash
cd vscode-extension
npm install
```

2. **Compiler l'extension**
```bash
npm run compile
```

3. **Tester en mode développement**
- Ouvrir le dossier `vscode-extension` dans VS Code
- Appuyer sur F5 pour lancer une nouvelle fenêtre VS Code avec l'extension
- Ouvrir un fichier `.bu` pour tester

4. **Compiler en mode watch (développement)**
```bash
npm run watch
```

## Créer le Package VSIX

### Installation de vsce
```bash
npm install -g @vscode/vsce
```

### Créer le package
```bash
cd vscode-extension
vsce package
```

Cela crée un fichier `bulu-language-0.1.0.vsix`

### Installer localement le package
```bash
code --install-extension bulu-language-0.1.0.vsix
```

## Publication sur le Marketplace VS Code

### 1. Créer un compte Publisher

1. Aller sur https://marketplace.visualstudio.com/
2. Se connecter avec un compte Microsoft
3. Créer un publisher sur https://marketplace.visualstudio.com/manage
4. Noter votre publisher ID

### 2. Obtenir un Personal Access Token (PAT)

1. Aller sur https://dev.azure.com/
2. Créer une organisation si nécessaire
3. User Settings > Personal Access Tokens
4. Créer un nouveau token avec les scopes :
   - **Marketplace** : Acquire, Manage

### 3. Se connecter avec vsce

```bash
vsce login <publisher-name>
# Entrer votre PAT quand demandé
```

### 4. Publier l'extension

```bash
cd vscode-extension
vsce publish
```

Ou publier une version spécifique :
```bash
vsce publish minor  # 0.1.0 -> 0.2.0
vsce publish patch  # 0.1.0 -> 0.1.1
vsce publish major  # 0.1.0 -> 1.0.0
```

## Publication sur Open VSX (Alternative)

Pour les utilisateurs de VSCodium et autres éditeurs compatibles :

### 1. Créer un compte sur Open VSX
https://open-vsx.org/

### 2. Obtenir un token
https://open-vsx.org/user-settings/tokens

### 3. Publier
```bash
npx ovsx publish bulu-language-0.1.0.vsix -p <token>
```

## Mise à Jour de l'Extension

### 1. Mettre à jour le code
- Faire les modifications nécessaires
- Mettre à jour CHANGELOG.md
- Tester localement

### 2. Incrémenter la version
Dans `package.json` :
```json
{
  "version": "0.2.0"
}
```

### 3. Créer un tag Git
```bash
git tag v0.2.0
git push origin v0.2.0
```

### 4. Publier
```bash
vsce publish
```

## Structure des Fichiers

```
vscode-extension/
├── package.json              # Manifeste de l'extension
├── tsconfig.json            # Configuration TypeScript
├── language-configuration.json  # Config du langage
├── README.md                # Documentation utilisateur
├── CHANGELOG.md             # Historique des versions
├── .vscodeignore           # Fichiers à exclure du package
├── src/
│   └── extension.ts        # Code principal de l'extension
├── syntaxes/
│   └── bulu.tmLanguage.json  # Grammaire TextMate
├── snippets/
│   └── bulu.json           # Snippets de code
└── images/
    ├── icon.png            # Icône de l'extension (128x128)
    └── file-icon.svg       # Icône des fichiers .bu
```

## Checklist Avant Publication

- [ ] Tester toutes les fonctionnalités
- [ ] Vérifier la coloration syntaxique
- [ ] Tester le LSP (completion, hover, etc.)
- [ ] Vérifier les snippets
- [ ] Mettre à jour README.md
- [ ] Mettre à jour CHANGELOG.md
- [ ] Incrémenter la version dans package.json
- [ ] Créer les icônes (128x128 pour l'extension)
- [ ] Tester le package VSIX localement
- [ ] Créer un tag Git
- [ ] Publier sur le Marketplace
- [ ] Publier sur Open VSX (optionnel)

## Icônes Requises

### Icon de l'extension (icon.png)
- Taille : 128x128 pixels
- Format : PNG
- Fond transparent recommandé

### Icône de fichier (file-icon.svg)
- Format : SVG
- Utilisé pour les fichiers .bu dans l'explorateur

Exemple de création avec ImageMagick :
```bash
# Créer un icône simple
convert -size 128x128 xc:transparent \
  -fill "#4A90E2" -draw "circle 64,64 64,10" \
  -fill white -pointsize 60 -gravity center \
  -annotate +0+0 "B" \
  images/icon.png
```

## Dépannage

### Erreur "Publisher not found"
- Vérifier que vous êtes connecté : `vsce login`
- Vérifier le publisher dans package.json

### Erreur de compilation TypeScript
```bash
npm run compile
# Vérifier les erreurs dans la sortie
```

### Extension ne se charge pas
- Vérifier les logs : Help > Toggle Developer Tools > Console
- Vérifier que bulu_lsp est installé

### Package trop volumineux
- Vérifier .vscodeignore
- Exclure node_modules, src/, etc.

## Ressources

- [VS Code Extension API](https://code.visualstudio.com/api)
- [Publishing Extensions](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)
- [Extension Manifest](https://code.visualstudio.com/api/references/extension-manifest)
- [TextMate Grammars](https://macromates.com/manual/en/language_grammars)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)

## Support

Pour toute question ou problème :
- GitHub Issues : https://github.com/bulu-lang/bulu/issues
- Documentation : https://github.com/bulu-lang/bulu
