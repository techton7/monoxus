use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::score::compute_command_score;
use super::types::{
    CommandFilteredState, CommandGroupEntry, CommandItemEntry, CommandNavKey, CommandNavModifiers,
    CommandStateSnapshot,
};

#[inline]
pub fn should_render_item(
    item_force_mount: bool,
    group_force_mount: bool,
    should_filter: bool,
    search: &str,
    score: Option<f64>,
) -> bool {
    if item_force_mount || group_force_mount || !should_filter || search.is_empty() {
        return true;
    }
    score.is_some_and(|s| s > 0.0)
}

#[inline]
pub fn should_render_group(
    group_force_mount: bool,
    should_filter: bool,
    search: &str,
    is_visible_group: bool,
) -> bool {
    if group_force_mount || !should_filter || search.is_empty() {
        return true;
    }
    is_visible_group
}

#[inline]
pub fn should_render_empty(filtered_count: usize, settled: bool, force_mount: bool) -> bool {
    force_mount || (filtered_count == 0 && settled)
}

#[inline]
pub fn should_render_separator(search: &str, force_mount: bool) -> bool {
    force_mount || search.is_empty()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CommandRegistry {
    items: Vec<CommandItemEntry>,
    groups: Vec<CommandGroupEntry>,
    filtered: CommandFilteredState,
    next_order: usize,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items(&self) -> &[CommandItemEntry] {
        &self.items
    }

    pub fn groups(&self) -> &[CommandGroupEntry] {
        &self.groups
    }

    pub fn filtered(&self) -> &CommandFilteredState {
        &self.filtered
    }

    /// Registers or updates an item by its normalized trimmed `value` key.
    /// Returns `true` if a new entry was inserted, `false` if an existing entry was updated.
    pub fn register_item(
        &mut self,
        value: String,
        dom_id: String,
        group_value: Option<String>,
        disabled: bool,
        force_mount: bool,
        keywords: Vec<String>,
    ) -> bool {
        let normalized = value.trim().to_string();
        if normalized.is_empty() {
            return false;
        }
        let cleaned_keywords: Vec<String> = keywords
            .into_iter()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();

        if let Some(existing) = self.items.iter_mut().find(|item| item.value == normalized) {
            existing.dom_id = dom_id;
            existing.group_value = group_value;
            existing.disabled = disabled;
            existing.force_mount = force_mount;
            existing.keywords = cleaned_keywords;
            return false;
        }

        let order = self.next_order;
        self.next_order += 1;
        self.items.push(CommandItemEntry {
            value: normalized,
            dom_id,
            group_value,
            disabled,
            force_mount,
            keywords: cleaned_keywords,
            order,
        });
        true
    }

    pub fn unregister_item(&mut self, value: &str) -> bool {
        let normalized = value.trim();
        let before = self.items.len();
        self.items.retain(|item| item.value != normalized);
        if self.items.len() != before {
            self.filtered.item_scores.remove(normalized);
            true
        } else {
            false
        }
    }

    /// Registers or updates a group by its `value` key.
    pub fn register_group(
        &mut self,
        value: String,
        dom_id: String,
        heading_id: Option<String>,
        force_mount: bool,
    ) -> bool {
        let normalized = value.trim().to_string();
        if normalized.is_empty() {
            return false;
        }
        if let Some(existing) = self.groups.iter_mut().find(|g| g.value == normalized) {
            existing.dom_id = dom_id;
            if heading_id.is_some() {
                existing.heading_id = heading_id;
            }
            existing.force_mount = force_mount;
            return false;
        }

        let order = self.next_order;
        self.next_order += 1;
        self.groups.push(CommandGroupEntry {
            value: normalized,
            dom_id,
            heading_id,
            force_mount,
            order,
        });
        true
    }

    pub fn set_group_heading(&mut self, group_value: &str, heading_id: Option<String>) {
        let normalized = group_value.trim();
        if let Some(group) = self.groups.iter_mut().find(|g| g.value == normalized) {
            group.heading_id = heading_id;
        }
    }

    pub fn unregister_group(&mut self, value: &str) -> bool {
        let normalized = value.trim();
        let before = self.groups.len();
        self.groups.retain(|g| g.value != normalized);
        if self.groups.len() != before {
            self.filtered.visible_groups.remove(normalized);
            true
        } else {
            false
        }
    }

    pub fn group_force_mount(&self, group_value: Option<&str>) -> bool {
        let Some(gv) = group_value else {
            return false;
        };
        self.groups
            .iter()
            .find(|g| g.value == gv)
            .is_some_and(|g| g.force_mount)
    }

    pub fn item_score(&self, value: &str) -> Option<f64> {
        self.filtered.item_scores.get(value.trim()).copied()
    }

    pub fn is_group_visible(&self, group_value: &str) -> bool {
        self.filtered.visible_groups.contains(group_value.trim())
    }

    pub fn item_dom_id(&self, value: &str) -> Option<&str> {
        let normalized = value.trim();
        self.items
            .iter()
            .find(|item| item.value == normalized)
            .map(|item| item.dom_id.as_str())
    }

    /// Recomputes `filtered` counts, item scores, and visible groups.
    pub fn recompute_filtered<F>(&mut self, search: &str, should_filter: bool, mut scorer: F)
    where
        F: FnMut(&str, &str, &[String]) -> f64,
    {
        let mut item_scores = BTreeMap::new();
        let mut visible_groups = BTreeSet::new();
        let mut count = 0usize;

        if !should_filter || search.is_empty() {
            for item in &self.items {
                item_scores.insert(item.value.clone(), 1.0);
                if !item.force_mount {
                    count += 1;
                }
            }
            for group in &self.groups {
                visible_groups.insert(group.value.clone());
            }
        } else {
            for group in &self.groups {
                if group.force_mount {
                    visible_groups.insert(group.value.clone());
                }
            }
            for item in &self.items {
                let score = scorer(&item.value, search, &item.keywords);
                let score = if score.is_nan() { 0.0 } else { score };
                item_scores.insert(item.value.clone(), score);
                let group_fm = self.group_force_mount(item.group_value.as_deref());
                if score > 0.0 {
                    if !item.force_mount {
                        count += 1;
                    }
                    if let Some(ref gv) = item.group_value {
                        visible_groups.insert(gv.clone());
                    }
                } else if item.force_mount || group_fm {
                    if let Some(ref gv) = item.group_value {
                        visible_groups.insert(gv.clone());
                    }
                }
            }
        }

        self.filtered = CommandFilteredState {
            count,
            item_scores,
            visible_groups,
        };
    }

    pub fn recompute_filtered_default(&mut self, search: &str, should_filter: bool) {
        self.recompute_filtered(search, should_filter, compute_command_score);
    }

    /// Returns visible items in deterministic ranked order without DOM reparenting (`4B D-1`).
    ///
    /// - When `!should_filter || search.is_empty()`: preserves original registration order.
    /// - When filtering: ungrouped items first (sorted by descending score, then registration order),
    ///   followed by groups sorted by descending max item score (then group registration order),
    ///   with items within each group sorted by descending score (then item registration order).
    pub fn ranked_visible_items(&self, search: &str, should_filter: bool) -> Vec<&CommandItemEntry> {
        let mut visible: Vec<&CommandItemEntry> = self
            .items
            .iter()
            .filter(|item| {
                let group_fm = self.group_force_mount(item.group_value.as_deref());
                let score = self.filtered.item_scores.get(&item.value).copied();
                should_render_item(item.force_mount, group_fm, should_filter, search, score)
            })
            .collect();

        if !should_filter || search.is_empty() {
            visible.sort_by_key(|item| item.order);
            return visible;
        }

        let score_of = |item: &CommandItemEntry| -> f64 {
            self.filtered
                .item_scores
                .get(&item.value)
                .copied()
                .unwrap_or(0.0)
        };

        let mut ungrouped: Vec<&CommandItemEntry> = Vec::new();
        let mut by_group: BTreeMap<&str, Vec<&CommandItemEntry>> = BTreeMap::new();

        for item in visible {
            match item.group_value.as_deref() {
                None => ungrouped.push(item),
                Some(gv) => by_group.entry(gv).or_default().push(item),
            }
        }

        ungrouped.sort_by(|a, b| {
            score_of(b)
                .partial_cmp(&score_of(a))
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.order.cmp(&b.order))
        });

        let mut group_buckets: Vec<(f64, usize, Vec<&CommandItemEntry>)> = by_group
            .into_iter()
            .map(|(gv, mut items)| {
                items.sort_by(|a, b| {
                    score_of(b)
                        .partial_cmp(&score_of(a))
                        .unwrap_or(Ordering::Equal)
                        .then_with(|| a.order.cmp(&b.order))
                });
                let max_score = items.first().map(|it| score_of(it)).unwrap_or(0.0);
                let group_order = self
                    .groups
                    .iter()
                    .find(|g| g.value == gv)
                    .map(|g| g.order)
                    .unwrap_or(usize::MAX);
                (max_score, group_order, items)
            })
            .collect();

        group_buckets.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.1.cmp(&b.1))
        });

        let mut ranked = ungrouped;
        for (_, _, mut items) in group_buckets {
            ranked.append(&mut items);
        }
        ranked
    }

    /// Returns all visible, non-disabled items in ranked navigation order.
    pub fn valid_items(&self, search: &str, should_filter: bool) -> Vec<&CommandItemEntry> {
        self.ranked_visible_items(search, should_filter)
            .into_iter()
            .filter(|item| !item.disabled)
            .collect()
    }

    /// If `value` is the first visible item in its group, returns that group's heading DOM ID.
    pub fn first_in_group_heading_id(
        &self,
        value: &str,
        search: &str,
        should_filter: bool,
    ) -> Option<&str> {
        let normalized = value.trim();
        let ranked = self.ranked_visible_items(search, should_filter);
        let item = ranked.iter().find(|it| it.value == normalized)?;
        let gv = item.group_value.as_deref()?;
        let first_in_group = ranked
            .iter()
            .find(|it| it.group_value.as_deref() == Some(gv))?;
        if first_in_group.value != normalized {
            return None;
        }
        self.groups
            .iter()
            .find(|g| g.value == gv)
            .and_then(|g| g.heading_id.as_deref())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandState {
    pub search: String,
    pub value: String,
    pub should_filter: bool,
    pub loop_focus: bool,
    pub vim_bindings: bool,
    pub columns: Option<usize>,
    pub registry: CommandRegistry,
}

impl Default for CommandState {
    fn default() -> Self {
        Self {
            search: String::new(),
            value: String::new(),
            should_filter: true,
            loop_focus: false,
            vim_bindings: true,
            columns: None,
            registry: CommandRegistry::new(),
        }
    }
}

impl CommandState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_should_filter(mut self, should_filter: bool) -> Self {
        self.should_filter = should_filter;
        self
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub fn with_vim_bindings(mut self, vim_bindings: bool) -> Self {
        self.vim_bindings = vim_bindings;
        self
    }

    pub fn with_columns(mut self, columns: Option<usize>) -> Self {
        self.columns = columns.filter(|&c| c > 0);
        self
    }

    pub fn snapshot(&self) -> CommandStateSnapshot {
        CommandStateSnapshot {
            search: self.search.clone(),
            value: self.value.clone(),
            filtered: self.registry.filtered().clone(),
        }
    }

    pub fn recompute(&mut self) {
        self.registry
            .recompute_filtered_default(&self.search, self.should_filter);
    }

    pub fn navigate(&mut self, key: CommandNavKey, mods: CommandNavModifiers) -> Option<String> {
        let next = resolve_command_navigation(
            &self.registry,
            &self.search,
            self.should_filter,
            &self.value,
            key,
            mods,
            self.columns,
            self.loop_focus,
            self.vim_bindings,
        )?;
        self.value = next.clone();
        Some(next)
    }
}

pub fn select_first_valid_item(
    registry: &CommandRegistry,
    search: &str,
    should_filter: bool,
) -> Option<String> {
    registry
        .valid_items(search, should_filter)
        .first()
        .map(|item| item.value.clone())
}

pub fn select_last_valid_item(
    registry: &CommandRegistry,
    search: &str,
    should_filter: bool,
) -> Option<String> {
    registry
        .valid_items(search, should_filter)
        .last()
        .map(|item| item.value.clone())
}

pub fn select_by_index(
    registry: &CommandRegistry,
    search: &str,
    should_filter: bool,
    index: usize,
) -> Option<String> {
    registry
        .valid_items(search, should_filter)
        .get(index)
        .map(|item| item.value.clone())
}

pub fn select_by_item(
    registry: &CommandRegistry,
    search: &str,
    should_filter: bool,
    current_value: &str,
    delta: isize,
    loop_focus: bool,
) -> Option<String> {
    let items = registry.valid_items(search, should_filter);
    if items.is_empty() {
        return None;
    }

    let current_idx = items.iter().position(|it| it.value == current_value);
    let len = items.len() as isize;

    let target_idx = match current_idx {
        Some(idx) => {
            let raw = idx as isize + delta;
            if raw < 0 {
                if loop_focus {
                    len - 1
                } else {
                    return None;
                }
            } else if raw >= len {
                if loop_focus {
                    0
                } else {
                    return None;
                }
            } else {
                raw
            }
        }
        None => {
            if delta >= 0 {
                0
            } else {
                len - 1
            }
        }
    };

    items
        .get(target_idx as usize)
        .map(|item| item.value.clone())
}

pub fn select_by_group(
    registry: &CommandRegistry,
    search: &str,
    should_filter: bool,
    current_value: &str,
    delta: isize,
    loop_focus: bool,
) -> Option<String> {
    let valid = registry.valid_items(search, should_filter);
    if valid.is_empty() {
        return None;
    }

    // Collect ordered distinct group keys that have at least one valid item.
    let mut ordered_groups: Vec<&str> = Vec::new();
    for item in &valid {
        if let Some(gv) = item.group_value.as_deref() {
            if !ordered_groups.contains(&gv) {
                ordered_groups.push(gv);
            }
        }
    }

    let current_group = valid
        .iter()
        .find(|it| it.value == current_value)
        .and_then(|it| it.group_value.as_deref());

    if let Some(cur_gv) = current_group {
        if let Some(g_idx) = ordered_groups.iter().position(|&g| g == cur_gv) {
            let raw = g_idx as isize + delta;
            let target_group = if raw < 0 || raw >= ordered_groups.len() as isize {
                if loop_focus && !ordered_groups.is_empty() {
                    let wrapped = raw.rem_euclid(ordered_groups.len() as isize) as usize;
                    Some(ordered_groups[wrapped])
                } else {
                    None
                }
            } else {
                Some(ordered_groups[raw as usize])
            };

            if let Some(tg) = target_group {
                if let Some(first) = valid
                    .iter()
                    .find(|it| it.group_value.as_deref() == Some(tg))
                {
                    return Some(first.value.clone());
                }
            }
        }
    }

    select_by_item(registry, search, should_filter, current_value, delta, loop_focus)
}

fn build_items_grid<'a>(
    ranked_visible: &[&'a CommandItemEntry],
    columns: usize,
) -> Vec<Vec<&'a CommandItemEntry>> {
    let cols = columns.max(1);
    let mut grid: Vec<Vec<&'a CommandItemEntry>> = Vec::new();
    let mut current_group: Option<Option<&str>> = None;

    for &item in ranked_visible {
        let item_group = item.group_value.as_deref();
        let group_changed = current_group.is_some_and(|g| g != item_group);
        let needs_new_row =
            grid.is_empty() || group_changed || grid.last().is_some_and(|row| row.len() >= cols);
        if needs_new_row {
            grid.push(Vec::with_capacity(cols));
        }
        grid.last_mut().unwrap().push(item);
        current_group = Some(item_group);
    }

    grid
}

