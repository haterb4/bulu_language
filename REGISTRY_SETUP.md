# Configuration du Registry Bulu

Guide complet pour configurer et utiliser le registry de packages Bulu.

## 🚀 Démarrage du Registry Server

### 1. Compiler et lancer le serveur

```bash
cd registry-server
cargo build --release
cargo run --release
```

Le serveur démarre sur `http://localhost:3000`

Vous devriez voir:
```
🚀 Bulu Registry Server starting on http://127.0.0.1:3000
📦 API available at http://127.0.0.1:3000/api
```

### 2. Tester le serveur

```bash
# Dans un autre terminal
./registry-server/test_registry.sh
```

Ou manuellement:
```bash
curl http://localhost:3000/
```

## 📦 Publier un Package

### Étape 1: Préparer votre package

Créez un projet avec `lang.toml`:

```toml
[package]
name = "mon-package"
version = "1.0.0"
authors = ["Votre Nom <email@example.com>"]
description = "Description de votre package"
license = "MIT"
repository = "https://github.com/username/mon-package"
keywords = ["keyword1", "keyword2"]

[dependencies]
# Vos dépendances
```

### Étape 2: Configurer le registry

Créez `~/.bulu/config.toml`:

```toml
[registry]
url = "http://localhost:3000"
```

### Étape 3: Publier

```bash
cd mon-package
lang publish
```

## 🔍 Rechercher des Packages

```bash
# Rechercher
lang search math

# Rechercher avec limite
lang search http --limit 10
```

## ➕ Ajouter des Dépendances

```bash
# Ajouter la dernière version
lang add math-utils

# Ajouter une version spécifique
lang add http-client --version "^2.0.0"

# Ajouter depuis un chemin local
lang add my-lib --path ../my-lib

# Ajouter depuis Git
lang add async-lib --git https://github.com/user/async-lib
```

## 📥 Installer les Dépendances

```bash
# Installer toutes les dépendances du projet
lang install

# Mettre à jour les dépendances
lang update

# Lister les dépendances
lang list
```

## 🧪 Exemple Complet

### 1. Démarrer le registry

```bash
cd registry-server
cargo run &
```

### 2. Publier le package math-utils

```bash
cd example-package
lang publish --registry http://localhost:3000
```

### 3. Créer un nouveau projet qui utilise math-utils

```bash
lang new my-app
cd my-app

# Ajouter math-utils comme dépendance
lang add math-utils --registry http://localhost:3000
```

### 4. Utiliser le package

Éditez `src/main.bu`:

```bulu
import { Point2D, distance, sqrt, PI } from "math-utils"

func main() {
    println("=== Test math-utils ===")
    
    // Utiliser Point2D
    let p1 = Point2D.new(0.0, 0.0)
    let p2 = Point2D.new(3.0, 4.0)
    
    let dist = distance(p1, p2)
    println("Distance: " + string(dist))
    
    // Utiliser sqrt
    let root = sqrt(16.0)
    println("sqrt(16) = " + string(root))
    
    // Utiliser PI
    let circle_area = PI * 5.0 * 5.0
    println("Aire du cercle (r=5): " + string(circle_area))
}
```

### 5. Compiler et exécuter

```bash
lang build
lang run
```

## 🔧 API du Registry

### Endpoints disponibles

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/` | Informations sur le registry |
| GET | `/api/packages` | Liste tous les packages |
| GET | `/api/packages/:name` | Infos sur un package |
| GET | `/api/packages/:name/versions` | Versions d'un package |
| GET | `/api/packages/:name/:version` | Infos sur une version |
| POST | `/api/publish` | Publier un package |
| GET | `/api/search?q=query` | Rechercher des packages |
| GET | `/api/download/:name/:version` | Télécharger un package |

### Exemples avec curl

```bash
# Lister les packages
curl http://localhost:3000/api/packages | jq .

# Rechercher
curl "http://localhost:3000/api/search?q=math" | jq .

# Obtenir les versions
curl http://localhost:3000/api/packages/math-utils/versions | jq .

# Télécharger
curl http://localhost:3000/api/download/math-utils/1.0.0 -o package.tar.gz
```

## 🐛 Dépannage

### Le serveur ne démarre pas

```bash
# Vérifier que le port 3000 est libre
lsof -i :3000

# Utiliser un autre port
REGISTRY_PORT=8080 cargo run
```

### Erreur de publication

```bash
# Vérifier la configuration
cat ~/.bulu/config.toml

# Publier avec verbosité
lang publish --verbose

# Dry-run pour tester
lang publish --dry-run
```

### Package non trouvé

```bash
# Vérifier que le package existe
curl http://localhost:3000/api/packages/nom-package

# Vérifier la configuration du registry
lang config get registry.url
```

## 📊 Monitoring

### Logs du serveur

Le serveur affiche des logs pour chaque opération:

```
📦 Publishing package: math-utils v1.0.0
✅ Published: math-utils v1.0.0
📥 Download: math-utils v1.0.0 (total: 1)
```

### Statistiques

```bash
# Nombre de packages
curl http://localhost:3000/api/packages | jq '.total'

# Package le plus téléchargé
curl http://localhost:3000/api/packages | jq '.packages | sort_by(.downloads) | reverse | .[0]'
```

## 🚀 Production

Pour un déploiement en production:

1. **Base de données**: Remplacer le HashMap en mémoire par SQLite/PostgreSQL
2. **Authentification**: Ajouter des tokens d'API
3. **HTTPS**: Utiliser un reverse proxy (nginx, caddy)
4. **Stockage**: Utiliser S3 ou un stockage objet pour les tarballs
5. **Cache**: Ajouter Redis pour les métadonnées
6. **CDN**: Distribuer les packages via un CDN

## 📝 Notes

- Le registry en mémoire perd les données au redémarrage
- Pour la persistance, utilisez une base de données
- Les tarballs sont stockés en base64 dans la mémoire
- Pour la production, stockez les fichiers sur disque ou S3

## 🎯 Prochaines Étapes

1. ✅ Registry HTTP fonctionnel
2. ⏳ Intégration avec `lang add/install`
3. ⏳ Authentification et autorisation
4. ⏳ Persistance en base de données
5. ⏳ Interface web pour parcourir les packages
6. ⏳ CI/CD pour publication automatique
7. ⏳ Mirroring et réplication

## 🤝 Contribution

Pour contribuer au registry:

1. Fork le projet
2. Créez une branche (`git checkout -b feature/amazing-feature`)
3. Commit vos changements (`git commit -m 'Add amazing feature'`)
4. Push vers la branche (`git push origin feature/amazing-feature`)
5. Ouvrez une Pull Request

## 📄 Licence

MIT
