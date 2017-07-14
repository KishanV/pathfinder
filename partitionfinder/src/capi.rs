// partitionfinder/capi.rs

use env_logger;
use euclid::{Point2D, Transform2D};
use legalizer::Legalizer;
use partitioner::Partitioner;
use tessellator::{QuadTessLevels, Tessellator};
use std::mem;
use std::slice;
use {AntialiasingMode, BQuad, EdgeInstance, Endpoint, Subpath, Vertex};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Point2DF32 {
    pub x: f32,
    pub y: f32,
}

impl Point2DF32 {
    #[inline]
    pub fn to_point2d(&self) -> Point2D<f32> {
        Point2D::new(self.x, self.y)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Matrix2DF32 {
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_new() -> *mut Legalizer {
    let mut legalizer = Box::new(Legalizer::new());
    let legalizer_ptr: *mut Legalizer = &mut *legalizer;
    mem::forget(legalizer);
    legalizer_ptr
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_destroy(legalizer: *mut Legalizer) {
    drop(mem::transmute::<*mut Legalizer, Box<Legalizer>>(legalizer))
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_endpoints(legalizer: *const Legalizer,
                                            out_endpoint_count: *mut u32)
                                            -> *const Endpoint {
    let endpoints = (*legalizer).endpoints();
    if !out_endpoint_count.is_null() {
        *out_endpoint_count = endpoints.len() as u32
    }
    endpoints.as_ptr()
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_control_points(legalizer: *const Legalizer,
                                                 out_control_points_count: *mut u32)
                                                 -> *const Point2DF32 {
    let control_points = (*legalizer).control_points();
    if !out_control_points_count.is_null() {
        *out_control_points_count = control_points.len() as u32
    }
    // FIXME(pcwalton): This is unsafe! `Point2D<f32>` and `Point2DF32` may have different layouts!
    control_points.as_ptr() as *const Point2DF32
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_subpaths(legalizer: *const Legalizer,
                                           out_subpaths_count: *mut u32)
                                           -> *const Subpath {
    let subpaths = (*legalizer).subpaths();
    if !out_subpaths_count.is_null() {
        *out_subpaths_count = subpaths.len() as u32
    }
    subpaths.as_ptr()
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_move_to(legalizer: *mut Legalizer,
                                          position: *const Point2DF32) {
    (*legalizer).move_to(&(*position).to_point2d())
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_close_path(legalizer: *mut Legalizer) {
    (*legalizer).close_path()
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_line_to(legalizer: *mut Legalizer,
                                          endpoint: *const Point2DF32) {
    (*legalizer).line_to(&(*endpoint).to_point2d())
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_quadratic_curve_to(legalizer: *mut Legalizer,
                                                     control_point: *const Point2DF32,
                                                     endpoint: *const Point2DF32) {
    (*legalizer).quadratic_curve_to(&(*control_point).to_point2d(), &(*endpoint).to_point2d())
}

#[no_mangle]
pub unsafe extern fn pf_legalizer_bezier_curve_to(legalizer: *mut Legalizer,
                                                  point1: *const Point2DF32,
                                                  point2: *const Point2DF32,
                                                  endpoint: *const Point2DF32) {
    (*legalizer).bezier_curve_to(&(*point1).to_point2d(),
                                 &(*point2).to_point2d(),
                                 &(*endpoint).to_point2d())
}

#[no_mangle]
pub unsafe extern fn pf_partitioner_new() -> *mut Partitioner<'static> {
    let mut partitioner = Box::new(Partitioner::new());
    let partitioner_ptr: *mut Partitioner<'static> = &mut *partitioner;
    mem::forget(partitioner);
    partitioner_ptr
}

#[no_mangle]
pub unsafe extern fn pf_partitioner_destroy<'a>(partitioner: *mut Partitioner<'a>) {
    drop(mem::transmute::<*mut Partitioner<'a>, Box<Partitioner>>(partitioner))
}

#[no_mangle]
pub unsafe extern fn pf_partitioner_init<'a>(partitioner: *mut Partitioner<'a>,
                                             endpoints: *const Endpoint,
                                             endpoint_count: u32,
                                             control_points: *const Point2DF32,
                                             control_point_count: u32,
                                             subpaths: *const Subpath,
                                             subpath_count: u32) {
    // FIXME(pcwalton): This is unsafe! `Point2D<f32>` and `Point2DF32` may have different layouts!
    (*partitioner).init(slice::from_raw_parts(endpoints, endpoint_count as usize),
                        slice::from_raw_parts(control_points as *const Point2D<f32>,
                                              control_point_count as usize),
                        slice::from_raw_parts(subpaths, subpath_count as usize))
}

#[no_mangle]
pub unsafe extern fn pf_partitioner_partition<'a>(partitioner: *mut Partitioner<'a>,
                                                  path_id: u32,
                                                  first_subpath_index: u32,
                                                  last_subpath_index: u32) {
    (*partitioner).partition(path_id, first_subpath_index, last_subpath_index)
}

#[no_mangle]
pub unsafe extern fn pf_partitioner_b_quads<'a>(partitioner: *mut Partitioner<'a>,
                                                out_b_quad_count: *mut u32)
                                                -> *const BQuad {
    let b_quads = (*partitioner).b_quads();
    if !out_b_quad_count.is_null() {
        *out_b_quad_count = b_quads.len() as u32
    }
    b_quads.as_ptr()
}

#[no_mangle]
pub unsafe extern fn pf_partitioner_b_vertices<'a>(partitioner: *mut Partitioner<'a>,
                                                   out_b_vertex_count: *mut u32)
                                                   -> *const Point2DF32 {
    // FIXME(pcwalton): This is unsafe! `Point2D<f32>` and `Point2DF32` may have different layouts!
    let b_vertices = (*partitioner).b_vertices();
    if !out_b_vertex_count.is_null() {
        *out_b_vertex_count = b_vertices.len() as u32
    }
    b_vertices.as_ptr() as *const Point2DF32
}

#[no_mangle]
pub unsafe extern fn pf_tessellator_new(antialiasing_mode: AntialiasingMode)
                                        -> *mut Tessellator<'static> {
    let mut tessellator = Box::new(Tessellator::new(antialiasing_mode));
    let tessellator_ptr: *mut Tessellator<'static> = &mut *tessellator;
    mem::forget(tessellator);
    tessellator_ptr
}

#[no_mangle]
pub unsafe extern fn pf_tessellator_destroy<'a>(tessellator: *mut Tessellator<'a>) {
    drop(mem::transmute::<*mut Tessellator<'a>, Box<Tessellator>>(tessellator))
}

#[no_mangle]
pub unsafe extern fn pf_tessellator_init<'a>(tessellator: *mut Tessellator<'a>,
                                             b_quads: *const BQuad,
                                             b_quad_count: u32,
                                             b_vertices: *const Point2DF32,
                                             b_vertex_count: u32) {
    // FIXME(pcwalton): This is unsafe! `Point2D<f32>` and `Point2DF32` may have different layouts!
    (*tessellator).init(slice::from_raw_parts(b_quads, b_quad_count as usize),
                        slice::from_raw_parts(b_vertices as *const Point2D<f32>,
                                              b_vertex_count as usize))
}

#[no_mangle]
pub unsafe extern fn pf_tessellator_compute_hull<'a>(tessellator: *mut Tessellator<'a>,
                                                     transform: *const Matrix2DF32) {
    (*tessellator).compute_hull(&Transform2D::column_major((*transform).m00,
                                                           (*transform).m01,
                                                           (*transform).m02,
                                                           (*transform).m10,
                                                           (*transform).m11,
                                                           (*transform).m12))
}

#[no_mangle]
pub unsafe extern fn pf_tessellator_compute_domain<'a>(tessellator: *mut Tessellator<'a>) {
    (*tessellator).compute_domain()
}

#[no_mangle]
pub unsafe extern fn pf_tessellator_tess_levels<'a>(tessellator: *mut Tessellator<'a>,
                                                    out_tess_levels_count: *mut u32)
                                                    -> *const QuadTessLevels {
    let tess_levels = (*tessellator).tess_levels();
    if !out_tess_levels_count.is_null() {
        *out_tess_levels_count = tess_levels.len() as u32
    }
    tess_levels.as_ptr()
}

#[no_mangle]
pub unsafe extern fn pf_tessellator_vertices<'a>(tessellator: *mut Tessellator<'a>,
                                                 out_vertex_count: *mut u32)
                                                 -> *const Vertex {
    let vertices = (*tessellator).vertices();
    if !out_vertex_count.is_null() {
        *out_vertex_count = vertices.len() as u32
    }
    vertices.as_ptr()
}

#[no_mangle]
pub unsafe extern fn pf_tessellator_msaa_indices<'a>(tessellator: *mut Tessellator<'a>,
                                                     out_msaa_index_count: *mut u32)
                                                     -> *const u32 {
    let msaa_indices = (*tessellator).msaa_indices();
    if !out_msaa_index_count.is_null() {
        *out_msaa_index_count = msaa_indices.len() as u32
    }
    msaa_indices.as_ptr()
}

#[no_mangle]
pub unsafe extern fn pf_tessellator_edge_instances<'a>(tessellator: *mut Tessellator<'a>,
                                                       out_edge_instance_count: *mut u32)
                                                       -> *const EdgeInstance {
    let edge_instances = (*tessellator).edge_instances();
    if !out_edge_instance_count.is_null() {
        *out_edge_instance_count = edge_instances.len() as u32
    }
    edge_instances.as_ptr()
}

#[no_mangle]
pub unsafe extern fn pf_init_env_logger() -> u32 {
    env_logger::init().is_ok() as u32
}
