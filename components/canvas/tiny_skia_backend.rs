/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::*;
use cssparser::RGBA;
use euclid::Angle;
use euclid::default::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use lyon_geom::Arc;
use tiny_skia::{Paint, Pixmap, PixmapRef, PixmapPaint, Mask};

use crate::canvas_data::{self};
use crate::{canvas_data::{Backend, DrawOptions, CompositionOp, CanvasPaintState, GenericDrawTarget, Color, GradientStop, Path, GenericPathBuilder, GradientStops, Filter, StrokeOptions}, canvas_paint_thread::AntialiasMode};

pub struct TinySkiaBackend;

impl Backend for TinySkiaBackend {
    fn get_composition_op(&self, opts: &DrawOptions) -> CompositionOp {
        CompositionOp::TinySkia(opts.as_tiny_skia().blend_mode)
    }

    fn need_to_draw_shadow(&self, color: &Color) -> bool {
        color.as_tiny_skia().alpha() != 0.0
    }

    fn set_shadow_color<'a>(&mut self, color: RGBA, state: &mut CanvasPaintState<'a>) {
        state.shadow_color = Color::TinySkia(tiny_skia::Color::from_rgba8(
            color.red,
            color.green,
            color.blue,
            color.alpha,
        ).premultiply());
    }

    #[allow(unsafe_code)]
    fn set_fill_style<'a>(
        &mut self,
        style: canvas_traits::canvas::FillOrStrokeStyle,
        state: &mut CanvasPaintState<'a>,
        drawtarget: &dyn GenericDrawTarget,
    ) {
        state.fill_style = canvas_data::Pattern::TinySkia(match style {
            FillOrStrokeStyle::Color(color) => tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(
                color.red,
                color.green,
                color.blue,
                color.alpha,
            )),
            FillOrStrokeStyle::LinearGradient(linear) => {
                tiny_skia::LinearGradient::new(
                    tiny_skia::Point { x: linear.x0 as f32, y: linear.y0 as f32 },
                    tiny_skia::Point { x: linear.x1 as f32, y: linear.y1 as f32 },
                    linear.stops.into_iter().map(|stop| tiny_skia::GradientStop::new(stop.offset as f32, tiny_skia::Color::from_rgba8(
                        stop.color.red,
                        stop.color.green,
                        stop.color.blue,
                        stop.color.alpha,
                    ))).collect::<Vec<tiny_skia::GradientStop>>(),
                    tiny_skia::SpreadMode::Pad,
                    drawtarget.get_transform().to_tiny_skia(),
                ).unwrap()
            },
            FillOrStrokeStyle::RadialGradient(radial) => tiny_skia::RadialGradient::new(
                tiny_skia::Point { x: radial.x0 as f32, y: radial.y0 as f32 },
                tiny_skia::Point { x: radial.x1 as f32, y: radial.y1 as f32 },
                // TODO - tiny_skia will need support for 2 radii, especially if resvg wants to support SVG 2
                radial.r1 as f32,
                radial.stops.into_iter().map(|stop| tiny_skia::GradientStop::new(stop.offset as f32, tiny_skia::Color::from_rgba8(
                    stop.color.red,
                    stop.color.green,
                    stop.color.blue,
                    stop.color.alpha,
                ))).collect::<Vec<tiny_skia::GradientStop>>(),
                tiny_skia::SpreadMode::Pad,
                drawtarget.get_transform().to_tiny_skia(),
            ).unwrap(),
            FillOrStrokeStyle::Surface(surface) => {
                tiny_skia::Pattern::new(
                    PixmapRef::from_bytes(
                        unsafe {
                            std::slice::from_raw_parts(surface.surface_data.as_ptr() as *const u8, surface.surface_data.len())
                        },
                        surface.surface_size.width,
                        surface.surface_size.height,
                    ).unwrap(),
                    tiny_skia::SpreadMode::Pad,
                    tiny_skia::FilterQuality::Bilinear,
                    1.0,
                    drawtarget.get_transform().to_tiny_skia()
                )
            },
        });
    }

    #[allow(unsafe_code)]
    fn set_stroke_style<'a>(
        &mut self,
        style: canvas_traits::canvas::FillOrStrokeStyle,
        state: &mut CanvasPaintState<'a>,
        drawtarget: &dyn GenericDrawTarget,
    ) {
        state.stroke_style = canvas_data::Pattern::TinySkia(match style {
            FillOrStrokeStyle::Color(color) => tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(
                color.red,
                color.green,
                color.blue,
                color.alpha,
            )),
            FillOrStrokeStyle::LinearGradient(linear) => {
                tiny_skia::LinearGradient::new(
                    tiny_skia::Point { x: linear.x0 as f32, y: linear.y0 as f32 },
                    tiny_skia::Point { x: linear.x1 as f32, y: linear.y1 as f32 },
                    linear.stops.into_iter().map(|stop| tiny_skia::GradientStop::new(stop.offset as f32, tiny_skia::Color::from_rgba8(
                        stop.color.red,
                        stop.color.green,
                        stop.color.blue,
                        stop.color.alpha
                    ))).collect::<Vec<tiny_skia::GradientStop>>(),
                    tiny_skia::SpreadMode::Pad,
                    drawtarget.get_transform().to_tiny_skia(),
                ).unwrap()
            },
            FillOrStrokeStyle::RadialGradient(radial) => tiny_skia::RadialGradient::new(
                tiny_skia::Point { x: radial.x0 as f32, y: radial.y0 as f32 },
                tiny_skia::Point { x: radial.x1 as f32, y: radial.y1 as f32 },
                // TODO - Fix this
                radial.r1 as f32,
                radial.stops.into_iter().map(|stop| tiny_skia::GradientStop::new(stop.offset as f32, tiny_skia::Color::from_rgba8(
                    stop.color.red,
                    stop.color.green,
                    stop.color.blue,
                    stop.color.alpha
                ))).collect::<Vec<tiny_skia::GradientStop>>(),
                tiny_skia::SpreadMode::Pad,
                drawtarget.get_transform().to_tiny_skia(),
            ).unwrap(),
            FillOrStrokeStyle::Surface(surface) => {
                tiny_skia::Pattern::new(
                    PixmapRef::from_bytes(
                        unsafe {
                            std::slice::from_raw_parts(surface.surface_data.as_ptr() as *const u8, surface.surface_data.len())
                        },
                        surface.surface_size.width,
                        surface.surface_size.height,
                    ).unwrap(),
                    tiny_skia::SpreadMode::Pad,
                    tiny_skia::FilterQuality::Bilinear,
                    1.0,
                    drawtarget.get_transform().to_tiny_skia(),
                )
            },
        });
    }

    fn set_global_composition<'a>(
        &mut self,
        op: canvas_traits::canvas::CompositionOrBlending,
        state: &mut CanvasPaintState<'a>,
    ) {
        state.draw_options.as_tiny_skia_mut().blend_mode = match op {
            canvas_traits::canvas::CompositionOrBlending::Composition(composition) => match composition {
                CompositionStyle::SrcIn => tiny_skia::BlendMode::SourceIn,
                CompositionStyle::SrcOut => tiny_skia::BlendMode::SourceOut,
                CompositionStyle::SrcOver => tiny_skia::BlendMode::SourceOver,
                CompositionStyle::SrcAtop => tiny_skia::BlendMode::SourceAtop,
                CompositionStyle::DestIn => tiny_skia::BlendMode::DestinationIn,
                CompositionStyle::DestOut => tiny_skia::BlendMode::DestinationOut,
                CompositionStyle::DestOver => tiny_skia::BlendMode::DestinationOver,
                CompositionStyle::DestAtop => tiny_skia::BlendMode::DestinationAtop,
                CompositionStyle::Copy => tiny_skia::BlendMode::Source,
                CompositionStyle::Lighter => tiny_skia::BlendMode::Plus,
                CompositionStyle::Xor => tiny_skia::BlendMode::Xor,
                CompositionStyle::Clear => tiny_skia::BlendMode::Clear,
            },
            canvas_traits::canvas::CompositionOrBlending::Blending(blending) => match blending {
                BlendingStyle::Multiply => tiny_skia::BlendMode::Multiply,
                BlendingStyle::Screen => tiny_skia::BlendMode::Screen,
                BlendingStyle::Overlay => tiny_skia::BlendMode::Overlay,
                BlendingStyle::Darken => tiny_skia::BlendMode::Darken,
                BlendingStyle::Lighten => tiny_skia::BlendMode::Lighten,
                BlendingStyle::ColorDodge => tiny_skia::BlendMode::ColorDodge,
                BlendingStyle::ColorBurn => tiny_skia::BlendMode::ColorBurn,
                BlendingStyle::HardLight => tiny_skia::BlendMode::HardLight,
                BlendingStyle::SoftLight => tiny_skia::BlendMode::SoftLight,
                BlendingStyle::Difference => tiny_skia::BlendMode::Difference,
                BlendingStyle::Exclusion => tiny_skia::BlendMode::Exclusion,
                BlendingStyle::Hue => tiny_skia::BlendMode::Hue,
                BlendingStyle::Saturation => tiny_skia::BlendMode::Saturation,
                BlendingStyle::Color => tiny_skia::BlendMode::Color,
                BlendingStyle::Luminosity => tiny_skia::BlendMode::Luminosity,
            },
        } 
    }

    fn create_drawtarget(&self, size: Size2D<u64>) -> Box<dyn GenericDrawTarget> {
        Box::new(PixmapTarget {
            pixmap: tiny_skia::Pixmap::new(
                size.width as u32,
                size.height as u32,
            ).unwrap(),
            transform: tiny_skia::Transform::identity(),
            opacity: 1.0,
            mask: None,
            mask_paths: vec![],
        })
    }

    fn recreate_paint_state<'a>(&self, state: &CanvasPaintState<'a>) -> CanvasPaintState<'a> {
        CanvasPaintState::new(AntialiasMode::Default)
    }
}

