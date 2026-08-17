use nalgebra::{Matrix3, Vector3};

#[derive(Default, Clone)]
pub struct QEFData {
    /// Accumulates the A^T * A matrix (3x3)
    ata: Matrix3<f32>,
    /// Accumulates the A^T * b vector (3x1)
    atb: Vector3<f32>,
    /// Accumulates the raw intersection positions for our fallback
    mass_point: Vector3<f32>,
    /// How many edges intersected the surface in this cell
    pub intersection_count: f32,
}

impl QEFData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Call this for every edge of the cell that intersects the surface
    pub fn add_intersection(&mut self, intersection_point: [f32; 3], normal: [f32; 3]) {
        let p = Vector3::new(
            intersection_point[0],
            intersection_point[1],
            intersection_point[2],
        );
        let n = Vector3::new(normal[0], normal[1], normal[2]);

        // Accumulate A^T * A (outer product of the normal)
        self.ata += n * n.transpose();

        // Accumulate A^T * b (normal scaled by its dot product with the point)
        self.atb += n * n.dot(&p);

        // Keep track of the center of mass as a fallback
        self.mass_point += p;
        self.intersection_count += 1.0;
    }

    /// Solves the QEF and returns the optimal vertex position for this cell
    pub fn solve(&self) -> [f32; 3] {
        if self.intersection_count == 0.0 {
            return [0.0, 0.0, 0.0];
        }

        let mass_center = self.mass_point / self.intersection_count;

        // Perform Singular Value Decomposition on the 3x3 matrix
        let svd = self.ata.svd(true, true);

        // Calculate the pseudo-inverse.
        // The epsilon (1e-5) safely ignores near-zero singular values (flat walls).
        let pseudo_inv = svd
            .pseudo_inverse(1e-5)
            .unwrap_or_else(|_| Matrix3::zeros());

        //  Solve for a correction around the mass center.
        // This keeps under-constrained axes near the cell's edge intersections instead of
        // collapsing them toward zero on flat faces.
        let correction = pseudo_inv * (self.atb - self.ata * mass_center);
        let optimal_vertex = mass_center + correction;

        // Safety Check
        // If the math blows up, fall back to the simple average of the intersection
        // points (Surface Nets style).
        if !optimal_vertex.x.is_finite()
            || !optimal_vertex.y.is_finite()
            || !optimal_vertex.z.is_finite()
        {
            return [mass_center.x, mass_center.y, mass_center.z];
        }

        [optimal_vertex.x, optimal_vertex.y, optimal_vertex.z]
    }
}
