# Configuration du Registry Bulu

## Registry par défaut

Le CLI `lang` utilise maintenant le registry officiel déployé sur Render :

```
https://bulu-language.onrender.com
```

## Utilisation

### Publier un package

```bash
lang package publish
```

Le package sera automatiquement publié sur le registry officiel.

### Installer un package

```bash
lang package add nom-du-package
```

### Rechercher des packages

```bash
lang package search "mot-clé"
```

### Mettre à jour les dépendances

```bash
lang package update
```

## Utiliser un registry personnalisé

Si vous voulez utiliser un registry différent (par exemple pour le développement local), définissez la variable d'environnement `BULU_REGISTRY` :

```bash
# Registry local
export BULU_REGISTRY=http://localhost:3000

# Ou directement dans la commande
BULU_REGISTRY=http://localhost:3000 lang package publish
```

## Architecture du Registry

### Backend
- **Hébergement** : Render (https://render.com)
- **Base de données** : Neon PostgreSQL (serverless)
- **Stockage** : Cloudflare R2
- **ORM** : SeaORM (type-safe)
- **Framework** : Axum

### Endpoints disponibles

```
GET  /health                           - Health check
GET  /api/packages                     - Liste tous les packages
GET  /api/packages/:name               - Info sur un package
POST /api/packages/:name/:version      - Publier une version
GET  /api/download/:name/:version      - Télécharger un package
GET  /api/search?q=query&limit=20      - Rechercher des packages
```

## Avantages de cette stack

### Neon PostgreSQL
- ✅ Serverless (pas de serveur à gérer)
- ✅ Gratuit pour commencer
- ✅ Backups automatiques
- ✅ Scaling automatique

### Cloudflare R2
- ✅ Pas de frais de sortie de données
- ✅ Performance globale (CDN)
- ✅ Compatible S3
- ✅ Coût très bas

### Render
- ✅ Déploiement automatique depuis Git
- ✅ HTTPS gratuit
- ✅ Logs en temps réel
- ✅ Scaling automatique

## Développement local

Pour tester localement avec le registry de production :

```bash
# Le registry est déjà configuré par défaut
cargo build --release
./target/release/lang package search "test"
```

Pour tester avec un registry local :

```bash
# Terminal 1 : Démarrer le registry local
cd registry-server
cargo run

# Terminal 2 : Utiliser le registry local
export BULU_REGISTRY=http://localhost:3000
cargo run --bin lang -- package publish
```

## Monitoring

### Render Dashboard
- URL : https://dashboard.render.com
- Logs en temps réel
- Métriques de performance
- Gestion des déploiements

### Neon Dashboard
- URL : https://console.neon.tech
- Requêtes SQL
- Métriques de la base de données
- Gestion des branches

### Cloudflare Dashboard
- URL : https://dash.cloudflare.com
- Stockage R2
- Statistiques de bande passante
- Gestion des buckets

## Sécurité

### Variables d'environnement (Render)

Les secrets suivants doivent être configurés dans Render :

```bash
DATABASE_URL=postgresql://...@neon.tech/neondb?sslmode=require
CLOUDFLARE_ACCOUNT_ID=...
CLOUDFLARE_BUCKET_NAME=bulang
CLOUDFLARE_ACCESS_KEY_ID=...
CLOUDFLARE_SECRET_ACCESS_KEY=...
PORT=3000
```

### HTTPS

Toutes les communications avec le registry utilisent HTTPS automatiquement via Render.

## Limites

### Plan gratuit Render
- Inactivité après 15 minutes (cold start ~30s)
- 750 heures/mois
- 512 MB RAM

### Plan gratuit Neon
- 0.5 GB de stockage
- 1 projet
- Branches illimitées

### Plan gratuit Cloudflare R2
- 10 GB de stockage
- 1 million de requêtes/mois
- Pas de frais de sortie

## Mise à jour du registry

Le registry se met à jour automatiquement à chaque push sur la branche `main` :

```bash
git add .
git commit -m "Update registry"
git push origin main
```

Render détecte le changement et redéploie automatiquement.

## Support

Pour toute question ou problème :
- Issues GitHub : https://github.com/votre-repo/issues
- Documentation : https://bulu-language.onrender.com/docs (à venir)
