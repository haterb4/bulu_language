# 🎨 Démonstration des Fonctionnalités - Extension Bulu

Ce document présente visuellement toutes les fonctionnalités de l'extension Bulu pour VS Code.

## 🎨 1. Coloration Syntaxique

### Exemple de Code Coloré

```bulu
// Commentaire ligne - gris
/* Commentaire bloc - gris */

// Imports - violet
import std.io
import std.math

// Constantes - bleu clair
const PI: float64 = 3.14159
const MAX_SIZE: int32 = 1000

// Structure - orange
struct Point {
    x: float64    // Types - vert
    y: float64
}

// Fonction - jaune
func distance(p1: Point, p2: Point): float64 {
    let dx = p2.x - p1.x  // Mots-clés - bleu
    let dy = p2.y - p1.y
    return math.sqrt(dx * dx + dy * dy)  // Built-in - cyan
}

// Fonction async - jaune + bleu
async func fetchData(url: string): string {
    let response = await http.get(url)  // await - bleu
    return response.body
}

// Concurrence - bleu
func main() {
    let ch: chan int32 = make(chan int32)  // chan - vert
    
    run {  // run - bleu
        ch <- 42  // Opérateurs - blanc
    }
    
    let value = <- ch
    println(value)  // Built-in - cyan
    
    // Pattern matching
    match value {
        0 -> println("zero")
        1..10 -> println("petit")  // Range - blanc
        _ -> println("autre")
    }
    
    // Gestion d'erreurs
    try {
        if value < 0 {
            fail "Valeur négative"  // String - orange
        }
    } fail err {
        println("Erreur: " + err)
    }
}
```

### Légende des Couleurs

- 🔵 **Bleu** : Mots-clés (if, func, let, async, await, run, etc.)
- 🟢 **Vert** : Types (int32, string, bool, chan, etc.)
- 🟡 **Jaune** : Noms de fonctions
- 🔷 **Cyan** : Fonctions built-in (print, len, make, etc.)
- 🟠 **Orange** : Chaînes de caractères et structures
- ⚪ **Blanc** : Opérateurs et ponctuation
- ⚫ **Gris** : Commentaires

## 📝 2. Snippets

### Utilisation des Snippets

Tapez le préfixe puis appuyez sur **Tab** :

#### `func` → Fonction
```bulu
func name(params): returnType {
    // body
}
```

#### `struct` → Structure
```bulu
struct Name {
    field: type
}
```

#### `if` → Condition
```bulu
if condition {
    
}
```

#### `for` → Boucle
```bulu
for item in collection {
    
}
```

#### `match` → Pattern Matching
```bulu
match value {
    pattern -> result
    _ -> default
}
```

#### `try` → Gestion d'Erreurs
```bulu
try {
    
} fail err {
    
}
```

#### `run` → Goroutine
```bulu
run function()
```

#### `chan` → Channel
```bulu
let ch: chan type = make(chan type)
```

### Navigation dans les Snippets

1. Tapez le préfixe (ex: `func`)
2. Appuyez sur **Tab**
3. Le snippet s'insère avec des placeholders
4. Tapez pour remplacer le premier placeholder
5. Appuyez sur **Tab** pour passer au suivant
6. Continuez jusqu'à la fin

## 🚀 3. Auto-Complétion

### Déclenchement

Appuyez sur **Ctrl+Space** (ou Cmd+Space sur Mac) pour voir les suggestions.

### Types de Suggestions

#### Mots-clés
```
Tapez: "fu"
Suggestions:
  - func (keyword)
  - function (snippet)
```

#### Types
```
Tapez: "int"
Suggestions:
  - int (type)
  - int8 (type)
  - int16 (type)
  - int32 (type)
  - int64 (type)
```

#### Fonctions Built-in
```
Tapez: "pr"
Suggestions:
  - print(args: ...any) (function)
  - println(args: ...any) (function)
  - printf(format: string, args: ...any) (function)
```

#### Après un Point (Member Access)
```
Tapez: "string."
Suggestions:
  - len (method)
  - toString (method)
```

#### Imports
```
Tapez: "import std."
Suggestions:
  - std.io (module)
  - std.fmt (module)
  - std.math (module)
  - std.http (module)
```

## 💡 4. Hover Information

### Survol avec la Souris

Survolez n'importe quel élément pour voir sa documentation :