impl<'a> CanvasPaintState<'a> {
    pub fn new(_antialias: AntialiasMode) -> CanvasPaintState<'a> {
        let pattern = tiny_skia::Shader::SolidColor(tiny_skia::Color::BLACK);
        CanvasPaintState {
            draw_options: DrawOptions::TinySkia(tiny_skia::Paint::default()),
            fill_style: canvas_data::Pattern::TinySkia(pattern.clone()),
            stroke_style: canvas_data::Pattern::TinySkia(pattern),
            stroke_opts: StrokeOptions::TinySkia(tiny_skia::Stroke::default()),
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: Color::TinySkia(tiny_skia::Color::TRANSPARENT.premultiply()),
            font_style: None,
            text_align: TextAlign::default(),
            text_baseline: TextBaseline::default(),
        }
    }
}

impl GradientStop {
    fn as_tiny_skia(&self) -> &tiny_skia::GradientStop {
        match self {
            GradientStop::TinySkia(s) => s,
            _ => todo!(),
        }
    }
}

impl GradientStops {
    fn as_tiny_skia(&self) -> &Vec<tiny_skia::GradientStop> {
        match self {
            GradientStops::TinySkia(s) => s,
            _ => todo!(),
        }
    }
}

impl Color {
    fn as_tiny_skia(&self) -> &tiny_skia::PremultipliedColor {
        match self {
            Color::TinySkia(c) => c,
            _ => todo!(),
        }
    }
}

