# Tests de Syntaxe Bulu

Ce répertoire contient une suite de tests pour vérifier la syntaxe et le comportement du langage Bulu.

## Structure

```
bulu-syntax-test/
├── src/                 # Fichiers de test (.bu)
├── run_tests.sh         # Script principal pour exécuter les tests
├── Makefile            # Makefile pour faciliter l'exécution
└── README.md           # Ce fichier
```

## Utilisation

### Avec le script bash

```bash
# Exécuter tous les tests
./run_tests.sh

# Exécuter avec sortie détaillée
./run_tests.sh --verbose

# Exécuter avec un timeout personnalisé
./run_tests.sh --timeout=10

# Filtrer les tests par nom
./run_tests.sh --filter=method
./run_tests.sh --filter=if
./run_tests.sh --filter=struct
```

### Avec Make

```bash
# Exécuter tous les tests
make test

# Exécuter avec sortie détaillée
make test-verbose

# Tests rapides (timeout réduit)
make test-quick

# Tests par catégorie
make test-method      # Tests de méthodes
make test-if          # Tests de conditions if
make test-struct      # Tests de structures
make test-negative    # Tests de nombres négatifs
make test-comparison  # Tests de comparaison

# Afficher l'aide
make help
```

## Types de Tests

### Tests de Méthodes
- `test_method_*.bu` - Tests d'appels de méthodes
- `test_simple_method*.bu` - Tests de méthodes simples

### Tests de Conditions
- `test_if_*.bu` - Tests de conditions if/else
- `test_comparison.bu` - Tests d'opérateurs de comparaison

### Tests de Structures
- `test_struct*.bu` - Tests de définition et utilisation de structures
- `test_visibility.bu` - Tests de visibilité des membres

### Tests de Nombres
- `test_negative*.bu` - Tests de nombres négatifs
- `test_power*.bu` - Tests d'opérateurs de puissance

### Tests Divers
- `test_functions.bu` - Tests de fonctions
- `test_variables.bu` - Tests de variables
- `test_debug*.bu` - Tests de débogage

## Sortie

Le script affiche pour chaque test :
- **PASS** (vert) : Le test s'est exécuté sans erreur
- **FAIL** (rouge) : Le test a échoué avec une erreur
- **TIMEOUT** (jaune) : Le test a dépassé le timeout

### Exemple de sortie

```
=== Exécution des tests de syntaxe Bulu ===
Répertoire des tests: ./src
Timeout: 5s

test_basic_negative                               PASS
test_comparison                                   PASS
test_method_in_if                                PASS
test_negative_numbers                            FAIL
test_struct_creation                             PASS

=== Résumé ===
Total des tests: 5
Réussis: 4
Échoués: 1
```

## Ajout de Nouveaux Tests

Pour ajouter un nouveau test :

1. Créez un fichier `.bu` dans le répertoire `src/`
2. Nommez-le de manière descriptive (ex: `test_nouvelle_fonctionnalite.bu`)
3. Le script le détectera automatiquement lors de la prochaine exécution

## Dépannage

### Le binaire lang n'existe pas
```bash
cd ..
cargo build
```

### Tests qui traînent
Augmentez le timeout :
```bash
./run_tests.sh --timeout=10
```

### Déboguer un test spécifique
```bash
./run_tests.sh --filter=nom_du_test --verbose
```