# SPEC-06.1 — Répondeur automatique (Vacation)

## Résumé

Le répondeur automatique envoie une réponse prédéfinie aux emails reçus pendant
l'absence de l'utilisateur. Intégration avec le transport Postfix `vacation` et
un système de déduplication pour ne pas répondre plusieurs fois au même expéditeur.

## Entité : `Vacation`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `email` | `VARCHAR(255)` | PK | Adresse email de la boîte |
| `subject` | `VARCHAR(255)` | NOT NULL | Sujet de la réponse automatique |
| `body` | `TEXT` | NOT NULL, default `''` | Corps du message |
| `domain` | `VARCHAR(255)` | FK → `domain.domain` | Domaine |
| `active` | `BOOLEAN` | NOT NULL, default `true` | Répondeur actif |
| `active_from` | `TIMESTAMPTZ` | NULLABLE | Date de début (optionnel) |
| `active_until` | `TIMESTAMPTZ` | NULLABLE | Date de fin (optionnel) |
| `interval_time` | `INTEGER` | NOT NULL, default `0` | Intervalle en secondes avant re-notification |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date de création |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Dernière modification |

## Entité : `VacationNotification`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `on_vacation` | `VARCHAR(255)` | PK (composite), FK → `vacation.email` ON DELETE CASCADE | Utilisateur en vacation |
| `notified` | `VARCHAR(255)` | PK (composite) | Adresse qui a été notifiée |
| `notified_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Timestamp de la notification |

## Mécanisme de transport Postfix

### Architecture
```
Mail entrant
    │
    ▼
Postfix (transport: vacation)
    │
    ▼
vacation.pl / vacation binary
    │
    ├─▶ Vérifie vacation active + dates
    ├─▶ Vérifie déduplication (vacation_notification)
    ├─▶ Envoie la réponse
    └─▶ Enregistre la notification
```

### Alias vacation
Quand le vacation est activé, un alias spécial est ajouté :
- `user@example.com` → `user@example.com, user#example.com@autoreply.example.com`

Le domaine `autoreply.example.com` est un domaine de transport vacation.

## Règles métier

### BR-VAC-01 : Activation
- L'utilisateur doit avoir une boîte mail active
- Le sujet est obligatoire
- Le corps peut être vide (mais déconseillé)
- L'activation crée/modifie l'alias pour inclure la destination vacation
- `active_from` et `active_until` permettent une programmation à l'avance

### BR-VAC-02 : Désactivation
- La désactivation supprime la destination vacation de l'alias
- Les entrées de notification sont conservées (nettoyage périodique)
- L'entrée vacation reste en base (pour réactivation rapide)

### BR-VAC-03 : Déduplication
- Un expéditeur ne reçoit la réponse automatique qu'une seule fois par intervalle
- `interval_time` configurable (par défaut : 0 = une seule fois)
- Si > 0 : re-notification après N secondes depuis la dernière notification

### BR-VAC-04 : Exclusions
- Pas de réponse aux adresses de type :
  - `MAILER-DAEMON@*`
  - `noreply@*`, `no-reply@*`
  - Adresses listées dans des headers `Precedence: bulk/list/junk`
  - Adresses `X-Loop` contenant l'adresse de l'utilisateur

### BR-VAC-05 : Périodes programmées
- Si `active_from` est défini : le vacation ne s'active qu'à cette date
- Si `active_until` est défini : le vacation se désactive automatiquement
- Un cron job vérifie périodiquement les vacations à activer/désactiver

## Cas d'utilisation

### UC-VAC-01 : Configurer le répondeur
- **Acteur** : Utilisateur, Admin du domaine
- **Entrée** : Sujet, corps, dates optionnelles, intervalle
- **Sortie** : Vacation configuré, alias modifié

### UC-VAC-02 : Activer/Désactiver le répondeur
- **Acteur** : Utilisateur, Admin du domaine
- **Entrée** : Toggle activation
- **Sortie** : Alias modifié en conséquence

## Routes Web

| Route | Méthode | Description |
|-------|---------|-------------|
| `/user/vacation` | GET | Formulaire vacation (utilisateur) |
| `/user/vacation` | POST | Enregistrement vacation |
| `/domains/{domain}/mailboxes/{username}/vacation` | GET/POST | Gestion admin |

## Endpoints API

| Méthode | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/mailboxes/{username}/vacation` | Voir le vacation |
| `PUT` | `/api/v1/mailboxes/{username}/vacation` | Configurer le vacation |
| `DELETE` | `/api/v1/mailboxes/{username}/vacation` | Désactiver |