impl Path {
    pub fn transformed_copy_to_builder(
        &self,
        transform: &Transform2D<f32>,
    ) -> Box<dyn GenericPathBuilder> {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.push_path(&self.as_tiny_skia().clone().transform(transform.to_tiny_skia()).unwrap());
        Box::new(PathBuilder(Some(pb)))
    }

    pub fn contains_point(&self, x: f64, y: f64, path_transform: &Transform2D<f32>) -> bool {
        self.as_tiny_skia()
            .clone()
            .transform(path_transform.to_tiny_skia())
            .unwrap()
            .points()
            .contains(&tiny_skia::Point::from_xy(x as f32, y as f32))
    }

    pub fn copy_to_builder(&self) -> Box<dyn GenericPathBuilder> {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.push_path(&self.as_tiny_skia().clone());
        Box::new(PathBuilder(Some(pb)))
    }

    fn as_tiny_skia(&self) -> &tiny_skia::Path {
        match self {
            Path::TinySkia(p) => p,
            _ => todo!(),
        }
    }
}

impl<'a> canvas_data::Pattern<'a> {
    pub fn is_zero_size_gradient(&self) -> bool {
        match self {
            // TODO - tiny_skia immediately converts any zero-sized gradients into anything but a gradient
            // Investigate how a non-gradient pattern can be made distinct from a zero-sized one
            canvas_data::Pattern::TinySkia(pattern) => match pattern {
                tiny_skia::Shader::SolidColor(_) => false,
                tiny_skia::Shader::LinearGradient(_) => false,
                tiny_skia::Shader::RadialGradient(_) => false,
                tiny_skia::Shader::Pattern(_) => false,
            },
            _ => todo!(),
        }
    }

