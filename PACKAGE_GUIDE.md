# Guide de Gestion des Packages Bulu

## Vue d'ensemble

Bulu dispose d'un système de gestion de packages complet similaire à:
- **pub.dev** (Dart)
- **npm** (Node.js)
- **crates.io** (Rust)

## Structure d'un Package

### 1. Fichier de Configuration (`lang.toml`)

```toml
[package]
name = "mon-package"
version = "1.0.0"
authors = ["Votre Nom <email@example.com>"]
description = "Description de votre package"
license = "MIT"
repository = "https://github.com/username/mon-package"
keywords = ["web", "http", "client"]
categories = ["network", "web"]

[dependencies]
# Dépendance simple avec version compatible (^)
http = "^1.2.0"

# Dépendance avec version exacte
json = "=2.0.0"

# Dépendance depuis un chemin local
utils = { path = "../utils" }

# Dépendance depuis Git
async-lib = { git = "https://github.com/user/async-lib", tag = "v1.0.0" }

# Dépendance optionnelle
logging = { version = "^0.5.0", optional = true }

[build]
optimization = "2"
target = "native"
incremental = true
parallel = true

[test]
parallel = true
timeout = 30
coverage = false
```

### 2. Structure de Répertoire

```
mon-package/
├── lang.toml           # Configuration du package
├── lang.lock           # Lockfile des dépendances
├── README.md           # Documentation
├── LICENSE             # Licence
├── src/
│   ├── lib.bu          # Point d'entrée de la bibliothèque
│   ├── module1.bu      # Modules
│   └── module2.bu
├── tests/
│   ├── test1.bu
│   └── test2.bu
├── examples/
│   └── example1.bu
└── vendor/             # Dépendances vendorisées (optionnel)
```

### 3. Fichier de Bibliothèque (`src/lib.bu`)

```bulu
// src/lib.bu - Point d'entrée du package

// Exporter des fonctions
export func add(a: int32, b: int32): int32 {
    return a + b
}

export func multiply(a: int32, b: int32): int32 {
    return a * b
}

// Exporter des types
export struct Point {
    x: float64
    y: float64
}

export func distance(p1: Point, p2: Point): float64 {
    let dx = p2.x - p1.x
    let dy = p2.y - p1.y
    return sqrt(dx * dx + dy * dy)
}

// Exporter des constantes
export const VERSION = "1.0.0"
export const MAX_SIZE = 1000

// Réexporter depuis d'autres modules
export { HttpClient, Request, Response } from "./http"
export { JsonParser, JsonValue } from "./json"
```

## Commandes de Gestion des Packages

### Créer un Nouveau Package

```bash
# Créer un nouveau projet/package
lang new mon-package

# Créer une bibliothèque
lang new mon-package --lib
```

### Ajouter des Dépendances

```bash
# Ajouter une dépendance avec la dernière version
lang add http

# Ajouter avec une version spécifique
lang add json --version "^2.0.0"

# Ajouter depuis un chemin local
lang add utils --path ../utils

# Ajouter depuis Git
lang add async-lib --git https://github.com/user/async-lib --tag v1.0.0
```

### Gérer les Dépendances

```bash
# Installer toutes les dépendances
lang install

# Mettre à jour les dépendances
lang update

# Supprimer une dépendance
lang remove http

# Lister les dépendances
lang list

# Vendoriser les dépendances (copier localement)
lang vendor
```

### Rechercher des Packages

```bash
# Rechercher des packages
lang search http

# Rechercher avec limite de résultats
lang search json --limit 10
```

### Publier un Package

```bash
# Vérifier que le package est prêt
lang build
lang test

# Publier (dry-run d'abord)
lang publish --dry-run

# Publier réellement
lang publish

# Publier avec verbosité
lang publish --verbose
```

## Utiliser un Package

### Dans votre Code

```bulu
// Importer un package entier
import "mon-package"

func main() {
    let result = mon-package.add(5, 3)
    println("Result: " + string(result))
}

// Importer avec alias
import "mon-package" as mp

func main() {
    let result = mp.add(5, 3)
    println("Result: " + string(result))
}

// Importer des éléments spécifiques
import { add, multiply, Point } from "mon-package"

func main() {
    let sum = add(5, 3)
    let product = multiply(5, 3)
    
    let p1 = Point { x: 0.0, y: 0.0 }
    let p2 = Point { x: 3.0, y: 4.0 }
}
```

## Contraintes de Version

### Syntaxe des Versions

- `^1.2.3` - Compatible (>= 1.2.3, < 2.0.0)
- `~1.2.3` - Tilde (>= 1.2.3, < 1.3.0)
- `=1.2.3` - Exacte (1.2.3 seulement)
- `>=1.2.3` - Supérieur ou égal
- `>1.2.3` - Strictement supérieur
- `<=1.2.3` - Inférieur ou égal
- `<1.2.3` - Strictement inférieur
- `*` - N'importe quelle version

