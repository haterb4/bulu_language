# Configuration Cloudflare R2 pour Bulu Registry

Ce guide explique comment configurer Cloudflare R2 pour stocker les tarballs de packages Bulu.

## Pourquoi Cloudflare R2 ?

- ✅ **Gratuit** : 10 GB de stockage gratuit par mois
- ✅ **Pas de frais de sortie** : Téléchargements illimités sans frais
- ✅ **Compatible S3** : Utilise l'API S3 standard
- ✅ **Global** : CDN Cloudflare intégré
- ✅ **Rapide** : Faible latence mondiale

## Étape 1 : Créer un compte Cloudflare

1. Allez sur https://dash.cloudflare.com/sign-up
2. Créez un compte gratuit

## Étape 2 : Activer R2

1. Dans le dashboard Cloudflare, allez dans **R2**
2. Cliquez sur **Purchase R2 Plan** (le plan gratuit est suffisant)
3. Acceptez les conditions

## Étape 3 : Créer un bucket

1. Dans R2, cliquez sur **Create bucket**
2. Nom du bucket : `bulu-packages` (ou votre choix)
3. Région : Choisissez **Automatic** (recommandé)
4. Cliquez sur **Create bucket**

## Étape 4 : Créer un API Token

1. Dans R2, cliquez sur **Manage R2 API Tokens**
2. Cliquez sur **Create API Token**
3. Configuration :
   - **Token name** : `bulu-registry`
   - **Permissions** : 
     - ✅ Object Read & Write
   - **TTL** : Laissez vide (pas d'expiration)
   - **Bucket** : Sélectionnez `bulu-packages` (ou votre bucket)
4. Cliquez sur **Create API Token**
5. **IMPORTANT** : Copiez les informations affichées :
   - Access Key ID
   - Secret Access Key
   - Endpoint URL

## Étape 5 : Configurer le Registry Server

1. Copiez `.env.example` vers `.env` :
   ```bash
   cd registry-server
   cp .env.example .env
   ```

2. Éditez `.env` et ajoutez vos credentials R2 :
   ```bash
   # Cloudflare R2 Configuration
   R2_ACCOUNT_ID=your_account_id_here
   R2_ACCESS_KEY_ID=your_access_key_id_here
   R2_SECRET_ACCESS_KEY=your_secret_access_key_here
   R2_BUCKET_NAME=bulu-packages
   ```

3. Trouvez votre Account ID :
   - Dans le dashboard Cloudflare
   - Cliquez sur votre profil (en haut à droite)
   - L'Account ID est affiché dans la section **Account ID**

## Étape 6 : (Optionnel) Configurer un domaine personnalisé

Pour permettre les téléchargements publics via un domaine personnalisé :

1. Dans R2, sélectionnez votre bucket `bulu-packages`
2. Allez dans **Settings** > **Public Access**
3. Cliquez sur **Connect Domain**
4. Entrez votre domaine : `packages.yourdomain.com`
5. Suivez les instructions pour configurer le DNS

6. Ajoutez dans `.env` :
   ```bash
   R2_PUBLIC_DOMAIN=packages.yourdomain.com
   ```

## Étape 7 : Tester la configuration

1. Démarrez le registry server :
   ```bash
   cd registry-server
   cargo run
   ```

2. Vous devriez voir :
   ```
   ☁️  Cloudflare R2 storage enabled
   🚀 Bulu Registry Server starting on http://127.0.0.1:3000
   ```

3. Publiez un package de test :
   ```bash
   cd ../example-package
   lang publish
   ```

4. Vérifiez dans le dashboard R2 que le tarball est bien uploadé

## Structure des fichiers dans R2

Les tarballs sont stockés avec cette structure :

```
packages/
  ├── package-name/
  │   ├── 1.0.0/
  │   │   └── package-name-1.0.0.tar.gz
  │   ├── 1.0.1/
  │   │   └── package-name-1.0.1.tar.gz
  │   └── 2.0.0/
  │       └── package-name-2.0.0.tar.gz
  └── another-package/
      └── 1.0.0/
          └── another-package-1.0.0.tar.gz
```

## Coûts

Avec le plan gratuit de Cloudflare R2 :

- **Stockage** : 10 GB/mois gratuit
- **Opérations de classe A** (PUT, LIST) : 1 million/mois gratuit
- **Opérations de classe B** (GET, HEAD) : 10 millions/mois gratuit
- **Sortie** : Illimité et gratuit ! 🎉

Pour un registry de packages, cela devrait être largement suffisant.

## Dépannage

### Erreur : "Failed to initialize Cloudflare R2"

- Vérifiez que toutes les variables d'environnement sont définies
- Vérifiez que l'Account ID est correct
- Vérifiez que les credentials sont valides

### Erreur : "Access Denied"

- Vérifiez que le token API a les bonnes permissions (Object Read & Write)
- Vérifiez que le token est associé au bon bucket

### Le registry utilise le stockage en mémoire

- Vérifiez que `R2_ACCOUNT_ID` est défini dans `.env`
- Redémarrez le registry server après avoir modifié `.env`

## Sécurité

⚠️ **Important** :

- Ne commitez JAMAIS votre fichier `.env` dans Git
- Gardez vos credentials R2 secrets
- Utilisez des tokens API avec des permissions minimales
- Considérez l'utilisation de secrets management en production

## Ressources

- [Documentation Cloudflare R2](https://developers.cloudflare.com/r2/)
- [API S3 compatible](https://developers.cloudflare.com/r2/api/s3/)
- [Pricing R2](https://developers.cloudflare.com/r2/pricing/)
