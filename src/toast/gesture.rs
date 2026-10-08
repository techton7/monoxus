use crate::foundation::overlay::Point2D;
use crate::toast::types::{SwipeDirection, ToastConfig};

/// The primary axis locked for a swipe gesture once movement exceeds the initial deadband.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeAxis {
    X,
    Y,
}

impl SwipeAxis {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
        }
    }
}

/// Configuration governing swipe threshold crossing, dampening, and velocity triggers.
#[derive(Clone, Debug, PartialEq)]
pub struct SwipeReducerConfig {
    /// Minimum distance in pixels required to dismiss a toast when moving slowly (default 45.0 px).
    pub threshold: f64,
    /// Minimum velocity in px/ms required to dismiss on a quick flick (default 0.11 px/ms).
    pub velocity_threshold: f64,
    /// Minimum absolute displacement in pixels required even if flick velocity is high (default 10.0 px).
    pub min_velocity_delta: f64,
    /// Allowed swipe directions for dismissing toasts.
    pub allowed_directions: Vec<SwipeDirection>,
}

impl SwipeReducerConfig {
    pub fn new(threshold: f64, allowed_directions: Vec<SwipeDirection>) -> Self {
        Self {
            threshold,
            velocity_threshold: 0.11,
            min_velocity_delta: 10.0,
            allowed_directions,
        }
    }

    pub fn from_toast_config(config: &ToastConfig) -> Self {
        Self {
            threshold: config.swipe_threshold,
            velocity_threshold: 0.11,
            min_velocity_delta: 10.0,
            allowed_directions: config.effective_swipe_directions(),
        }
    }

    pub fn is_direction_allowed(&self, direction: SwipeDirection) -> bool {
        self.allowed_directions.contains(&direction)
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_velocity_threshold(mut self, velocity_threshold: f64) -> Self {
        self.velocity_threshold = velocity_threshold;
        self
    }

    pub fn with_min_velocity_delta(mut self, min_velocity_delta: f64) -> Self {
        self.min_velocity_delta = min_velocity_delta;
        self
    }

    pub fn with_allowed_directions(mut self, directions: Vec<SwipeDirection>) -> Self {
        self.allowed_directions = directions;
        self
    }
}

impl Default for SwipeReducerConfig {
    fn default() -> Self {
        Self {
            threshold: 45.0,
            velocity_threshold: 0.11,
            min_velocity_delta: 10.0,
            allowed_directions: vec![SwipeDirection::Right, SwipeDirection::Bottom],
        }
    }
}

/// Dispatched gesture actions emitted by the pure Rust swipe state machine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwipeAction {
    /// Gesture met dismissal criteria (threshold or velocity) in an allowed direction.
    Dismiss(u64),
    /// Gesture was released below dismissal criteria or cancelled; card should animate back to origin.
    SnapBack,
    /// Intermediate position update while actively dragging.
    Update { delta_x: f64, delta_y: f64 },
}

/// Pure state snapshot tracking an in-flight pointer swipe gesture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwipeState {
    pub drag_start: Point2D,
    pub current_point: Point2D,
    pub start_time_ms: f64,
    pub axis: Option<SwipeAxis>,
    pub is_swiping: bool,
}

impl SwipeState {
    pub fn new(start_point: Point2D, start_time_ms: f64) -> Self {
        Self {
            drag_start: start_point,
            current_point: start_point,
            start_time_ms,
            axis: None,
            is_swiping: false,
        }
    }

    /// Horizontal displacement from start position in screen pixels.
    pub fn delta_x(&self) -> f64 {
        self.current_point.x - self.drag_start.x
    }

    /// Vertical displacement from start position in screen pixels.
    pub fn delta_y(&self) -> f64 {
        self.current_point.y - self.drag_start.y
    }

    /// Non-linear dampening formula matching Sonner reference behavior:
    /// `delta * (1.0 / (1.5 + (delta.abs() / 20.0)))`.
    pub fn compute_dampened_delta(delta: f64) -> f64 {
        delta * (1.0 / (1.5 + (delta.abs() / 20.0)))
    }

