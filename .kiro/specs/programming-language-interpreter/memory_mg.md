# Epochal Ownership & Tracing (EOT) - Version Simplifiée

## Philosophie core : Trois tiers clairs, pas de chevauchement

Le modèle se simplifie en **trois régions mémoire distinctes** avec des règles claires et **un seul mécanisme de gestion par région**.

### Tier 1 : Ownership statique (Stack + Inline)
### Tier 2 : Arenas epochales  
### Tier 3 : Heap GC (traced uniquement, pas de RC hybride)

---

## 1. Architecture Simplifiée

```
┌─────────────────────────────────────────────────────────┐
│  Stack/Inline Allocation (Ownership pur)                │
│  - Borrow checker classique                             │
│  - Zero overhead runtime                                │
│  - 80% des allocations en pratique                      │
└─────────────────────────────────────────────────────────┘
                        ↓ escape
┌─────────────────────────────────────────────────────────┐
│  Epoch Arenas (Régions temporaires)                     │
│  - Bump allocator O(1)                                   │
│  - Pas de write-barriers (immutables après création)    │
│  - Reset bulk en fin d'epoch                            │
│  - 15% des allocations                                   │
└─────────────────────────────────────────────────────────┘
                        ↓ promotion
┌─────────────────────────────────────────────────────────┐
│  Traced Heap (GC generationnel pause-bounded)           │
│  - Uniquement pour objets vraiment long-lived           │
│  - Write-barriers uniquement ici                        │
│  - Collection incrémentale par générations              │
│  - 5% des allocations                                    │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Règles de Transition Entre Tiers

### Stack → Arena (via escape analysis)
```rust
fn process_request() {
    let arena = Arena::new();  // Nouvelle epoch
    
    // Objets détectés comme "epoch-scoped"
    let data = arena.alloc(parse_json(input));  // Compile-time check
    
    process(data);  // Borrows valides dans l'epoch
    
    // Arena.drop() = reset O(1)
}
```

**Règle** : Un objet arena ne peut pas survivre à l'epoch. Le compilateur refuse :
```rust
let leaked = {
    let arena = Arena::new();
    arena.alloc(Data { ... })  // Erreur : arena-allocated value escapes
};
```

### Arena → Heap (promotion explicite)
```rust
fn cache_result(arena: &Arena) -> Gc<Data> {
    let temp = arena.alloc(compute_heavy());
    
    // Promotion explicite (copie vers heap tracé)
    Gc::promote(temp)  // Atomic allocation + write to young gen
}
```

**Règle** : `Gc::promote()` fait une copie deep. Pas de pointeurs dangling entre arenas et heap.

### Stack → Heap (via escape analysis + Box/Gc)
```rust
fn long_lived() -> Gc<Widget> {
    Gc::new(Widget { ... })  // Direct allocation in traced heap
}
```

---

## 3. Modèle Mémoire Détaillé

### 3.1 Stack/Inline (Tier 1)
- **Ownership strict** comme Rust
- **Lifetimes** vérifiées statiquement
- **Zero cost** : pas de runtime checks
- **Exemple** : Variables locales, arguments, retours

```rust
struct Point { x: i32, y: i32 }

fn distance(p1: Point, p2: Point) -> f64 {
    // Tout sur la stack, zero allocation
    let dx = (p1.x - p2.x) as f64;
    let dy = (p1.y - p2.y) as f64;
    (dx*dx + dy*dy).sqrt()
}
```

### 3.2 Epoch Arenas (Tier 2)

**Implémentation** :
```rust
pub struct Arena {
    chunks: Vec<Chunk>,
    current: *mut u8,
    end: *mut u8,
}

impl Arena {
    pub fn alloc<T>(&self, value: T) -> &T {
        // Fast path: bump pointer
        let ptr = self.current;
        self.current = self.current.add(size_of::<T>());
        
        if self.current > self.end {
            self.allocate_new_chunk();  // Rare slow path
        }
        
        ptr::write(ptr as *mut T, value);
        &*(ptr as *const T)
    }
    
    pub fn reset(&mut self) {
        // O(1) : juste réinitialiser le pointeur
        self.current = self.chunks[0].start;
    }
}
```

**Contraintes** :
- Objets dans arena sont **immutables après création** (sauf via `&mut` exclusive)
- Pas de write-barriers (pas de pointeurs vers heap tracé)
- Lifetime `'arena` imposée par le type-checker

**Use-cases** :
- Parsing (AST temporaire)
- Frames de jeu/UI (reset à 60Hz)
- Request handlers HTTP
- Compilation incrémentale (per-file arena)

