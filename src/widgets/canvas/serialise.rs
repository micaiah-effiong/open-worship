use gtk::glib;
use serde::{Deserialize, Serialize};

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
    pub justification: u32, // should be enum
    pub align: u32,         // should be enum
    pub color: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum CanvasItemType {
    #[serde(rename = "text")]
    Text(TextItemData),
    #[default]
    Unknown,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SlideData {
    pub transition: u32,
    pub items: Vec<CanvasItemData>,
    pub preview: String,
    #[serde(flatten)]
    pub canvas_data: CanvasData,
}

impl SlideData {
    pub fn new(
        transition: u32,
        items: Vec<CanvasItemData>,
        preview: String,
        canvas_data: CanvasData,
    ) -> Self {
        Self {
            transition,
            items,
            preview,
            canvas_data,
        }
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

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct SlideManagerData {
    #[serde(rename = "current-slide")]
    pub current_slide: u32,
    #[serde(rename = "preview-slide")]
    pub preview_slide: u32,
    // aspect_ratio
    pub slides: Vec<SlideData>,
}

impl SlideManagerData {
    pub fn new(current_slide: u32, preview_slide: u32, slides: impl Into<Vec<SlideData>>) -> Self {
        Self {
            current_slide,
            preview_slide,
            slides: slides.into(),
        }
    }
}