#### Mots-clés
```bulu
func main() {
    // Survoler "func" affiche :
    // ```bulu
    // func name(params): returnType { ... }
    // ```
    // Function definition
}
```

#### Fonctions Built-in
```bulu
println("Hello")
// Survoler "println" affiche :
// ```bulu
// func println(args: ...any)
// ```
// Print values with newline
```

#### Types
```bulu
let x: int32 = 42
// Survoler "int32" affiche :
// ```bulu
// int32
// ```
// 32-bit signed integer (-2,147,483,648 to 2,147,483,647)
```

## 🔍 5. Go-to-Definition

### Navigation Rapide

Appuyez sur **F12** ou **Ctrl+Click** sur un symbole :

```bulu
// Définition
func calculate(x: int32): int32 {
    return x * 2
}

func main() {
    let result = calculate(5)  // F12 ici → saute à la définition
}
```

### Peek Definition

Appuyez sur **Alt+F12** pour voir la définition sans naviguer :

```bulu
func main() {
    let result = calculate(5)  // Alt+F12 → affiche la définition inline
}
```

## 🔎 6. Find References

### Trouver Toutes les Utilisations

Appuyez sur **Shift+F12** sur un symbole :

```bulu
func add(a: int32, b: int32): int32 {  // Shift+F12 ici
    return a + b
}

func main() {
    let x = add(1, 2)      // ← Référence trouvée
    let y = add(3, 4)      // ← Référence trouvée
    println(add(5, 6))     // ← Référence trouvée
}
```

Résultat : Liste de toutes les références avec fichier et ligne.

## ✏️ 7. Rename Refactoring

### Renommer un Symbole

Appuyez sur **F2** sur un symbole :

```bulu
func oldName(x: int32): int32 {  // F2 ici, taper "newName"
    return x * 2
}

