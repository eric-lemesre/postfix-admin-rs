> **Language:** [English](../en/guidelines/GUIDELINES-CSS.md) | Francais

# Guidelines CSS / Tailwind — postfix-admin-rs

## Philosophie

Le projet utilise **Tailwind CSS** comme système de design utilitaire.
Le CSS custom est limité au strict minimum.

---

## 1. Principes

### Utility-first
- Utiliser les classes Tailwind en priorité
- Ne créer des classes CSS custom que pour les cas non couverts par Tailwind
- Les composants réutilisables sont des templates Askama, pas des abstractions CSS

### Mobile-first
- Les classes de base s'appliquent au mobile
- Les breakpoints (`sm:`, `md:`, `lg:`, `xl:`) ajoutent les variantes desktop
- L'interface doit être utilisable sur un écran de 375px de large minimum

### Mode sombre
- Utiliser la stratégie `class` (pas `media`)
- Toutes les pages doivent supporter le mode sombre
- Chaque couleur de fond et de texte doit avoir sa variante `dark:`

---

## 2. Conventions Tailwind

### Ordre des classes

Suivre l'ordre logique (layout → box → typographie → visuel → états) :

```html
<div class="
    flex items-center justify-between     <!-- Layout -->
    w-full max-w-4xl mx-auto p-4 mb-6    <!-- Dimensionnement / Espacement -->
    text-sm font-medium text-gray-700     <!-- Typographie -->
    bg-white rounded-lg shadow-sm border  <!-- Visuel -->
    hover:shadow-md transition-shadow     <!-- États / Animations -->
    dark:bg-gray-800 dark:text-gray-200   <!-- Mode sombre -->
">
```

### Composants Tailwind récurrents

#### Bouton primaire
```html
<button class="
    inline-flex items-center gap-2 px-4 py-2
    text-sm font-medium text-white
    bg-indigo-600 rounded-md shadow-sm
    hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2
    focus-visible:outline-offset-2 focus-visible:outline-indigo-600
    disabled:opacity-50 disabled:cursor-not-allowed
    dark:bg-indigo-500 dark:hover:bg-indigo-400
">
```

#### Bouton danger
```html
<button class="
    inline-flex items-center gap-2 px-4 py-2
    text-sm font-medium text-white
    bg-red-600 rounded-md shadow-sm
    hover:bg-red-500 focus-visible:outline focus-visible:outline-2
    focus-visible:outline-offset-2 focus-visible:outline-red-600
    dark:bg-red-500 dark:hover:bg-red-400
">
```

#### Input
```html
<input class="
    block w-full rounded-md border-0 py-1.5 px-3
    text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300
    placeholder:text-gray-400
    focus:ring-2 focus:ring-inset focus:ring-indigo-600
    dark:bg-gray-700 dark:text-white dark:ring-gray-600
    sm:text-sm sm:leading-6
" />
```

#### Tableau
```html
<table class="min-w-full divide-y divide-gray-300 dark:divide-gray-700">
    <thead class="bg-gray-50 dark:bg-gray-800">
        <tr>
            <th class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900 dark:text-gray-100">
```

---

## 3. Palette de couleurs

| Rôle | Light | Dark |
|------|-------|------|
| Fond principal | `bg-white` | `dark:bg-gray-900` |
| Fond secondaire | `bg-gray-50` | `dark:bg-gray-800` |
| Texte principal | `text-gray-900` | `dark:text-gray-100` |
| Texte secondaire | `text-gray-500` | `dark:text-gray-400` |
| Accent primaire | `indigo-600` | `dark:indigo-400` |
| Succès | `green-600` | `dark:green-400` |
| Avertissement | `amber-500` | `dark:amber-400` |
| Erreur | `red-600` | `dark:red-400` |
| Bordures | `border-gray-200` | `dark:border-gray-700` |

---

## 4. CSS custom (minimal)

```css
/* static/css/custom.css */

/* Indicateur de chargement HTMX */
.htmx-indicator {
    display: none;
}
.htmx-request .htmx-indicator,
.htmx-request.htmx-indicator {
    display: inline-flex;
}

/* Animation du spinner */
@keyframes spin {
    to { transform: rotate(360deg); }
}
.animate-spin {
    animation: spin 1s linear infinite;
}

/* Transitions HTMX */
.htmx-settling {
    opacity: 0;
}
.htmx-swapping {
    opacity: 0;
    transition: opacity 0.15s ease-out;
}
```

---

## 5. Responsive breakpoints

| Breakpoint | Largeur min | Usage |
|------------|------------|-------|
| (base) | 0px | Mobile |
| `sm:` | 640px | Tablette portrait |
| `md:` | 768px | Tablette paysage |
| `lg:` | 1024px | Desktop |
| `xl:` | 1280px | Grand écran |

### Règles responsive

- La sidebar est masquée sur mobile (drawer avec hamburger)
- Les tableaux sont scrollables horizontalement sur mobile
- Les formulaires passent en une colonne sur mobile
- Les statistiques du dashboard sont en grille adaptative

---

## 6. Configuration Tailwind

```javascript
// tailwind.config.js
module.exports = {
    content: [
        './templates/**/*.html',
        './static/js/**/*.js',
    ],
    darkMode: 'class',
    theme: {
        extend: {
            colors: {
                // Ajuster si le thème de base ne convient pas
            },
        },
    },
    plugins: [
        require('@tailwindcss/forms'),
        require('@tailwindcss/typography'),
    ],
};
```

---

## 7. Ce qui est interdit

- Pas de CSS-in-JS
- Pas de SCSS/SASS/LESS (Tailwind suffit)
- Pas de `!important` (signe d'un problème de spécificité)
- Pas de styles inline (`style="..."`) sauf cas exceptionnels documentés
- Pas de classes de positionnement absolu sans conteneur relatif
- Pas de `z-index` > 50 sans justification (gestion de la pile z-index)