    /// Evaluates whether the gesture qualifies for dismissal at release time.
    ///
    /// Requires an allowed swipe direction, and either:
    /// 1. `abs_delta >= config.threshold` (default 45.0 px), OR
    /// 2. `velocity > config.velocity_threshold` (0.11 px/ms) AND `abs_delta > config.min_velocity_delta` (10.0 px).
    pub fn evaluate_dismissal(&self, elapsed_ms: f64, config: &SwipeReducerConfig) -> bool {
        let (raw_delta, dir) = match self.axis {
            Some(SwipeAxis::X) => {
                let dx = self.delta_x();
                if dx > 0.0 {
                    (dx, SwipeDirection::Right)
                } else if dx < 0.0 {
                    (dx, SwipeDirection::Left)
                } else {
                    return false;
                }
            }
            Some(SwipeAxis::Y) => {
                let dy = self.delta_y();
                if dy > 0.0 {
                    (dy, SwipeDirection::Bottom)
                } else if dy < 0.0 {
                    (dy, SwipeDirection::Top)
                } else {
                    return false;
                }
            }
            None => return false,
        };

        if !config.is_direction_allowed(dir) {
            return false;
        }

        let abs_delta = raw_delta.abs();
        let elapsed = if elapsed_ms <= 0.0 { 1.0 } else { elapsed_ms };
        let velocity = abs_delta / elapsed;

        abs_delta >= config.threshold
            || (velocity > config.velocity_threshold && abs_delta > config.min_velocity_delta)
    }

    /// Updates pointer position, locks the gesture axis on first significant motion,
    /// filters direction, applies non-linear dampening when moving in disallowed directions,
    /// and returns the updated swipe action.
    pub fn update(&mut self, point: Point2D, config: &SwipeReducerConfig) -> SwipeAction {
        self.current_point = point;
        let raw_dx = self.delta_x();
        let raw_dy = self.delta_y();

        if self.axis.is_none() && (raw_dx.abs() > 1.0 || raw_dy.abs() > 1.0) {
            self.axis = Some(if raw_dx.abs() > raw_dy.abs() {
                SwipeAxis::X
            } else {
                SwipeAxis::Y
            });
        }

        let (effective_dx, effective_dy) = match self.axis {
            Some(SwipeAxis::X) => {
                let is_allowed = (config.is_direction_allowed(SwipeDirection::Right) && raw_dx > 0.0)
                    || (config.is_direction_allowed(SwipeDirection::Left) && raw_dx < 0.0);
                let dx = if is_allowed {
                    raw_dx
                } else {
                    let dampened = Self::compute_dampened_delta(raw_dx);
                    if dampened.abs() < raw_dx.abs() {
                        dampened
                    } else {
                        raw_dx
                    }
                };
                (dx, 0.0)
            }
            Some(SwipeAxis::Y) => {
                let is_allowed = (config.is_direction_allowed(SwipeDirection::Bottom) && raw_dy > 0.0)
                    || (config.is_direction_allowed(SwipeDirection::Top) && raw_dy < 0.0);
                let dy = if is_allowed {
                    raw_dy
                } else {
                    let dampened = Self::compute_dampened_delta(raw_dy);
                    if dampened.abs() < raw_dy.abs() {
                        dampened
                    } else {
                        raw_dy
                    }
                };
                (0.0, dy)
            }
            None => (0.0, 0.0),
        };

        if effective_dx.abs() > 0.0 || effective_dy.abs() > 0.0 {
            self.is_swiping = true;
        }

        SwipeAction::Update {
            delta_x: effective_dx,
            delta_y: effective_dy,
        }
    }

    /// Evaluates gesture termination upon pointer release.
    pub fn release(&self, toast_id: u64, elapsed_ms: f64, config: &SwipeReducerConfig) -> SwipeAction {
        if self.is_swiping && self.evaluate_dismissal(elapsed_ms, config) {
            SwipeAction::Dismiss(toast_id)
        } else {
            SwipeAction::SnapBack
        }
    }

    /// Cancels active swipe, requiring the card to snap back.
    pub fn cancel(&self) -> SwipeAction {
        SwipeAction::SnapBack
    }
}

/// Stateful reducer wrapping `SwipeState` and configuration for a specific toast.
#[derive(Clone, Debug, PartialEq)]
pub struct SwipeGestureReducer {
    pub toast_id: u64,
    pub state: SwipeState,
    pub config: SwipeReducerConfig,
}

impl SwipeGestureReducer {
    pub fn new(
        toast_id: u64,
        start_point: Point2D,
        start_time_ms: f64,
        config: SwipeReducerConfig,
    ) -> Self {
        Self {
            toast_id,
            state: SwipeState::new(start_point, start_time_ms),
            config,
        }
    }

    pub fn handle_move(&mut self, point: Point2D) -> SwipeAction {
        self.state.update(point, &self.config)
    }

    pub fn handle_release(&self, current_time_ms: f64) -> SwipeAction {
        let elapsed = (current_time_ms - self.state.start_time_ms).max(1.0);
        self.state.release(self.toast_id, elapsed, &self.config)
    }

    pub fn handle_cancel(&self) -> SwipeAction {
        self.state.cancel()
    }

    pub fn is_swiping(&self) -> bool {
        self.state.is_swiping
    }