fn find_grid_position(
    grid: &[Vec<&CommandItemEntry>],
    current_value: &str,
) -> Option<(usize, usize)> {
    for (r, row) in grid.iter().enumerate() {
        for (c, item) in row.iter().enumerate() {
            if item.value == current_value {
                return Some((r, c));
            }
        }
    }
    None
}

fn find_non_disabled_in_row<'a>(
    row: &[&'a CommandItemEntry],
    expected_col: usize,
) -> Option<&'a CommandItemEntry> {
    if let Some(&item) = row.get(expected_col) {
        if !item.disabled {
            return Some(item);
        }
        return None;
    }
    // Row is shorter than expected_col: fall back to highest non-disabled item in row.
    row.iter().rev().find(|item| !item.disabled).copied()
}

fn select_grid_vertical(
    registry: &CommandRegistry,
    search: &str,
    should_filter: bool,
    current_value: &str,
    columns: usize,
    down: bool,
    skip_row: bool,
    loop_focus: bool,
) -> Option<String> {
    let ranked = registry.ranked_visible_items(search, should_filter);
    let grid = build_items_grid(&ranked, columns);
    if grid.is_empty() {
        return None;
    }

    let Some((row_idx, col_idx)) = find_grid_position(&grid, current_value) else {
        return if down {
            select_first_valid_item(registry, search, should_filter)
        } else {
            select_last_valid_item(registry, search, should_filter)
        };
    };

    let skip = if skip_row { 1usize } else { 0usize };
    let num_rows = grid.len();

    let found = if down {
        if skip_row && num_rows >= 2 && row_idx == num_rows - 2 && !loop_focus {
            find_non_disabled_in_row(&grid[num_rows - 1], col_idx)
        } else if row_idx == num_rows - 1 {
            if !loop_focus {
                return None;
            }
            (skip..row_idx).find_map(|r| find_non_disabled_in_row(&grid[r], col_idx))
        } else {
            let primary = ((row_idx + 1 + skip)..num_rows)
                .find_map(|r| find_non_disabled_in_row(&grid[r], col_idx));
            if primary.is_some() {
                primary
            } else if loop_focus {
                (0..row_idx).find_map(|r| find_non_disabled_in_row(&grid[r], col_idx))
            } else {
                None
            }
        }
    } else if skip_row && row_idx == 1 && !loop_focus {
        find_non_disabled_in_row(&grid[0], col_idx)
    } else if row_idx == 0 {
        if !loop_focus {
            return None;
        }
        let start = num_rows.saturating_sub(1 + skip);
        ((row_idx + 1)..=start)
            .rev()
            .find_map(|r| find_non_disabled_in_row(&grid[r], col_idx))
    } else {
        let primary = if row_idx > skip {
            (0..=(row_idx - 1 - skip))
                .rev()
                .find_map(|r| find_non_disabled_in_row(&grid[r], col_idx))
        } else {
            None
        };
        if primary.is_some() {
            primary
        } else if loop_focus {
            ((row_idx + 1)..num_rows)
                .rev()
                .find_map(|r| find_non_disabled_in_row(&grid[r], col_idx))
        } else {
            None
        }
    };

    found.map(|item| item.value.clone())
}

