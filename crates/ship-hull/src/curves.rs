/// Centripetal Catmull-Rom interpolation with explicit endpoint tangents.
///
/// Control points are (u, value) pairs where u is normalized [0,1] position along hull length.
/// Interior points use centripetal Catmull-Rom. Endpoints use one-sided tangent computation
/// with configurable scale and slope cap for style-aware bow/stern behavior.

/// A single control point on a longitudinal curve.
#[derive(Debug, Clone, Copy)]
pub struct ControlPoint {
    pub u: f64,
    pub value: f64,
}

/// Configuration for endpoint tangent behavior.
#[derive(Debug, Clone, Copy)]
pub struct EndpointConfig {
    /// Tangent scale multiplier for the stern (u=0) end.
    pub stern_tangent_scale: f64,
    /// Maximum slope magnitude at stern.
    pub stern_slope_cap: f64,
    /// Tangent scale multiplier for the bow (u=1) end.
    pub bow_tangent_scale: f64,
    /// Maximum slope magnitude at bow.
    pub bow_slope_cap: f64,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            stern_tangent_scale: 1.0,
            stern_slope_cap: 3.0,
            bow_tangent_scale: 1.0,
            bow_slope_cap: 3.0,
        }
    }
}

/// A piecewise curve defined by control points with Catmull-Rom interior
/// and explicit endpoint tangents.
#[derive(Debug, Clone)]
pub struct LongitudinalCurve {
    points: Vec<ControlPoint>,
    endpoint_config: EndpointConfig,
}

impl LongitudinalCurve {
    /// Create a new curve from control points.
    /// Points must be sorted by u and have at least 2 entries.
    pub fn new(points: Vec<ControlPoint>, endpoint_config: EndpointConfig) -> Result<Self, CurveError> {
        if points.len() < 2 {
            return Err(CurveError::TooFewPoints(points.len()));
        }
        // Check sorted and finite
        for i in 0..points.len() {
            if !points[i].u.is_finite() || !points[i].value.is_finite() {
                return Err(CurveError::NonFinite(i));
            }
            if i > 0 && points[i].u <= points[i - 1].u {
                return Err(CurveError::NotSorted(i));
            }
        }
        Ok(Self { points, endpoint_config })
    }

    /// Evaluate the curve at parameter u.
    /// Clamps to the first/last control point value outside the defined range.
    pub fn eval(&self, u: f64) -> f64 {
        let n = self.points.len();

        // Clamp outside range
        if u <= self.points[0].u {
            return self.points[0].value;
        }
        if u >= self.points[n - 1].u {
            return self.points[n - 1].value;
        }

        // Find the segment [i, i+1] containing u
        let seg = self.find_segment(u);
        let i = seg;

        // Local parameter t within this segment
        let u0 = self.points[i].u;
        let u1 = self.points[i + 1].u;
        let t = (u - u0) / (u1 - u0);

        // Get the four values and tangents needed for Hermite interpolation
        let p0 = self.points[i].value;
        let p1 = self.points[i + 1].value;
        let m0 = self.tangent_at(i) * (u1 - u0);
        let m1 = self.tangent_at(i + 1) * (u1 - u0);

        // Cubic Hermite
        hermite(t, p0, p1, m0, m1)
    }

    /// Compute the tangent (slope) at control point index.
    fn tangent_at(&self, idx: usize) -> f64 {
        let n = self.points.len();

        if idx == 0 {
            // Stern endpoint: one-sided tangent
            let dp = self.points[1].value - self.points[0].value;
            let du = self.points[1].u - self.points[0].u;
            let slope = dp / du;
            let scaled = slope * self.endpoint_config.stern_tangent_scale;
            clamp_magnitude(scaled, self.endpoint_config.stern_slope_cap)
        } else if idx == n - 1 {
            // Bow endpoint: one-sided tangent
            let dp = self.points[n - 1].value - self.points[n - 2].value;
            let du = self.points[n - 1].u - self.points[n - 2].u;
            let slope = dp / du;
            let scaled = slope * self.endpoint_config.bow_tangent_scale;
            clamp_magnitude(scaled, self.endpoint_config.bow_slope_cap)
        } else {
            // Interior: centripetal Catmull-Rom tangent
            catmull_rom_tangent(
                &self.points[idx - 1],
                &self.points[idx],
                &self.points[idx + 1],
            )
        }
    }

