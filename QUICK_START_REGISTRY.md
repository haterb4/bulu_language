# 🚀 Quick Start - Registry Bulu

Guide rapide pour démarrer avec le registry de packages Bulu.

## ⚡ Démarrage Rapide (5 minutes)

### 1. Démarrer le Registry

```bash
# Terminal 1
cargo run --manifest-path registry-server/Cargo.toml
```

Attendez de voir:
```
🚀 Bulu Registry Server starting on http://127.0.0.1:3000
📦 API available at http://127.0.0.1:3000/api
```

### 2. Publier le Package Exemple

```bash
# Terminal 2
./publish_example.sh
```

Vous devriez voir:
```
✅ Publication terminée!
```

### 3. Vérifier

```bash
# Lister les packages
curl http://localhost:3000/api/packages | jq .

# Rechercher
curl "http://localhost:3000/api/search?q=math" | jq .

# Télécharger
curl http://localhost:3000/api/download/math-utils/1.0.0 -o test.tar.gz
tar tzf test.tar.gz
```

## 📦 Publier Votre Propre Package

### Étape 1: Créer le Package

```bash
lang new mon-package --lib
cd mon-package

# Éditer src/lib.bu
export func hello(): string {
    return "Hello from mon-package!"
}
```

### Étape 2: Créer le Tarball

```bash
tar czf ../mon-package-1.0.0.tar.gz src/ lang.toml README.md
cd ..
```

### Étape 3: Publier

```bash
TARBALL=$(base64 -w 0 mon-package-1.0.0.tar.gz)

curl -X POST http://localhost:3000/api/publish \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"mon-package\",
    \"version\": \"1.0.0\",
    \"description\": \"Mon super package\",
    \"authors\": [\"Moi <moi@example.com>\"],
    \"license\": \"MIT\",
    \"keywords\": [\"example\"],
    \"dependencies\": {},
    \"tarball\": \"$TARBALL\"
  }"
```

## 🔍 Commandes Utiles

```bash
# Lister tous les packages
curl http://localhost:3000/api/packages | jq .

# Détails d'un package
curl http://localhost:3000/api/packages/math-utils | jq .

# Versions disponibles
curl http://localhost:3000/api/packages/math-utils/versions | jq .

# Rechercher
curl "http://localhost:3000/api/search?q=math&limit=10" | jq .

# Télécharger
curl http://localhost:3000/api/download/math-utils/1.0.0 -o package.tar.gz

# Statistiques
curl http://localhost:3000/api/packages | jq '.total'
```

## 🧪 Tester avec curl

```bash
# Test complet
./registry-server/test_registry.sh
```

## 🛠️ Développement

### Modifier le Serveur

```bash
# Éditer registry-server/src/main.rs
# Puis recompiler
cargo build --manifest-path registry-server/Cargo.toml

# Redémarrer
cargo run --manifest-path registry-server/Cargo.toml
```

### Ajouter des Features

Le serveur est dans `registry-server/src/main.rs`. Vous pouvez:
- Ajouter de nouveaux endpoints
- Modifier la logique de recherche
- Ajouter de la persistance
- Implémenter l'authentification

## 📚 Documentation Complète

- `REGISTRY_SETUP.md` - Guide complet de configuration
- `REGISTRY_SUCCESS.md` - État actuel et accomplissements
- `PACKAGE_GUIDE.md` - Guide de création de packages
- `registry-server/README.md` - Documentation du serveur

## 🎯 Prochaines Étapes

1. **Intégrer avec `lang` CLI**
   - Implémenter `lang add` pour télécharger depuis le registry
   - Implémenter `lang publish` pour publier

2. **Ajouter la Persistance**
   - SQLite pour les métadonnées
   - Fichiers sur disque pour les tarballs

3. **Sécurité**
   - Authentification par token
   - Validation des packages

## ❓ Problèmes Courants

### Le serveur ne démarre pas

```bash
# Vérifier que le port 3000 est libre
lsof -i :3000

# Tuer le processus si nécessaire
kill -9 $(lsof -t -i:3000)
```

### Erreur de publication

```bash
# Vérifier le format du tarball
tar tzf mon-package.tar.gz

# Vérifier l'encodage base64
echo "$TARBALL" | base64 -d | tar tz
```

### Package non trouvé

```bash
# Vérifier qu'il est publié
curl http://localhost:3000/api/packages | jq '.packages[].name'
```

## 🎉 C'est Tout!

Vous avez maintenant un registry de packages Bulu fonctionnel!

Pour plus d'informations, consultez la documentation complète dans les fichiers mentionnés ci-dessus.

Happy coding! 🚀
