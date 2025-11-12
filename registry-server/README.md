# Bulu Package Registry Server

Serveur de registry HTTP local pour les packages Bulu.

## Démarrage

```bash
cd registry-server
cargo run
```

Le serveur démarre sur `http://localhost:3000`

## API Endpoints

### GET /
Informations sur le registry

### GET /api/packages
Liste tous les packages

### GET /api/packages/:name
Informations sur un package spécifique

### GET /api/packages/:name/versions
Liste les versions d'un package

### GET /api/packages/:name/:version
Informations sur une version spécifique

### POST /api/publish
Publier un nouveau package

Body:
```json
{
  "name": "package-name",
  "version": "1.0.0",
  "description": "Description",
  "authors": ["Author <email>"],
  "license": "MIT",
  "repository": "https://github.com/user/repo",
  "keywords": ["keyword1", "keyword2"],
  "dependencies": {
    "dep1": "^1.0.0"
  },
  "tarball": "base64_encoded_tarball"
}
```

### GET /api/search?q=query&limit=20
Rechercher des packages

### GET /api/download/:name/:version
Télécharger un package (tarball)

## Utilisation avec Bulu

Configurer le registry dans `~/.bulu/config.toml`:

```toml
[registry]
url = "http://localhost:3000"
```

Ou dans le projet `lang.toml`:

```toml
[package.registry]
url = "http://localhost:3000"
```

## Exemples

### Publier un package

```bash
# Depuis le dossier du package
lang publish --registry http://localhost:3000
```

### Rechercher des packages

```bash
lang search http --registry http://localhost:3000
```

### Installer un package

```bash
lang add math-utils --registry http://localhost:3000
```