    /// Find the segment index such that points[i].u <= u < points[i+1].u.
    fn find_segment(&self, u: f64) -> usize {
        for i in 0..self.points.len() - 1 {
            if u < self.points[i + 1].u {
                return i;
            }
        }
        self.points.len() - 2
    }

    /// Number of control points.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Access control points.
    pub fn points(&self) -> &[ControlPoint] {
        &self.points
    }
}

/// Centripetal Catmull-Rom tangent at the middle point of three consecutive points.
fn catmull_rom_tangent(p0: &ControlPoint, p1: &ControlPoint, p2: &ControlPoint) -> f64 {
    // Centripetal parameterization: use sqrt of chord length
    let d01 = ((p1.u - p0.u).powi(2) + (p1.value - p0.value).powi(2)).sqrt().sqrt();
    let d12 = ((p2.u - p1.u).powi(2) + (p2.value - p1.value).powi(2)).sqrt().sqrt();

    if d01 < 1e-12 || d12 < 1e-12 {
        // Degenerate: fall back to finite difference
        return (p2.value - p0.value) / (p2.u - p0.u);
    }

    // Centripetal tangent formula
    let t1 = (p1.value - p0.value) / d01 + (p2.value - p1.value) / d12
        - (p2.value - p0.value) / (d01 + d12);

    // Scale by the average segment parameter span for proper units
    let avg_du = (p2.u - p0.u) / 2.0;
    if avg_du.abs() < 1e-12 {
        return 0.0;
    }

    // The tangent in u-space
    let t_u1 = (p1.u - p0.u) / d01 + (p2.u - p1.u) / d12
        - (p2.u - p0.u) / (d01 + d12);

    if t_u1.abs() < 1e-12 {
        return (p2.value - p0.value) / (p2.u - p0.u);
    }

    t1 / t_u1
}

/// Cubic Hermite interpolation.
/// t in [0,1], p0/p1 are endpoint values, m0/m1 are tangents scaled by segment width.
fn hermite(t: f64, p0: f64, p1: f64, m0: f64, m1: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    h00 * p0 + h10 * m0 + h01 * p1 + h11 * m1
}