### 3.3 Traced Heap (Tier 3)

**Architecture GC** :
```
Young Generation (Nursery)
  - Allocation bump rapide
  - Collection fréquente (1-5ms pauses)
  - Promotion vers Old après N survivals

Old Generation  
  - Collection incrémentale (tri-color marking)
  - Pause bounded via work budgets (target: <10ms)
  - Compaction occasionnelle
```

**Write-Barrier** (uniquement pour Old → Young pointers) :
```rust
#[inline(always)]
fn write_barrier<T>(obj: *mut GcBox<T>, field_offset: usize, new_val: Gc<U>) {
    // Fast path: check si write traverse génération
    if obj.is_old() && new_val.is_young() {
        // Slow path: enregistrer dans remembered set
        REMEMBERED_SET.insert(obj);
    }
}
```

**Implémentation Gc<T>** :
```rust
pub struct Gc<T> {
    ptr: NonNull<GcBox<T>>,  // Pointer opaque
}

// Gc<T> est Copy + Clone (cheap)
impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self { *self }
}

// Borrow via Deref
impl<T> Deref for Gc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &self.ptr.as_ref().data }
    }
}

// Mutation nécessite un lock ou unsafe
impl<T> Gc<T> {
    pub fn get_mut(&mut self) -> Option<&mut T> {
        // Unsafe ou avec runtime check selon politique
    }
}
```

---

## 4. Escape Analysis & Allocation Decision

Le compilateur utilise un **flow-sensitive escape analysis** :

```rust
fn example() {
    let x = Data { ... };  
    // ↑ Analyse : x ne s'échappe pas → Stack
    
    let y = vec![1, 2, 3];
    send_to_thread(y);  
    // ↑ y s'échappe → Heap (Gc ou Arena selon context)
}
```

**Heuristiques** :
1. Si lifetime ≤ function scope → **Stack**
2. Si lifetime ≤ epoch boundary + pas partagé → **Arena**
3. Si lifetime indéfinie ou partagé entre threads → **Heap GC**

**Annotations pour override** :
```rust
#[arena]  // Force allocation arena (error si escape)
fn temp_compute(arena: &Arena) -> &Data { ... }

#[heap]   // Force GC heap allocation
fn cached() -> Gc<Data> { ... }

#[inline_always]  // Force stack/inline
fn hot_path(x: SmallStruct) { ... }
```

---

## 5. Algorithme GC Détaillé

### Phase 1 : Young Generation Collection (stop-the-world, 1-5ms)

```
1. Safepoint : arrêter tous les mutators
2. Scanner roots (stacks + registers + arenas)
3. Copier objets vivants Young → Survivor
4. Mettre à jour pointeurs
5. Résumer mutators
```

**Déclenchement** : quand Young atteint seuil (ex: 8MB)

### Phase 2 : Old Generation Collection (concurrent incremental)

```
1. Mark phase (tri-color, concurrent avec mutators)
   - Gris : à scanner
   - Noir : scanné
   - Blanc : non atteint (garbage)
   
2. Write-barriers maintiennent invariant tri-color
   
3. Sweep phase (concurrent)
   - Free liste des objets blancs
   
4. Compaction (optionnelle, rare)
```

**Déclenchement** : adaptatif basé sur heap size et allocation rate

**Budget de pause** :
```rust
const MAX_INCREMENTAL_WORK_MS: f64 = 2.0;
const TARGET_PAUSE_MS: f64 = 10.0;

fn gc_step() {
    let start = now();
    while now() - start < MAX_INCREMENTAL_WORK_MS {
        process_next_object();
        if work_done { break; }
    }
}
```

---

## 6. Gestion de la Concurrence

### Threads & Arenas
- Chaque thread a ses propres arenas (thread-local)
- Pas de sharing d'objets arena entre threads (compile error)

### Threads & Heap GC
- `Gc<T>` est `Send + Sync` si `T: Send + Sync`
- Sharing automatique via GC
- Mutations protégées par `GcCell<T>` (runtime-checked borrow) ou locks

```rust
use std::sync::Mutex;

let shared: Gc<Mutex<Vec<i32>>> = Gc::new(Mutex::new(vec![]));

thread::spawn(move || {
    shared.lock().unwrap().push(42);
});
```

### Alternative : Actor Model natif
Pour éviter locks explicites, proposer des `GcActor<T>` :
```rust
let actor = GcActor::new(State { ... });

actor.send(|state| {
    state.counter += 1;  // Exécuté séquentiellement
});
```

---

