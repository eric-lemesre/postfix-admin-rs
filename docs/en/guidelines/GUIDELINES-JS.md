> **Language:** English | [Francais](../fr/guidelines/GUIDELINES-JS.md)

# Guidelines JavaScript — postfix-admin-rs
---
## Philosophy

This project uses a **minimal JavaScript** approach. Most of the logic
is server-side (Rust). JavaScript is limited to:

1. **HTMX** — Declarative interactivity (AJAX requests, partial DOM updates)
2. **Alpine.js** — Micro-interactivity (dropdowns, modals, toggles)
3. **Utility scripts** — Specific logic not covered by HTMX/Alpine

No heavy frameworks (React, Vue, Angular) are used.

---
## Table of contents

1. [General principles](#1-general-principles)
2. [HTMX](#2-htmx)
3. [Alpine.js](#3-alpinejs)
4. [Custom scripts](#4-custom-scripts)
5. [File organization](#5-file-organization)
6. [Security](#6-security)
7. [Accessibility](#7-accessibility)
8. [Performance](#8-performance)
9. [Tests](#9-tests)

---
---
## 1. General principles

### Vanilla first
- Do not add JS dependency if the browser or HTMX/Alpine can handle it
- No jQuery, no lodash, no moment.js
- Use native browser APIs (`fetch`, `URLSearchParams`, `FormData`, etc.)

### Progressive enhancement
- Pages must work without JavaScript (standard HTML forms)
- HTMX and Alpine enhance the experience but are not mandatory
- Forms must have a standard HTML action and method

### No bundler
- No webpack, vite, parcel, esbuild
- Scripts are loaded directly (CDN or served statically)
- If a build is needed for Tailwind CSS, it is managed separately

### ECMAScript version
- Target: ES2020+ (modern browsers)
- No transpilation, no polyfills
- Use `const`/`let` (never `var`)
- Use arrow functions, template literals, destructuring, optional chaining

---
---
## 2. HTMX

### Conventions

```html
<!-- HTMX attributes are declared in the following order: -->
<button
    hx-delete="/api/v1/domains/example.com"
    hx-target="#domain-row-example"
    hx-swap="outerHTML"
    hx-confirm="Delete domain example.com ?"
    hx-indicator="#spinner"
    hx-headers='{"X-CSRF-Token": "..."}'
>
    Delete
</button>
```

Attribute order:
1. Method + URL (`hx-get`, `hx-post`, `hx-delete`, etc.)
2. Target (`hx-target`)
3. Swap strategy (`hx-swap`)
4. Confirmation (`hx-confirm`)
5. Indicator (`hx-indicator`)
6. Headers (`hx-headers`)

### Recommended patterns

#### Pagination
```html
<div id="domain-list">
    <nav hx-boost="true">
        <a href="/domains?page=2" hx-target="#domain-list">Next page</a>
    </nav>
</div>
```

#### Live search
```html
<input
    type="search"
    name="search"
    hx-get="/domains"
    hx-target="#domain-list"
    hx-trigger="keyup changed delay:300ms"
    hx-indicator="#search-spinner"
    placeholder="Search..."
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

#### Form with inline errors
```html
<form
    hx-post="/domains"
    hx-target="#form-errors"
    hx-swap="innerHTML"
    hx-target-error="#form-errors"
>
    <div id="form-errors"></div>
    <button type="submit" hx-indicator="#submit-spinner">
        Create
        <span id="submit-spinner" class="htmx-indicator">...</span>
    </button>
</form>
```

### HTMX rules

- Always explicitly define `hx-target`
- Use `hx-push-url="true"` for navigations that should modify the URL
- The server returns HTML fragments (not complete pages) for HTMX requests
- Detect HTMX requests on the server side via the header `HX-Request: true`
- Use `hx-swap="outerHTML"` to replace the element, `innerHTML` for content

### CSRF with HTMX

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

### When to use Alpine.js

| Use case                | HTMX | Alpine.js |
|-------------------------|------|-----------|
| Loading server data     | Yes  | No        |
| Dropdown/menu           | No   | Yes       |
| Confirmation modal      | No   | Yes       |
| Local visibility toggle | No   | Yes       |
| Client-side validation  | No   | Yes       |
| Local tabs              | No   | Yes       |
| Form submission         | Yes  | No        |
| List update             | Yes  | No        |

### Alpine.js Rules

- Keep `x-data` simple (no complex logic)
- If a component exceeds 10 lines of JS, extract it into a separate file
- Use `x-ref` for DOM references instead of selectors
- Prefer `x-show` over `x-if` unless the DOM should not exist at all

### Reusable Components

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
                <button @click="cancel()">Cancel</button>
                <button @click="confirm()">Confirm</button>
            </div>
        </div>
    </div>
</template>
```

---
---
## 4. Custom scripts

### Module structure

```javascript
// static/js/modules/password-strength.js

/**
 * Evaluates password strength and updates the visual indicator.
 */

const STRENGTH_LEVELS = {
    WEAK: { label: 'Weak', class: 'bg-red-500', min: 0 },
    FAIR: { label: 'Fair', class: 'bg-amber-500', min: 40 },
    GOOD: { label: 'Good', class: 'bg-green-400', min: 70 },
    STRONG: { label: 'Strong', class: 'bg-green-600', min: 90 },
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

### Code conventions

```javascript
// Use const by default, let only if reassignment is needed
const config = { timeout: 5000 };
let count = 0;

// Arrow functions for callbacks
items.filter(item => item.active);
items.map(item => ({ ...item, label: item.name.toUpperCase() }));

// Destructuring
const { domain, active } = response.data;

// Optional chaining and nullish coalescing
const name = user?.profile?.name ?? 'Anonymous';

// Template literals for string construction
const url = `/api/v1/domains/${encodeURIComponent(domain)}`;

// Async/await instead of .then()
async function fetchDomain(name) {
    const response = await fetch(`/api/v1/domains/${name}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}
```

### Forbidden practices

- **`var`** : Use `const` or `let` exclusively
- **`==`** : Always use `===` (exception: `x == null` for null/undefined)
- **Dynamic string execution as code** : No dynamic code interpretation APIs should be used (risk of code injection)
- **Unsanitized HTML injection** : Use `textContent` or DOM methods to insert content. Never inject raw HTML from user data
- **Direct document writing** : Use DOM methods (`createElement`, `appendChild`) to manipulate the DOM
- **`console.log` in production** : Remove before each commit
- **`setTimeout`/`setInterval` with strings** : Use callback functions

---

---
---
## 5. File Organization

```
static/
├── js/
│   ├── htmx.min.js           # HTMX (vendor, versioned)
│   ├── alpine.min.js          # Alpine.js (vendor, versioned)
│   ├── app.js                 # Global initialization (CSRF, HTMX config)
│   └── modules/               # Specific modules
│       ├── password-strength.js
│       ├── qr-code.js         # TOTP QR code display
│       └── clipboard.js       # Copy to clipboard
├── css/
│   ├── output.css             # Tailwind compiled
│   └── custom.css             # Custom styles (minimal)
└── images/
    ├── logo.svg
    └── favicon.ico
```

### Script Loading

```html
<script src="/static/js/htmx.min.js" defer></script>
<script src="/static/js/alpine.min.js" defer></script>
<script src="/static/js/app.js" defer></script>
```

Specific modules are loaded only on pages that need them via a template block `{% block scripts %}{% endblock %}`.

---
---
## 6. Security

### XSS
- Never inject raw HTML with unsanitized data
- Askama escapes by default on the server side
- Alpine.js `x-text` escapes by default (never use `x-html` with user data)
- HTMX attributes should not contain unencoded user data

### CSRF
- All POST/PUT/DELETE HTMX requests include the CSRF token (see HTMX section)
- The token is renewed on each session

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

### Third-party dependencies
- Only HTMX and Alpine.js are allowed as JS dependencies
- Vendor files are versioned and served locally (no CDN in production)
- Verify the integrity of vendor files (SRI hash)

---
---
## 7. Accessibility

### ARIA with HTMX
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

### ARIA with Alpine.js
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

### Accessibility rules

- All interactive elements must be keyboard accessible
- Error messages should be associated with inputs (`aria-describedby`)
- Dynamic regions (HTMX updates) use `aria-live`
- Focus is managed correctly after DOM updates

---

---
---
## 8. Performance

- Load scripts with `defer`
- HTMX and Alpine are lightweight (~14 Ko and ~15 Ko gzipped)
- HTMX responses are minimal HTML fragments
- Use browser cache for static assets (hash in filename)
- Debounce searches, throttle repeated actions

---
---
## 9. Tests

### E2E tests (recommended)

To validate the HTMX + server integration, use E2E tests with Playwright:

```javascript
// tests/e2e/domains.spec.js
test('can create a domain via the form', async ({ page }) => {
    await page.goto('/domains/new');
    await page.fill('input[name="domain"]', 'test-e2e.com');
    await page.fill('input[name="description"]', 'Test E2E');
    await page.click('button[type="submit"]');
    await page.waitForSelector('text=test-e2e.com');
});

test('search filters domains in real time', async ({ page }) => {
    await page.goto('/domains');
    await page.fill('input[name="search"]', 'example');
    await page.waitForTimeout(400);
    const rows = await page.locator('table tbody tr').count();
    expect(rows).toBeGreaterThan(0);
});
```

---