    fn as_tiny_skia(&self) -> &tiny_skia::Shader<'a> {
        match self {
            canvas_data::Pattern::TinySkia(s) => s,
            _ => todo!(),
        }
    }
}

impl StrokeOptions {
    pub fn set_line_width(&mut self, _val: f32) {
        match self {
            StrokeOptions::TinySkia(options) => options.width = _val,
            _ => todo!(),
        }
    }

    pub fn set_miter_limit(&mut self, _val: f32) {
        match self {
            StrokeOptions::TinySkia(options) => options.miter_limit = _val,
            _ => todo!(),
        }
    }

    pub fn set_line_join(&mut self, val: LineJoinStyle) {
        match self {
            StrokeOptions::TinySkia(options) => options.line_join = match val {
                LineJoinStyle::Round => tiny_skia::LineJoin::Round,
                LineJoinStyle::Bevel => tiny_skia::LineJoin::Bevel,
                LineJoinStyle::Miter => tiny_skia::LineJoin::Miter,
            },
            _ => todo!(),
        }
    }

    pub fn set_line_cap(&mut self, val: LineCapStyle) {
        match self {
            StrokeOptions::TinySkia(options) => options.line_cap = match val {
                LineCapStyle::Butt => tiny_skia::LineCap::Butt,
                LineCapStyle::Round => tiny_skia::LineCap::Round,
                LineCapStyle::Square => tiny_skia::LineCap::Square,
            },
            _ => todo!(),
        }
    }

    pub fn as_tiny_skia(&self) -> &tiny_skia::Stroke {
        match self {
            StrokeOptions::TinySkia(options) => options,
            _ => todo!(),
        }
    }
}

impl<'a> DrawOptions<'a> {
    pub fn set_alpha(&mut self, val: f32) {
        /*
        match self {
            DrawOptions::TinySkia(draw_options) => draw_options.shader.apply_opacity(val),
            _ => todo!(),
        }
        */
    }

    fn as_tiny_skia(&self) -> &tiny_skia::Paint<'a> {
        match self {
            DrawOptions::TinySkia(paint) => paint,
            _ => todo!(),
        }
    }

    fn as_tiny_skia_mut(&mut self) -> &mut tiny_skia::Paint<'a> {
        match self {
            DrawOptions::TinySkia(paint) => paint,
            _ => todo!(),
        }
    }
}

#[derive(Clone)]
pub struct PixmapTarget {
    pixmap: Pixmap,
    transform: tiny_skia::Transform,
    opacity: f32,
    mask: Option<Mask>,
    mask_paths: Vec<tiny_skia::Path>,
}

impl GenericDrawTarget for PixmapTarget {
    fn clear_rect(&mut self, rect: &Rect<f32>) {
        let mut paint = tiny_skia::Paint::default();
        paint.blend_mode = tiny_skia::BlendMode::Clear;

        self.pixmap.fill_rect(
            tiny_skia::Rect::from_xywh(
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            ).unwrap(),
            &paint,
            self.transform,
            self.mask.as_ref()
        );
    }

    fn copy_surface(
        &mut self,
        surface: &[u8],
        source: Rect<i32>,
        destination: lyon_geom::Point<i32>,
    ) {
        let pixmap = PixmapRef::from_bytes(surface, source.width() as u32, source.height() as u32).unwrap();
        let mut paint = PixmapPaint::default();
        paint.blend_mode = tiny_skia::BlendMode::Source;

        self.pixmap.draw_pixmap(destination.x, destination.y, pixmap, &paint, tiny_skia::Transform::default(), None);
    }

    fn create_gradient_stops(
        &self,
        gradient_stops: Vec<GradientStop>,
    ) -> GradientStops {
        let stops = gradient_stops
            .into_iter()
            .map(|item| item.as_tiny_skia().clone())
            .collect::<Vec<tiny_skia::GradientStop>>();
        // https://www.w3.org/html/test/results/2dcontext/annotated-spec/canvas.html#testrefs.2d.gradient.interpolate.overlap
        //stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
        GradientStops::TinySkia(stops)
    }

