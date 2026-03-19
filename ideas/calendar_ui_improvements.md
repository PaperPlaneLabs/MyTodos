# Calendar UI Improvements

Improvements are grouped by component and ordered by visual impact. Check off each one as it is done.

---

## 🗓️ CalendarDayCell.svelte

- [x] **Hover-only `+` button** — The add-task `+` button is always in the DOM. Set `opacity: 0` by default and `opacity: 1` on cell hover so it only appears when needed.
- [ ] **Drag-over drop zone** — No visual feedback when dragging a task over a cell. Add a highlighted drop zone style (accent border + faint accent background) using an `ondragenter`/`ondragleave` state toggle.
- [ ] **Colored project dots in month view** — The count badge (`●  3`) is generic. Replace it with stacked colored mini-dots per project (up to 3 unique project colors), like Google Calendar's event pills, so users can see which projects have tasks at a glance.
- [ ] **Clean selected-cell border** — The `selected` state uses `border-width: 2px` which shifts layout. Replace with `box-shadow: inset 0 0 0 2px var(--accent)` for a layout-stable highlight.
- [ ] **Weekend column tint** — Sunday and Saturday cells have identical background to weekdays. Apply a very subtle tint on `other-month` + weekend to form a visual rhythm.
- [ ] **Today cell glow** — The today number has an accent circle. Add a soft `box-shadow` glow on the whole `.day-cell.today` to make it pop further.

---

## 📅 CalendarWeek.svelte

- [ ] **"Nothing scheduled" empty state** — Empty day rows show a blank 20px placeholder. Replace with a faint italic "No tasks" label in `var(--text-tertiary)`.
- [ ] **Today row tint** — Only the date number is highlighted for today. Add a very faint accent tint to the whole row background for the current day.
- [ ] **Timed vs all-day section separator** — All items are equally spaced. Add a subtle divider or slightly larger gap between timed items and the all-day section.
- [ ] **Task chip background** — Items use only a 4px left color bar. Complement it with a low-opacity `color-mix` background using the same project color, so the row identity is clearer without being garish.

---

## 🔠 CalendarHeader.svelte

- [ ] **Circular nav arrow hover** — The prev/next arrow buttons have a square-ish hover box. Change to `border-radius: 50%` for a circular ripple effect like Google Calendar.
- [ ] **"Not on current month" indicator** — When the user has navigated away from the current month, show a subtle dot or "↩ Today" hint below the month label so they know they've drifted.
- [ ] **View toggle slide animation** — The active segment jumps instantly. Use a CSS `::before` pseudo-element sliding indicator for a smooth transition.

---

## ✨ Global / Cross-cutting

- [ ] **Month↔Week slide transition** — Switching views has no animation. Wrap `CalendarMonth` and `CalendarWeek` in a `crossfade` or directional `fly` transition.
- [ ] **Skeleton loading state** — The loading state is just a spinner. Replace with a 7-column ghost grid with a shimmer animation so the calendar shape is visible immediately while data loads.
- [ ] **Custom thin scrollbar** — `.calendar-content` uses the browser default scrollbar. Add a slim custom scrollbar using `::-webkit-scrollbar` to match the rest of the app.
