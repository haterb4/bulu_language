# Extension VS Code pour Bulu

## Vue d'Ensemble

L'extension officielle VS Code pour le langage Bulu fournit une expérience de développement complète avec :

- 🎨 **Coloration syntaxique** complète et précise
- 🚀 **Intégration LSP** pour toutes les fonctionnalités IDE
- 📝 **Snippets** pour accélérer le développement
- ⚙️ **Configuration** flexible et personnalisable

## Structure du Projet

```
vscode-extension/
├── package.json                    # Manifeste de l'extension
├── tsconfig.json                   # Configuration TypeScript
├── language-configuration.json     # Configuration du langage
├── README.md                       # Documentation utilisateur
├── CHANGELOG.md                    # Historique des versions
├── GUIDE_RAPIDE.md                # Guide de démarrage rapide
├── INSTALLATION.md                # Guide d'installation détaillé
├── .vscodeignore                  # Fichiers exclus du package
├── .eslintrc.json                 # Configuration ESLint
├── .gitignore                     # Fichiers Git ignorés
│
├── src/
│   └── extension.ts               # Code principal TypeScript
│
├── syntaxes/
│   └── bulu.tmLanguage.json       # Grammaire TextMate
│
├── snippets/
│   └── bulu.json                  # Snippets de code
│
├── images/
│   ├── icon.png                   # Icône extension (128x128)
│   └── file-icon.svg              # Icône fichiers .bu
│
└── scripts/
    └── build.sh                   # Script de build automatisé
```

## Fonctionnalités Implémentées

### 1. Coloration Syntaxique (TextMate Grammar)

La grammaire TextMate dans `syntaxes/bulu.tmLanguage.json` fournit :

- **Commentaires** : Ligne (`//`) et bloc (`/* */`)
- **Chaînes** : Double quote, simple quote, backtick avec échappements
- **Nombres** : Décimal, hexadécimal, binaire, octal, flottants
- **Mots-clés** :
  - Contrôle : `if`, `else`, `while`, `for`, `break`, `continue`, `return`, `match`, `select`, `defer`, `try`, `fail`
  - Déclaration : `let`, `const`, `func`, `struct`, `interface`, `type`, `import`, `export`
  - Opérateurs : `and`, `or`, `not`, `as`, `in`
  - Autres : `async`, `await`, `run`, `chan`, `lock`, `yield`, `pub`, `priv`
- **Types** :
  - Primitifs : `int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`, `int`, `uint`, `float32`, `float64`, `bool`, `char`, `string`, `byte`, `rune`, `any`, `void`
  - Collections : `array`, `slice`, `map`, `chan`
- **Fonctions** : Détection des appels et fonctions built-in
- **Constantes** : `true`, `false`, `null`, `nil`
- **Opérateurs** : Arithmétiques, comparaison, logiques, bitwise, assignation

### 2. Configuration du Langage

`language-configuration.json` définit :

- **Commentaires** : Ligne et bloc
- **Brackets** : `{}`, `[]`, `()`
- **Auto-closing pairs** : Brackets, quotes
- **Surrounding pairs** : Pour sélection
- **Folding** : Régions de code pliables
- **Indentation** : Règles automatiques

### 3. Snippets

20+ snippets dans `snippets/bulu.json` :

| Préfixe | Description |
|---------|-------------|
| `func` | Fonction standard |
| `afunc` | Fonction async |
| `struct` | Structure |
| `interface` | Interface |
| `if` | Condition if |
| `ifelse` | If-else |
| `for` | Boucle for |
| `while` | Boucle while |
| `match` | Pattern matching |
| `try` | Try-fail |
| `let` | Variable |
| `const` | Constante |
| `print` | Print |
| `println` | Print line |
| `main` | Fonction main |
| `import` | Import |
| `run` | Goroutine |
| `chan` | Channel |
| `select` | Select |
| `defer` | Defer |

### 4. Intégration LSP

Le code TypeScript dans `src/extension.ts` :

- **Démarre automatiquement** le serveur LSP `bulu_lsp`
- **Gère les erreurs** avec messages utilisateur clairs
- **Fournit des commandes** :
  - `Bulu: Restart Language Server`
  - `Bulu: Show Output Channel`
- **Configuration flexible** :
  - Chemin personnalisable vers `bulu_lsp`
  - Activation/désactivation du LSP
  - Niveaux de trace pour débogage

### 5. Configuration Utilisateur

Paramètres disponibles :

```json
{
  "bulu.lsp.enabled": true,
  "bulu.lsp.path": "bulu_lsp",
  "bulu.lsp.trace.server": "off",
  "bulu.format.onSave": false,
  "bulu.lint.onSave": true
}
```

## Développement

### Prérequis

- Node.js 18+
- npm
- VS Code 1.75+
- TypeScript 5.0+

### Installation des Dépendances

```bash
cd vscode-extension
npm install
```

### Compilation

```bash
# Compilation unique
npm run compile

# Mode watch (recompile automatiquement)
npm run watch
```

### Test en Mode Développement

1. Ouvrir `vscode-extension` dans VS Code
2. Appuyer sur `F5`
3. Une nouvelle fenêtre VS Code s'ouvre avec l'extension chargée
4. Ouvrir un fichier `.bu` pour tester

