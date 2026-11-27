---
title: Roadmap
description: Status of all 44 Monoxus headless primitives implemented in v0.1.2 across Batch 1, Batch 2, and Batch 3 waves.
---

All **44 Monoxus headless primitives** are implemented in `monoxus v0.1.2` across Batch 1 (Baseline + Track 2 Batches 1A–5A), Batch 2, and Batch 3. Five MVP component documentation pages (`Button`, `Dialog`, `Select`, `Tabs`, and `Tooltip`) are currently live on this site, with the remaining 39 component docs in active expansion.

## Summary

| Wave | Count | Status | Scope |
|---|---:|---|---|
| **Batch 1 (Baseline + Track 2 Batches 1A–5A)** | 30 | Implemented | Core controls, overlays, forms, display, slider, scroll area, hover card, toolbar, pagination, pin input, a11y utilities |
| **Batch 2 (Tier 3B–3C & Tier 4B)** | 6 | Implemented | Menus (`dropdown_menu`, `context_menu`, `menubar`), `navigation_menu`, `command`, `combobox` |
| **Batch 3 (Tier 5B–5C)** | 8 | Implemented | `calendar`, `range_calendar`, segmented date/time fields, `date_picker`, `date_range_picker` |
| **Total** | **44** | **Implemented** | 5 MVP component docs live; 39 remaining docs in expansion |

## Batch 1 — Core, Overlays, Forms & Layout (30)

| Primitive | Module | Category | Status |
|---|---|---|---|
| `Accordion` | `monoxus::accordion` | Disclosure | Implemented |
| `AlertDialog` | `monoxus::alert_dialog` | Overlay | Implemented |
| `AspectRatio` | `monoxus::aspect_ratio` | Layout | Implemented |
| `Avatar` | `monoxus::avatar` | Media | Implemented |
| `Button` | `monoxus::button` | Action | Implemented |
| `Checkbox` | `monoxus::checkbox` | Form | Implemented |
| `Collapsible` | `monoxus::collapsible` | Disclosure | Implemented |
| `Dialog` | `monoxus::dialog` | Overlay | Implemented |
| `HoverCard` | `monoxus::hover_card` | Overlay | Implemented |
| `Label` | `monoxus::label` | Form | Implemented |
| `Meter` | `monoxus::meter` | Display | Implemented |
| `Pagination` | `monoxus::pagination` | Navigation | Implemented |
| `PinInput` | `monoxus::pin_input` | Form | Implemented |
| `Popover` | `monoxus::popover` | Overlay | Implemented |
| `Progress` | `monoxus::progress` | Display | Implemented |
| `RadioGroup` | `monoxus::radio_group` | Form | Implemented |
| `RatingGroup` | `monoxus::rating_group` | Form | Implemented |
| `ScrollArea` | `monoxus::scroll_area` | Layout | Implemented |
| `Select` | `monoxus::select` | Form / Overlay | Implemented |
| `Separator` | `monoxus::separator` | Layout | Implemented |
| `Slider` | `monoxus::slider` | Form | Implemented |
| `Switch` | `monoxus::switch` | Form | Implemented |
| `Tabs` | `monoxus::tabs` | Navigation | Implemented |
| `Toast` | `monoxus::toast` | Feedback | Implemented |
| `Toggle` | `monoxus::toggle` | Action | Implemented |
| `ToggleGroup` | `monoxus::toggle_group` | Action | Implemented |
| `Toolbar` | `monoxus::toolbar` | Navigation | Implemented |
| `Tooltip` | `monoxus::tooltip` | Overlay | Implemented |
| `VisuallyHidden` | `monoxus::visually_hidden` | Accessibility | Implemented |
| `AccessibleIcon` | `monoxus::accessible_icon` | Accessibility | Implemented |

## Batch 2 — Menus, Navigation & Filtering (6)

| Primitive | Module | Tier | Status |
|---|---|---|---|
| `DropdownMenu` | `monoxus::dropdown_menu` | Tier 3B | Implemented |
| `ContextMenu` | `monoxus::context_menu` | Tier 3B | Implemented |
| `Menubar` | `monoxus::menubar` | Tier 3B | Implemented |
| `NavigationMenu` | `monoxus::navigation_menu` | Tier 3C | Implemented |
| `Command` | `monoxus::command` | Tier 4B | Implemented |
| `Combobox` | `monoxus::combobox` | Tier 4B | Implemented |

## Batch 3 — Calendar & Date/Time Suite (8)

| Primitive | Module | Tier | Status |
|---|---|---|---|
| `Calendar` | `monoxus::calendar` | Tier 5B | Implemented |
| `RangeCalendar` | `monoxus::range_calendar` | Tier 5B | Implemented |
| `DateField` | `monoxus::date_field` | Tier 5C | Implemented |
| `TimeField` | `monoxus::time_field` | Tier 5C | Implemented |
| `DateRangeField` | `monoxus::date_range_field` | Tier 5C | Implemented |
| `TimeRangeField` | `monoxus::time_range_field` | Tier 5C | Implemented |
| `DatePicker` | `monoxus::date_picker` | Tier 5C | Implemented |
| `DateRangePicker` | `monoxus::date_range_picker` | Tier 5C | Implemented |