/// Clamp a value's magnitude without changing sign.
fn clamp_magnitude(v: f64, max_mag: f64) -> f64 {
    if v.abs() > max_mag {
        v.signum() * max_mag
    } else {
        v
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CurveError {
    #[error("need at least 2 control points, got {0}")]
    TooFewPoints(usize),
    #[error("control point {0} has non-finite value")]
    NonFinite(usize),
    #[error("control points not sorted at index {0}")]
    NotSorted(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_points(pairs: &[(f64, f64)]) -> Vec<ControlPoint> {
        pairs.iter().map(|&(u, v)| ControlPoint { u, value: v }).collect()
    }

    #[test]
    fn two_point_linear() {
        let curve = LongitudinalCurve::new(
            make_points(&[(0.0, 0.0), (1.0, 1.0)]),
            EndpointConfig::default(),
        ).unwrap();

        // With only 2 points, should interpolate between them
        let v = curve.eval(0.5);
        assert!((v - 0.5).abs() < 0.01, "midpoint should be ~0.5, got {v}");
    }

    #[test]
    fn endpoints_return_boundary_values() {
        let curve = LongitudinalCurve::new(
            make_points(&[(0.0, 2.0), (0.5, 3.0), (1.0, 1.0)]),
            EndpointConfig::default(),
        ).unwrap();

        assert_eq!(curve.eval(0.0), 2.0);
        assert_eq!(curve.eval(1.0), 1.0);
        assert_eq!(curve.eval(-0.5), 2.0); // clamp below
        assert_eq!(curve.eval(1.5), 1.0);  // clamp above
    }

    #[test]
    fn smooth_through_control_points() {
        let points = make_points(&[
            (0.0, 0.0),
            (0.25, 0.5),
            (0.5, 1.0),
            (0.75, 0.8),
            (1.0, 0.0),
        ]);
        let curve = LongitudinalCurve::new(points, EndpointConfig::default()).unwrap();

        // Should pass through control points
        assert!((curve.eval(0.25) - 0.5).abs() < 1e-10, "should pass through p1");
        assert!((curve.eval(0.5) - 1.0).abs() < 1e-10, "should pass through p2");
        assert!((curve.eval(0.75) - 0.8).abs() < 1e-10, "should pass through p3");
    }

    #[test]
    fn stern_tangent_scale_affects_output() {
        let points = make_points(&[(0.0, 0.0), (0.5, 1.0), (1.0, 0.5)]);

        let default_curve = LongitudinalCurve::new(
            points.clone(),
            EndpointConfig::default(),
        ).unwrap();

        let scaled_curve = LongitudinalCurve::new(
            points,
            EndpointConfig {
                stern_tangent_scale: 2.0,
                ..EndpointConfig::default()
            },
        ).unwrap();

        // Values near the stern should differ
        let v_default = default_curve.eval(0.1);
        let v_scaled = scaled_curve.eval(0.1);
        assert!((v_default - v_scaled).abs() > 0.001,
            "tangent scale should affect near-stern values: {v_default} vs {v_scaled}");
    }

    #[test]
    fn bow_slope_cap_clamps() {
        let points = make_points(&[(0.0, 0.0), (0.9, 0.0), (1.0, 10.0)]);

        let uncapped = LongitudinalCurve::new(
            points.clone(),
            EndpointConfig {
                bow_slope_cap: 1000.0,
                ..EndpointConfig::default()
            },
        ).unwrap();

        let capped = LongitudinalCurve::new(
            points,
            EndpointConfig {
                bow_slope_cap: 1.0,
                ..EndpointConfig::default()
            },
        ).unwrap();

        let v_uncapped = uncapped.eval(0.95);
        let v_capped = capped.eval(0.95);
        // Capped should be closer to a linear interp (less aggressive approach)
        assert!(v_capped != v_uncapped, "cap should change behavior");
    }

    #[test]
    fn too_few_points_rejected() {
        let result = LongitudinalCurve::new(
            make_points(&[(0.0, 1.0)]),
            EndpointConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn unsorted_points_rejected() {
        let result = LongitudinalCurve::new(
            make_points(&[(0.5, 1.0), (0.2, 0.5), (0.8, 0.0)]),
            EndpointConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn nan_points_rejected() {
        let result = LongitudinalCurve::new(
            make_points(&[(0.0, 0.0), (f64::NAN, 1.0)]),
            EndpointConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn monotonic_for_monotonic_control_points() {
        // If all control point values are increasing, output should be mostly increasing
        let curve = LongitudinalCurve::new(
            make_points(&[(0.0, 0.0), (0.25, 0.3), (0.5, 0.6), (0.75, 0.8), (1.0, 1.0)]),
            EndpointConfig::default(),
        ).unwrap();

        let samples: Vec<f64> = (0..=100).map(|i| curve.eval(i as f64 / 100.0)).collect();
        let mut violations = 0;
        for w in samples.windows(2) {
            if w[1] < w[0] - 1e-10 {
                violations += 1;
            }
        }
        assert!(violations == 0,
            "monotonic control points should produce monotonic output, got {violations} violations");
    }

    #[test]
    fn nine_station_beam_envelope() {
        // Realistic beam envelope for a sloop
        let curve = LongitudinalCurve::new(
            make_points(&[
                (0.00, 0.10),
                (0.08, 0.48),
                (0.18, 0.82),
                (0.32, 0.96),
                (0.48, 1.00),
                (0.64, 0.92),
                (0.78, 0.72),
                (0.90, 0.38),
                (1.00, 0.02),
            ]),
            EndpointConfig::default(),
        ).unwrap();

        // Peak should be near midship
        let peak_u = (0..=100)
            .map(|i| {
                let u = i as f64 / 100.0;
                (u, curve.eval(u))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
            .0;

        assert!(peak_u > 0.3 && peak_u < 0.7,
            "beam peak should be near midship, got u={peak_u}");

        // Stern and bow should be narrow
        assert!(curve.eval(0.0) < 0.15, "stern should be narrow");
        assert!(curve.eval(1.0) < 0.10, "bow should be narrow");
    }
}
