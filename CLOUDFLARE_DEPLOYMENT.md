# Déploiement du Registry Bulu sur Cloudflare

Ce guide explique comment déployer le registry Bulu sur votre cluster Cloudflare avec R2 pour le stockage des tarballs.

## Architecture

```
┌─────────────────┐
│  Bulu CLI       │
│  (lang publish) │
└────────┬────────┘
         │
         │ HTTP POST /api/publish
         ▼
┌─────────────────────────┐
│  Registry Server        │
│  (Cloudflare Workers)   │
│  - API REST             │
│  - Métadonnées (D1)     │
└────────┬────────────────┘
         │
         │ Upload tarball
         ▼
┌─────────────────────────┐
│  Cloudflare R2          │
│  - Stockage tarballs    │
│  - CDN global           │
└─────────────────────────┘
```

## Option 1 : Déploiement sur Cloudflare Workers (Recommandé)

### Prérequis

- Compte Cloudflare (gratuit)
- Wrangler CLI installé : `npm install -g wrangler`
- R2 activé sur votre compte

### Étapes

1. **Créer un Worker Rust**

```bash
cd registry-server
wrangler init bulu-registry --type rust
```

2. **Configurer wrangler.toml**

```toml
name = "bulu-registry"
main = "src/main.rs"
compatibility_date = "2024-01-01"

[env.production]
name = "bulu-registry"
route = "registry.yourdomain.com/*"

[[r2_buckets]]
binding = "PACKAGES"
bucket_name = "bulu-packages"

[[d1_databases]]
binding = "DB"
database_name = "bulu-registry-db"
database_id = "your-database-id"
```

3. **Créer la base de données D1**

```bash
wrangler d1 create bulu-registry-db
wrangler d1 execute bulu-registry-db --file=./migrations/001_initial_schema.sql
```

4. **Déployer**

```bash
wrangler publish
```

5. **Configurer le DNS**

Dans le dashboard Cloudflare :
- Ajoutez un enregistrement CNAME : `registry.yourdomain.com` → `bulu-registry.workers.dev`

## Option 2 : Déploiement sur un VPS avec R2

Si vous préférez héberger le registry sur votre propre serveur :

### 1. Préparer le serveur

```bash
# Sur votre serveur
git clone https://github.com/yourusername/bulu.git
cd bulu/registry-server
```

### 2. Configurer R2

```bash
cp .env.example .env
nano .env
```

Ajoutez vos credentials R2 :

```bash
R2_ACCOUNT_ID=your_account_id
R2_ACCESS_KEY_ID=your_access_key
R2_SECRET_ACCESS_KEY=your_secret_key
R2_BUCKET_NAME=bulu-packages
R2_PUBLIC_DOMAIN=packages.yourdomain.com
```

### 3. Compiler et démarrer

```bash
cargo build --release
./target/release/bulu-registry
```

### 4. Configurer un reverse proxy (nginx)

```nginx
server {
    listen 80;
    server_name registry.yourdomain.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 5. Configurer systemd (optionnel)

```ini
[Unit]
Description=Bulu Registry Server
After=network.target

[Service]
Type=simple
User=bulu
WorkingDirectory=/opt/bulu/registry-server
Environment="R2_ACCOUNT_ID=your_account_id"
Environment="R2_ACCESS_KEY_ID=your_access_key"
Environment="R2_SECRET_ACCESS_KEY=your_secret_key"
Environment="R2_BUCKET_NAME=bulu-packages"
ExecStart=/opt/bulu/registry-server/target/release/bulu-registry
Restart=always

[Install]
WantedBy=multi-user.target
```

## Configuration du client Bulu

Une fois le registry déployé, configurez le client :

```bash
# Dans votre projet Bulu
export BULU_REGISTRY_URL=https://registry.yourdomain.com
```

Ou ajoutez dans `~/.bulu/config.toml` :

```toml
[registry]
url = "https://registry.yourdomain.com"
```

## Test du déploiement

```bash
# Publier un package
cd example-package
lang publish

# Chercher des packages
lang search geometry

# Installer dans un nouveau projet
cd ../test-project
lang add example-package
lang install
```

## Monitoring

### Cloudflare Dashboard

- **R2 Storage** : Voir l'utilisation du stockage
- **Workers Analytics** : Voir les requêtes et la latence
- **D1 Database** : Voir les requêtes SQL

### Logs

```bash
# Logs du Worker
wrangler tail

# Logs du serveur VPS
journalctl -u bulu-registry -f
```

## Coûts estimés

### Cloudflare Workers + R2 (Plan gratuit)

- **Workers** : 100,000 requêtes/jour gratuit
- **R2 Storage** : 10 GB gratuit
- **R2 Operations** : 1M PUT + 10M GET/mois gratuit
- **Bandwidth** : Illimité et gratuit

Pour un registry de packages, le plan gratuit devrait suffire pour des milliers d'utilisateurs.

### VPS + R2

- **VPS** : ~$5-10/mois (DigitalOcean, Hetzner, etc.)
- **R2** : Gratuit jusqu'à 10 GB

## Sécurité

### Production

1. **Activer HTTPS** : Utilisez Cloudflare SSL ou Let's Encrypt
2. **Authentification** : Ajoutez un token pour `lang publish`
3. **Rate limiting** : Limitez les requêtes par IP
4. **Validation** : Vérifiez les checksums des tarballs

### Variables d'environnement sensibles

Ne commitez jamais :
- `.env`
- Credentials R2
- Tokens d'authentification

Utilisez :
- Cloudflare Secrets pour Workers
- Variables d'environnement système pour VPS

## Backup

Les tarballs dans R2 sont automatiquement répliqués sur plusieurs datacenters Cloudflare. Pour une sécurité supplémentaire :

1. **Activer le versioning R2** (si disponible)
2. **Exporter régulièrement la base D1** :
   ```bash
   wrangler d1 export bulu-registry-db --output=backup.sql
   ```

## Mise à jour

```bash
# Workers
cd registry-server
wrangler publish

# VPS
git pull
cargo build --release
systemctl restart bulu-registry
```

## Support

Pour des questions ou problèmes :
- GitHub Issues : https://github.com/yourusername/bulu/issues
- Documentation : https://bulu-lang.org/docs/registry
