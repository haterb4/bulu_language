# Protocole d'ajout d'une fonction built-in dans Bulu

Ce document décrit la procédure complète pour ajouter une nouvelle fonction built-in au langage Bulu. Une fonction built-in est une fonction fournie par le langage lui-même, disponible sans import.

## Vue d'ensemble

Pour qu'une fonction built-in fonctionne correctement dans Bulu, elle doit être ajoutée dans **5 composants principaux** :

1. **Symbol Resolver** - Pour la résolution des symboles
2. **Type Checker** - Pour la vérification des types
3. **Runtime Builtins** - Pour l'interpréteur IR/bytecode
4. **AST Interpreter** - Pour l'interpréteur AST (mode source)
5. **Native Backend** - Pour la compilation native

## Étape 1 : Symbol Resolver

**Fichier** : `src/compiler/symbol_resolver.rs`

**Localisation** : Fonction `is_builtin()`

**Action** : Ajouter le nom de la fonction à la liste des built-ins reconnus.

```rust
fn is_builtin(&self, name: &str) -> bool {
    if matches!(
        name,
        // ... autres built-ins
        // String functions
        | "ord" | "chr"  // <-- Ajouter ici
    ) {
        return true;
    }
    // ...
}
```

**Pourquoi** : Le symbol resolver doit reconnaître la fonction comme un symbole valide pour éviter les erreurs "Undefined symbol".

---

## Étape 2 : Type Checker

**Fichier** : `src/types/checker.rs`

**Localisation** : Fonction `add_builtin_functions()`

**Action** : Ajouter la signature de type de la fonction.

```rust
fn add_builtin_functions(&mut self) {
    let builtin_functions = vec![
        // ... autres built-ins
        // String functions
        ("ord", vec![TypeId::String], Some(TypeId::Int64)),
        ("chr", vec![TypeId::Int64], Some(TypeId::String)),
        // ...
    ];
    // ...
}
```

**Format** : `(nom, vec![types_params], type_retour_optionnel)`

**Pourquoi** : Le type checker doit connaître la signature pour valider les appels et inférer les types de retour.

---

## Étape 3 : Runtime Builtins (Interpréteur IR)

### 3.1 Enregistrement

**Fichier** : `src/runtime/builtins.rs`

**Localisation** : Fonction d'enregistrement appropriée (ex: `register_memory_functions()`)

**Action** : Enregistrer la fonction dans le registre des built-ins.

```rust
fn register_memory_functions(&mut self) {
    self.register("len", builtin_len);
    self.register("ord", builtin_ord);  // <-- Ajouter ici
    self.register("chr", builtin_chr);  // <-- Ajouter ici
}
```

### 3.2 Implémentation

**Fichier** : `src/runtime/builtins.rs`

**Localisation** : Après les autres fonctions built-in

**Action** : Implémenter la fonction.

```rust
/// Convert a character to its ASCII code
pub fn builtin_ord(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "ord() expects exactly 1 argument".to_string(),
        });
    }

    match &args[0] {
        RuntimeValue::String(s) => {
            if s.is_empty() {
                return Err(BuluError::RuntimeError {
                    file: None,
                    message: "ord() requires a non-empty string".to_string(),
                });
            }
            let first_char = s.chars().next().unwrap();
            Ok(RuntimeValue::Int64(first_char as i64))
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: format!("ord() expects a string, got {:?}", args[0].get_type()),
        }),
    }
}
```

**Signature** : `pub fn builtin_xxx(args: &[RuntimeValue]) -> Result<RuntimeValue>`

**Pourquoi** : L'interpréteur IR/bytecode utilise ce registre pour exécuter les built-ins.

---

## Étape 4 : AST Interpreter

### 4.1 Déclaration du cas

**Fichier** : `src/runtime/ast_interpreter.rs`

**Localisation** : Fonction `execute_call_expr()`, dans le match sur `ident.name.as_str()`

**Action** : Ajouter le cas pour la fonction.

```rust
if let Expression::Identifier(ident) = expr.callee.as_ref() {
    match ident.name.as_str() {
        "make" => return self.execute_make_call(expr),
        "println" => return self.execute_println_call(expr),
        "ord" => return self.execute_ord_call(expr),  // <-- Ajouter ici
        "chr" => return self.execute_chr_call(expr),  // <-- Ajouter ici
        _ => {}
    }
    // ...
}
```

### 4.2 Implémentation

**Fichier** : `src/runtime/ast_interpreter.rs`

**Localisation** : Après les autres fonctions `execute_xxx_call()`

**Action** : Implémenter la fonction d'exécution.

```rust
fn execute_ord_call(&mut self, expr: &CallExpr) -> Result<RuntimeValue> {
    if expr.args.len() != 1 {
        return Err(BuluError::RuntimeError {
            message: "ord() requires exactly one argument".to_string(),
            file: self.current_file.clone(),
        });
    }

    let value = self.execute_expression(&expr.args[0])?;
    match value {
        RuntimeValue::String(s) => {
            if s.is_empty() {
                return Err(BuluError::RuntimeError {
                    message: "ord() requires a non-empty string".to_string(),
                    file: self.current_file.clone(),
                });
            }
            let first_char = s.chars().next().unwrap();
            Ok(RuntimeValue::Int64(first_char as i64))
        }
        _ => Err(BuluError::RuntimeError {
            message: "ord() can only be called on strings".to_string(),
            file: self.current_file.clone(),
        }),
    }
}
```

### 4.3 Reconnaissance comme identifiant

