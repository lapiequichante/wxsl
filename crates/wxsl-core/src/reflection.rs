//! Device-free planar reflection geometry.

/// An infinite mirror plane, `dot(normal, position) = distance`.
///
/// Stored normalized, so distances are in world units. Coordinates use
/// the frame's origin; rebased applications translate the plane too.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanarReflection {
    normal: [f32; 3],
    distance: f32,
}

impl PlanarReflection {
    /// Normalize a plane's coefficients. Reject non-finite or zero normals.
    pub fn new(normal: [f32; 3], distance: f32) -> Option<Self> {
        let length = normal.iter().map(|n| n * n).sum::<f32>().sqrt();
        if !distance.is_finite() || !length.is_finite() || length < 1e-6 {
            return None;
        }
        let plane = Self {
            normal: normal.map(|n| n / length),
            distance: distance / length,
        };
        plane.distance.is_finite().then_some(plane)
    }

    /// The unit normal, pointing towards the scene that is reflected.
    pub fn normal(self) -> [f32; 3] {
        self.normal
    }

    /// Signed distance from the origin along the unit normal.
    pub fn distance(self) -> f32 {
        self.distance
    }

    /// The plane equation `(nx, ny, nz, -distance)`.
    pub fn equation(self) -> [f32; 4] {
        [
            self.normal[0],
            self.normal[1],
            self.normal[2],
            -self.distance,
        ]
    }

    /// Reflect a point across the plane.
    pub fn reflect_point(self, point: [f32; 3]) -> [f32; 3] {
        let distance = point
            .iter()
            .zip(self.normal)
            .map(|(p, n)| p * n)
            .sum::<f32>()
            - self.distance;
        std::array::from_fn(|i| point[i] - 2.0 * distance * self.normal[i])
    }

    /// Column-major homogeneous reflection matrix, without a GPU dependency.
    pub fn matrix(self) -> [[f32; 4]; 4] {
        let mut matrix = [[0.0; 4]; 4];
        for (column, values) in matrix.iter_mut().enumerate().take(3) {
            for (row, value) in values.iter_mut().enumerate().take(3) {
                *value = if row == column { 1.0 } else { 0.0 }
                    - 2.0 * self.normal[row] * self.normal[column];
            }
        }
        matrix[3] = [
            2.0 * self.distance * self.normal[0],
            2.0 * self.distance * self.normal[1],
            2.0 * self.distance * self.normal[2],
            1.0,
        ];
        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reflection_fixes_the_plane_and_is_an_involution() {
        let plane = PlanarReflection::new([0.0, 2.0, 0.0], 4.0).unwrap();
        assert_eq!(plane.reflect_point([3.0, 2.0, -1.0]), [3.0, 2.0, -1.0]);
        assert_eq!(plane.reflect_point([3.0, 5.0, -1.0]), [3.0, -1.0, -1.0]);
        let tilted = PlanarReflection::new([1.0, 2.0, 3.0], -2.0).unwrap();
        let point = [4.0, 5.0, 6.0];
        let twice = tilted.reflect_point(tilted.reflect_point(point));
        assert!(twice.iter().zip(point).all(|(a, b)| (a - b).abs() < 1e-5));
        let matrix = tilted.matrix();
        let reflected = tilted.reflect_point(point);
        for row in 0..3 {
            let transformed =
                (0..3).map(|col| matrix[col][row] * point[col]).sum::<f32>() + matrix[3][row];
            assert!((transformed - reflected[row]).abs() < 1e-5);
        }
    }
    #[test]
    fn invalid_planes_are_refused() {
        for (normal, distance) in [
            ([0.0; 3], 0.0),
            ([f32::NAN, 0.0, 0.0], 0.0),
            ([0.0, 1.0, 0.0], f32::INFINITY),
        ] {
            assert!(PlanarReflection::new(normal, distance).is_none());
        }
    }
}