## 7. Syntaxe & Ergonomie

### Syntax minimale (proche Rust)
```rust
// Ownership classique
let mut v = Vec::new();
v.push(1);

// Arena
let arena = Arena::new();
let node = arena.alloc(Node { data: 42 });

// GC heap
let shared = Gc::new(RefCell::new(HashMap::new()));
```

### Modes de compilation
```toml
[profile.realtime]
# Interdit GC allocations, force arenas
gc_mode = "forbidden"

[profile.server]
# Balance perf/simplicité
gc_mode = "adaptive"

[profile.script]
# Max simplicité, GC agressif
gc_mode = "always"
```

---

## 8. Avantages vs Modèle Original

| Aspect | EOT Original | EOT Simplifié |
|--------|--------------|---------------|
| Mécanismes GC | RC + Tracing hybride | Tracing pur |
| Complexité | Très élevée | Moyenne |
| Write-barriers | Partout | Seulement heap GC |
| Pauses GC | "Bornées" (flou) | <10ms garanties |
| Interop tiers | Floue (promotions) | Claire (copies) |
| Overhead RC | Présent | Absent |
| Cycles | Cycle collector nécessaire | Tracé natif |

---

## 9. Implémentation Pragmatique

### Phase 1 (3-6 mois)
- Borrow checker basique
- Stack allocation uniquement
- Arenas bump simples
- Heap avec malloc/free (pas de GC)

### Phase 2 (6-12 mois)
- Escape analysis
- Young gen GC (stop-the-world)
- Write-barriers

### Phase 3 (12-18 mois)
- Old gen incremental
- Concurrent marking
- Tuning pauses

### Phase 4 (18-24 mois)
- Compaction
- NUMA-aware allocation
- Profiling tools

---

## 10. Benchmarks Clés

```rust
// Microbenchmarks
bench_stack_allocation()      // vs Rust
bench_arena_reset()            // vs jemalloc
bench_gc_young_collection()    // vs Go
bench_gc_pause_distribution()  // p50/p99/p999

// Macrobenchmarks  
bench_http_server()            // requests/sec, latency
bench_game_frame()             // 60fps stability
bench_compiler()               // throughput
bench_long_running_cache()     // memory stability
```

**Cibles** :
- Stack/arena : <5% overhead vs Rust
- GC pauses : p99 < 10ms, p999 < 50ms
- Throughput : >80% de Rust pour workloads ownership-heavy

---

## 11. Limites Acceptées

1. **Pas de RC** : cycles sont gérés par tracing, pas de contrôle fin de destruction
2. **Promotions sont des copies** : overhead sur arena→heap, mais sécurité garantie
3. **Write-barriers obligatoires** sur heap GC : 5-10% overhead sur mutations
4. **Escape analysis imparfaite** : peut forcer heap alors que stack suffirait

Ces limites sont acceptables car :
- Elles simplifient drastiquement l'implémentation
- Elles évitent les pires cas (cycle detector, deadlocks RC)
- Le profiler peut identifier les hot paths à optimiser

---

## 12. Outils Développeur

### Visualiseur d'allocations
```bash
$ mycompiler --explain-alloc main.rs
fn process() {
  let x = Data { ... };     // ✓ Stack (does not escape)
  let y = compute();        // ⚠ Heap (escapes via return)
  arena.alloc(parse(file))  // ✓ Arena (epoch-scoped)
}
```

### Profiler GC
```bash
$ myruntime --gc-profile app
GC Stats (60s runtime):
  Young collections: 1203 (avg 2.3ms, p99 4.1ms)
  Old collections: 12 (avg 8.7ms, p99 9.8ms)
  Total pause: 3.1s (5.2% of runtime)
  Heap size: 45MB (peak 67MB)
```

### Linter arenas
```bash
$ mylint --check-arena-leaks
Warning: possible arena escape in function 'cache_build'
  Consider using Gc::promote() if intentional
```

---

## Conclusion

Cette version simplifiée de EOT est **viable et implémentable** car :

1. **Séparation claire** : chaque tier a un seul mécanisme
2. **Complexité réduite** : pas d'hybrides RC/Tracing
3. **Pauses bornées** : GC generationnel bien compris
4. **Escape analysis** : déjà prouvé dans Go/Java
5. **Path incremental** : chaque phase est utilisable

Le modèle reste **puissant** :
- 80%+ allocations zero-cost (stack/arena)
- 15% allocations temporaires O(1) (arena)
- 5% seulement payent le GC

**Prochaine étape** : prototyper le borrow checker + arenas en Rust, mesurer overhead réel.