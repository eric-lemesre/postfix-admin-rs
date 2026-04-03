> **Language:** [English](../en/guidelines/CODE-REVIEW.md) | Francais

# Guidelines Code Review — postfix-admin-rs

## Objectif

Assurer la qualité, la sécurité et la maintenabilité du code via une revue
systématique avant intégration.

---

## 1. Rôles

### Auteur de la PR
- S'assure que la PR compile, que les tests passent et que Clippy est satisfait
- Fournit un contexte clair dans la description
- Répond aux commentaires dans un délai raisonnable
- Ne merge pas sa propre PR (sauf hotfix critique avec accord)

### Reviewer
- Vérifie la logique métier et la cohérence architecturale
- Identifie les problèmes de sécurité, de performance et de maintenabilité
- Propose des améliorations constructives (pas de critique personnelle)
- Approuve explicitement ou demande des modifications

---

## 2. Checklist de revue

### Architecture et design

- [ ] Le code respecte la séparation des responsabilités entre crates
- [ ] Les dépendances entre crates vont dans le bon sens (pas de cycle)
- [ ] Les nouveaux types publics sont justifiés
- [ ] Le pattern Repository est respecté pour l'accès aux données
- [ ] Les traits sont utilisés pour l'abstraction (pas de couplage direct aux implémentations)

### Logique métier

- [ ] Les règles métier documentées dans les specs sont respectées
- [ ] Les cas limites sont gérés (valeurs nulles, chaînes vides, limites numériques)
- [ ] Les validations sont faites au bon niveau (DTO → service → repository)
- [ ] Les transactions SQL englobent les opérations atomiques

### Sécurité

- [ ] Pas d'injection SQL (requêtes paramétrées uniquement)
- [ ] Pas de XSS (données échappées dans les templates)
- [ ] Pas de secrets dans le code (mots de passe, clés API)
- [ ] CSRF protection sur les formulaires POST
- [ ] Les mots de passe ne sont jamais loggés ou sérialisés
- [ ] Les comparaisons de secrets sont constantes en temps
- [ ] Les entrées utilisateur sont validées

### Performance

- [ ] Les requêtes SQL sont paginées (LIMIT/OFFSET)
- [ ] Pas de N+1 queries
- [ ] Les index nécessaires sont créés
- [ ] Les allocations inutiles sont évitées (clones, String vs &str)
- [ ] Le hashing de mot de passe utilise spawn_blocking

### Tests

- [ ] Les nouvelles fonctions publiques ont des tests
- [ ] Les cas d'erreur sont testés
- [ ] Les tests sont indépendants et reproductibles
- [ ] Les noms de tests sont descriptifs

### Style et lisibilité

- [ ] Le code suit les conventions de nommage du projet
- [ ] Les imports sont organisés par groupe
- [ ] Les fonctions font moins de 50 lignes (préférence)
- [ ] Le code est auto-documenté (noms clairs, structure lisible)
- [ ] Les commentaires expliquent le **pourquoi**, pas le **quoi**
- [ ] `cargo fmt` et `cargo clippy` passent

### Documentation

- [ ] Les fonctions publiques ont un `///` doc comment
- [ ] Les modules ont un `//!` en en-tête
- [ ] Les changements d'API sont documentés
- [ ] Les migrations SQL sont documentées
- [ ] Le CHANGELOG est mis à jour si nécessaire

---

## 3. Niveaux de commentaires

| Préfixe       | Signification                                       | Action requise                         |
|---------------|-----------------------------------------------------|----------------------------------------|
| `blocker:`    | Problème critique (sécurité, bug, perte de données) | Doit être corrigé avant merge          |
| `issue:`      | Problème significatif (logique, performance)        | Devrait être corrigé                   |
| `nit:`        | Détail mineur (style, nommage)                      | Optionnel, à la discrétion de l'auteur |
| `question:`   | Demande de clarification                            | Réponse attendue                       |
| `suggestion:` | Proposition d'amélioration                          | À discuter                             |
| `praise:`     | Point positif à souligner                           | Encouragement                          |

### Exemples

```
blocker: Cette requête SQL concatène une entrée utilisateur sans paramétrage.
Utiliser sqlx::query! avec un paramètre $1.

issue: Cette fonction fait 120 lignes. Envisager de l'extraire en sous-fonctions
pour la lisibilité.

nit: Préférer `DomainName` (newtype) plutôt que `String` ici pour la sécurité
des types.

question: Pourquoi utiliser `fetch_one` plutôt que `fetch_optional` ici ?
L'enregistrement pourrait ne pas exister.

suggestion: On pourrait utiliser `impl From<DomainRow> for Domain` plutôt
qu'une méthode `to_domain()` pour suivre les conventions Rust.

praise: Le pattern de rehash transparent des mots de passe est bien conçu,
la gestion des cas limites est complète.
```

---

## 4. Processus

```
1. L'auteur ouvre une PR avec une description complète
2. Le CI exécute les vérifications automatiques
3. Un reviewer est assigné (ou se porte volontaire)
4. Le reviewer examine le code et laisse des commentaires
5. L'auteur répond et apporte les modifications
6. Le reviewer re-examine les modifications
7. Approbation → Merge
```

### Délais

- Première revue : objectif 24h ouvrées
- Réponse aux commentaires : objectif 48h ouvrées
- Si le reviewer n'est pas disponible, un autre peut prendre le relais

---

## 5. Ce qu'on ne fait PAS en revue

- Critiquer la personne (on critique le code, pas l'auteur)
- Imposer des préférences de style non documentées dans les guidelines
- Bloquer une PR pour des détails cosmétiques
- Réécrire la PR dans les commentaires (proposer, ne pas imposer)
- Approuver sans lire le code ("LGTM" sans examen)
