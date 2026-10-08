//! Safe pointer grace area geometry calculations.
//!
//! Provides headless, pure-Rust geometry primitives for computing pointer grace areas
//! (safe corridors) between triggers and floating overlays (e.g., tooltips, popovers, hover cards).
//! Contains zero host dependencies or DOM assumptions.

/// A 2D point in screen coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A 2D axis-aligned bounding rectangle in screen coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect2D {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect2D {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn left(&self) -> f64 {
        self.x
    }

    pub fn top(&self) -> f64 {
        self.y
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// Returns the 4 corners of the rectangle: [top-left, top-right, bottom-right, bottom-left].
    pub fn corners(&self) -> [Point2D; 4] {
        [
            Point2D::new(self.left(), self.top()),
            Point2D::new(self.right(), self.top()),
            Point2D::new(self.right(), self.bottom()),
            Point2D::new(self.left(), self.bottom()),
        ]
    }

    /// Checks if a point lies within or on the boundary of the rectangle.
    pub fn contains_point(&self, point: Point2D) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }
}

/// The cardinal side of a rectangle through which a pointer exited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardinalSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// Default padding applied to the pointer exit point when creating a grace area.
pub const DEFAULT_GRACE_PADDING: f64 = 5.0;

/// Determines which cardinal edge of `rect` is closest to the given `point`.
///
/// In the case of equidistant sides (e.g. exact corners), precedence is
/// Left, Right, Top, Bottom.
pub fn get_exit_side_from_rect(point: Point2D, rect: &Rect2D) -> CardinalSide {
    let top = (rect.top() - point.y).abs();
    let bottom = (rect.bottom() - point.y).abs();
    let right = (rect.right() - point.x).abs();
    let left = (rect.left() - point.x).abs();

    let min_dist = left.min(right).min(top).min(bottom);
    if (left - min_dist).abs() < 1e-9 {
        CardinalSide::Left
    } else if (right - min_dist).abs() < 1e-9 {
        CardinalSide::Right
    } else if (top - min_dist).abs() < 1e-9 {
        CardinalSide::Top
    } else {
        CardinalSide::Bottom
    }
}

/// Generates a 3-point triangular wedge around the exit point extending in the direction
/// of pointer exit to provide a forgiving corridor base.
pub fn get_padded_exit_points(point: Point2D, side: CardinalSide, padding: f64) -> [Point2D; 3] {
    let tip_padding = padding * 1.5;
    match side {
        CardinalSide::Top => [
            Point2D::new(point.x - padding, point.y + padding),
            Point2D::new(point.x, point.y - tip_padding),
            Point2D::new(point.x + padding, point.y + padding),
        ],
        CardinalSide::Bottom => [
            Point2D::new(point.x - padding, point.y - padding),
            Point2D::new(point.x, point.y + tip_padding),
            Point2D::new(point.x + padding, point.y - padding),
        ],
        CardinalSide::Left => [
            Point2D::new(point.x + padding, point.y - padding),
            Point2D::new(point.x - tip_padding, point.y),
            Point2D::new(point.x + padding, point.y + padding),
        ],
        CardinalSide::Right => [
            Point2D::new(point.x - padding, point.y - padding),
            Point2D::new(point.x + tip_padding, point.y),
            Point2D::new(point.x - padding, point.y + padding),
        ],
    }
}

/// Helper function to check if a point lies on a 2D line segment between `a` and `b`.
pub fn is_point_on_segment(p: Point2D, a: Point2D, b: Point2D, epsilon: f64) -> bool {
    let seg_len = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
    if seg_len < epsilon {
        return (p.x - a.x).abs() <= epsilon && (p.y - a.y).abs() <= epsilon;
    }
    let cross = ((b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)).abs();
    if cross / seg_len > epsilon {
        return false;
    }

    let min_x = a.x.min(b.x) - epsilon;
    let max_x = a.x.max(b.x) + epsilon;
    let min_y = a.y.min(b.y) - epsilon;
    let max_y = a.y.max(b.y) + epsilon;

    p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y
}

