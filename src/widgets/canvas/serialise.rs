use gtk::glib;
use serde::{Deserialize, Serialize};

use crate::services;

fn default_font_weight() -> String {
    "regular".into()
}
fn default_text_decoration() -> String {
    "none".into()
}
fn default_text_shadow() -> String {
    "#0000 0px  0px 0px".into()
}
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, glib::Boxed)]
#[boxed_type(name = "TextItemData")]
pub struct TextItemData {
    #[serde(rename = "text-data")]
    pub text_data: String,
    pub font: String,
    #[serde(rename = "font-size")]
    pub font_size: u32,
    #[serde(rename = "font-style")]
    pub font_style: String,
    #[serde(rename = "font-weight", default = "default_font_weight")]
    pub font_weight: String,
    pub justification: u32, // should be enum
    pub align: u32,         // should be enum
    pub color: String,
    #[serde(rename = "text-underline")]
    pub text_underline: bool,
    #[serde(rename = "text-outline")]
    pub text_outline: bool,
    #[serde(rename = "text-shadow")]
    pub text_shadow: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CanvasItemData {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    #[serde(flatten)]
    pub item_type: CanvasItemType,
}
impl CanvasItemData {
    pub fn new(x: i32, y: i32, w: i32, h: i32, item_type: CanvasItemType) -> Self {
        Self {
            x,
            y,
            w,
            h,
            item_type,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, glib::Boxed)]
#[boxed_type(name = "CanvasData")]
pub struct CanvasData {
    #[serde(rename = "background-color")]
    pub background_color: String,
    #[serde(rename = "background-pattern")]
    pub background_pattern: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum CanvasItemType {
    #[serde(rename = "text")]
    Text(TextItemData),
    #[default]
    Unknown,
}

const DEFAULT_SLIDE: &str = services::slide::EMPTY_SLIDE;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, glib::Boxed)]
#[boxed_type(name = "SlideData")]
pub struct SlideData {
    pub transition: u32,
    pub items: Vec<CanvasItemData>,
    pub preview: String,
    #[serde(flatten)]
    pub canvas_data: CanvasData,
}

impl SlideData {
    pub fn new<I: IntoIterator<Item = CanvasItemData>>(
        transition: u32,
        items: I,
        preview: String,
        canvas_data: CanvasData,
    ) -> Self {
        Self {
            transition,
            items: items.into_iter().collect(),
            preview,
            canvas_data,
        }
    }

    pub fn from_default() -> Self {
        serde_json::from_str(DEFAULT_SLIDE).expect("Could not parse DEFAULT_SLIDE")
    }
}

impl From<SlideData> for CanvasData {
    fn from(value: SlideData) -> Self {
        Self {
            background_color: value.canvas_data.background_color,
            background_pattern: value.canvas_data.background_pattern,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, glib::Boxed)]
#[boxed_type(name = "SlideManagerData")]
pub struct SlideManagerData {
    #[serde(rename = "current-slide")]
    pub current_slide: u32,
    #[serde(rename = "preview-slide")]
    pub preview_slide: u32,
    pub title: String,
    // aspect_ratio
    pub slides: Vec<SlideData>,
}

impl SlideManagerData {
    pub fn new<I: IntoIterator<Item = SlideData>>(
        current_slide: u32,
        preview_slide: u32,
        slides: I,
    ) -> Self {
        Self {
            current_slide,
            preview_slide,
            slides: slides.into_iter().collect(),
            title: "".into(),
        }
    }
}