### Linting

```bash
npm run lint
```

### Création du Package

```bash
# Avec le script automatisé
./scripts/build.sh

# Ou manuellement
npm run compile
npx vsce package
```

Cela crée `bulu-language-0.1.0.vsix`

### Installation Locale

```bash
code --install-extension bulu-language-0.1.0.vsix
```

## Publication

### Sur le Marketplace VS Code

1. **Créer un compte publisher** sur https://marketplace.visualstudio.com/
2. **Obtenir un PAT** (Personal Access Token) depuis Azure DevOps
3. **Se connecter** :
   ```bash
   npx vsce login <publisher-name>
   ```
4. **Publier** :
   ```bash
   npx vsce publish
   ```

### Sur Open VSX (VSCodium)

```bash
npx ovsx publish bulu-language-0.1.0.vsix -p <token>
```

Voir [INSTALLATION.md](../vscode-extension/INSTALLATION.md) pour les détails complets.

## Architecture Technique

### Extension TypeScript

L'extension utilise :
- **vscode-languageclient** : Client LSP officiel
- **Async/await** : Gestion asynchrone moderne
- **Error handling** : Gestion robuste des erreurs
- **Output channel** : Logs détaillés pour débogage

### Activation

L'extension s'active automatiquement quand :
- Un fichier `.bu` est ouvert
- La commande `onLanguage:bulu` est déclenchée

### Communication LSP

```
VS Code Extension (TypeScript)
    ↓ stdio
bulu_lsp (Rust)
    ↓ tower-lsp
LSP Protocol
```

### Grammaire TextMate

Format JSON avec :
- **Patterns** : Expressions régulières pour matching
- **Scopes** : Noms sémantiques pour coloration
- **Repository** : Réutilisation de patterns

## Tests

### Tests Manuels

1. **Coloration** :
   - Ouvrir un fichier `.bu`
   - Vérifier que tous les éléments sont colorés correctement

2. **Snippets** :
   - Taper chaque préfixe + Tab
   - Vérifier l'expansion et la navigation

3. **LSP** :
   - Auto-complétion : `Ctrl+Space`
   - Hover : Survol souris
   - Go-to-definition : `F12`
   - Find references : `Shift+F12`
   - Rename : `F2`

4. **Commandes** :
   - `Ctrl+Shift+P` → "Bulu: Restart Language Server"
   - `Ctrl+Shift+P` → "Bulu: Show Output Channel"

### Tests Automatisés (À venir)

```bash
npm test
```

## Maintenance

### Mise à Jour de la Grammaire

Éditer `syntaxes/bulu.tmLanguage.json` :
1. Ajouter/modifier les patterns
2. Tester avec `F5`
3. Valider avec https://macromates.com/manual/en/language_grammars

### Ajout de Snippets

Éditer `snippets/bulu.json` :
```json
{
  "Nom du Snippet": {
    "prefix": "prefixe",
    "body": [
      "ligne 1",
      "ligne 2 avec ${1:placeholder}"
    ],
    "description": "Description"
  }
}
```

### Mise à Jour du LSP

Le client LSP se met à jour automatiquement si `bulu_lsp` est mis à jour.

## Roadmap

### Version 0.2.0
- [ ] Thème de couleurs Bulu personnalisé
- [ ] Support du formatage automatique
- [ ] Linting avancé
- [ ] Tests d'intégration

### Version 0.3.0
- [ ] Refactorings avancés
- [ ] Code lens
- [ ] Inlay hints
- [ ] Semantic highlighting

### Version 1.0.0
- [ ] Débogueur intégré
- [ ] Profiler
- [ ] Documentation generator
- [ ] Support multi-workspace complet

## Ressources

### Documentation
- [VS Code Extension API](https://code.visualstudio.com/api)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [TextMate Grammars](https://macromates.com/manual/en/language_grammars)
- [vscode-languageclient](https://github.com/microsoft/vscode-languageserver-node)

### Outils
- [vsce](https://github.com/microsoft/vscode-vsce) - Packaging et publication
- [ovsx](https://github.com/eclipse/openvsx) - Open VSX Registry
- [yo code](https://github.com/microsoft/vscode-generator-code) - Générateur d'extensions

### Exemples
- [Rust Extension](https://github.com/rust-lang/rust-analyzer/tree/master/editors/code)
- [Go Extension](https://github.com/golang/vscode-go)
- [Python Extension](https://github.com/microsoft/vscode-python)

## Contribution

Les contributions sont bienvenues ! Voir le guide principal de contribution du projet Bulu.

### Zones d'Amélioration

1. **Grammaire** : Améliorer la précision de la coloration
2. **Snippets** : Ajouter plus de snippets utiles
3. **Tests** : Ajouter des tests automatisés
4. **Documentation** : Améliorer les exemples
5. **Performance** : Optimiser le démarrage

## Support

- GitHub Issues : https://github.com/bulu-lang/bulu/issues
- Documentation : https://github.com/bulu-lang/bulu
- Email : support@bulu-lang.org

## Licence

MIT License - Voir LICENSE dans le répertoire racine du projet.