/// Tests whether a point lies within or on the boundary of a 2D polygon using
/// ray-casting algorithm (crossing number) with boundary edge inclusion.
pub fn is_point_in_polygon(point: Point2D, polygon: &[Point2D]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    // Check boundary inclusion (points directly on polygon edges or vertices)
    const EPSILON: f64 = 1e-9;
    for i in 0..polygon.len() {
        let j = if i == 0 { polygon.len() - 1 } else { i - 1 };
        if is_point_on_segment(point, polygon[j], polygon[i], EPSILON) {
            return true;
        }
    }

    let x = point.x;
    let y = point.y;
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let xi = polygon[i].x;
        let yi = polygon[i].y;
        let xj = polygon[j].x;
        let yj = polygon[j].y;

        let intersect = ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Computes the convex hull of 2D points using Andrew's monotone chain algorithm
/// in screen-space coordinates (Y pointing downwards).
///
/// Returns the vertices of the convex hull in cyclic boundary traversal order.
pub fn get_convex_hull(points: &[Point2D]) -> Vec<Point2D> {
    if points.len() <= 1 {
        return points.to_vec();
    }

    let mut new_points = points.to_vec();
    new_points.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });

    // Upper hull
    let mut upper_hull: Vec<Point2D> = Vec::new();
    for &p in &new_points {
        while upper_hull.len() >= 2 {
            let q = upper_hull[upper_hull.len() - 1];
            let r = upper_hull[upper_hull.len() - 2];
            if (q.x - r.x) * (p.y - r.y) >= (q.y - r.y) * (p.x - r.x) {
                upper_hull.pop();
            } else {
                break;
            }
        }
        upper_hull.push(p);
    }
    upper_hull.pop();

    // Lower hull
    let mut lower_hull: Vec<Point2D> = Vec::new();
    for &p in new_points.iter().rev() {
        while lower_hull.len() >= 2 {
            let q = lower_hull[lower_hull.len() - 1];
            let r = lower_hull[lower_hull.len() - 2];
            if (q.x - r.x) * (p.y - r.y) >= (q.y - r.y) * (p.x - r.x) {
                lower_hull.pop();
            } else {
                break;
            }
        }
        lower_hull.push(p);
    }
    lower_hull.pop();

    if upper_hull.len() == 1 && lower_hull.len() == 1 && upper_hull[0] == lower_hull[0] {
        return upper_hull;
    }

    upper_hull.extend(lower_hull);
    upper_hull
}

