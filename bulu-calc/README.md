# Bulu Calculator - Version Avancée

Une application de calculatrice complète et avancée écrite en Bulu, démontrant les capacités du langage.

## Fonctionnalités

### 🧮 Calculatrice de Base
- Opérations arithmétiques de base (addition, soustraction, multiplication, division, modulo)
- Gestion du dernier résultat avec opérations chainées
- Fonctions utilitaires (valeur absolue, min/max, moyenne)
- Validation des entrées et gestion d'erreurs

### 🔬 Calculatrice Scientifique
- Opérations mathématiques avancées (puissance, racine carrée)
- Fonctions mathématiques (factorielle, suite de Fibonacci)
- Approximations trigonométriques (sinus, cosinus)
- Système de mémoire (store, recall, clear, add)
- Historique des opérations

### 📐 Constantes Mathématiques
- Constantes prédéfinies (π, e, nombre d'or, √2)
- Conversions d'angles (degrés ↔ radians)
- Accès facile aux constantes courantes

### 🛠 Utilitaires
- Formatage des nombres et résultats
- Validation des entrées utilisateur
- Gestion d'erreurs robuste
- Interface utilisateur claire

## Structure du Projet

```
bulu-calc/
├── src/
│   ├── main.bu                    # Point d'entrée principal
│   ├── calculator/
│   │   ├── calculator.bu          # Calculatrice de base
│   │   └── scientific.bu          # Calculatrice scientifique
│   ├── math/
│   │   ├── operations.bu          # Opérations mathématiques
│   │   └── constants.bu           # Constantes mathématiques
│   └── utils/
│       ├── formatter.bu           # Formatage des sorties
│       └── validator.bu           # Validation des entrées
├── lang.toml                      # Configuration du projet
└── README.md                      # Documentation
```

## Compilation et Exécution

### Compilation
```bash
langc build
```

### Exécution
```bash
# Exécuter l'exécutable compilé
lang run

# Ou exécuter directement le code source
lang run --source
```

## Exemples d'Utilisation

Le programme démontre automatiquement toutes les fonctionnalités :

1. **Opérations de base** : Addition, soustraction, multiplication, division
2. **Opérations avancées** : Puissances, racines carrées, factorielles
3. **Fonctions trigonométriques** : Approximations de sinus et cosinus
4. **Gestion de mémoire** : Stockage et rappel de valeurs
5. **Constantes mathématiques** : Utilisation de π, e, etc.
6. **Calculs complexes** : Aire d'un cercle, suites mathématiques

## Fonctionnalités Démontrées

### Système de Modules
- Import/export entre modules
- Organisation hiérarchique du code
- Réutilisabilité des composants

### Programmation Orientée Objet
- Structures avec méthodes
- Encapsulation des données
- Constructeurs personnalisés

### Gestion d'Erreurs
- Validation des entrées
- Messages d'erreur informatifs
- Gestion des cas limites

### Types de Données
- Entiers (int32, int64)
- Nombres flottants (float64)
- Chaînes de caractères (string)
- Booléens (bool)
- Tableaux ([]string)

### Structures de Contrôle
- Boucles for avec ranges
- Conditions if/else
- Gestion des exceptions (panic)

## Architecture

L'application suit une architecture modulaire claire :

- **Séparation des responsabilités** : Chaque module a un rôle spécifique
- **Réutilisabilité** : Les composants peuvent être utilisés indépendamment
- **Extensibilité** : Facile d'ajouter de nouvelles fonctionnalités
- **Maintenabilité** : Code organisé et bien documenté

## Technologies Utilisées

- **Langage** : Bulu
- **Compilation** : langc (compilateur Bulu vers C puis natif)
- **Exécution** : lang (runtime Bulu)
- **Architecture** : Modulaire avec imports/exports