    fn create_path_builder(&self) -> Box<dyn GenericPathBuilder> {
        Box::new(PathBuilder::new())
    }

    fn create_similar_draw_target(
        &self,
        size: &Size2D<i32>,
    ) -> Box<dyn GenericDrawTarget> {
        Box::new(PixmapTarget {
            pixmap: tiny_skia::Pixmap::new(size.width as u32, size.height as u32).unwrap(),
            transform: self.transform,
            opacity: self.opacity,
            mask: self.mask.clone(),
            mask_paths: self.mask_paths.clone(),
        })
    }

    fn draw_surface(
            &mut self,
            surface: &[u8],
            dest: Rect<f64>,
            source: Rect<f64>,
            filter: Filter,
            draw_options: &DrawOptions,
        ) {
            let image = PixmapRef::from_bytes(surface, source.width() as u32, source.height() as u32).unwrap();
            let transform = tiny_skia::Transform::from_row(
                dest.width() as f32 / image.width() as f32,
                0.0,
                0.0,
                dest.height() as f32 / image.height() as f32,
                dest.min_x() as f32,
                dest.min_y() as f32
            );

            let drop = draw_options.as_tiny_skia();
            
            /*
            let mut test = tiny_skia::Shader::SolidColor(tiny_skia::Color::WHITE);
            test.apply_opacity(0.2);

            self.pixmap.fill_rect(
                tiny_skia::Rect::from_xywh(
                    dest.origin.x as f32,
                    dest.origin.y as f32,
                    dest.width() as f32,
                    dest.height() as f32
                ).unwrap(),
                &tiny_skia::Paint {
                    shader: test,
                    blend_mode: drop.blend_mode,
                    anti_alias: false,
                    force_hq_pipeline: false,
                },
                self.transform,
                self.mask.as_ref(),
            );
            */
            //self.pixmap.draw_pixmap(dest.max_x() as i32, dest.max_y() as i32, image, &paint, transform.pre_concat(self.transform), None);
            self.pixmap.fill_rect(
                tiny_skia::Rect::from_xywh(
                    dest.min_x() as f32,
                    dest.min_y() as f32,
                    dest.width() as f32,
                    dest.height() as f32
                ).unwrap(),
                &tiny_skia::Paint {
                    shader: tiny_skia::Pattern::new(
                        image,
                        tiny_skia::SpreadMode::Pad,
                        filter.as_tiny_skia(),
                        self.opacity,
                        transform,
                    ),
                    blend_mode: drop.blend_mode,
                    anti_alias: false,
                    force_hq_pipeline: false,
                },
                self.transform,
                self.mask.as_ref(),
            );
            //self.fill_rect(tiny_skia::Rect::from_xywh(dest.origin.x as f32, dest.origin.y as f32, dest.width() as f32, dest.height() as f32).unwrap(), &tiny_skia::Paint::default(), transform, None);
            //self.pixmap.draw_pixmap(dest.max_x() as i32, dest.max_y() as i32, image, &paint, transform, None);
    }

    fn draw_surface_with_shadow(
        &self,
        _surface: &[u8],
        _dest: &Point2D<f32>,
        _color: &Color,
        _offset: &Vector2D<f32>,
        _sigma: f32,
        _operator: CompositionOp,
    ) {
        println!("no support for drawing shadows");
    }

    fn fill(&mut self, path: &Path, pattern: canvas_data::Pattern, draw_options: &DrawOptions) {
        let mut draw_options = draw_options.clone();
        let mut draw_options2 = draw_options.as_tiny_skia_mut();
        draw_options2.shader = pattern.as_tiny_skia().to_owned();
        draw_options2.shader.apply_opacity(self.opacity);

        self.pixmap.fill_path(path.as_tiny_skia(), &draw_options2, tiny_skia::FillRule::default(), self.transform, self.mask.as_ref())
    }

    fn fill_text(
        &mut self,
        font: &font_kit::font::Font,
        point_size: f32,
        text: &str,
        start: lyon_geom::Point<f32>,
        pattern: &crate::canvas_data::Pattern,
        draw_options: &DrawOptions,
    ) {
        //unimplemented!();
    }