/// Constructs a safe pointer grace polygon connecting an exit point from `current_rect`
/// to the target boundary of `target_rect`.
///
/// If `padding` is omitted, [`DEFAULT_GRACE_PADDING`] (5.0px) is used.
pub fn create_grace_area(
    exit_point: Point2D,
    current_rect: &Rect2D,
    target_rect: &Rect2D,
    padding: Option<f64>,
) -> Vec<Point2D> {
    let padding = padding.unwrap_or(DEFAULT_GRACE_PADDING);
    let exit_side = get_exit_side_from_rect(exit_point, current_rect);
    let padded_exit_points = get_padded_exit_points(exit_point, exit_side, padding);
    let target_corners = target_rect.corners();

    let mut points = Vec::with_capacity(7);
    points.extend_from_slice(&padded_exit_points);
    points.extend_from_slice(&target_corners);

    get_convex_hull(&points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect2d_corners_and_contains() {
        let rect = Rect2D::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.left(), 10.0);
        assert_eq!(rect.top(), 20.0);
        assert_eq!(rect.right(), 110.0);
        assert_eq!(rect.bottom(), 70.0);

        let corners = rect.corners();
        assert_eq!(
            corners,
            [
                Point2D::new(10.0, 20.0),
                Point2D::new(110.0, 20.0),
                Point2D::new(110.0, 70.0),
                Point2D::new(10.0, 70.0),
            ]
        );

        assert!(rect.contains_point(Point2D::new(50.0, 40.0)));
        assert!(rect.contains_point(Point2D::new(10.0, 20.0)));
        assert!(rect.contains_point(Point2D::new(110.0, 70.0)));
        assert!(!rect.contains_point(Point2D::new(9.9, 40.0)));
        assert!(!rect.contains_point(Point2D::new(110.1, 40.0)));
    }

    #[test]
    fn test_exit_side_detection() {
        let rect = Rect2D::new(100.0, 100.0, 100.0, 50.0);
        // top edge is at y=100, bottom at y=150, left at x=100, right at x=200

        // Exiting top
        assert_eq!(
            get_exit_side_from_rect(Point2D::new(150.0, 99.0), &rect),
            CardinalSide::Top
        );
        // Exiting bottom
        assert_eq!(
            get_exit_side_from_rect(Point2D::new(150.0, 151.0), &rect),
            CardinalSide::Bottom
        );
        // Exiting left
        assert_eq!(
            get_exit_side_from_rect(Point2D::new(99.0, 125.0), &rect),
            CardinalSide::Left
        );
        // Exiting right
        assert_eq!(
            get_exit_side_from_rect(Point2D::new(201.0, 125.0), &rect),
            CardinalSide::Right
        );

        // Precedence on exact corner tie (Left, Right, Top, Bottom)
        // At top-left corner (100, 100)
        assert_eq!(
            get_exit_side_from_rect(Point2D::new(100.0, 100.0), &rect),
            CardinalSide::Left
        );
    }

    #[test]
    fn test_padded_exit_points() {
        let pt = Point2D::new(100.0, 100.0);
        let padding = 10.0;
        let tip_padding = 15.0;

        let top_pts = get_padded_exit_points(pt, CardinalSide::Top, padding);
        assert_eq!(
            top_pts,
            [
                Point2D::new(90.0, 110.0),
                Point2D::new(100.0, 85.0),
                Point2D::new(110.0, 110.0),
            ]
        );

        let bottom_pts = get_padded_exit_points(pt, CardinalSide::Bottom, padding);
        assert_eq!(
            bottom_pts,
            [
                Point2D::new(90.0, 90.0),
                Point2D::new(100.0, 100.0 + tip_padding),
                Point2D::new(110.0, 90.0),
            ]
        );

        let left_pts = get_padded_exit_points(pt, CardinalSide::Left, padding);
        assert_eq!(
            left_pts,
            [
                Point2D::new(110.0, 90.0),
                Point2D::new(85.0, 100.0),
                Point2D::new(110.0, 110.0),
            ]
        );

        let right_pts = get_padded_exit_points(pt, CardinalSide::Right, padding);
        assert_eq!(
            right_pts,
            [
                Point2D::new(90.0, 90.0),
                Point2D::new(115.0, 100.0),
                Point2D::new(90.0, 110.0),
            ]
        );
    }

    #[test]
    fn test_point_in_polygon_inside_outside_edge() {
        let poly = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
        ];

        // Inside
        assert!(is_point_in_polygon(Point2D::new(5.0, 5.0), &poly));
        assert!(is_point_in_polygon(Point2D::new(1.0, 1.0), &poly));
        assert!(is_point_in_polygon(Point2D::new(9.0, 9.0), &poly));

        // Outside
        assert!(!is_point_in_polygon(Point2D::new(15.0, 5.0), &poly));
        assert!(!is_point_in_polygon(Point2D::new(-5.0, 5.0), &poly));
        assert!(!is_point_in_polygon(Point2D::new(5.0, -1.0), &poly));
        assert!(!is_point_in_polygon(Point2D::new(5.0, 11.0), &poly));

        // On edge
        assert!(is_point_in_polygon(Point2D::new(5.0, 0.0), &poly)); // Top edge
        assert!(is_point_in_polygon(Point2D::new(10.0, 5.0), &poly)); // Right edge
        assert!(is_point_in_polygon(Point2D::new(5.0, 10.0), &poly)); // Bottom edge
        assert!(is_point_in_polygon(Point2D::new(0.0, 5.0), &poly)); // Left edge

        // On vertices
        assert!(is_point_in_polygon(Point2D::new(0.0, 0.0), &poly));
        assert!(is_point_in_polygon(Point2D::new(10.0, 0.0), &poly));
        assert!(is_point_in_polygon(Point2D::new(10.0, 10.0), &poly));
        assert!(is_point_in_polygon(Point2D::new(0.0, 10.0), &poly));

        // Degenerate polygons
        assert!(!is_point_in_polygon(Point2D::new(0.0, 0.0), &[]));
        assert!(!is_point_in_polygon(
            Point2D::new(0.0, 0.0),
            &[Point2D::new(0.0, 0.0)]
        ));
        assert!(!is_point_in_polygon(
            Point2D::new(0.0, 0.0),
            &[Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)]
        ));
    }

    #[test]
    fn test_point_on_segment() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(10.0, 10.0);

        assert!(is_point_on_segment(Point2D::new(5.0, 5.0), a, b, 1e-9));
        assert!(is_point_on_segment(a, a, b, 1e-9));
        assert!(is_point_on_segment(b, a, b, 1e-9));
        assert!(!is_point_on_segment(Point2D::new(11.0, 11.0), a, b, 1e-9));
        assert!(!is_point_on_segment(Point2D::new(5.0, 6.0), a, b, 1e-9));
    }

    #[test]
    fn test_convex_hull_degenerate_and_collinear() {
        // Empty
        assert_eq!(get_convex_hull(&[]), vec![]);

        // Single point
        let single = Point2D::new(42.0, 42.0);
        assert_eq!(get_convex_hull(&[single]), vec![single]);

        // Duplicate identical points
        let dupes = vec![single, single, single];
        assert_eq!(get_convex_hull(&dupes), vec![single]);

        // Two points
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(10.0, 10.0);
        let two = get_convex_hull(&[p1, p2]);
        assert_eq!(two.len(), 2);
        assert!(two.contains(&p1) && two.contains(&p2));

        // Collinear points
        let collinear = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(3.0, 3.0),
            Point2D::new(7.0, 7.0),
            Point2D::new(10.0, 10.0),
        ];
        let hull = get_convex_hull(&collinear);
        assert_eq!(hull.len(), 2);
        assert_eq!(hull[0], Point2D::new(0.0, 0.0));
        assert_eq!(hull[1], Point2D::new(10.0, 10.0));
    }

    #[test]
    fn test_convex_hull_triangle_and_rectangle() {
        // Simple triangle
        let triangle = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(5.0, 10.0),
            // Interior point
            Point2D::new(5.0, 3.0),
        ];
        let hull = get_convex_hull(&triangle);
        assert_eq!(hull.len(), 3);
        assert!(!hull.contains(&Point2D::new(5.0, 3.0)));

        // Rectangle with interior points
        let rect_pts = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
            Point2D::new(5.0, 5.0),
            Point2D::new(2.0, 8.0),
        ];
        let hull_rect = get_convex_hull(&rect_pts);
        assert_eq!(hull_rect.len(), 4);
        assert!(!hull_rect.contains(&Point2D::new(5.0, 5.0)));
        assert!(!hull_rect.contains(&Point2D::new(2.0, 8.0)));
    }

    #[test]
    fn test_create_grace_area() {
        // Trigger is at top: (100, 50, 50, 20) -> y in [50, 70]
        let trigger = Rect2D::new(100.0, 50.0, 50.0, 20.0);
        // Tooltip is below trigger: (80, 100, 90, 40) -> y in [100, 140]
        let tooltip = Rect2D::new(80.0, 100.0, 90.0, 40.0);

        // Exit through bottom of trigger
        let exit_point = Point2D::new(125.0, 70.0);
        let grace_area = create_grace_area(exit_point, &trigger, &tooltip, Some(5.0));

        // Must form a valid polygon with at least 3 vertices
        assert!(grace_area.len() >= 3);

        // A point midway in the corridor between trigger and tooltip should be inside the grace area
        let corridor_point = Point2D::new(125.0, 85.0);
        assert!(is_point_in_polygon(corridor_point, &grace_area));

        // Tooltip center should be inside
        let tooltip_center = Point2D::new(125.0, 120.0);
        assert!(is_point_in_polygon(tooltip_center, &grace_area));

        // A point far to the left or right should be outside
        assert!(!is_point_in_polygon(Point2D::new(0.0, 85.0), &grace_area));
        assert!(!is_point_in_polygon(Point2D::new(300.0, 85.0), &grace_area));
    }
}
