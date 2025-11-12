# Configuration Neon PostgreSQL pour le Registry Bulu

Ce guide explique comment configurer une base de données PostgreSQL avec Neon pour le registry de packages Bulu.

## Pourquoi Neon ?

- **Gratuit** : Plan gratuit généreux avec 0.5 GB de stockage
- **Serverless** : Pas de serveur à gérer
- **Rapide** : Démarrage instantané
- **Moderne** : Branching de base de données, autoscaling
- **Compatible** : PostgreSQL standard

## Étapes de configuration

### 1. Créer un compte Neon

1. Allez sur [https://console.neon.tech](https://console.neon.tech)
2. Créez un compte (gratuit)
3. Créez un nouveau projet

### 2. Obtenir la chaîne de connexion

1. Dans votre projet Neon, cliquez sur **Connection Details**
2. Copiez la **Connection string** (elle ressemble à ceci) :
   ```
   postgresql://username:password@ep-xxx-xxx.region.aws.neon.tech/neondb?sslmode=require
   ```

### 3. Configurer le Registry

Créez un fichier `.env` dans le dossier `registry-server/` :

```bash
# Database - Neon PostgreSQL
DATABASE_URL=postgresql://username:password@ep-xxx-xxx.region.aws.neon.tech/neondb?sslmode=require

# Storage (local par défaut)
STORAGE_PATH=./storage

# Cloudflare R2 (optionnel)
# CLOUDFLARE_ACCOUNT_ID=your_account_id
# CLOUDFLARE_BUCKET_NAME=bulu-packages
# CLOUDFLARE_ACCESS_KEY_ID=your_access_key_id
# CLOUDFLARE_SECRET_ACCESS_KEY=your_secret_access_key

# Server
PORT=3000
```

### 4. Démarrer le serveur

```bash
cd registry-server
cargo run
```

Le serveur va automatiquement :
- Se connecter à Neon
- Créer les tables nécessaires
- Démarrer sur le port 3000

## Architecture avec SeaORM

Le registry utilise **SeaORM**, un ORM moderne pour Rust qui offre :

- **Type-safe** : Toutes les requêtes sont vérifiées à la compilation
- **Async/await** : Performance optimale
- **Relations** : Gestion automatique des relations entre tables
- **Migrations** : Gestion de schéma simplifiée

### Entités définies

```
packages
├── package_versions
│   ├── package_authors
│   ├── package_dependencies
│   └── download_stats
└── package_keywords
```

### Exemple de requête avec SeaORM

Au lieu d'écrire du SQL manuel :
```rust
// ❌ Avant (SQL manuel)
sqlx::query("SELECT * FROM packages WHERE name = $1")
    .bind(name)
    .fetch_one(&pool)
    .await?;
```

Maintenant avec SeaORM :
```rust
// ✅ Maintenant (Type-safe)
package::Entity::find()
    .filter(package::Column::Name.eq(name))
    .one(&db)
    .await?;
```

## Avantages de cette stack

### Neon PostgreSQL
- ✅ Gratuit pour commencer
- ✅ Pas de serveur à gérer
- ✅ Backups automatiques
- ✅ Scaling automatique
- ✅ Branching de base de données

### SeaORM
- ✅ Type-safe (erreurs détectées à la compilation)
- ✅ Pas de SQL manuel à écrire
- ✅ Relations automatiques
- ✅ Migrations intégrées
- ✅ Performance optimale

## Test de l'installation

### 1. Vérifier la connexion

```bash
curl http://localhost:3000/health
# Devrait retourner: OK
```

### 2. Lister les packages

```bash
curl http://localhost:3000/api/packages
# Devrait retourner: []
```

### 3. Publier un package de test

```bash
cd ../example-package
cargo run --bin lang -- package publish --registry http://localhost:3000
```

### 4. Vérifier dans Neon

1. Allez dans votre console Neon
2. Cliquez sur **SQL Editor**
3. Exécutez :
   ```sql
   SELECT * FROM packages;
   SELECT * FROM package_versions;
   ```

## Monitoring

### Dans Neon Console

- **Metrics** : CPU, mémoire, connexions
- **Queries** : Requêtes lentes
- **Logs** : Logs de la base de données

### Dans le Registry

Les logs du serveur montrent :
```
📊 Connecting to database: postgresql://...
💾 Using local storage at: ./storage
🚀 Registry server listening on 0.0.0.0:3000
```

## Dépannage

### Erreur de connexion

```
Error: Database(ConnectionError)
```

**Solutions** :
1. Vérifiez que votre DATABASE_URL est correct
2. Vérifiez que `?sslmode=require` est présent
3. Vérifiez que votre projet Neon est actif

### Tables non créées

Les migrations s'exécutent automatiquement au démarrage. Si vous avez des problèmes :

```bash
# Supprimez et recréez la base de données dans Neon Console
# Puis redémarrez le serveur
cargo run
```

### Performance

Neon offre :
- **Free tier** : 0.25 vCPU, 1 GB RAM
- **Pro tier** : Autoscaling jusqu'à 4 vCPU

Pour la plupart des cas d'usage, le free tier est suffisant.

## Migration depuis SQLite

Si vous aviez une base SQLite locale :

1. Exportez vos données (si nécessaire)
2. Configurez Neon comme ci-dessus
3. Redémarrez le serveur
4. Les tables seront créées automatiquement

## Prochaines étapes

- [ ] Configurer Cloudflare R2 pour le stockage des tarballs
- [ ] Déployer sur un serveur (Fly.io, Railway, etc.)
- [ ] Configurer un domaine personnalisé
- [ ] Ajouter l'authentification

## Ressources

- [Documentation Neon](https://neon.tech/docs)
- [Documentation SeaORM](https://www.sea-ql.org/SeaORM/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
