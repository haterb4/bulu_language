#!/bin/bash
# Script de test rapide pour l'extension Bulu VS Code

set -e

echo "🧪 Testing Bulu VS Code Extension..."
echo ""

# Vérifier que nous sommes dans le bon répertoire
if [ ! -f "package.json" ]; then
    echo "❌ Error: package.json not found. Run this script from vscode-extension directory."
    exit 1
fi

# Créer un fichier de test
TEST_FILE="test-example.bu"
echo "📝 Creating test file: $TEST_FILE"

cat > $TEST_FILE << 'EOF'
// Test file for Bulu extension
import std.io
import std.math

const PI: float64 = 3.14159
const MAX_SIZE: int32 = 1000

struct Point {
    x: float64
    y: float64
}

func distance(p1: Point, p2: Point): float64 {
    let dx = p2.x - p1.x
    let dy = p2.y - p1.y
    return math.sqrt(dx * dx + dy * dy)
}

async func fetchData(url: string): string {
    let response = await http.get(url)
    return response.body
}

func main() {
    println("Hello, Bulu!")
    
    let p1 = Point { x: 0.0, y: 0.0 }
    let p2 = Point { x: 3.0, y: 4.0 }
    
    let dist = distance(p1, p2)
    println("Distance: " + dist)
    
    // Test concurrency
    let ch: chan int32 = make(chan int32)
    
    run {
        ch <- 42
    }
    
    let value = <- ch
    println(value)
    
    // Test pattern matching
    match value {
        0 -> println("zero")
        1..10 -> println("small")
        _ -> println("other")
    }
    
    // Test error handling
    try {
        if value < 0 {
            fail "Negative value"
        }
    } fail err {
        println("Error: " + err)
    }
}
EOF

echo "✅ Test file created"
echo ""

# Vérifier la grammaire
echo "🔍 Checking grammar file..."
if [ -f "syntaxes/bulu.tmLanguage.json" ]; then
    echo "✅ Grammar file exists"
    
    # Vérifier que c'est du JSON valide
    if command -v jq &> /dev/null; then
        if jq empty syntaxes/bulu.tmLanguage.json 2>/dev/null; then
            echo "✅ Grammar is valid JSON"
        else
            echo "❌ Grammar has JSON errors"
            exit 1
        fi
    else
        echo "⚠️  jq not installed, skipping JSON validation"
    fi
else
    echo "❌ Grammar file not found"
    exit 1
fi
echo ""

# Vérifier les snippets
echo "🔍 Checking snippets..."
if [ -f "snippets/bulu.json" ]; then
    echo "✅ Snippets file exists"
    
    if command -v jq &> /dev/null; then
        SNIPPET_COUNT=$(jq 'length' snippets/bulu.json)
        echo "✅ Found $SNIPPET_COUNT snippets"
    fi
else
    echo "❌ Snippets file not found"
    exit 1
fi
echo ""

# Vérifier la configuration du langage
echo "🔍 Checking language configuration..."
if [ -f "language-configuration.json" ]; then
    echo "✅ Language configuration exists"
    
    if command -v jq &> /dev/null; then
        if jq empty language-configuration.json 2>/dev/null; then
            echo "✅ Language configuration is valid JSON"
        else
            echo "❌ Language configuration has JSON errors"
            exit 1
        fi
    fi
else
    echo "❌ Language configuration not found"
    exit 1
fi
echo ""

# Vérifier le code TypeScript
echo "🔍 Checking TypeScript code..."
if [ -f "src/extension.ts" ]; then
    echo "✅ Extension source exists"
    
    # Vérifier la compilation
    if [ -d "node_modules" ]; then
        echo "📦 Compiling TypeScript..."
        npm run compile 2>&1 | tail -5
        
        if [ -f "out/extension.js" ]; then
            echo "✅ TypeScript compiled successfully"
        else
            echo "❌ TypeScript compilation failed"
            exit 1
        fi
    else
        echo "⚠️  node_modules not found, run 'npm install' first"
    fi
else
    echo "❌ Extension source not found"
    exit 1
fi
echo ""

# Vérifier le package.json
echo "🔍 Checking package.json..."
if command -v jq &> /dev/null; then
    NAME=$(jq -r '.name' package.json)
    VERSION=$(jq -r '.version' package.json)
    DISPLAY_NAME=$(jq -r '.displayName' package.json)
    
    echo "✅ Package: $NAME"
    echo "✅ Version: $VERSION"
    echo "✅ Display Name: $DISPLAY_NAME"
    
    # Vérifier les contributions
    LANGUAGES=$(jq '.contributes.languages | length' package.json)
    GRAMMARS=$(jq '.contributes.grammars | length' package.json)
    SNIPPETS=$(jq '.contributes.snippets | length' package.json)
    COMMANDS=$(jq '.contributes.commands | length' package.json)
    
    echo "✅ Languages: $LANGUAGES"
    echo "✅ Grammars: $GRAMMARS"
    echo "✅ Snippets: $SNIPPETS"
    echo "✅ Commands: $COMMANDS"
fi
echo ""

# Tester avec VS Code (si disponible)
echo "🚀 Testing with VS Code..."
if command -v code &> /dev/null; then
    echo "Opening test file in VS Code..."
    code $TEST_FILE
    echo ""
    echo "✅ Test file opened in VS Code"
    echo ""
    echo "📋 Manual tests to perform:"
    echo "  1. Check syntax highlighting (colors)"
    echo "  2. Type 'func' and press Tab (snippet)"
    echo "  3. Type 'pr' and press Ctrl+Space (completion)"
    echo "  4. Hover over 'println' (hover info)"
    echo "  5. F12 on 'distance' (go-to-definition)"
    echo "  6. Shift+F12 on 'Point' (find references)"
    echo "  7. F2 on 'value' (rename)"
    echo "  8. Check diagnostics in Problems panel"
else
    echo "⚠️  VS Code not found in PATH"
    echo "   Install VS Code or add it to PATH to test"
fi
echo ""

# Résumé
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Extension tests completed!"
echo ""
echo "📁 Test file: $TEST_FILE"
echo ""
echo "Next steps:"
echo "  1. Open $TEST_FILE in VS Code"
echo "  2. Test all features manually"
echo "  3. Check the Output channel: 'Bulu Language Server'"
echo "  4. Run: ./scripts/build.sh to create VSIX package"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
