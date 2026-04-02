> **Language:** English | [Francais](../fr/guidelines/GUIDELINES-CSS.md)

# Guidelines CSS / Tailwind — postfix-admin-rs

## Philosophy

The project uses **Tailwind CSS** as the utility-based design system.
Custom CSS is limited to the strict minimum.

---

## 1. Principles

### Utility-first
- Use Tailwind classes as a priority
- Only create custom CSS classes for cases not covered by Tailwind
- Reusable components are Askama templates, not CSS abstractions

### Mobile-first
- Base classes apply to mobile
- Breakpoints (`sm:`, `md:`, `lg:`, `xl:`) add desktop variants
- The interface must be usable on screens with a minimum width of 375px

### Dark mode
- Use the `class` strategy (not `media`)
- All pages must support dark mode
- Every background and text color must have its `dark:` variant

---

## 2. Tailwind Conventions

### Order of classes

Follow the logical order (layout → box → typography → visual → states):

```html
<div class="
    flex items-center justify-between     <!-- Layout -->
    w-full max-w-4xl mx-auto p-4 mb-6    <!-- Sizing / Spacing -->
    text-sm font-medium text-gray-700     <!-- Typography -->
    bg-white rounded-lg shadow-sm border  <!-- Visual -->
    hover:shadow-md transition-shadow     <!-- States / Animations -->
    dark:bg-gray-800 dark:text-gray-200   <!-- Dark mode -->
">
```

### Recurring Tailwind components

#### Primary button
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

#### Danger button
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

#### Table
```html
<table class="min-w-full divide-y divide-gray-300 dark:divide-gray-700">
    <thead class="bg-gray-50 dark:bg-gray-800">
        <tr>
            <th class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900 dark:text-gray-100">
```

---

## 3. Color Palette

| Role | Light | Dark |
|------|-------|------|
| Primary background | `bg-white` | `dark:bg-gray-900` |
| Secondary background | `bg-gray-50` | `dark:bg-gray-800` |
| Primary text | `text-gray-900` | `dark:text-gray-100` |
| Secondary text | `text-gray-500` | `dark:text-gray-400` |
| Primary accent | `indigo-600` | `dark:indigo-400` |
| Success | `green-600` | `dark:green-400` |
| Warning | `amber-500` | `dark:amber-400` |
| Error | `red-600` | `dark:red-400` |
| Borders | `border-gray-200` | `dark:border-gray-700` |

---

## 4. Custom CSS (minimal)

```css
/* static/css/custom.css */

/* HTMX loading indicator */
.htmx-indicator {
    display: none;
}
.htmx-request .htmx-indicator,
.htmx-request.htmx-indicator {
    display: inline-flex;
}

/* Spinner animation */
@keyframes spin {
    to { transform: rotate(360deg); }
}
.animate-spin {
    animation: spin 1s linear infinite;
}

/* HTMX transitions */
.htmx-settling {
    opacity: 0;
}
.htmx-swapping {
    opacity: 0;
    transition: opacity 0.15s ease-out;
}
```

---

## 5. Responsive Breakpoints

| Breakpoint | Min width | Usage |
|------------|-----------|-------|
| (base)     | 0px       | Mobile |
| `sm:`      | 640px     | Tablet portrait |
| `md:`      | 768px     | Tablet landscape |
| `lg:`      | 1024px    | Desktop |
| `xl:`      | 1280px    | Large screen |

### Responsive rules

- The sidebar is hidden on mobile (drawer with hamburger)
- Tables are horizontally scrollable on mobile
- Forms switch to a single column on mobile
- Dashboard statistics are in an adaptive grid

---

## 6. Tailwind Configuration

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
                // Adjust if the base theme doesn't suit
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

## 7. What is Prohibited

- No CSS-in-JS
- No SCSS/SASS/LESS (Tailwind is sufficient)
- No `!important` (sign of a specificity problem)
- No inline styles (`style="..."`) except for documented exceptional cases
- No absolute positioning classes without relative container
- No `z-index` > 50 without justification (management of the z-index stack)

---
