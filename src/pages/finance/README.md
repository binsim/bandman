# Finance page

**Status:** Deferred (stub route only — build much later)

## Purpose

Generate **invoice PDFs** and track **income / expenses** for the band.

## Planned behaviour (later)

- Ledger of income and expenses (amount, category, date, note)
- Create invoice PDFs (band details, line items, totals)
- Simple filters and totals overview
- Admin-only or trusted members

## Suggested data (when ready)

- `finance_entries` — kind (income/expense), amount, currency, category, occurred_on, created_by
- `invoices` / `invoice_lines` — for PDF generation

## Notes

- Schema intentionally **not** created in the foundation migrations
- Keep this route as a placeholder until core band features ship
