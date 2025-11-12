# math-utils

Bibliothèque d'utilitaires mathématiques pour Bulu.

## Installation

```bash
lang add math-utils
```

## Utilisation

```bulu
import { Point2D, Vector2D, distance, sqrt, PI } from "math-utils"

func main() {
    // Géométrie
    let p1 = Point2D.new(0.0, 0.0)
    let p2 = Point2D.new(3.0, 4.0)
    let dist = distance(p1, p2)
    
    // Vecteurs
    let v = Vector2D.new(3.0, 4.0)
    let mag = v.magnitude()
    
    // Mathématiques
    let root = sqrt(16.0)
    let circle_area = PI * pow(5.0, 2)
}
```

## Fonctionnalités

- ✅ Constantes mathématiques (PI, E, GOLDEN_RATIO)
- ✅ Fonctions de base (abs, max, min, pow, sqrt)
- ✅ Géométrie 2D (Point2D, distance, midpoint)
- ✅ Vecteurs 2D (Vector2D, magnitude, normalize, dot product)
- ✅ Statistiques (mean, median)
- ✅ Utilitaires (factorial, gcd, isEven, isOdd)

## Exemples

Voir le dossier `examples/` pour des exemples complets.

## Licence

MIT