    fn fill_rect(&mut self, rect: &Rect<f32>, pattern: canvas_data::Pattern, draw_options: &DrawOptions) {
        let mut draw_options = draw_options.clone();
        let mut draw_options2 = draw_options.as_tiny_skia_mut();
        draw_options2.shader = pattern.as_tiny_skia().to_owned();
        draw_options2.shader.apply_opacity(self.opacity);

        self.pixmap.fill_rect(
            tiny_skia::Rect::from_xywh(
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            ).unwrap(),
            &draw_options2,
            self.transform,
            self.mask.as_ref(),
        )
    }

    fn get_size(&self) -> Size2D<i32> {
        Size2D::new(self.pixmap.width() as i32, self.pixmap.height() as i32)
    }

    fn get_transform(&self) -> Transform2D<f32> {
        Transform2D::new(self.transform.sx, self.transform.ky, self.transform.kx, self.transform.sy, self.transform.tx, self.transform.ty)
    }

    fn pop_clip(&mut self) {
        self.mask_paths.pop();

        if !self.mask_paths.is_empty() {
            let mut mask = tiny_skia::Mask::new(self.pixmap.width(), self.pixmap.height()).unwrap();
            for path in &self.mask_paths {
                mask.fill_path(&path, tiny_skia::FillRule::default(), true, self.transform);
            }
            self.mask = Some(mask);
        } else {
            self.mask = None;
        }
    }

    fn push_clip(&mut self, path: &Path) {
        self.mask_paths.push(path.as_tiny_skia().clone());

        if !self.mask_paths.is_empty() {
            let mut mask = tiny_skia::Mask::new(self.pixmap.width(), self.pixmap.height()).unwrap();
            for path in &self.mask_paths {
                mask.fill_path(&path, tiny_skia::FillRule::default(), true, self.transform);
            }
            self.mask = Some(mask);
        } else {
            self.mask = None;
        }
    }

    fn set_transform(&mut self, matrix: &Transform2D<f32>) {
        self.transform = tiny_skia::Transform {
            sx: matrix.m11,
            ky: matrix.m12,
            kx: matrix.m21,
            sy: matrix.m22,
            tx: matrix.m31,
            ty: matrix.m32,
        };
    }

    fn get_opacity(&self) -> f32 {
        self.opacity
    }

    fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    fn snapshot(&self) -> &[u8] {
        self.pixmap.data()
    }

    fn stroke(
        &mut self,
        path: &Path,
        pattern: crate::canvas_data::Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        // TODO - This pattern is too common; address it
        let mut draw_options = draw_options.clone();
        let mut draw_options2 = draw_options.as_tiny_skia_mut();
        draw_options2.shader = pattern.as_tiny_skia().to_owned();
        draw_options2.shader.apply_opacity(self.opacity);

        self.pixmap.stroke_path(path.as_tiny_skia(), &draw_options2, stroke_options.as_tiny_skia(), self.transform, self.mask.as_ref())
    }

    fn stroke_line(
        &mut self,
        start: lyon_geom::Point<f32>,
        end: lyon_geom::Point<f32>,
        pattern: crate::canvas_data::Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(start.x, start.y);
        pb.line_to(end.x, end.y);
        let mut stroke_options = stroke_options.as_tiny_skia().clone();
        let line_cap = match stroke_options.line_join {
            tiny_skia::LineJoin::Round => tiny_skia::LineCap::Round,
            _ => tiny_skia::LineCap::Butt,
        };
        stroke_options.line_cap = line_cap;

        let mut draw_options = draw_options.clone();
        let mut draw_options2 = draw_options.as_tiny_skia_mut();
        draw_options2.shader = pattern.as_tiny_skia().to_owned();
        draw_options2.shader.apply_opacity(self.opacity);

        self.pixmap.stroke_path(
            &pb.finish().unwrap(),
            &draw_options2,
            &stroke_options,
            self.transform,
            self.mask.as_ref(),
        );
    }

    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: crate::canvas_data::Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.push_rect(tiny_skia::Rect::from_xywh(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        ).unwrap());

        let mut draw_options = draw_options.clone();
        let mut draw_options2 = draw_options.as_tiny_skia_mut();
        draw_options2.shader = pattern.as_tiny_skia().to_owned();
        draw_options2.shader.apply_opacity(self.opacity);

