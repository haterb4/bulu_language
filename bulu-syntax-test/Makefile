# Makefile pour les tests de syntaxe Bulu

.PHONY: test test-verbose test-quick test-method test-if test-struct clean help

# Variables
LANG_BINARY = ../target/debug/lang
TEST_SCRIPT = ./run_tests.sh

# Cible par défaut
test: build
	@echo "Exécution de tous les tests..."
	@$(TEST_SCRIPT)

# Test avec sortie détaillée
test-verbose: build
	@echo "Exécution de tous les tests (mode verbose)..."
	@$(TEST_SCRIPT) --verbose

# Test rapide avec timeout réduit
test-quick: build
	@echo "Exécution rapide des tests..."
	@$(TEST_SCRIPT) --timeout=2

# Tests spécifiques par catégorie
test-method: build
	@echo "Exécution des tests de méthodes..."
	@$(TEST_SCRIPT) --filter=method

test-if: build
	@echo "Exécution des tests de conditions if..."
	@$(TEST_SCRIPT) --filter=if

test-struct: build
	@echo "Exécution des tests de structures..."
	@$(TEST_SCRIPT) --filter=struct

test-negative: build
	@echo "Exécution des tests de nombres négatifs..."
	@$(TEST_SCRIPT) --filter=negative

test-comparison: build
	@echo "Exécution des tests de comparaison..."
	@$(TEST_SCRIPT) --filter=comparison

# Construire le binaire lang si nécessaire
build:
	@if [ ! -f "$(LANG_BINARY)" ]; then \
		echo "Construction du binaire lang..."; \
		cd .. && cargo build; \
	fi

# Nettoyer les fichiers temporaires
clean:
	@echo "Nettoyage des fichiers temporaires..."
	@find . -name "*.tmp" -delete 2>/dev/null || true
	@find . -name "*.log" -delete 2>/dev/null || true

# Afficher l'aide
help:
	@echo "Makefile pour les tests de syntaxe Bulu"
	@echo ""
	@echo "Cibles disponibles:"
	@echo "  test          - Exécuter tous les tests"
	@echo "  test-verbose  - Exécuter tous les tests avec sortie détaillée"
	@echo "  test-quick    - Exécuter tous les tests avec timeout réduit"
	@echo "  test-method   - Exécuter seulement les tests de méthodes"
	@echo "  test-if       - Exécuter seulement les tests de conditions if"
	@echo "  test-struct   - Exécuter seulement les tests de structures"
	@echo "  test-negative - Exécuter seulement les tests de nombres négatifs"
	@echo "  test-comparison - Exécuter seulement les tests de comparaison"
	@echo "  build         - Construire le binaire lang"
	@echo "  clean         - Nettoyer les fichiers temporaires"
	@echo "  help          - Afficher cette aide"
	@echo ""
	@echo "Exemples:"
	@echo "  make test                    # Tous les tests"
	@echo "  make test-verbose            # Tous les tests avec détails"
	@echo "  make test-method             # Seulement les tests de méthodes"