func main() {
    let result = oldName(5)  // ← Automatiquement renommé en "newName"
}
```

Tous les usages sont mis à jour automatiquement !

## 🛠️ 8. Code Actions

### Corrections Rapides

Appuyez sur **Ctrl+.** quand vous voyez une erreur :

#### Exemple : Variable non définie
```bulu
func main() {
    println(undefined_var)  // Erreur soulignée en rouge
    // Ctrl+. → "Add import statement"
}
```

#### Exemple : Variable inutilisée
```bulu
func main() {
    let unused = 42  // Warning souligné en jaune
    // Ctrl+. → "Remove unused variable"
}
```

## 📋 9. Diagnostics en Temps Réel

### Erreurs de Syntaxe

```bulu
func main() {
    let x = 42
    println(x  // ← Erreur : Parenthèse manquante
}              // ← Erreur : Accolade manquante
```

Les erreurs apparaissent immédiatement avec :
- Soulignement rouge ondulé
- Message d'erreur au survol
- Liste dans le panneau "Problems"

### Warnings

```bulu
func main() {
    let unused = 42  // ← Warning : Variable non utilisée
}
```

Les warnings apparaissent avec :
- Soulignement jaune ondulé
- Message au survol

## 📝 10. Signature Help

### Aide sur les Paramètres

Tapez une fonction et ouvrez la parenthèse :

```bulu
func main() {
    printf(  // ← Affiche : printf(format: string, args: ...any)
           // Paramètre actif souligné
}
```

Naviguez entre les paramètres avec les virgules.

## 🗂️ 11. Document Symbols

### Navigation dans le Fichier

Appuyez sur **Ctrl+Shift+O** pour voir tous les symboles :

```bulu
// Fichier: example.bu

const MAX = 100        // ← Symbole : Constant

struct Point {         // ← Symbole : Struct
    x: float64
    y: float64
}

func distance() {      // ← Symbole : Function
    // ...
}

func main() {          // ← Symbole : Function
    // ...
}
```

Liste affichée :
- MAX (constant)
- Point (struct)
- distance (function)
- main (function)

## 🔧 12. Configuration

### Paramètres Disponibles

```json
{
  // Activer/désactiver le LSP
  "bulu.lsp.enabled": true,
  
  // Chemin vers bulu_lsp
  "bulu.lsp.path": "bulu_lsp",
  
  // Niveau de trace (off, messages, verbose)
  "bulu.lsp.trace.server": "off",
  
  // Formater à la sauvegarde
  "bulu.format.onSave": false,
  
  // Linter à la sauvegarde
  "bulu.lint.onSave": true
}
```

### Configuration par Fichier

```json
{
  "[bulu]": {
    "editor.formatOnSave": true,
    "editor.tabSize": 4,
    "editor.insertSpaces": true,
    "editor.rulers": [80, 120]
  }
}
```

## 🎯 13. Commandes VS Code

### Palette de Commandes

Appuyez sur **Ctrl+Shift+P** et tapez "Bulu" :

- `Bulu: Restart Language Server` - Redémarrer le LSP
- `Bulu: Show Output Channel` - Afficher les logs

## 📦 14. Icônes et Thème

### Icône de Fichier

Les fichiers `.bu` ont une icône personnalisée dans l'explorateur :

```
📁 src/
  📄 main.bu     ← Icône Bulu (B bleu)
  📄 utils.bu    ← Icône Bulu
  📄 types.bu    ← Icône Bulu
```

## 🚀 15. Workflow Complet

### Exemple de Développement

1. **Créer un fichier** : `example.bu`
2. **Taper un snippet** : `func` + Tab
3. **Compléter le code** : Ctrl+Space pour suggestions
4. **Voir la doc** : Survoler les fonctions
5. **Naviguer** : F12 pour aller aux définitions
6. **Refactorer** : F2 pour renommer
7. **Corriger** : Ctrl+. pour quick fixes
8. **Sauvegarder** : Ctrl+S (diagnostics mis à jour)

## 💡 16. Astuces Productivité

### Multi-Curseurs
```bulu
let x = 1  // Alt+Click pour ajouter un curseur
let y = 2  // Éditer plusieurs lignes en même temps
let z = 3
```

### Sélection Rapide
```bulu
func calculate(x: int32): int32 {
    // Ctrl+D pour sélectionner l'occurrence suivante
    return x * x
}
```

### Breadcrumbs
En haut de l'éditeur : `example.bu > main > calculate`
Cliquez pour naviguer rapidement.

### Minimap
À droite de l'éditeur : Vue d'ensemble du fichier avec coloration.

## 🎨 17. Thèmes Recommandés

L'extension fonctionne avec tous les thèmes VS Code :

- **Dark+** (par défaut) - Excellent contraste
- **Monokai** - Couleurs vives
- **Solarized Dark** - Doux pour les yeux
- **One Dark Pro** - Populaire et élégant
- **Dracula** - Violet et rose

## 📊 18. Comparaison Avant/Après

### Sans Extension
```
// Tout en blanc, pas de coloration
// Pas d'auto-complétion
// Pas de navigation
// Pas de diagnostics
```

### Avec Extension
```bulu
// Coloration complète et précise
// Auto-complétion intelligente
// Navigation fluide (F12, Shift+F12)
// Diagnostics en temps réel
// Snippets rapides
// Refactoring facile
```

## 🎯 19. Cas d'Usage Réels

### Développement d'API
```bulu
import std.http

struct User {
    id: int32
    name: string
    email: string
}

async func getUser(id: int32): User {
    let response = await http.get("/api/users/" + id)
    return json.decode(response.body)
}
```

Fonctionnalités utilisées :
- ✅ Coloration des imports
- ✅ Complétion des types
- ✅ Hover sur http.get
- ✅ Snippets pour struct

### Concurrence
```bulu
func worker(id: int32, jobs: chan int32, results: chan int32) {
    for job in jobs {
        results <- job * 2
    }
}

func main() {
    let jobs: chan int32 = make(chan int32, 100)
    let results: chan int32 = make(chan int32, 100)
    
    for i in 1..3 {
        run worker(i, jobs, results)
    }
}
```

Fonctionnalités utilisées :
- ✅ Snippets chan et run
- ✅ Coloration des mots-clés concurrence
- ✅ Complétion des types chan
- ✅ Navigation entre fonctions

## 🎓 20. Apprentissage

### Pour Débutants

L'extension aide à apprendre Bulu :

1. **Snippets** : Voir la structure correcte
2. **Hover** : Comprendre les fonctions
3. **Complétion** : Découvrir les APIs
4. **Diagnostics** : Corriger les erreurs
5. **Exemples** : Code coloré lisible

### Pour Experts

L'extension accélère le développement :

1. **Navigation** : Trouver rapidement le code
2. **Refactoring** : Modifier efficacement
3. **Multi-curseurs** : Éditions en masse
4. **Snippets** : Templates personnalisés
5. **Raccourcis** : Workflow optimisé

---

## 🎉 Conclusion

L'extension Bulu transforme VS Code en un **IDE complet** pour Bulu, offrant une expérience de développement **moderne, fluide et productive** !

**Installez-la maintenant et profitez de toutes ces fonctionnalités ! 🚀**
