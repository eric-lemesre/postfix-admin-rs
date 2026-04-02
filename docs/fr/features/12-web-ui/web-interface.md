> **Language:** [English](../en/features/12-web-ui/web-interface.md) | Francais

# SPEC-12.1 — Interface Web

## Résumé

Interface d'administration web moderne utilisant le rendu côté serveur (SSR) avec
Askama, enrichi par HTMX pour l'interactivité et stylé avec Tailwind CSS.
Approche "HTML over the wire" : pas de framework JavaScript lourd.

## Stack frontend

| Technologie | Rôle | Version |
|-------------|------|---------|
| **Askama** | Templates compilés Rust | Latest |
| **HTMX** | Interactivité (requêtes AJAX déclaratives) | 2.x |
| **Tailwind CSS** | Système de design utilitaire | 3.x |
| **Alpine.js** | Micro-interactivité (dropdowns, modales) | 3.x |
| **Heroicons** | Icônes SVG | 2.x |

## Layout et navigation

### Structure de page
```
┌─────────────────────────────────────────────────┐
│  Header : Logo + Nav principale + User menu     │
├──────────┬──────────────────────────────────────┤
│          │                                      │
│ Sidebar  │        Contenu principal             │
│ (nav     │                                      │
│  context)│  ┌────────────────────────────────┐  │
│          │  │  Breadcrumb                    │  │
│          │  ├────────────────────────────────┤  │
│          │  │  Titre + Actions               │  │
│          │  ├────────────────────────────────┤  │
│          │  │                                │  │
│          │  │  Contenu (table, form, etc.)   │  │
│          │  │                                │  │
│          │  └────────────────────────────────┘  │
│          │                                      │
├──────────┴──────────────────────────────────────┤
│  Footer : Version + Liens                       │
└─────────────────────────────────────────────────┘
```

### Navigation principale (Superadmin)
- Dashboard (vue d'ensemble)
- Domaines
- Admins
- Alias de domaines
- Logs
- Configuration
- DKIM

### Navigation contextuelle (après sélection d'un domaine)
- Boîtes mail
- Alias
- Fetchmail
- DKIM du domaine

### Navigation utilisateur
- Mon mot de passe
- Mon vacation
- Mes app passwords
- Mon 2FA

## Pages principales

### Dashboard
- Statistiques globales (nb domaines, boîtes, alias)
- Graphiques d'utilisation quotas (barres empilées)
- Dernières actions (5 derniers logs)
- Alertes (quotas proches de la limite, mots de passe expirés)

### Liste générique (domaines, boîtes, alias, admins)
- Tableau paginé avec tri par colonnes (HTMX)
- Barre de recherche avec filtrage en temps réel (HTMX debounce)
- Filtres rapides (actif/inactif, domaine)
- Actions en masse (activer/désactiver, supprimer)
- Toggle inline actif/inactif (HTMX PATCH)

### Formulaire générique (création/modification)
- Validation côté serveur avec messages d'erreur inline
- Validation côté client légère (HTML5 required, pattern)
- Bouton de soumission avec indicateur de chargement (HTMX)
- Confirmation pour les actions destructives (modale Alpine.js)

## Fonctionnalités HTMX

| Fonctionnalité | Attribut HTMX | Description |
|---------------|---------------|-------------|
| Pagination | `hx-get`, `hx-target` | Charge la page suivante sans reload |
| Recherche live | `hx-get`, `hx-trigger="keyup changed delay:300ms"` | Filtre la liste en temps réel |
| Toggle actif | `hx-patch`, `hx-swap="outerHTML"` | Bascule le statut inline |
| Suppression | `hx-delete`, `hx-confirm` | Supprime avec confirmation |
| Formulaires | `hx-post`, `hx-target="#errors"` | Soumission AJAX avec erreurs inline |
| Navigation | `hx-get`, `hx-push-url` | Navigation SPA-like |

## Design system

### Couleurs

| Usage | Couleur Tailwind |
|-------|-----------------|
| Primaire | `indigo-600` |
| Succès | `green-600` |
| Avertissement | `amber-500` |
| Erreur | `red-600` |
| Fond | `gray-50` (clair) / `gray-900` (sombre) |

### Mode sombre
- Support natif via `class="dark"` sur `<html>`
- Toggle stocké en `localStorage`
- Respecte `prefers-color-scheme` par défaut

### Responsive
- Mobile-first avec breakpoints Tailwind standards
- Sidebar repliable sur mobile (hamburger menu)
- Tableaux scrollables horizontalement sur petit écran

## Accessibilité

- Sémantique HTML5 (`<nav>`, `<main>`, `<aside>`, `<table>`)
- Labels associés aux inputs (`<label for="...">`)
- Focus visible sur les éléments interactifs
- Attributs ARIA pour les composants dynamiques
- Contraste de couleurs conforme WCAG 2.1 AA
- Navigation au clavier complète

## Internationalisation (i18n)

- Système de traduction basé sur des fichiers TOML par langue
- Langue détectée depuis : header `Accept-Language`, cookie, configuration
- Langues initiales : français, anglais
- Les clés de traduction sont utilisées dans les templates Askama

## Sécurité frontend

- CSRF token sur tous les formulaires POST
- Content Security Policy (CSP) strict
- X-Frame-Options: DENY
- X-Content-Type-Options: nosniff
- Referrer-Policy: same-origin
- Sanitisation de toutes les données affichées (Askama échappe par défaut)