        self.pixmap.stroke_path(
            &pb.finish().unwrap(),
            &draw_options2,
            stroke_options.as_tiny_skia(),
            self.transform,
            self.mask.as_ref(),
        );
    }

    fn snapshot_data(&self, f: &dyn Fn(&[u8]) -> Vec<u8>) -> Vec<u8> {
        let v = self.pixmap.data();
        f(v)
    }

    fn snapshot_data_owned(&self) -> Vec<u8> {
        let v = self.pixmap.data();
        v.to_vec()
    }
}

struct PathBuilder(Option<tiny_skia::PathBuilder>);

impl PathBuilder {
    fn new() -> PathBuilder {
        PathBuilder(Some(tiny_skia::PathBuilder::new()))
    }
}

impl GenericPathBuilder for PathBuilder {
    fn arc(
        &mut self,
        origin: lyon_geom::Point<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    ) {
        self.ellipse(
            origin,
            radius,
            radius,
            0.,
            start_angle,
            end_angle,
            anticlockwise,
        );
    }

    fn bezier_curve_to(
            &mut self,
            control_point1: &lyon_geom::Point<f32>,
            control_point2: &lyon_geom::Point<f32>,
            control_point3: &lyon_geom::Point<f32>,
        ) {
        self.0.as_mut().unwrap().cubic_to(
            control_point1.x,
            control_point1.y,
            control_point2.x,
            control_point2.y,
            control_point3.x,
            control_point3.y,
        )
    }

    fn close(&mut self) {
        self.0.as_mut().unwrap().close();
    }

    fn ellipse(
        &mut self,
        origin: Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    ) {
        let mut start = Angle::radians(start_angle);
        let mut end = Angle::radians(end_angle);

        // Wrap angles mod 2 * PI if necessary
        if !anticlockwise && start > end + Angle::two_pi() ||
            anticlockwise && end > start + Angle::two_pi()
        {
            start = start.positive();
            end = end.positive();
        }

        // Calculate the total arc we're going to sweep.
        let sweep = match anticlockwise {
            true => {
                if end - start == Angle::two_pi() {
                    -Angle::two_pi()
                } else if end > start {
                    -(Angle::two_pi() - (end - start))
                } else {
                    -(start - end)
                }
            },
            false => {
                if start - end == Angle::two_pi() {
                    Angle::two_pi()
                } else if start > end {
                    Angle::two_pi() - (start - end)
                } else {
                    end - start
                }
            },
        };

        let arc: Arc<f32> = Arc {
            center: origin,
            radii: Vector2D::new(radius_x, radius_y),
            start_angle: start,
            sweep_angle: sweep,
            x_rotation: Angle::radians(rotation_angle),
        };

        self.line_to(arc.from());

        arc.for_each_quadratic_bezier(&mut |q| {
            self.quadratic_curve_to(&q.ctrl, &q.to);
        });
    }

    fn get_current_point(&mut self) -> Option<Point2D<f32>> {
        self.0.as_mut().unwrap().last_point().map(|p| Point2D::new(p.x, p.y))
    }

    fn line_to(&mut self, point: Point2D<f32>) {
        self.0.as_mut().unwrap().line_to(point.x, point.y);
    }

    fn move_to(&mut self, point: lyon_geom::Point<f32>) {
        self.0.as_mut().unwrap().move_to(point.x, point.y);
    }

    fn quadratic_curve_to(&mut self, control_point: &lyon_geom::Point<f32>, end_point: &lyon_geom::Point<f32>) {
        self.0.as_mut().unwrap().quad_to(
            control_point.x,
            control_point.y,
            end_point.x,
            end_point.y,
        );
    }

    fn finish(&mut self) -> Option<Path> {
        self.0.take().unwrap().finish().map(|path| Path::TinySkia(path))
    }
}

impl Filter {
    fn as_tiny_skia(&self) -> tiny_skia::FilterQuality {
        match self {
            Filter::Bilinear => tiny_skia::FilterQuality::Bilinear,
            Filter::Nearest => tiny_skia::FilterQuality::Nearest,
        }
    }
}

pub trait ToTinySkia {
    type Target;

    fn to_tiny_skia(self) -> Self::Target;
}

impl ToTinySkia for Transform2D<f32> {
    type Target = tiny_skia::Transform;

    fn to_tiny_skia(self) -> Self::Target {
        tiny_skia::Transform {
            sx: self.m11,
            ky: self.m12,
            kx: self.m21,
            sy: self.m22,
            tx: self.m31,
            ty: self.m32,
        }
    }
}