/// Resolves keyboard navigation across 1D list and 2D `columns` grid modes,
/// honoring `vim_bindings`, `Meta`/`Alt` modifiers, `Home`/`End`, and `loop_focus`.
#[allow(clippy::too_many_arguments)]
pub fn resolve_command_navigation(
    registry: &CommandRegistry,
    search: &str,
    should_filter: bool,
    current_value: &str,
    key: CommandNavKey,
    mods: CommandNavModifiers,
    columns: Option<usize>,
    loop_focus: bool,
    vim_bindings: bool,
) -> Option<String> {
    let grid_cols = columns.filter(|&c| c > 0);
    let is_grid = grid_cols.is_some();

    let next_item = || -> Option<String> {
        if mods.meta {
            select_last_valid_item(registry, search, should_filter)
        } else if mods.alt {
            select_by_group(registry, search, should_filter, current_value, 1, loop_focus)
        } else {
            select_by_item(registry, search, should_filter, current_value, 1, loop_focus)
        }
    };

    let prev_item = || -> Option<String> {
        if mods.meta {
            select_first_valid_item(registry, search, should_filter)
        } else if mods.alt {
            select_by_group(registry, search, should_filter, current_value, -1, loop_focus)
        } else {
            select_by_item(registry, search, should_filter, current_value, -1, loop_focus)
        }
    };

    let grid_down = |cols: usize| -> Option<String> {
        if mods.meta {
            select_by_group(registry, search, should_filter, current_value, 1, loop_focus)
        } else {
            select_grid_vertical(
                registry,
                search,
                should_filter,
                current_value,
                cols,
                true,
                mods.alt,
                loop_focus,
            )
        }
    };

    let grid_up = |cols: usize| -> Option<String> {
        if mods.meta {
            select_by_group(registry, search, should_filter, current_value, -1, loop_focus)
        } else {
            select_grid_vertical(
                registry,
                search,
                should_filter,
                current_value,
                cols,
                false,
                mods.alt,
                loop_focus,
            )
        }
    };

    match key {
        CommandNavKey::ArrowDown => match grid_cols {
            Some(cols) => grid_down(cols),
            None => next_item(),
        },
        CommandNavKey::ArrowUp => match grid_cols {
            Some(cols) => grid_up(cols),
            None => prev_item(),
        },
        CommandNavKey::ArrowRight => {
            if is_grid {
                next_item()
            } else {
                None
            }
        }
        CommandNavKey::ArrowLeft => {
            if is_grid {
                prev_item()
            } else {
                None
            }
        }
        CommandNavKey::VimNext => {
            if !vim_bindings || !mods.ctrl {
                return None;
            }
            match grid_cols {
                Some(cols) => grid_down(cols),
                None => next_item(),
            }
        }
        CommandNavKey::VimPrev => {
            if !vim_bindings || !mods.ctrl {
                return None;
            }
            match grid_cols {
                Some(cols) => grid_up(cols),
                None => prev_item(),
            }
        }
        CommandNavKey::VimRight => {
            if vim_bindings && mods.ctrl && is_grid {
                next_item()
            } else {
                None
            }
        }
        CommandNavKey::VimLeft => {
            if vim_bindings && mods.ctrl && is_grid {
                prev_item()
            } else {
                None
            }
        }
        CommandNavKey::Home => select_first_valid_item(registry, search, should_filter),
        CommandNavKey::End => select_last_valid_item(registry, search, should_filter),
    }
}
