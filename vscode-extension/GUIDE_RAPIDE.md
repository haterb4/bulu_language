# Guide Rapide - Extension Bulu pour VS Code

## 🚀 Installation Rapide

### 1. Installer le serveur LSP Bulu

```bash
# Depuis le répertoire racine du projet Bulu
cargo build --release --bin bulu_lsp
cargo install --path . --bin bulu_lsp
```

### 2. Installer l'extension VS Code

#### Option A : Depuis les sources (développement)
```bash
cd vscode-extension
npm install
npm run compile
npm run package
code --install-extension bulu-language-*.vsix
```

#### Option B : Depuis le Marketplace (quand publié)
1. Ouvrir VS Code
2. Extensions (Ctrl+Shift+X)
3. Chercher "Bulu Language Support"
4. Cliquer "Install"

### 3. Vérifier l'installation

1. Créer un fichier `test.bu`
2. Taper `func` et appuyer sur Tab
3. Vous devriez voir le snippet de fonction se compléter

## 📝 Premiers Pas

### Créer votre premier programme

```bulu
// test.bu
func main() {
    println("Bonjour, Bulu!")
}
```

### Tester les fonctionnalités

1. **Auto-complétion** : Tapez `pr` puis Ctrl+Space
2. **Hover** : Survolez `println` avec la souris
3. **Go-to-Definition** : Ctrl+Click sur une fonction
4. **Snippets** : Tapez `func` puis Tab

## 🎨 Coloration Syntaxique

L'extension colore automatiquement :
- 🔵 Mots-clés : `func`, `let`, `if`, `for`, etc.
- 🟢 Types : `int32`, `string`, `bool`, etc.
- 🟡 Fonctions : `print`, `len`, `make`, etc.
- 🟠 Commentaires : `//` et `/* */`
- 🔴 Chaînes : `"texte"`, `'char'`, `` `template` ``

## ⌨️ Raccourcis Essentiels

| Action | Raccourci | Description |
|--------|-----------|-------------|
| Complétion | `Ctrl+Space` | Afficher les suggestions |
| Définition | `F12` | Aller à la définition |
| Références | `Shift+F12` | Trouver les références |
| Renommer | `F2` | Renommer un symbole |
| Actions | `Ctrl+.` | Corrections rapides |
| Formater | `Shift+Alt+F` | Formater le document |

## 📚 Snippets Disponibles

Tapez le préfixe puis Tab :

- `func` → Fonction
- `afunc` → Fonction async
- `struct` → Structure
- `if` → Condition if
- `for` → Boucle for
- `while` → Boucle while
- `match` → Pattern matching
- `try` → Gestion d'erreurs
- `main` → Fonction main
- `print` → Print
- `println` → Print line

## 🔧 Configuration

### Paramètres Recommandés

Ouvrir les paramètres (Ctrl+,) et ajouter :

```json
{
  // Bulu
  "bulu.lsp.enabled": true,
  "bulu.lsp.path": "bulu_lsp",
  "bulu.format.onSave": true,
  "bulu.lint.onSave": true,
  
  // Éditeur
  "[bulu]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "bulu-lang.bulu-language",
    "editor.tabSize": 4,
    "editor.insertSpaces": true
  }
}
```

### Chemin Personnalisé pour LSP

Si `bulu_lsp` n'est pas dans votre PATH :

```json
{
  "bulu.lsp.path": "/chemin/complet/vers/bulu_lsp"
}
```

## 🐛 Dépannage Rapide

### Le LSP ne démarre pas

```bash
# Vérifier l'installation
which bulu_lsp

# Tester manuellement
bulu_lsp --version
```

Si non trouvé, réinstaller :
```bash
cargo install --path . --bin bulu_lsp --force
```

### Pas de coloration syntaxique

1. Vérifier que le fichier a l'extension `.bu`
2. Recharger VS Code : `Ctrl+Shift+P` → "Reload Window"

### Pas d'auto-complétion

1. Sauvegarder le fichier (Ctrl+S)
2. Vérifier les logs : `Ctrl+Shift+P` → "Bulu: Show Output Channel"
3. Redémarrer le LSP : `Ctrl+Shift+P` → "Bulu: Restart Language Server"

## 💡 Astuces

### 1. Utiliser les Snippets Efficacement
Tapez le début d'un snippet et appuyez sur Tab pour naviguer entre les placeholders.

### 2. Navigation Rapide
- `Ctrl+P` : Ouvrir un fichier rapidement
- `Ctrl+Shift+O` : Naviguer entre les symboles du fichier
- `Ctrl+T` : Chercher un symbole dans tout le projet

### 3. Multi-curseurs
- `Alt+Click` : Ajouter un curseur
- `Ctrl+Alt+↑/↓` : Ajouter un curseur au-dessus/en-dessous
- `Ctrl+D` : Sélectionner l'occurrence suivante

### 4. Refactoring Rapide
1. Sélectionner du code
2. `Ctrl+.` pour voir les actions disponibles
3. Choisir "Extract function" ou autre

## 📖 Exemples de Code

### Hello World
```bulu
func main() {
    println("Hello, World!")
}
```

### Fonction avec Types
```bulu
func add(a: int32, b: int32): int32 {
    return a + b
}
```

### Structure
```bulu
struct Person {
    name: string
    age: int32
}

func (p: Person) greet() {
    println("Bonjour, je suis " + p.name)
}
```

### Concurrence
```bulu
func main() {
    let ch: chan int32 = make(chan int32)
    
    run {
        ch <- 42
    }
    
    let result = <- ch
    println(result)
}
```

### Gestion d'Erreurs
```bulu
func divide(a: int32, b: int32): int32 {
    try {
        if b == 0 {
            fail "Division par zéro"
        }
        return a / b
    } fail err {
        println("Erreur: " + err)
        return 0
    }
}
```

## 🎯 Prochaines Étapes

1. ✅ Installer l'extension
2. ✅ Créer votre premier fichier `.bu`
3. ✅ Tester les snippets et l'auto-complétion
4. 📚 Lire la [documentation complète](README.md)
5. 🚀 Commencer à coder en Bulu !

## 🆘 Besoin d'Aide ?

- 📖 [Documentation complète](README.md)
- 🐛 [Rapporter un bug](https://github.com/bulu-lang/bulu/issues)
- 💬 [Discussions](https://github.com/bulu-lang/bulu/discussions)
- 📧 Support : support@bulu-lang.org

---

**Bon codage avec Bulu ! 🎉**
