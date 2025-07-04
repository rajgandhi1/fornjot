use fj_interop::Tolerance;
use fj_math::{Aabb, Point};

use crate::{
    extra::triangulate::{TriangulationPoint, delaunay::triangles},
    topology::surface::Surface,
};

#[derive(Debug)]
pub struct SurfaceMesh {
    pub points: Vec<TriangulationPoint>,
    pub triangles: Vec<MeshTriangle>,
}

impl SurfaceMesh {
    pub fn from_surface(
        surface: &Surface,
        boundary: &Aabb<2>,
        _: impl Into<Tolerance>,
    ) -> Self {
        let surface_points = surface
            .geometry
            .approximate(boundary)
            .into_iter()
            .map(|point_surface| {
                TriangulationPoint::from_surface_point(
                    point_surface,
                    surface.geometry.as_ref(),
                )
            })
            .collect::<Vec<_>>();

        let boundary_points = {
            let [[min_u, min_v], [max_u, max_v]] = [boundary.min, boundary.max]
                .map(|point| point.coords.components);

            [
                [min_u, min_v],
                [min_u, max_v],
                [max_u, min_v],
                [max_u, max_v],
            ]
            .map(Point::from)
            .map(|point_surface| {
                TriangulationPoint::from_surface_point(
                    point_surface,
                    surface.geometry.as_ref(),
                )
            })
        };

        let mut all_points = surface_points.clone();
        all_points.extend(boundary_points);

        let triangles = triangles([], all_points)
            .into_iter()
            .map(|triangle| MeshTriangle { points: triangle })
            .collect();

        Self {
            points: surface_points,
            triangles,
        }
    }
}

#[derive(Debug)]
pub struct MeshTriangle {
    pub points: [TriangulationPoint; 3],
}