**Fichier** : `src/runtime/ast_interpreter.rs`

**Localisation** : Fonction `execute_identifier_expr()`

**Action** : Ajouter le nom à la liste des built-ins reconnus.

```rust
fn execute_identifier_expr(&mut self, expr: &IdentifierExpr) -> Result<RuntimeValue> {
    if let Some(value) = self.environment.get(&expr.name) {
        Ok(value.clone())
    } else {
        // Check if it's a built-in function name
        if matches!(expr.name.as_str(), "ord" | "chr" | "len" | "println" | "print" | "make" | "append" | "close") {
            Ok(RuntimeValue::Null)  // Placeholder
        } else {
            Err(BuluError::RuntimeError {
                message: format!("Undefined variable '{}'", expr.name),
                file: self.current_file.clone(),
            })
        }
    }
}
```

**Pourquoi** : L'AST interpreter doit reconnaître le nom comme valide avant d'évaluer l'appel.

---

## Étape 5 : Native Backend

**Fichier** : `src/compiler/native_backend.rs`

**Localisation** : Fonction `generate_instruction()`, dans le match sur `IrOpcode::Call`

**Action** : Ajouter un cas `else if` pour la fonction.

```rust
} else if name == "ord" {
    // Built-in: ord(c) - convert character to ASCII code
    if let Some(arg) = inst.operands.get(1) {
        match arg {
            IrValue::Register(reg) => {
                if let Some(&offset) = reg_map.get(&reg.id) {
                    // Load the string pointer
                    asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                    // Check if it's a valid string
                    asm.push_str("    cmp $0x1000, %rdi\n");
                    let label_id = *label_counter;
                    *label_counter += 1;
                    asm.push_str(&format!("    jb .{}_ord_error_{}\n", name, label_id));
                    // Get first character (at offset 8)
                    asm.push_str("    movzbq 8(%rdi), %rax\n");
                    asm.push_str(&format!("    jmp .{}_ord_done_{}\n", name, label_id));
                    asm.push_str(&format!(".{}_ord_error_{}:\n", name, label_id));
                    asm.push_str("    mov $0, %rax\n");
                    asm.push_str(&format!(".{}_ord_done_{}:\n", name, label_id));
                    // Store result
                    if let Some(result) = inst.result {
                        if let Some(&res_offset) = reg_map.get(&result.id) {
                            asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                        }
                    }
                }
            }
            IrValue::Constant(IrConstant::String(s)) => {
                // String constant - return ASCII of first character
                if !s.is_empty() {
                    let ascii = s.chars().next().unwrap() as i64;
                    asm.push_str(&format!("    mov ${}, %rax\n", ascii));
                    if let Some(result) = inst.result {
                        if let Some(&res_offset) = reg_map.get(&result.id) {
                            asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
```

**Considérations importantes** :
- Gérer les arguments constants ET variables (Register)
- Utiliser `label_counter` pour générer des labels uniques
- Préfixer les labels avec le nom de la fonction pour éviter les conflits
- Stocker le résultat dans le registre approprié

**Pourquoi** : Le backend natif doit générer le code assembleur x86-64 pour la fonction.

---

## Checklist complète

Avant de considérer qu'une fonction built-in est complète, vérifier :

- [ ] **Symbol Resolver** : Ajouté dans `is_builtin()`
- [ ] **Type Checker** : Ajouté dans `add_builtin_functions()` avec signature correcte
- [ ] **Runtime Builtins** : 
  - [ ] Enregistré dans `register_xxx_functions()`
  - [ ] Implémenté `builtin_xxx()`
- [ ] **AST Interpreter** :
  - [ ] Ajouté dans le match de `execute_call_expr()`
  - [ ] Implémenté `execute_xxx_call()`
  - [ ] Ajouté dans `execute_identifier_expr()`
- [ ] **Native Backend** : Ajouté dans `generate_instruction()` avec génération d'assembleur
- [ ] **Tests** : Testé dans les 3 modes (interpréteur AST, IR, et natif)

---

## Tests de validation

Pour valider qu'une fonction built-in fonctionne correctement :

### Test 1 : Interpréteur AST (mode source)
```bash
lang run --source test.bu
```

### Test 2 : Interpréteur IR (mode bytecode)
```bash
lang run test.bu
```

### Test 3 : Backend natif
```bash
langc build --release
./target/release/program
```

---

## Erreurs courantes à éviter

1. **Oublier un composant** : La fonction ne fonctionnera que partiellement
2. **Mauvaise signature de type** : Erreurs de type checking
3. **Labels non uniques** : Conflits dans le backend natif (utiliser `label_counter`)
4. **Ne pas gérer tous les types d'arguments** : Constantes ET variables
5. **Oublier `execute_identifier_expr()`** : "Undefined variable" dans l'AST interpreter

---

## Exemple complet : Fonction `ord()`

Voir l'implémentation de `ord()` et `chr()` dans le commit correspondant pour un exemple complet et fonctionnel.

---

## Notes importantes

- **JAMAIS** mapper automatiquement des fonctions utilisateur vers des fonctions natives
- Les fonctions built-in doivent être des primitives du langage, pas des fonctions de bibliothèque
- Pour les fonctions de bibliothèque, utiliser la stdlib (`std.xxx`) avec import explicite
- Les built-ins doivent être documentés dans la spécification du langage

---

## Maintenance

Ce protocole doit être mis à jour si l'architecture du compilateur change. Dernière mise à jour : Novembre 2024.