    pub fn axis(&self) -> Option<SwipeAxis> {
        self.state.axis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axis_lock_on_movement() {
        let start = Point2D::new(100.0, 100.0);
        let mut state = SwipeState::new(start, 0.0);
        let config = SwipeReducerConfig::default();

        assert_eq!(state.axis, None);

        // Movement with dx > dy locks to X axis
        let action = state.update(Point2D::new(105.0, 101.0), &config);
        assert_eq!(state.axis, Some(SwipeAxis::X));
        match action {
            SwipeAction::Update { delta_x, delta_y } => {
                assert!((delta_x - 5.0).abs() < 1e-4);
                assert_eq!(delta_y, 0.0);
            }
            _ => panic!("Expected Update action"),
        }

        // Subsequent move with large vertical shift preserves locked X axis
        let action2 = state.update(Point2D::new(110.0, 180.0), &config);
        assert_eq!(state.axis, Some(SwipeAxis::X));
        match action2 {
            SwipeAction::Update { delta_x, delta_y } => {
                assert!((delta_x - 10.0).abs() < 1e-4);
                assert_eq!(delta_y, 0.0);
            }
            _ => panic!("Expected Update action"),
        }

        // Repeat with vertical dominant motion locking to Y axis
        let mut state_y = SwipeState::new(start, 0.0);
        let action_y = state_y.update(Point2D::new(101.0, 106.0), &config);
        assert_eq!(state_y.axis, Some(SwipeAxis::Y));
        match action_y {
            SwipeAction::Update { delta_x, delta_y } => {
                assert_eq!(delta_x, 0.0);
                assert!((delta_y - 6.0).abs() < 1e-4);
            }
            _ => panic!("Expected Update action"),
        }

        // Subsequent move keeps Y axis locked
        let action_y2 = state_y.update(Point2D::new(190.0, 120.0), &config);
        assert_eq!(state_y.axis, Some(SwipeAxis::Y));
        match action_y2 {
            SwipeAction::Update { delta_x, delta_y } => {
                assert_eq!(delta_x, 0.0);
                assert!((delta_y - 20.0).abs() < 1e-4);
            }
            _ => panic!("Expected Update action"),
        }
    }

    #[test]
    fn test_direction_filtering_against_allowed_directions() {
        let start = Point2D::new(50.0, 50.0);
        // Only allow Right swipe
        let config = SwipeReducerConfig::new(45.0, vec![SwipeDirection::Right]);

        let mut state_right = SwipeState::new(start, 0.0);
        let action_right = state_right.update(Point2D::new(80.0, 50.0), &config);
        match action_right {
            SwipeAction::Update { delta_x, delta_y } => {
                assert_eq!(delta_x, 30.0, "Allowed right swipe must not be dampened");
                assert_eq!(delta_y, 0.0);
            }
            _ => panic!("Expected Update action"),
        }

        let mut state_left = SwipeState::new(start, 0.0);
        let action_left = state_left.update(Point2D::new(20.0, 50.0), &config);
        match action_left {
            SwipeAction::Update { delta_x, delta_y } => {
                let expected_dampened = -30.0 * (1.0 / (1.5 + (30.0 / 20.0))); // -30 / 3.0 = -10.0
                assert!(
                    (delta_x - expected_dampened).abs() < 1e-4,
                    "Disallowed left swipe must be dampened"
                );
                assert!(delta_x.abs() < 30.0);
                assert_eq!(delta_y, 0.0);
            }
            _ => panic!("Expected Update action"),
        }

        // Only allow Bottom swipe
        let config_y = SwipeReducerConfig::new(45.0, vec![SwipeDirection::Bottom]);
        let mut state_down = SwipeState::new(start, 0.0);
        let action_down = state_down.update(Point2D::new(50.0, 80.0), &config_y);
        match action_down {
            SwipeAction::Update { delta_x, delta_y } => {
                assert_eq!(delta_x, 0.0);
                assert_eq!(delta_y, 30.0, "Allowed bottom swipe must not be dampened");
            }
            _ => panic!("Expected Update action"),
        }

        let mut state_up = SwipeState::new(start, 0.0);
        let action_up = state_up.update(Point2D::new(50.0, 20.0), &config_y);
        match action_up {
            SwipeAction::Update { delta_x, delta_y } => {
                assert_eq!(delta_x, 0.0);
                let expected_dampened = -30.0 * (1.0 / (1.5 + (30.0 / 20.0))); // -10.0
                assert!(
                    (delta_y - expected_dampened).abs() < 1e-4,
                    "Disallowed up swipe must be dampened"
                );
            }
            _ => panic!("Expected Update action"),
        }
    }

    #[test]
    fn test_non_linear_dampening_formula() {
        assert_eq!(SwipeState::compute_dampened_delta(0.0), 0.0);

        // delta = 20.0: factor = 20.0 / 20.0 = 1.0; 20.0 / (1.5 + 1.0) = 8.0
        let dampened_20 = SwipeState::compute_dampened_delta(20.0);
        assert!((dampened_20 - 8.0).abs() < 1e-4);

        // delta = -20.0: -8.0
        let dampened_neg20 = SwipeState::compute_dampened_delta(-20.0);
        assert!((dampened_neg20 - (-8.0)).abs() < 1e-4);

        // delta = 40.0: factor = 40.0 / 20.0 = 2.0; 40.0 / (1.5 + 2.0) = 40.0 / 3.5 ≈ 11.42857
        let dampened_40 = SwipeState::compute_dampened_delta(40.0);
        assert!((dampened_40 - (40.0 / 3.5)).abs() < 1e-4);

        // Verify diminishing returns (non-linear resistance):
        // Going from 20 -> 40 increases dampened delta by ~3.43 px, whereas initial 20 gave 8.0 px
        assert!(dampened_40 - dampened_20 < dampened_20);
    }

    #[test]
    fn test_threshold_crossing_vs_velocity_fast_swipe() {
        let config = SwipeReducerConfig::new(45.0, vec![SwipeDirection::Right]);
        let start = Point2D::new(0.0, 0.0);

        // 1. Slow swipe crossing 45.0 threshold -> dismiss
        let mut state = SwipeState::new(start, 0.0);
        state.update(Point2D::new(50.0, 0.0), &config);
        assert!(
            state.evaluate_dismissal(1000.0, &config),
            "Displacement 50px >= 45px threshold must trigger dismissal even at low velocity"
        );

        // 2. Slow swipe below 45.0 threshold -> snap back
        let mut state_slow_short = SwipeState::new(start, 0.0);
        state_slow_short.update(Point2D::new(40.0, 0.0), &config);
        assert!(
            !state_slow_short.evaluate_dismissal(1000.0, &config),
            "Displacement 40px < 45px at low velocity (0.04 px/ms) must not dismiss"
        );

        // 3. Fast flick below threshold (e.g. 15px in 50ms = 0.3 px/ms > 0.11 px/ms and abs_delta > 10.0) -> dismiss
        let mut state_fast = SwipeState::new(start, 0.0);
        state_fast.update(Point2D::new(15.0, 0.0), &config);
        assert!(
            state_fast.evaluate_dismissal(50.0, &config),
            "Fast velocity (0.3 px/ms) with displacement > 10px must trigger dismissal"
        );

        // 4. Fast flick with tiny delta (<= 10.0 px) -> snap back
        let mut state_tiny_fast = SwipeState::new(start, 0.0);
        state_tiny_fast.update(Point2D::new(8.0, 0.0), &config);
        assert!(
            !state_tiny_fast.evaluate_dismissal(20.0, &config),
            "Flick with displacement <= 10px must not dismiss even if velocity is high"
        );

        // 5. Swipe in disallowed direction (Left when only Right allowed) -> snap back
        let mut state_disallowed = SwipeState::new(start, 0.0);
        state_disallowed.update(Point2D::new(-60.0, 0.0), &config);
        assert!(
            !state_disallowed.evaluate_dismissal(50.0, &config),
            "High velocity / delta in disallowed direction must never trigger dismissal"
        );
    }

    #[test]
    fn test_snap_back_on_cancelled_or_insufficient_swipe() {
        let config = SwipeReducerConfig::default();
        let start = Point2D::new(0.0, 0.0);
        let mut reducer = SwipeGestureReducer::new(42, start, 0.0, config);

        // Movement below threshold at slow speed
        reducer.handle_move(Point2D::new(15.0, 0.0));
        assert!(reducer.is_swiping());

        // Release produces SnapBack
        let action = reducer.handle_release(1000.0);
        assert_eq!(
            action,
            SwipeAction::SnapBack,
            "Insufficient swipe must produce SnapBack"
        );

        // Cancellation produces SnapBack
        let mut cancel_reducer = SwipeGestureReducer::new(42, start, 0.0, SwipeReducerConfig::default());
        cancel_reducer.handle_move(Point2D::new(60.0, 0.0));
        let cancel_action = cancel_reducer.handle_cancel();
        assert_eq!(
            cancel_action,
            SwipeAction::SnapBack,
            "Explicit cancel must produce SnapBack"
        );

        // Sufficient swipe produces Dismiss(toast_id)
        let mut dismiss_reducer = SwipeGestureReducer::new(99, start, 0.0, SwipeReducerConfig::default());
        dismiss_reducer.handle_move(Point2D::new(60.0, 0.0));
        let dismiss_action = dismiss_reducer.handle_release(500.0);
        assert_eq!(
            dismiss_action,
            SwipeAction::Dismiss(99),
            "Sufficient swipe must produce Dismiss with corresponding ID"
        );
    }
}
