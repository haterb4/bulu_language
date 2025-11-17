# Changelog

Toutes les modifications notables de l'extension Bulu pour VS Code seront documentées ici.

## [0.1.0] - 2025-11-17

### Ajouté
- 🎨 Coloration syntaxique complète pour Bulu
  - Mots-clés (if, func, struct, async, etc.)
  - Types primitifs et collections
  - Fonctions et fonctions built-in
  - Commentaires (ligne et bloc)
  - Chaînes de caractères avec échappements
  - Nombres (décimal, hexadécimal, binaire, octal)
  - Opérateurs arithmétiques, logiques, et de comparaison
  
- 🚀 Intégration Language Server Protocol (LSP)
  - Auto-complétion intelligente
  - Go-to-definition (F12)
  - Find references (Shift+F12)
  - Hover documentation
  - Diagnostics en temps réel
  - Rename refactoring (F2)
  - Code actions et quick fixes
  - Signature help
  
- 📝 Snippets pour constructions courantes
  - Déclarations de fonction (func, afunc)
  - Structures et interfaces
  - Structures de contrôle (if, for, while, match)
  - Gestion d'erreurs (try-fail)
  - Concurrence (run, chan, select)
  - Variables et constantes
  
- ⚙️ Configuration personnalisable
  - Chemin vers bulu_lsp
  - Activation/désactivation du LSP
  - Options de trace pour débogage
  - Format et lint on save
  
- 🛠️ Commandes VS Code
  - Restart Language Server
  - Show Output Channel
  
- 📚 Documentation complète
  - README avec exemples
  - Guide d'installation
  - Dépannage
  
- 🎯 Support des fichiers .bu
  - Détection automatique
  - Icône de fichier personnalisé
  - Configuration de langage (brackets, indentation, etc.)

### Technique
- Configuration TypeScript stricte
- Client LSP basé sur vscode-languageclient
- Gestion d'erreurs robuste
- Logs détaillés pour débogage
- Package optimisé pour distribution

## [À venir]

### Version 0.2.0
- [ ] Thème de couleurs Bulu personnalisé
- [ ] Intégration du débogueur
- [ ] Support du formatage automatique
- [ ] Linting avancé avec règles configurables
- [ ] Snippets additionnels
- [ ] Tests d'intégration
- [ ] Support multi-workspace

### Version 0.3.0
- [ ] Refactorings avancés (extract method, inline variable)
- [ ] Code lens pour tests
- [ ] Inlay hints pour types
- [ ] Semantic highlighting
- [ ] Import organization
- [ ] Documentation generator

---

Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/)
