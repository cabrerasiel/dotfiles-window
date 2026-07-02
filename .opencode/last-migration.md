# eCare360 Migration Session

## Goal
Migrate legacy eCare ASPX application to Angular (eCare360) by building new feature components.

## Branches
- `migrate/member-medication` (current, uncommitted changes)
- `migrate/member-diagnosis` (committed: c47308f)
- `~/dotfiles/` (standalone, outside repo)

---

## Changes Today (2026-07-02)

### Member Diagnosis (`migrate/member-diagnosis`) [committed]
**Commit:** `c47308f Member Diagnosis: list, dialog, mock service`

**Files:**
- `src/app/core/services/member-diagnosis.service.ts` — mock service (5 fake records, no HTTP)
- `src/app/core/services/member-diagnosis.service.spec.ts` — tests
- `src/app/shared/models/member-diagnosis.ts` — model with `diagnosisId`
- `src/app/members/member-diagnosis/member-diagnosis-list/` — list component (table: Code, Description, Date Recorded, Start/End Date, Primary, Treatment, Actions)
- `src/app/members/member-diagnosis/member-diagnosis-dialog/` — dialog (date picker, diagnosis code autocomplete via real `DiagnosesService`, description, treatment, primary checkbox, start/end dates, notes)
- `src/app/layouts/site-routes.ts` — added route `member/:memberId/diagnosis`

**Post-commit fix:** `sortField` typo fixed `datedRecorded` → `dateRecorded`

### Medication Catalog (`migrate/member-medication`) [uncommitted]
**Made but not committed yet:**

**Files:**
- `src/app/members/member-medication/member-medication-list/` — list component (table: Medication Name, MedSpan ID; search with Enter/lupa; server-side pagination)
- `src/app/layouts/site-routes.ts` — route `member/:memberId/medication` added

**Fixes applied:**
1. `isLoading` starts `true` (avoid flash)
2. `[pageSizeOptions]="[5,10,25,50]"`
3. `[showFirstLastButtons]="true"`
4. `.table-scroll-container` with `max-height` + `overflow-y: auto`
5. `resetScroll()` on data load/search/page change
6. `totalCount` fallback to `paginated.result.length` when X-Pagination header missing

### Dotfiles (`~/dotfiles/`)
Contiene configuraciones personales completas con bootstrap.

**Nuevos componentes agregados:**
- `git/config` + `git/gitignore_global` + `git/setup.ps1` — Git config exportable + gitignore global para .NET, Node, Angular, VS
- `vscode/extensions.txt` — 30 extensiones de VS Code (Angular, C#, Copilot, SonarLint, Prettier, Catppuccin, etc.)
- `winget-packages.txt` — lista de paquetes winget instalados (para import manual)
- `scripts/monitor-glazewm-display.ps1`, `scripts/switch-glazewm-config.ps1` — helpers movidos a `scripts/`
- `scripts/setup-wsl.ps1` — crea symlinks desde dotfiles hacia WSL ~/.config/
- Fonts agregadas a `choco-packages.txt`: FiraCode, Nerd Fonts FiraCode + JetBrainsMono

**Bootstrap actualizado con:**
- Restaura git config + global gitignore
- Instala VS Code extensions automáticamente
- Auto `npm install` en Zebar si faltan node_modules
- Copia helpers a `~/.glzr/scripts/`

---

## Identified Issues
- **MemberDiagnoses/ API** → 404 (blocked, needs backend controller)
- **Medications/ API** — pagination works but `X-Pagination` header likely not exposed via CORS; multi-page total count shows `0` without `result.length` fallback
- **Pre-existing console error**: `ExpressionChangedAfterItHasBeenCheckedError` in `ToolbarMemberInfoComponent` (not related to our work)

## Progress Summary

### Done
- Member Diagnosis: list + dialog + mock service (API 404)
- Medication Catalog: list with search + pagination (API exists, missing X-Pagination)
- Dotfiles: bootstrap install script + configs

### Blocked
- `MemberDiagnoses/` endpoint needed in ILS.API
- `X-Pagination` CORS exposure needed in ILS.API

### Next Steps
Fase 1 independent components: Member Notes, Allergies, Vital Signs, Vaccinations, Insurance, Documents, User Management, etc.

## Key Context
- API base: `https://localhost:44316/api/`
- OIDC auth handled by Angular interceptor
- `migrate/` branch prefix — can rename to `ILS006-{nro}-{desc}` later
