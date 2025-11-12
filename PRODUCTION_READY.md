# 🚀 Bulu Language - Production Ready

Le langage Bulu est maintenant prêt pour la production avec un registry officiel déployé !

## ✅ Ce qui est configuré

### Registry de Production
- **URL** : https://bulu-language.onrender.com
- **Hébergement** : Render (déploiement automatique)
- **Base de données** : Neon PostgreSQL (serverless)
- **Stockage** : Cloudflare R2 (CDN global)
- **ORM** : SeaORM (type-safe, pas de SQL manuel)

### CLI `lang`
Le CLI est maintenant configuré pour utiliser le registry de production par défaut.

```bash
# Compiler le CLI
cargo build --release

# Le CLI utilise automatiquement le registry de production
./target/release/lang package search "test"
./target/release/lang package publish
./target/release/lang package add nom-package
```

## 🎯 Utilisation

### Publier un package

```bash
cd mon-package
lang package publish
```

Le package sera :
1. Compilé et empaqueté en tarball
2. Uploadé sur Cloudflare R2
3. Métadonnées enregistrées dans Neon PostgreSQL
4. Disponible immédiatement pour tous les utilisateurs

### Installer un package

```bash
lang package add nom-package
```

Le package sera :
1. Téléchargé depuis Cloudflare R2
2. Extrait dans `bulu_modules/`
3. Ajouté aux dépendances dans `lang.toml`

### Rechercher des packages

```bash
lang package search "mot-clé"
```

### Mettre à jour les dépendances

```bash
lang package update
```

## 🔧 Configuration avancée

### Utiliser un registry local pour le développement

```bash
# Terminal 1 : Démarrer le registry local
cd registry-server
cargo run

# Terminal 2 : Utiliser le registry local
export BULU_REGISTRY=http://localhost:3000
lang package publish
```

### Variables d'environnement

```bash
# Registry personnalisé
export BULU_REGISTRY=https://mon-registry.com

# Désactiver les couleurs
export NO_COLOR=1
```

## 📊 Architecture

```
┌─────────────────┐
│   Utilisateur   │
│   (lang CLI)    │
└────────┬────────┘
         │
         │ HTTPS
         ▼
┌─────────────────────────────────────┐
│  Registry (Render)                  │
│  https://bulu-language.onrender.com │
│                                     │
│  ┌──────────────┐  ┌─────────────┐ │
│  │   Axum API   │  │   SeaORM    │ │
│  └──────┬───────┘  └──────┬──────┘ │
│         │                 │         │
└─────────┼─────────────────┼─────────┘
          │                 │
          │                 ▼
          │         ┌───────────────┐
          │         │ Neon PostgreSQL│
          │         │  (Metadata)    │
          │         └───────────────┘
          │
          ▼
  ┌──────────────────┐
  │ Cloudflare R2    │
  │  (Tarballs)      │
  └──────────────────┘
```

## 🎨 Avantages de cette stack

### Pour les développeurs
- ✅ Pas de configuration complexe
- ✅ CLI simple et intuitif
- ✅ Packages disponibles instantanément
- ✅ Recherche rapide

### Pour l'infrastructure
- ✅ Coût minimal (plans gratuits)
- ✅ Scaling automatique
- ✅ Pas de serveur à gérer
- ✅ Déploiement automatique
- ✅ HTTPS gratuit

### Pour la performance
- ✅ CDN global (Cloudflare)
- ✅ Base de données serverless (Neon)
- ✅ Pas de cold start pour les packages
- ✅ Requêtes optimisées (SeaORM)

## 📈 Métriques

### Plans gratuits actuels

**Render** :
- 750 heures/mois
- 512 MB RAM
- Cold start après 15 min d'inactivité

**Neon** :
- 0.5 GB de stockage
- Connexions illimitées
- Branches de base de données

**Cloudflare R2** :
- 10 GB de stockage
- 1 million de requêtes/mois
- Pas de frais de sortie

## 🧪 Tests

### Test rapide

```bash
# Vérifier que le registry est en ligne
curl https://bulu-language.onrender.com/health

# Lister les packages
curl https://bulu-language.onrender.com/api/packages | jq '.'
```

### Test complet

```bash
./test_production_registry.sh
```

Ce script teste :
1. Health check du registry
2. API endpoints
3. Build du CLI
4. Configuration
5. Publication (optionnel)

## 🔐 Sécurité

### HTTPS
Toutes les communications utilisent HTTPS automatiquement.

### Secrets
Les secrets (clés Cloudflare, DATABASE_URL) sont stockés dans Render et ne sont jamais exposés.

### Validation
- Checksums SHA-256 pour tous les packages
- Validation des métadonnées
- Rate limiting (à venir)

## 📚 Documentation

- **Registry Config** : `REGISTRY_CONFIG.md`
- **Neon Setup** : `registry-server/NEON_SETUP.md`
- **Cloudflare Setup** : `registry-server/CLOUDFLARE_R2_SETUP.md`
- **Package Guide** : `PACKAGE_GUIDE.md`

## 🚀 Déploiement

Le registry se déploie automatiquement à chaque push sur `main` :

```bash
git add .
git commit -m "Update registry"
git push origin main
```

Render détecte le changement et redéploie en ~2 minutes.

## 🐛 Dépannage

### Le registry ne répond pas
- Vérifier : https://bulu-language.onrender.com/health
- Le service peut être en cold start (attendre 30s)

### Erreur de publication
```bash
# Vérifier la connexion
curl https://bulu-language.onrender.com/health

# Vérifier les logs
RUST_LOG=debug lang package publish
```

### Package non trouvé
```bash
# Rechercher le package
lang package search "nom-package"

# Vérifier dans le registry
curl https://bulu-language.onrender.com/api/packages/nom-package | jq '.'
```

## 🎉 Prochaines étapes

- [ ] Ajouter l'authentification (tokens)
- [ ] Implémenter le rate limiting
- [ ] Ajouter des badges pour les packages
- [ ] Créer une interface web
- [ ] Ajouter des statistiques de téléchargement
- [ ] Implémenter les versions sémantiques
- [ ] Ajouter la documentation des packages

## 📞 Support

Pour toute question ou problème :
- GitHub Issues
- Documentation en ligne (à venir)
- Email : support@bulu-lang.com (à venir)

---

**Le langage Bulu est maintenant prêt pour la production ! 🎉**
