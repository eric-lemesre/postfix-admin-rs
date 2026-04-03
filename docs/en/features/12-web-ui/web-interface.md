> **Language:** English | [Francais](../fr/features/12-web-ui/web-interface.md)

---
# SPEC-12.1 — Web Interface

## Implementation Status

| Component                    | Crate               | Status  | Milestone |
|------------------------------|---------------------|---------|-----------|
| Askama template base layout  | `postfix-admin-web` | Done    | M6        |
| Tailwind CSS build pipeline  | `postfix-admin-web` | Pending | M6        |
| HTMX + Alpine.js integration | `postfix-admin-web` | Partial | M6        |
| i18n system (EN + FR)        | `postfix-admin-web` | Pending | M6        |
| All CRUD pages               | `postfix-admin-web` | Partial | M6        |
| Security headers middleware  | `postfix-admin-web` | Pending | M6        |

## Summary

Modern web administration interface using server-side rendering (SSR) with Askama,
enhanced by HTMX for interactivity and styled with Tailwind CSS.
"HTML over the wire" approach: no heavy JavaScript framework.

## Frontend stack

| Technology       | Role                                      | Version |
|------------------|-------------------------------------------|---------|
| **Askama**       | Rust compiled templates                   | Latest  |
| **HTMX**         | Interactivity (declarative AJAX requests) | 2.x     |
| **Tailwind CSS** | Utility design system                     | 3.x     |
| **Alpine.js**    | Micro-interactivity (dropdowns, modals)   | 3.x     |
| **Heroicons**    | SVG icons                                 | 2.x     |

## Layout and navigation

### Page structure
```
┌─────────────────────────────────────────────────┐
│  Header : Logo + Main Nav + User menu           │
├──────────┬──────────────────────────────────────┤
│          │                                      │
│ Sidebar  │        Main content                  │
│ (context)│  ┌────────────────────────────────┐  │
│          │  │  Breadcrumb                    │  │
│          │  ├────────────────────────────────┤  │
│          │  │  Title + Actions               │  │
│          │  ├────────────────────────────────┤  │
│          │  │                                │  │
│          │  │  Content (table, form, etc.)   │  │
│          │  │                                │  │
│          │  └────────────────────────────────┘  │
│          │                                      │
├──────────┴──────────────────────────────────────┤
│  Footer : Version + Links                       │
└─────────────────────────────────────────────────┘
```

### Main navigation (Superadmin)
- Dashboard (overview)
- Domains
- Admins
- Domain aliases
- Logs
- Configuration
- DKIM

### Contextual navigation (after domain selection)
- Mailboxes
- Aliases
- Fetchmail
- Domain DKIM

### User navigation
- My password
- My vacation
- My app passwords
- My 2FA

## Main pages

### Dashboard
- Global statistics (number of domains, mailboxes, aliases)
- Quota usage charts (stacked bars)
- Recent actions (5 latest logs)
- Alerts (near-limit quotas, expired passwords)

### Generic list (domains, mailboxes, aliases, admins)
- Paginated table with column sorting (HTMX)
- Search bar with real-time filtering (HTMX debounce)
- Quick filters (active/inactive, domain)
- Bulk actions (enable/disable, delete)
- Inline active/inactive toggle (HTMX PATCH)

### Generic form (creation/modification)
- Server-side validation with inline error messages
- Light client-side validation (HTML5 required, pattern)
- Submit button with loading indicator (HTMX)
- Confirmation for destructive actions (Alpine.js modal)

## HTMX features

| Feature       | HTMX attribute                                     | Description                        |
|---------------|----------------------------------------------------|------------------------------------|
| Pagination    | `hx-get`, `hx-target`                              | Loads next page without reload     |
| Live search   | `hx-get`, `hx-trigger="keyup changed delay:300ms"` | Real-time list filtering           |
| Active toggle | `hx-patch`, `hx-swap="outerHTML"`                  | Toggles status inline              |
| Deletion      | `hx-delete`, `hx-confirm`                          | Deletes with confirmation          |
| Forms         | `hx-post`, `hx-target="#errors"`                   | AJAX submission with inline errors |
| Navigation    | `hx-get`, `hx-push-url`                            | SPA-like navigation                |

## Design system

### Colors

| Usage      | Tailwind color                        |
|------------|---------------------------------------|
| Primary    | `indigo-600`                          |
| Success    | `green-600`                           |
| Warning    | `amber-500`                           |
| Error      | `red-600`                             |
| Background | `gray-50` (light) / `gray-900` (dark) |

### Dark mode
- Native support via `class="dark"` on `<html>`
- Toggle stored in `localStorage`
- Respects `prefers-color-scheme` by default

### Responsive
- Mobile-first with standard Tailwind breakpoints
- Collapsible sidebar on mobile (hamburger menu)
- Horizontally scrollable tables on small screens

## Accessibility

- HTML5 semantics (`<nav>`, `<main>`, `<aside>`, `<table>`)
- Input labels associated via `for` attribute
- Visible focus on interactive elements
- ARIA attributes for dynamic components
- WCAG 2.1 AA compliant color contrast
- Full keyboard navigation

## Internationalization (i18n)

- Translation system based on language-specific TOML files
- Language detection from: `Accept-Language` header, cookie, configuration
- Initial languages: French, English
- Translation keys used in Askama templates

## Frontend security

- CSRF token on all POST forms
- Strict Content Security Policy (CSP)
- X-Frame-Options: DENY
- X-Content-Type-Options: nosniff
- Referrer-Policy: same-origin
- All displayed data sanitized (Askama escapes by default)

---
