use approx::AbsDiffEq;
use parry3d_f64::query::{Ray, RayCast as _};

use crate::Vector;

use super::{Point, Scalar};

/// # A triangle
///
/// The dimensionality of the triangle is defined by the const generic `D`
/// parameter.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(C)]
pub struct Triangle<const D: usize> {
    /// # The points that make up the triangle
    pub points: [Point<D>; 3],
}

impl<const D: usize> Triangle<D> {
    /// # Construct a triangle from three points
    pub fn from_points(points: [impl Into<Point<D>>; 3]) -> Self {
        let points = points.map(Into::into);
        Self { points }
    }

    /// # Determine whether the triangle is valid
    ///
    /// A triangle is valid, if it is not degenerate. In a degenerate triangle,
    /// the three points do not form an actual triangle, but are collapsed into
    /// a line or even a single point.
    ///
    /// ## Implementation Note
    ///
    /// Right now, this function computes the area of the triangle, and compares
    /// it against [`Scalar`]'s default epsilon value. This might not be
    /// flexible enough for all use cases.
    ///
    /// Long-term, it might become necessary to add some way to override the
    /// epsilon value used within this function.
    pub fn is_valid(&self) -> bool {
        let [a, b, c] = self.points;
        let area = (b - a).outer(&(c - a)).magnitude();
        area > Scalar::default_epsilon()
    }

    /// # Compute the center point of the triangle
    pub fn center(&self) -> Point<D> {
        let [a, b, c] = self.points;
        let coords = (a.coords + b.coords + c.coords) / 3.;
        Point { coords }
    }

    /// # Convert a set of barycentric coordinates on the triangle into a point
    pub fn point_from_barycentric_coords(
        &self,
        [wa, wb, wc]: [Scalar; 3],
    ) -> Point<D> {
        let [a, b, c] = self.points;
        let coords = a.coords * wa + b.coords * wb + c.coords * wc;
        Point { coords }
    }

    /// # Project a point into the triangle
    ///
    /// Returns barycentric coordinates of the projected point.
    ///
    /// ## Panics
    ///
    /// Panics, if the triangle is degenerate.
    pub fn point_to_barycentric_coords(
        &self,
        point: impl Into<Point<D>>,
    ) -> [Scalar; 3] {
        // Algorithm from Real-Time Collision Detection by Christer Ericson,
        // pages 47-48.

        let p = point.into();
        let [a, b, c] = self.points;

        let v0 = b - a;
        let v1 = c - a;
        let v2 = p - a;

        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);
        let d20 = v2.dot(&v0);
        let d21 = v2.dot(&v1);

        let denom = d00 * d11 - d01 * d01;

        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = Scalar::ONE - v - w;

        [u, v, w]
    }

    /// # Normalize the triangle
    ///
    /// Returns a new `Triangle` instance with the same points, but the points
    /// ordered such that they are ordered according to their `Ord`/`PartialOrd`
    /// implementation.
    ///
    /// This is useful for comparing triangles when the order of points is not
    /// important.
    pub fn normalize(mut self) -> Self {
        self.points.sort();
        self
    }
}

impl Triangle<2> {
    /// # Compute the winding of the triangle
    ///
    /// A winding can only be computed, if the triangle is valid. Returns
    /// `None`, if it isn't.
    pub fn winding(&self) -> Option<Winding> {
        let [pa, pb, pc] = self.points.map(|point| robust::Coord {
            x: point.u,
            y: point.v,
        });
        let orient2d = robust::orient2d(pa, pb, pc);

        if orient2d < 0. {
            return Some(Winding::Cw);
        }
        if orient2d > 0. {
            return Some(Winding::Ccw);
        }

        None
    }
}

impl Triangle<3> {
    /// # Convert the triangle to a Parry triangle
    pub fn to_parry(self) -> parry3d_f64::shape::Triangle {
        self.points.map(|vertex| vertex.to_na()).into()
    }

    /// # Cast a ray against the Triangle
    pub fn cast_local_ray(
        &self,
        origin: Point<3>,
        dir: Vector<3>,
        max_toi: f64,
        solid: bool,
    ) -> Option<Scalar> {
        let ray = Ray {
            origin: origin.to_na(),
            dir: dir.to_na(),
        };

        self.to_parry()
            .cast_local_ray(&ray, max_toi, solid)
            .map(Into::into)
    }

    /// # Compute the triangle's normal
    pub fn normal(&self) -> Vector<3> {
        self.to_parry()
            .normal()
            .expect("triangle is valid (validated on construction)")
            .into_inner()
            .into()
    }
}

impl<P, const D: usize> From<[P; 3]> for Triangle<D>
where
    P: Into<Point<D>>,
{
    fn from(points: [P; 3]) -> Self {
        Self::from_points(points)
    }
}

/// # Winding direction of a triangle.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Winding {
    /// # Counter-clockwise winding
    Ccw,

    /// # Clockwise winding
    Cw,
}

impl Winding {
    /// # Indicate whether the winding is counter-clockwise
    pub fn is_ccw(&self) -> bool {
        matches!(self, Self::Ccw)
    }

    /// # Indicate whether the winding is clockwise
    pub fn is_cw(&self) -> bool {
        matches!(self, Self::Cw)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Point, Vector};

    use super::Triangle;

    #[test]
    fn valid_triangle_2d() {
        let a = Point::from([0.0, 0.0]);
        let b = Point::from([1.0, 0.0]);
        let c = Point::from([0.0, 1.0]);

        assert!(Triangle::from_points([a, b, c]).is_valid());
    }

    #[test]
    fn valid_triangle_3d() {
        let a = Point::from([0.0, 0.0, 0.0]);
        let b = Point::from([0.0, 1.0, 0.0]);
        let c = Point::from([1.0, 0.0, 0.0]);

        assert!(Triangle::from_points([a, b, c]).is_valid());
    }

    #[test]
    fn invalid_triangle_2d() {
        let a = Point::from([0.0, 0.0]);
        let b = Point::from([1.0, 0.0]);
        let c = Point::from([2.0, 0.0]);

        assert!(!Triangle::from_points([a, b, c]).is_valid());
    }

    #[test]
    fn invalid_triangle_3d() {
        let a = Point::from([0.0, 0.0, 0.0]);
        let b = Point::from([1.0, 0.0, 0.0]);
        let c = Point::from([2.0, 0.0, 0.0]);

        assert!(!Triangle::from_points([a, b, c]).is_valid());
    }

    #[test]
    fn normal() {
        let triangle =
            Triangle::from([[0.0, 0.0, 0.0], [2.0, 1.0, 0.0], [2.0, 0.0, 0.0]]);
        assert_eq!(triangle.normal(), Vector::from([0.0, 0.0, -1.0]));
    }
}
