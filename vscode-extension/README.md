# Bulu Language Support for VS Code

Extension officielle pour le langage de programmation Bulu, offrant une coloration syntaxique complète, l'intégration LSP, et des fonctionnalités avancées d'édition.

## Fonctionnalités

### 🎨 Coloration Syntaxique
- Mise en évidence complète de la syntaxe Bulu
- Support des commentaires, chaînes de caractères, nombres
- Coloration des mots-clés, types, et fonctions
- Thèmes clairs et sombres

### 🚀 Language Server Protocol (LSP)
- **Auto-complétion** : Suggestions intelligentes pour les mots-clés, fonctions, et types
- **Go-to-Definition** : Navigation rapide vers les définitions (F12)
- **Find References** : Trouver toutes les utilisations d'un symbole (Shift+F12)
- **Hover Information** : Documentation au survol
- **Diagnostics en temps réel** : Détection d'erreurs pendant la frappe
- **Rename Refactoring** : Renommer des symboles partout (F2)
- **Code Actions** : Corrections rapides et refactorings (Ctrl+.)
- **Signature Help** : Aide sur les paramètres de fonction

### 📝 Snippets
Snippets prédéfinis pour :
- `func` - Déclaration de fonction
- `struct` - Déclaration de structure
- `if`, `for`, `while` - Structures de contrôle
- `match` - Pattern matching
- `try` - Gestion d'erreurs
- Et bien plus...

### ⚙️ Configuration
- Chemin personnalisable vers le serveur LSP
- Activation/désactivation du LSP
- Options de formatage et linting

## Installation

### Prérequis
1. Installer le compilateur Bulu et le serveur LSP :
```bash
cargo build --release --bin bulu_lsp
cargo install --path . --bin bulu_lsp
```

2. Vérifier que `bulu_lsp` est dans votre PATH :
```bash
bulu_lsp --version
```

### Installer l'extension

#### Depuis le Marketplace VS Code
1. Ouvrir VS Code
2. Aller dans Extensions (Ctrl+Shift+X)
3. Chercher "Bulu Language Support"
4. Cliquer sur "Install"

#### Installation manuelle
1. Télécharger le fichier `.vsix`
2. Dans VS Code : `Extensions > ... > Install from VSIX`
3. Sélectionner le fichier téléchargé

#### Depuis les sources
```bash
cd vscode-extension
npm install
npm run compile
npm run package
code --install-extension bulu-language-*.vsix
```

## Utilisation

1. Ouvrir un fichier `.bu`
2. L'extension s'active automatiquement
3. Le serveur LSP démarre en arrière-plan
4. Profiter des fonctionnalités !

### Raccourcis Clavier

| Fonctionnalité | Raccourci |
|----------------|-----------|
| Auto-complétion | `Ctrl+Space` |
| Go-to-Definition | `F12` |
| Find References | `Shift+F12` |
| Rename | `F2` |
| Code Actions | `Ctrl+.` |
| Hover Info | Survol souris |

### Commandes

- `Bulu: Restart Language Server` - Redémarrer le serveur LSP
- `Bulu: Show Output Channel` - Afficher les logs du serveur

## Configuration

Ouvrir les paramètres VS Code (Ctrl+,) et chercher "Bulu" :

```json
{
  // Activer/désactiver le serveur LSP
  "bulu.lsp.enabled": true,
  
  // Chemin vers l'exécutable bulu_lsp
  "bulu.lsp.path": "bulu_lsp",
  
  // Niveau de trace pour le débogage
  "bulu.lsp.trace.server": "off",
  
  // Formater à la sauvegarde
  "bulu.format.onSave": false,
  
  // Linter à la sauvegarde
  "bulu.lint.onSave": true
}
```

## Exemples

### Fonction simple
```bulu
func add(a: int32, b: int32): int32 {
    return a + b
}
```

### Structure avec méthodes
```bulu
struct Point {
    x: float64
    y: float64
}

func (p: Point) distance(): float64 {
    return math.sqrt(p.x * p.x + p.y * p.y)
}
```

### Concurrence
```bulu
func main() {
    let ch: chan int32 = make(chan int32)
    
    run {
        ch <- 42
    }
    
    let value = <- ch
    println(value)
}
```

## Dépannage

### Le serveur LSP ne démarre pas
1. Vérifier que `bulu_lsp` est installé :
   ```bash
   which bulu_lsp
   ```
2. Vérifier les logs : `Bulu: Show Output Channel`
3. Configurer le chemin manuellement dans les paramètres

### Pas d'auto-complétion
1. Sauvegarder le fichier
2. Vérifier que l'extension est activée
3. Redémarrer le serveur : `Bulu: Restart Language Server`

### Erreurs de syntaxe non détectées
1. Sauvegarder le fichier pour déclencher l'analyse
2. Vérifier que le LSP est activé dans les paramètres

## Contribuer

Les contributions sont les bienvenues !

1. Fork le projet
2. Créer une branche (`git checkout -b feature/amelioration`)
3. Commit les changements (`git commit -am 'Ajout fonctionnalité'`)
4. Push vers la branche (`git push origin feature/amelioration`)
5. Créer une Pull Request

## Licence

MIT License - voir le fichier LICENSE pour plus de détails.

## Liens

- [Documentation Bulu](https://github.com/bulu-lang/bulu)
- [Rapporter un bug](https://github.com/bulu-lang/bulu/issues)
- [Guide LSP](../docs/LSP_GUIDE.md)

## Changelog

### 0.1.0 (Initial Release)
- ✨ Coloration syntaxique complète
- ✨ Intégration LSP avec toutes les fonctionnalités
- ✨ Snippets pour les constructions courantes
- ✨ Configuration personnalisable
- ✨ Support des fichiers .bu
