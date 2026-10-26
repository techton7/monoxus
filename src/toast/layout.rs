use serde::{Deserialize, Serialize};

/// Layout coordinates and transform properties published to CSS custom variables for each toast card.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToastComputedStyle {
    pub height: f64,
    pub offset: f64,
    pub scale: f64,
    pub z_index: usize,
    pub index: usize,
}

impl ToastComputedStyle {
    /// Formats the layout properties into CSS custom properties string.
    pub fn to_style_string(&self) -> String {
        let height_str = if self.height > 0.0 {
            format!("--height: {}px; ", self.height)
        } else {
            String::new()
        };
        format!(
            "{height_str}--offset: {}px; --scale: {:.3}; --z-index: {}; --index: {}; --toasts-before: {};",
            self.offset, self.scale, self.z_index, self.index, self.index
        )
    }
}

/// Pure Rust stacking geometry calculation engine for declarative toast layout.
pub struct ToastStackLayout;

impl ToastStackLayout {
    /// Computes declarative stacking geometry for an array of toast heights.
    ///
    /// In collapsed mode:
    /// - Offset uses fixed gap increment (`i * gap`).
    /// - Scale attenuates progressively (`1.0 - i * 0.05`), clamped to `0.7`.
    /// - Z-index descends from front to back (`total - i`).
    ///
    /// In expanded mode:
    /// - Offset accumulates cumulative heights plus gap spacing.
    /// - Scale is locked at `1.0`.
    /// - Z-index descends from front to back (`total - i`).
    pub fn compute(heights: &[f64], gap: f64, is_expanded: bool) -> Vec<ToastComputedStyle> {
        let total = heights.len();
        let mut results = Vec::with_capacity(total);
        let mut accumulated_h = 0.0;
        for (i, &h) in heights.iter().enumerate() {
            let (offset, scale) = if is_expanded {
                let current_offset = accumulated_h + (i as f64) * gap;
                accumulated_h += h;
                (current_offset, 1.0)
            } else {
                ((i as f64) * gap, (1.0 - (i as f64) * 0.05).max(0.7))
            };
            let z_index = total.saturating_sub(i);
            results.push(ToastComputedStyle {
                height: h,
                offset,
                scale,
                z_index,
                index: i,
            });
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_empty() {
        let collapsed = ToastStackLayout::compute(&[], 14.0, false);
        assert!(collapsed.is_empty());

        let expanded = ToastStackLayout::compute(&[], 14.0, true);
        assert!(expanded.is_empty());
    }

    #[test]
    fn test_compute_single_item() {
        let collapsed = ToastStackLayout::compute(&[60.0], 14.0, false);
        assert_eq!(collapsed.len(), 1);
        assert_eq!(
            collapsed[0],
            ToastComputedStyle {
                height: 60.0,
                offset: 0.0,
                scale: 1.0,
                z_index: 1,
                index: 0,
            }
        );

        let expanded = ToastStackLayout::compute(&[60.0], 14.0, true);
        assert_eq!(expanded.len(), 1);
        assert_eq!(
            expanded[0],
            ToastComputedStyle {
                height: 60.0,
                offset: 0.0,
                scale: 1.0,
                z_index: 1,
                index: 0,
            }
        );
    }

    #[test]
    fn test_compute_collapsed_mode() {
        let heights = vec![50.0, 60.0, 70.0, 80.0, 50.0, 50.0, 50.0, 50.0];
        let gap = 14.0;
        let results = ToastStackLayout::compute(&heights, gap, false);

        assert_eq!(results.len(), 8);

        // Front item (index 0)
        assert_eq!(results[0].offset, 0.0);
        assert_eq!(results[0].scale, 1.0);
        assert_eq!(results[0].z_index, 8);
        assert_eq!(results[0].index, 0);

        // Second item (index 1)
        assert_eq!(results[1].offset, 14.0);
        assert!((results[1].scale - 0.95).abs() < 1e-6);
        assert_eq!(results[1].z_index, 7);
        assert_eq!(results[1].index, 1);

        // Third item (index 2)
        assert_eq!(results[2].offset, 28.0);
        assert!((results[2].scale - 0.90).abs() < 1e-6);
        assert_eq!(results[2].z_index, 6);
        assert_eq!(results[2].index, 2);

        // Sixth item (index 5) -> scale = 1.0 - 0.25 = 0.75
        assert_eq!(results[5].offset, 5.0 * 14.0);
        assert!((results[5].scale - 0.75).abs() < 1e-6);
        assert_eq!(results[5].z_index, 3);

        // Seventh item (index 6) -> scale = 1.0 - 0.30 = 0.70 (clamped)
        assert_eq!(results[6].offset, 6.0 * 14.0);
        assert!((results[6].scale - 0.70).abs() < 1e-6);
        assert_eq!(results[6].z_index, 2);

        // Eighth item (index 7) -> scale = (1.0 - 0.35).max(0.7) = 0.70 (clamped)
        assert_eq!(results[7].offset, 7.0 * 14.0);
        assert!((results[7].scale - 0.70).abs() < 1e-6);
        assert_eq!(results[7].z_index, 1);

        // Verify descending z_index across all
        for i in 0..results.len() {
            assert_eq!(results[i].z_index, 8 - i);
            assert_eq!(results[i].offset, (i as f64) * gap);
            assert_eq!(results[i].height, heights[i]);
            assert!(results[i].scale >= 0.7);
        }
    }

    #[test]
    fn test_compute_expanded_mode() {
        let heights = vec![50.0, 60.0, 70.0];
        let gap = 10.0;
        let results = ToastStackLayout::compute(&heights, gap, true);

        assert_eq!(results.len(), 3);

        // Front item (index 0): offset = 0.0, scale = 1.0, z_index = 3
        assert_eq!(
            results[0],
            ToastComputedStyle {
                height: 50.0,
                offset: 0.0,
                scale: 1.0,
                z_index: 3,
                index: 0,
            }
        );

        // Second item (index 1): offset = 50.0 + 1 * 10.0 = 60.0, scale = 1.0, z_index = 2
        assert_eq!(
            results[1],
            ToastComputedStyle {
                height: 60.0,
                offset: 60.0,
                scale: 1.0,
                z_index: 2,
                index: 1,
            }
        );

        // Third item (index 2): offset = (50.0 + 60.0) + 2 * 10.0 = 130.0, scale = 1.0, z_index = 1
        assert_eq!(
            results[2],
            ToastComputedStyle {
                height: 70.0,
                offset: 130.0,
                scale: 1.0,
                z_index: 1,
                index: 2,
            }
        );

        // Verify scale locked at 1.0 for all items in expanded mode
        for item in &results {
            assert_eq!(item.scale, 1.0);
        }
    }

    #[test]
    fn test_to_style_string() {
        let cs = ToastComputedStyle {
            height: 50.0,
            offset: 60.0,
            scale: 0.95,
            z_index: 3,
            index: 1,
        };
        let style_str = cs.to_style_string();
        assert_eq!(
            style_str,
            "--height: 50px; --offset: 60px; --scale: 0.950; --z-index: 3; --index: 1; --toasts-before: 1;"
        );
    }
}
