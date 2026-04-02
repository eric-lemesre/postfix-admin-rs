> **Language:** [English](../en/guidelines/GUIDELINES-JS.md) | Francais

# Guidelines JavaScript — postfix-admin-rs

## Philosophie

Ce projet utilise une approche **minimal JavaScript**. L'essentiel de la logique
est cote serveur (Rust). Le JavaScript est limite a :

1. **HTMX** — Interactivite declarative (requetes AJAX, mises a jour partielles du DOM)
2. **Alpine.js** — Micro-interactivite (dropdowns, modales, toggles)
3. **Scripts utilitaires** — Logique specifique non couverte par HTMX/Alpine

Aucun framework lourd (React, Vue, Angular) n'est utilise.

---

## Table des matieres

1. [Principes generaux](#1-principes-generaux)
2. [HTMX](#2-htmx)
3. [Alpine.js](#3-alpinejs)
4. [Scripts custom](#4-scripts-custom)
5. [Organisation des fichiers](#5-organisation-des-fichiers)
6. [Securite](#6-securite)
7. [Accessibilite](#7-accessibilite)
8. [Performance](#8-performance)
9. [Tests](#9-tests)

---

## 1. Principes generaux

### Vanilla first
- Ne pas ajouter de dependance JS si le navigateur ou HTMX/Alpine peut le faire
- Pas de jQuery, pas de lodash, pas de moment.js
- Utiliser les APIs natives du navigateur (`fetch`, `URLSearchParams`, `FormData`, etc.)

### Progressive enhancement
- Les pages doivent fonctionner sans JavaScript (formulaires HTML classiques)
- HTMX et Alpine enrichissent l'experience mais ne sont pas indispensables
- Les formulaires doivent avoir une action et un method HTML standard

### Pas de bundler
- Pas de webpack, vite, parcel, esbuild
- Les scripts sont charges directement (CDN ou servis statiquement)
- Si un build est necessaire pour Tailwind CSS, il est gere separement

### Version ECMAScript
- Cible : ES2020+ (navigateurs modernes)
- Pas de transpilation, pas de polyfills
- Utiliser `const`/`let` (jamais `var`)
- Utiliser les arrow functions, template literals, destructuring, optional chaining

---

## 2. HTMX

### Conventions

```html
<!-- Les attributs HTMX sont declares dans l'ordre suivant : -->
<button
    hx-delete="/api/v1/domains/example.com"
    hx-target="#domain-row-example"
    hx-swap="outerHTML"
    hx-confirm="Supprimer le domaine example.com ?"
    hx-indicator="#spinner"
    hx-headers='{"X-CSRF-Token": "..."}'
>
    Supprimer
</button>
```

Ordre des attributs :
1. Methode + URL (`hx-get`, `hx-post`, `hx-delete`, etc.)
2. Cible (`hx-target`)
3. Strategie de swap (`hx-swap`)
4. Confirmation (`hx-confirm`)
5. Indicateur (`hx-indicator`)
6. Headers (`hx-headers`)

### Patterns recommandes

#### Pagination
```html
<div id="domain-list">
    <nav hx-boost="true">
        <a href="/domains?page=2" hx-target="#domain-list">Page suivante</a>
    </nav>
</div>
```

#### Recherche live
```html
<input
    type="search"
    name="search"
    hx-get="/domains"
    hx-target="#domain-list"
    hx-trigger="keyup changed delay:300ms"
    hx-indicator="#search-spinner"
    placeholder="Rechercher..."
/>
```

#### Toggle inline
```html
<button
    hx-patch="/domains/example.com/active"
    hx-target="this"
    hx-swap="outerHTML"
    class="toggle-btn"
>
</button>
```

#### Formulaire avec erreurs inline
```html
<form
    hx-post="/domains"
    hx-target="#form-errors"
    hx-swap="innerHTML"
    hx-target-error="#form-errors"
>
    <div id="form-errors"></div>
    <button type="submit" hx-indicator="#submit-spinner">
        Creer
        <span id="submit-spinner" class="htmx-indicator">...</span>
    </button>
</form>
```

### Regles HTMX

- Toujours definir `hx-target` explicitement
- Utiliser `hx-push-url="true"` pour les navigations qui doivent modifier l'URL
- Le serveur retourne des fragments HTML (pas des pages completes) pour les requetes HTMX
- Detecter les requetes HTMX cote serveur via le header `HX-Request: true`
- Utiliser `hx-swap="outerHTML"` pour remplacer l'element, `innerHTML` pour le contenu

### CSRF avec HTMX

```html
<meta name="csrf-token" content="{{ csrf_token }}">
<script>
    document.body.addEventListener('htmx:configRequest', function(event) {
        const token = document.querySelector('meta[name="csrf-token"]').content;
        event.detail.headers['X-CSRF-Token'] = token;
    });
</script>
```

---

## 3. Alpine.js

### Conventions

```html
<div x-data="{ open: false }" class="relative">
    <button @click="open = !open">Menu</button>
    <div x-show="open" @click.away="open = false" x-transition>
    </div>
</div>
```

### Quand utiliser Alpine.js

| Cas d'usage | HTMX | Alpine.js |
|-------------|------|-----------|
| Charger des donnees serveur | Oui | Non |
| Dropdown/menu | Non | Oui |
| Modal de confirmation | Non | Oui |
| Toggle de visibilite locale | Non | Oui |
| Validation cote client | Non | Oui |
| Onglets locaux | Non | Oui |
| Soumission de formulaire | Oui | Non |
| Mise a jour de liste | Oui | Non |

### Regles Alpine.js

- Garder `x-data` simple (pas de logique complexe)
- Si un composant depasse 10 lignes de JS, l'extraire dans un fichier separe
- Utiliser `x-ref` pour les references DOM plutot que les selecteurs
- Preferer `x-show` a `x-if` sauf si le DOM ne doit pas exister du tout

### Composants reutilisables

```html
<template x-teleport="body">
    <div
        x-data="confirmModal()"
        x-show="isOpen"
        x-transition
        class="fixed inset-0 z-50 flex items-center justify-center"
        @confirm-delete.window="open($event.detail)"
    >
        <div class="bg-white rounded-lg shadow-xl p-6">
            <h3 x-text="title"></h3>
            <p x-text="message"></p>
            <div class="flex gap-2 mt-4">
                <button @click="cancel()">Annuler</button>
                <button @click="confirm()">Confirmer</button>
            </div>
        </div>
    </div>
</template>
```

---

## 4. Scripts custom

### Structure d'un module

```javascript
// static/js/modules/password-strength.js

/**
 * Evalue la force d'un mot de passe et met a jour l'indicateur visuel.
 */

const STRENGTH_LEVELS = {
    WEAK: { label: 'Faible', class: 'bg-red-500', min: 0 },
    FAIR: { label: 'Moyen', class: 'bg-amber-500', min: 40 },
    GOOD: { label: 'Bon', class: 'bg-green-400', min: 70 },
    STRONG: { label: 'Fort', class: 'bg-green-600', min: 90 },
};

function calculateStrength(password) {
    let score = 0;
    if (password.length >= 8) score += 20;
    if (password.length >= 12) score += 15;
    if (/[a-z]/.test(password) && /[A-Z]/.test(password)) score += 20;
    if (/\d/.test(password)) score += 20;
    if (/[^a-zA-Z0-9]/.test(password)) score += 25;
    return Math.min(score, 100);
}

function getStrengthLevel(score) {
    if (score >= STRENGTH_LEVELS.STRONG.min) return STRENGTH_LEVELS.STRONG;
    if (score >= STRENGTH_LEVELS.GOOD.min) return STRENGTH_LEVELS.GOOD;
    if (score >= STRENGTH_LEVELS.FAIR.min) return STRENGTH_LEVELS.FAIR;
    return STRENGTH_LEVELS.WEAK;
}

window.passwordStrength = { calculateStrength, getStrengthLevel };
```

### Conventions de code

```javascript
// Utiliser const par defaut, let uniquement si reassignation necessaire
const config = { timeout: 5000 };
let count = 0;

// Arrow functions pour les callbacks
items.filter(item => item.active);
items.map(item => ({ ...item, label: item.name.toUpperCase() }));

// Destructuring
const { domain, active } = response.data;

// Optional chaining et nullish coalescing
const name = user?.profile?.name ?? 'Anonyme';

// Template literals pour la construction de chaines
const url = `/api/v1/domains/${encodeURIComponent(domain)}`;

// Async/await plutot que .then()
async function fetchDomain(name) {
    const response = await fetch(`/api/v1/domains/${name}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}
```

### Pratiques interdites

- **`var`** : Utiliser `const` ou `let` exclusivement
- **`==`** : Toujours utiliser `===` (exception : `x == null` pour null/undefined)
- **Execution dynamique de chaines comme code** : Aucune API d'interpretation
  dynamique de code ne doit etre utilisee (risque d'injection de code)
- **Injection de HTML non sanitise** : Utiliser `textContent` ou les methodes DOM
  pour inserer du contenu. Ne jamais injecter du HTML brut provenant de donnees utilisateur
- **Ecriture directe dans le document** : Utiliser les methodes DOM (`createElement`,
  `appendChild`) pour manipuler le DOM
- **`console.log` en production** : Retirer avant chaque commit
- **`setTimeout`/`setInterval` avec des chaines** : Utiliser des fonctions callback

---

## 5. Organisation des fichiers

```
static/
├── js/
│   ├── htmx.min.js           # HTMX (vendor, versionne)
│   ├── alpine.min.js          # Alpine.js (vendor, versionne)
│   ├── app.js                 # Initialisation globale (CSRF, HTMX config)
│   └── modules/               # Modules specifiques
│       ├── password-strength.js
│       ├── qr-code.js         # Affichage QR code TOTP
│       └── clipboard.js       # Copier dans le presse-papier
├── css/
│   ├── output.css             # Tailwind compile
│   └── custom.css             # Styles custom (minimal)
└── images/
    ├── logo.svg
    └── favicon.ico
```

### Chargement des scripts

```html
<script src="/static/js/htmx.min.js" defer></script>
<script src="/static/js/alpine.min.js" defer></script>
<script src="/static/js/app.js" defer></script>
```

Les modules specifiques sont charges uniquement sur les pages qui en ont besoin
via un bloc template `{% block scripts %}{% endblock %}`.

---

## 6. Securite

### XSS
- Ne jamais injecter du HTML brut avec des donnees non sanitisees
- Askama echappe par defaut cote serveur
- Alpine.js `x-text` echappe par defaut (jamais `x-html` avec des donnees utilisateur)
- Les attributs HTMX ne doivent pas contenir de donnees utilisateur non encodees

### CSRF
- Tous les POST/PUT/DELETE HTMX incluent le token CSRF (voir section HTMX)
- Le token est renouvele a chaque session

### CSP (Content Security Policy)
```
Content-Security-Policy:
    default-src 'self';
    script-src 'self';
    style-src 'self' 'unsafe-inline';
    img-src 'self' data:;
    connect-src 'self';
    font-src 'self';
    frame-src 'none';
    object-src 'none';
```

### Dependances tierces
- Seuls HTMX et Alpine.js sont autorises comme dependances JS
- Les fichiers vendors sont versionnes et servis localement (pas de CDN en prod)
- Verifier l'integrite des fichiers vendors (SRI hash)

---

## 7. Accessibilite

### ARIA avec HTMX
```html
<div
    id="domain-list"
    aria-live="polite"
    aria-busy="false"
    hx-get="/domains"
    hx-indicator="#spinner"
>
```

```javascript
document.body.addEventListener('htmx:beforeRequest', function(event) {
    const target = event.detail.target;
    if (target) target.setAttribute('aria-busy', 'true');
});
document.body.addEventListener('htmx:afterRequest', function(event) {
    const target = event.detail.target;
    if (target) target.setAttribute('aria-busy', 'false');
});
```

### ARIA avec Alpine.js
```html
<div x-data="{ open: false }">
    <button
        @click="open = !open"
        :aria-expanded="open"
        aria-controls="dropdown-menu"
    >
        Menu
    </button>
    <div id="dropdown-menu" x-show="open" role="menu">
        <a role="menuitem" href="...">Option 1</a>
    </div>
</div>
```

### Regles d'accessibilite

- Tous les elements interactifs doivent etre accessibles au clavier
- Les messages d'erreur doivent etre associes aux inputs (`aria-describedby`)
- Les regions dynamiques (mises a jour HTMX) utilisent `aria-live`
- Le focus est gere correctement apres les mises a jour DOM

---

## 8. Performance

- Charger les scripts avec `defer`
- HTMX et Alpine sont legers (~14 Ko et ~15 Ko gzippes)
- Les reponses HTMX sont des fragments HTML minimaux
- Utiliser le cache navigateur pour les assets statiques (hash dans le nom de fichier)
- Debounce les recherches, throttle les actions repetees

---

## 9. Tests

### Tests E2E (recommandes)

Pour valider l'integration HTMX + serveur, utiliser des tests E2E avec Playwright :

```javascript
// tests/e2e/domains.spec.js
test('peut creer un domaine via le formulaire', async ({ page }) => {
    await page.goto('/domains/new');
    await page.fill('input[name="domain"]', 'test-e2e.com');
    await page.fill('input[name="description"]', 'Test E2E');
    await page.click('button[type="submit"]');
    await page.waitForSelector('text=test-e2e.com');
});

test('la recherche filtre les domaines en temps reel', async ({ page }) => {
    await page.goto('/domains');
    await page.fill('input[name="search"]', 'example');
    await page.waitForTimeout(400);
    const rows = await page.locator('table tbody tr').count();
    expect(rows).toBeGreaterThan(0);
});
```
