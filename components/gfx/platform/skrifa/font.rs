/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::sync::Arc;

use app_units::Au;
use skrifa::attribute::Attributes;
use skrifa::font::FontRef;
use skrifa::instance::{LocationRef, Size};
use skrifa::string::StringId;
use skrifa::{MetadataProvider, Tag};
use style::values::computed::font::{FontStretch, FontStyle, FontWeight};

use super::font_template::FontTemplateData;
use crate::font::{FontHandleMethods, FontMetrics, FontTableMethods, FractionalPixel};
use crate::font_context::FontContextHandle;
use crate::text::glyph::GlyphId;

#[derive(Debug)]
pub struct FontTable {
    buffer: Vec<u8>,
}

impl FontTableMethods for FontTable {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}

// I hate lifetimes
#[derive(Debug)]
pub struct Test(Vec<u8>);

impl Test {
    pub fn get_font_ref(&self) -> FontRef {
        FontRef::new(&self.0).unwrap()
    }
}

#[derive(Debug)]
pub struct FontInfo {
    family_name: Option<String>,
    subfamily_name: Option<String>,
    attributes: Attributes,
}

#[derive(Debug)]
pub struct FontHandle {
    font_data: Arc<FontTemplateData>,
    test: Test,
    info: FontInfo,
    em_size: Size,
}

impl FontHandleMethods for FontHandle {
    fn new_from_template(
        _fctx: &FontContextHandle,
        template: Arc<FontTemplateData>,
        pt_size: Option<app_units::Au>,
    ) -> Result<Self, ()> {
        let test = if let Some(ref bytes) = template.bytes {
            Test(bytes.to_owned())
        } else {
            let bytes = std::fs::read(Path::new(template.identifier.as_ref())).unwrap();
            Test(bytes)
        };
        let font = test.get_font_ref();

        let family_name = font
            .localized_strings(StringId::FAMILY_NAME)
            .english_or_first()
            .map(|locstr| locstr.to_string());
        let subfamily_name = font
            .localized_strings(StringId::SUBFAMILY_NAME)
            .english_or_first()
            .map(|locstr| locstr.to_string());
        let attributes = font.attributes();

        let info = FontInfo {
            family_name,
            subfamily_name,
            attributes,
        };

        let pt_size = pt_size.unwrap_or(Au::from_f32_px(16.0));
        let em_size = Size::new(pt_size.to_f32_px());

        Ok(Self {
            font_data: template,
            test,
            info,
            em_size,
        })
    }

    fn template(&self) -> Arc<FontTemplateData> {
        self.font_data.clone()
    }

    fn family_name(&self) -> Option<String> {
        self.info.family_name.clone()
    }

    fn face_name(&self) -> Option<String> {
        self.info.subfamily_name.clone()
    }

    fn style(&self) -> FontStyle {
        match self.info.attributes.style {
            skrifa::attribute::Style::Normal => FontStyle::NORMAL,
            skrifa::attribute::Style::Italic => FontStyle::ITALIC,
            skrifa::attribute::Style::Oblique(degrees) => {
                FontStyle::oblique(degrees.unwrap_or(FontStyle::DEFAULT_OBLIQUE_DEGREES.into()))
            },
        }
    }

    fn boldness(&self) -> FontWeight {
        FontWeight::from_float(self.info.attributes.weight.value())
    }

    fn stretchiness(&self) -> FontStretch {
        FontStretch::from_percentage(self.info.attributes.stretch.ratio())
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        self.test
            .get_font_ref()
            .charmap()
            .map(codepoint)
            .map(|id| id.to_u16() as u32)
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        self.test
            .get_font_ref()
            .glyph_metrics(self.em_size, LocationRef::default())
            .advance_width(skrifa::GlyphId::new(glyph as u16))
            .map(|adv| adv as f64)
    }

    fn glyph_h_kerning(&self, _: GlyphId, _: GlyphId) -> FractionalPixel {
        0.0
    }

    fn can_do_fast_shaping(&self) -> bool {
        false
    }

    fn metrics(&self) -> FontMetrics {
        let metrics = self
            .test
            .get_font_ref()
            .metrics(self.em_size, LocationRef::default());
        let (underline_thickness, underline_offset) = if let Some(underline) = metrics.underline {
            (underline.thickness, underline.offset)
        } else {
            (0.0, 0.0)
        };

        let (strikeout_thickness, strikeout_offset) = if let Some(strikeout) = metrics.strikeout {
            (strikeout.thickness, strikeout.offset)
        } else {
            (0.0, 0.0)
        };

        FontMetrics {
            underline_size: Au::from_f32_px(underline_thickness),
            underline_offset: Au::from_f32_px(underline_offset),
            strikeout_size: Au::from_f32_px(strikeout_thickness),
            strikeout_offset: Au::from_f32_px(strikeout_offset),
            leading: Au::from_f32_px(metrics.leading),
            x_height: Au::from_f32_px(metrics.x_height.unwrap_or(0.0)),
            em_size: Au::from_f32_px(self.em_size.ppem().unwrap_or(0.0)),
            ascent: Au::from_f32_px(metrics.ascent),
            descent: Au::from_f32_px(-metrics.descent),
            max_advance: Au::from_f32_px(metrics.max_width.unwrap_or(0.0)),
            average_advance: Au::from_f32_px(metrics.average_width.unwrap_or(0.0)),
            line_gap: Au::from_f32_px(metrics.ascent + -metrics.descent + metrics.leading),
        }
    }

    fn table_for_tag(&self, tag: crate::font::FontTableTag) -> Option<FontTable> {
        self.test
            .get_font_ref()
            .table_data(Tag::from_u32(tag))
            .map(|data| FontTable {
                buffer: data.as_bytes().to_owned(),
            })
    }

    fn identifier(&self) -> servo_atoms::Atom {
        self.font_data.identifier.clone()
    }
}
