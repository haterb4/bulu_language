#!/bin/bash

# Script pour exécuter tous les tests de syntaxe Bulu
# Usage: ./run_tests.sh [--verbose] [--timeout=N]

set -e

# Configuration par défaut
VERBOSE=false
TIMEOUT=5

# Détecter le répertoire de base du projet
SCRIPT_DIR="$(dirname "$0")"
if [[ -f "$SCRIPT_DIR/../target/debug/lang" ]]; then
    # Appelé depuis bulu-syntax-test/
    LANG_BINARY="$SCRIPT_DIR/../target/debug/lang"
    TEST_DIR="$SCRIPT_DIR/src"
elif [[ -f "./target/debug/lang" ]]; then
    # Appelé depuis la racine du projet
    LANG_BINARY="./target/debug/lang"
    TEST_DIR="bulu-syntax-test/src"
else
    echo "Erreur: Impossible de trouver le binaire lang"
    echo "Veuillez exécuter ce script depuis la racine du projet ou depuis bulu-syntax-test/"
    exit 1
fi
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Couleurs pour l'affichage
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Fonction d'aide
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --verbose, -v     Afficher la sortie complète des tests"
    echo "  --timeout=N       Timeout en secondes (défaut: 5)"
    echo "  --filter=PATTERN  Exécuter seulement les tests correspondant au pattern"
    echo "  --help, -h        Afficher cette aide"
    echo ""
    echo "Exemples:"
    echo "  $0                    # Exécuter tous les tests"
    echo "  $0 --verbose          # Exécuter avec sortie détaillée"
    echo "  $0 --filter=method    # Exécuter seulement les tests contenant 'method'"
    echo "  $0 --timeout=10       # Utiliser un timeout de 10 secondes"
}

# Analyser les arguments
FILTER=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --timeout=*)
            TIMEOUT="${1#*=}"
            shift
            ;;
        --filter=*)
            FILTER="${1#*=}"
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            echo "Option inconnue: $1"
            show_help
            exit 1
            ;;
    esac
done

# Vérifier que le binaire lang existe
if [[ ! -f "$LANG_BINARY" ]]; then
    echo -e "${RED}Erreur: Le binaire lang n'existe pas à $LANG_BINARY${NC}"
    echo "Veuillez d'abord compiler avec: cargo build"
    exit 1
fi

# Fonction pour exécuter un test
run_test() {
    local test_file="$1"
    local test_name=$(basename "$test_file" .bu)
    
    # Appliquer le filtre si spécifié
    if [[ -n "$FILTER" && "$test_name" != *"$FILTER"* ]]; then
        return 0
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    printf "%-50s " "$test_name"
    
    # Exécuter le test avec timeout
    local output
    local exit_code
    
    if output=$(timeout "$TIMEOUT" "$LANG_BINARY" run "$test_file" 2>&1); then
        exit_code=0
    else
        exit_code=$?
    fi
    
    # Analyser le résultat
    case $exit_code in
        0)
            echo -e "${GREEN}PASS${NC}"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            if [[ "$VERBOSE" == "true" ]]; then
                echo -e "${BLUE}Output:${NC}"
                echo "$output" | sed 's/^/  /'
                echo ""
            fi
            ;;
        124)
            echo -e "${YELLOW}TIMEOUT${NC}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            if [[ "$VERBOSE" == "true" ]]; then
                echo -e "${YELLOW}Test timed out after ${TIMEOUT}s${NC}"
                echo ""
            fi
            ;;
        *)
            echo -e "${RED}FAIL${NC}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            if [[ "$VERBOSE" == "true" ]]; then
                echo -e "${RED}Error output:${NC}"
                echo "$output" | sed 's/^/  /'
                echo ""
            fi
            ;;
    esac
}

# En-tête
echo -e "${BLUE}=== Exécution des tests de syntaxe Bulu ===${NC}"
echo "Répertoire des tests: $TEST_DIR"
echo "Timeout: ${TIMEOUT}s"
if [[ -n "$FILTER" ]]; then
    echo "Filtre: $FILTER"
fi
echo ""

# Exécuter tous les tests .bu (sauf main.bu)
for test_file in "$TEST_DIR"/*.bu; do
    if [[ -f "$test_file" && "$(basename "$test_file")" != "main.bu" ]]; then
        run_test "$test_file"
    fi
done

# Résumé
echo ""
echo -e "${BLUE}=== Résumé ===${NC}"
echo "Total des tests: $TOTAL_TESTS"
echo -e "Réussis: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Échoués: ${RED}$FAILED_TESTS${NC}"

if [[ $FAILED_TESTS -eq 0 ]]; then
    echo -e "${GREEN}Tous les tests sont passés ! 🎉${NC}"
    exit 0
else
    echo -e "${RED}$FAILED_TESTS test(s) ont échoué${NC}"
    exit 1
fi