### Exemples

```toml
[dependencies]
# Compatible: accepte 1.2.3, 1.2.4, 1.3.0, mais pas 2.0.0
http = "^1.2.3"

# Tilde: accepte 1.2.3, 1.2.4, mais pas 1.3.0
json = "~1.2.3"

# Exacte: seulement 2.0.0
parser = "=2.0.0"

# Range: >= 1.0.0 et < 2.0.0
utils = ">=1.0.0, <2.0.0"
```

## Lockfile (`lang.lock`)

Le fichier `lang.lock` est généré automatiquement et contient:
- Les versions exactes de toutes les dépendances
- Les checksums pour la vérification d'intégrité
- L'arbre complet des dépendances transitives

**Important**: Commitez toujours `lang.lock` dans votre dépôt!

## Exemple Complet: Package HTTP Client

### Structure

```
http-client/
├── lang.toml
├── README.md
├── src/
│   ├── lib.bu
│   ├── client.bu
│   ├── request.bu
│   └── response.bu
├── tests/
│   └── client_test.bu
└── examples/
    └── simple_get.bu
```

### `lang.toml`

```toml
[package]
name = "http-client"
version = "1.0.0"
authors = ["Dev Team <dev@example.com>"]
description = "Simple HTTP client for Bulu"
license = "MIT"
keywords = ["http", "client", "web"]
categories = ["network"]

[dependencies]
json = "^2.0.0"
async = "^1.5.0"
```

### `src/lib.bu`

```bulu
// Réexporter les types principaux
export { HttpClient } from "./client"
export { Request, Method } from "./request"
export { Response, StatusCode } from "./response"

export const VERSION = "1.0.0"
```

### `src/client.bu`

```bulu
import { Request, Response } from "./request"

export struct HttpClient {
    base_url: string
    timeout: int32
}

export func HttpClient.new(base_url: string): HttpClient {
    return HttpClient {
        base_url: base_url,
        timeout: 30000
    }
}

export async func HttpClient.get(self: HttpClient, path: string): Response {
    let request = Request.new("GET", self.base_url + path)
    return await self.send(request)
}

export async func HttpClient.post(self: HttpClient, path: string, body: string): Response {
    let request = Request.new("POST", self.base_url + path)
    request.body = body
    return await self.send(request)
}

async func HttpClient.send(self: HttpClient, request: Request): Response {
    // Implémentation de l'envoi HTTP
    // ...
}
```

### Utilisation

```bulu
import { HttpClient } from "http-client"

async func main() {
    let client = HttpClient.new("https://api.example.com")
    
    let response = await client.get("/users")
    println("Status: " + string(response.status))
    println("Body: " + response.body)
}
```

## Bonnes Pratiques

### 1. Versioning Sémantique

Suivez [SemVer](https://semver.org/):
- **MAJOR**: Changements incompatibles
- **MINOR**: Nouvelles fonctionnalités compatibles
- **PATCH**: Corrections de bugs

### 2. Documentation

- README.md complet avec exemples
- Commentaires de documentation dans le code
- Exemples dans `examples/`

### 3. Tests

```bulu
// tests/client_test.bu
import { HttpClient } from "../src/lib"

func test_http_get() {
    let client = HttpClient.new("https://httpbin.org")
    let response = await client.get("/get")
    assert(response.status == 200)
}
```

### 4. Exports Clairs

```bulu
// Exporter seulement l'API publique
export { PublicType, publicFunction } from "./internal"

// Ne pas exporter les détails d'implémentation
func internalHelper() {
    // Fonction privée
}
```

### 5. Gestion des Erreurs

```bulu
export struct Result<T> {
    value: T?
    error: string?
}

export func divide(a: float64, b: float64): Result<float64> {
    if b == 0.0 {
        return Result { value: null, error: "Division by zero" }
    }
    return Result { value: a / b, error: null }
}
```

## Configuration du Registry

Le registry par défaut est `https://pkg.lang-lang.org`. Pour utiliser un registry privé:

```toml
# ~/.bulu/config.toml
[registry]
url = "https://my-private-registry.com"
auth_token = "your-token-here"
```

## Conclusion

Le système de packages Bulu permet de:
- ✅ Créer et publier des bibliothèques réutilisables
- ✅ Gérer les dépendances avec des contraintes de version
- ✅ Partager du code avec la communauté
- ✅ Utiliser des packages tiers facilement
- ✅ Vendoriser les dépendances pour un contrôle total

Pour plus d'informations, consultez la documentation officielle sur https://lang-lang.org/docs/packages
