use gtk::gdk::prelude::DisplayExt;
use gtk::gdk_pixbuf::prelude::PixbufLoaderExt;
use gtk::gio::prelude::ListModelExt;
use gtk::glib::object::{Cast, CastNone, IsA};
use gtk::glib::types::StaticType;
use gtk::prelude::{
    AccessibleExt, SelectionModelExt, SnapshotExt, StyleContextExt, TextBufferExt, WidgetExt,
};
use gtk::{CssProvider, gio, glib};

use crate::widgets::canvas::canvas::Canvas;
use crate::widgets::canvas::canvas_item::CanvasItem;
use crate::widgets::canvas::serialise::{CanvasItemData, CanvasItemType};
use crate::widgets::canvas::text_item::TextItem;

pub trait TextBufferExtraExt: IsA<gtk::TextBuffer> {
    fn full_text(&self) -> glib::GString {
        self.text(&self.start_iter(), &self.end_iter(), true)
    }
}
impl<O: IsA<gtk::TextBuffer>> TextBufferExtraExt for O {}

pub struct ChildrenIterator<W> {
    model: gio::ListModel,
    pub front_index: u32,
    pub back_index: u32,
    _marker: std::marker::PhantomData<W>,
}

impl<W: IsA<gtk::Widget> + IsA<glib::Object>> Iterator for ChildrenIterator<W> {
    type Item = W;

    fn next(&mut self) -> Option<Self::Item> {
        while self.front_index < self.back_index {
            let item = self.model.item(self.front_index);
            self.front_index += 1;

            if let Some(child) = item.and_downcast::<W>() {
                return Some(child);
            }
        }

        None
    }
}
impl<W: IsA<gtk::Widget> + IsA<glib::Object>> DoubleEndedIterator for ChildrenIterator<W> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while self.front_index < self.back_index {
            self.back_index -= 1;

            let item = self.model.item(self.back_index);
            if let Some(child) = item.and_downcast::<W>() {
                return Some(child);
            }
        }
        None
    }
}

pub trait WidgetChildrenExt {
    fn get_children<W: IsA<gtk::Widget> + IsA<glib::Object>>(&self) -> ChildrenIterator<W>;
    fn children(&self) -> ChildrenIterator<gtk::Widget>;
    // fn get_all_children_iter<W: IsA<gtk::Widget>>(widget: &gtk::Widget) -> Vec<W>;
}
impl<O: IsA<gtk::Widget>> WidgetChildrenExt for O {
    fn get_children<W: IsA<gtk::Widget> + IsA<glib::Object>>(&self) -> ChildrenIterator<W> {
        let model = self.observe_children();
        let size = model.n_items();
        ChildrenIterator {
            model: model,
            front_index: 0,
            back_index: size,
            _marker: std::marker::PhantomData,
        }
    }
    fn children(&self) -> ChildrenIterator<gtk::Widget> {
        self.get_children::<gtk::Widget>()
    }
    // fn get_all_children_iter<W: IsA<gtk::Widget>>(widget: &gtk::Widget) -> Vec<W> {
    //     let mut result = Vec::new();
    //     let mut stack = vec![widget.clone()];
    //
    //     while let Some(current) = stack.pop() {
    //         let mut child = current.first_child();
    //
    //         while let Some(current_child) = child {
    //             if let Ok(c) = current_child.clone().downcast::<W>()
    //                 && c.is_drawable()
    //             {
    //                 result.push(c);
    //             }
    //             stack.push(current_child.clone());
    //             child = current_child.next_sibling();
    //         }
    //     }
    //
    //     result
    // }
}

pub mod rect {
    use gtk::{gdk, glib};

    #[derive(Debug, Default, Clone, Copy, PartialEq, glib::Boxed)]
    #[boxed_type(name = "OwRect")]
    pub struct Rect {
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }

    impl From<gdk::Rectangle> for Rect {
        fn from(value: gdk::Rectangle) -> Self {
            Self::new(value.x(), value.y(), value.width(), value.height())
        }
    }
    impl Into<gdk::Rectangle> for Rect {
        fn into(self) -> gdk::Rectangle {
            gdk::Rectangle::new(self.x, self.y, self.width, self.height)
        }
    }
    impl Rect {
        pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
            Self {
                x,
                y,
                width,
                height,
            }
        }
    }
}

pub fn set_style(widget: &impl IsA<gtk::Widget>, css: &str) {
    let provider = CssProvider::new();
    provider.load_from_string(css);

    widget
        .style_context()
        .add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}

/// Convert a JSON `Value` object describing a canvas item into a concrete CanvasItem instance.
///
/// The application-specific types `TextItem`, `ColorItem`, `ImageItem` are placeholders.
/// Return `None` if the type is unknown or if construction fails.
pub fn canvas_item_from_data(data: CanvasItemData, canvas: Option<&Canvas>) -> Option<CanvasItem> {
    match data.item_type {
        CanvasItemType::Text(_) => {
            let item = TextItem::new(canvas, Some(data)).upcast::<CanvasItem>();
            Some(item)
        }
        // Some("color") => {
        //     let item = ColorItem::new(canvas, Some(data)).upcast::<CanvasItem>();
        //     Some(item)
        // }
        // Some("image") => {
        //     let item = ImageItem::new(canvas, Some(data)).upcast::<CanvasItem>();
        //     Some(item)
        // }
        _ => None,
    }
}

pub fn int_to_transition(value: u32) -> gtk::StackTransitionType {
    match value {
        0 => gtk::StackTransitionType::None,
        1 => gtk::StackTransitionType::Crossfade,
        2 => gtk::StackTransitionType::SlideLeft,
        3 => gtk::StackTransitionType::SlideRight,
        4 => gtk::StackTransitionType::SlideUp,
        5 => gtk::StackTransitionType::SlideDown,
        6 => gtk::StackTransitionType::SlideLeftRight,
        7 => gtk::StackTransitionType::SlideUpDown,
        8 => gtk::StackTransitionType::OverUp,
        9 => gtk::StackTransitionType::OverDown,
        10 => gtk::StackTransitionType::OverLeft,
        11 => gtk::StackTransitionType::OverRight,
        12 => gtk::StackTransitionType::UnderUp,
        13 => gtk::StackTransitionType::UnderDown,
        14 => gtk::StackTransitionType::UnderLeft,
        15 => gtk::StackTransitionType::UnderRight,
        16 => gtk::StackTransitionType::OverUpDown,
        17 => gtk::StackTransitionType::OverDownUp,
        18 => gtk::StackTransitionType::OverLeftRight,
        19 => gtk::StackTransitionType::OverRightLeft,
        20 => gtk::StackTransitionType::RotateLeft,
        21 => gtk::StackTransitionType::RotateRight,
        22 => gtk::StackTransitionType::RotateLeftRight,
        _ => gtk::StackTransitionType::None,
    }
}

pub fn transition_to_int(value: gtk::StackTransitionType) -> u32 {
    match value {
        gtk::StackTransitionType::None => 0,
        gtk::StackTransitionType::Crossfade => 1,
        gtk::StackTransitionType::SlideLeft => 2,
        gtk::StackTransitionType::SlideRight => 3,
        gtk::StackTransitionType::SlideUp => 4,
        gtk::StackTransitionType::SlideDown => 5,
        gtk::StackTransitionType::SlideLeftRight => 6,
        gtk::StackTransitionType::SlideUpDown => 7,
        gtk::StackTransitionType::OverUp => 8,
        gtk::StackTransitionType::OverDown => 9,
        gtk::StackTransitionType::OverLeft => 10,
        gtk::StackTransitionType::OverRight => 11,
        gtk::StackTransitionType::UnderUp => 12,
        gtk::StackTransitionType::UnderDown => 13,
        gtk::StackTransitionType::UnderLeft => 14,
        gtk::StackTransitionType::UnderRight => 15,
        gtk::StackTransitionType::OverUpDown => 16,
        gtk::StackTransitionType::OverDownUp => 17,
        gtk::StackTransitionType::OverLeftRight => 18,
        gtk::StackTransitionType::OverRightLeft => 19,
        gtk::StackTransitionType::RotateLeft => 20,
        gtk::StackTransitionType::RotateRight => 21,
        gtk::StackTransitionType::RotateLeftRight => 22,
        _ => 23,
    }
}

pub fn base64_to_pixbuf(b64: &str) -> Option<gtk::gdk_pixbuf::Pixbuf> {
    let bytes = glib::base64_decode(b64);
    let loader = gtk::gdk_pixbuf::PixbufLoader::new();

    match loader.write(&bytes) {
        Ok(_) => (),
        Err(e) => {
            glib::g_critical!("Utile", "Error loading pixbuf loader byte: {:?}", e);
            return None;
        }
    };

    match loader.close() {
        Ok(_) => (),
        Err(e) => {
            glib::g_critical!("Utile", "Error closing pixbuf loader byte: {:?}", e);
            return None;
        }
    };
    return loader.pixbuf();
}

//
fn get_list_store(list_view: &gtk::ListView) -> Option<gio::ListStore> {
    let Some(selection_model) = list_view.model() else {
        return None;
    };

    let mut model = if let Ok(single) = selection_model.clone().downcast::<gtk::SingleSelection>() {
        single.model()?
    } else if let Ok(multi) = selection_model.clone().downcast::<gtk::MultiSelection>() {
        multi.model()?
    } else if let Ok(none) = selection_model.clone().downcast::<gtk::NoSelection>() {
        none.model()?
    } else {
        return None;
    };

    if let Ok(filter_model) = model.clone().downcast::<gtk::FilterListModel>() {
        model = filter_model.model()?;
    }

    if let Ok(sort_model) = model.clone().downcast::<gtk::SortListModel>() {
        model = sort_model.model()?;
    }

    model.downcast::<gtk::gio::ListStore>().ok()
}

pub trait ListViewExtra: IsA<gtk::ListView> {
    fn append_item(&self, item: &impl IsA<glib::Object>) {
        let list_view = self.upcast_ref::<gtk::ListView>();
        let Some(list_store) = get_list_store(&list_view) else {
            return;
        };

        list_store.append(item);
    }
    fn insert_item(&self, position: u32, item: &impl IsA<glib::Object>) {
        let list_view = self.upcast_ref::<gtk::ListView>();
        let Some(list_store) = get_list_store(&list_view) else {
            return;
        };

        list_store.insert(position, item);
    }
    fn get_items(&self) -> Vec<glib::Object> {
        let list_view = self.upcast_ref::<gtk::ListView>();
        let mut items = Vec::new();

        let Some(list_store) = get_list_store(&list_view) else {
            return items;
        };

        for index in 0..=list_store.n_items() {
            if let Some(item) = list_store.item(index) {
                items.push(item);
            }
        }

        items
    }
    fn get_selected_items(&self) -> Vec<glib::Object> {
        let list_view = self.upcast_ref::<gtk::ListView>();
        let mut items = Vec::new();

        let Some(model) = list_view.model() else {
            return items;
        };
        let Some(list_store) = get_list_store(&list_view) else {
            return items;
        };

        for index in 0..=list_store.n_items() {
            if model.is_selected(index)
                && let Some(item) = list_store.item(index)
            {
                items.push(item);
            }
        }

        items
    }
    fn remove_selected_items(&self) {
        let list_view = self.upcast_ref::<gtk::ListView>();
        let Some(selection_model) = list_view.model() else {
            return;
        };

        let Some(list_store) = get_list_store(&list_view) else {
            return;
        };

        let bitset = selection_model.selection();
        let Some((iter, first)) = gtk::BitsetIter::init_first(&bitset) else {
            return;
        };

        let mut iter_list = iter.collect::<Vec<u32>>();
        iter_list.insert(0, first);

        for item in iter_list {
            list_store.remove(item);
        }
    }

    fn remove_item(&self, item: &impl IsA<glib::Object>) {
        let list_view = self.upcast_ref::<gtk::ListView>();

        let Some(list_store) = get_list_store(&list_view) else {
            return;
        };

        if let Some(position) = list_store.find(item) {
            list_store.remove(position);
        };
    }

    fn remove_all(&self) {
        let list_view = self.upcast_ref::<gtk::ListView>();
        if let Some(list_store) = get_list_store(&list_view) {
            list_store.remove_all();
        };
    }
}
impl<O: IsA<gtk::ListView>> ListViewExtra for O {}

//
pub trait WidgetExtrasExt: IsA<gtk::Widget> {
    fn set_margin_all(&self, value: i32) {
        self.set_margin_start(value);
        self.set_margin_end(value);
        self.set_margin_top(value);
        self.set_margin_bottom(value);
    }

    fn set_expand(&self, value: bool) {
        self.set_hexpand(value);
        self.set_vexpand(value);
    }
    fn set_tooltip(&self, text: &str) {
        self.set_has_tooltip(true);
        self.set_tooltip_text(Some(text));
    }
    fn snap_selected(
        &self,
        children: Vec<&impl IsA<gtk::Widget>>,
    ) -> Option<impl IsA<gtk::gdk::Paintable>> {
        let obj = self.upcast_ref::<gtk::Widget>();
        let snap = gtk::Snapshot::new();
        for c in children {
            obj.snapshot_child(c, &snap);
        }

        let Some((_, _, w, h)) = obj.bounds() else {
            glib::g_log!("Snapshot", glib::LogLevel::Warning, "Could not get bounds",);
            return None;
        };

        let size = gtk::graphene::Size::new(w as f32, h as f32);
        snap.to_paintable(Some(&size))
    }
    fn snap(&self) -> Option<impl IsA<gtk::gdk::Paintable>> {
        let obj = self.upcast_ref::<gtk::Widget>();
        let snap = gtk::Snapshot::new();
        for c in obj.children() {
            obj.snapshot_child(&c, &snap);
        }

        let Some((_, _, w, h)) = obj.bounds() else {
            glib::g_log!("Snapshot", glib::LogLevel::Warning, "Could not get bounds",);
            return None;
        };

        let size = gtk::graphene::Size::new(w as f32, h as f32);
        snap.to_paintable(Some(&size))
    }

    fn toplevel_window(&self) -> Option<gtk::Window> {
        self.ancestor(gtk::Window::static_type())
            .and_then(|widget| widget.dynamic_cast::<gtk::Window>().ok())
    }
}
impl<O: IsA<gtk::Widget>> WidgetExtrasExt for O {}

pub trait RGBExtra {
    fn to_hex(&self) -> String;
}

impl RGBExtra for gtk::gdk::RGBA {
    fn to_hex(&self) -> String {
        let rgba = self.clone();
        let r = (rgba.red() * 255.0) as u8;
        let g = (rgba.green() * 255.0) as u8;
        let b = (rgba.blue() * 255.0) as u8;
        let a = (rgba.alpha() * 255.0) as u8;
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a).to_lowercase()
    }
}

//
#[macro_export]
macro_rules! tr {
    ($msg:expr) => {
        glib::dgettext(Some("myapp"), $msg)
    };
}

/// Constructs a resource path for the Openworship.
///
/// This macro simplifies resource path construction by prepending the standard
/// application resource base path `/com/openworship/app/` to the provided path segments.
///
/// # Examples
///
/// Basic usage with a single path:
///
/// ```rust
/// let path = format_resource!("ui/main.ui");
/// assert_eq!(path, "/com/openworship/app/ui/main.ui");
/// ```
///
/// Usage with directory and file separated:
///
/// ```rust
/// let path = format_resource!("ui", "main.ui");
/// assert_eq!(path, "/com/openworship/app/ui/main.ui");
///
/// ```
///
/// # Arguments
///
/// * `$path` - A string literal representing the full relative path
/// * `$dir` - A string literal representing the directory
/// * `$file` - A string literal representing the filename
#[macro_export]
macro_rules! format_resource {
    ($path:literal) => {
        concat!("/com/openworship/app/", $path)
    };
    ($dir:literal, $file:literal) => {
        concat!("/com/openworship/app/", $dir, "/", $file)
    };
}

#[macro_export]
macro_rules! accels {
    ($key:literal) => {
        if cfg!(target_os = "macos") {
            concat!("<Meta>", $key)
        } else {
            concat!("<Primary>", $key)
        }
    };
}

pub fn setup_theme_listener() {
    let Some(settings) = gtk::Settings::default() else {
        return;
    };
    let css_provider = gtk::CssProvider::new();

    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };

    gtk::style_context_add_provider_for_display(
        &display,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Apply initial theme
    update_css(
        &css_provider,
        settings.is_gtk_application_prefer_dark_theme(),
    );

    // Listen for changes
    display.connect_setting_changed(move |_, _| {
        update_css(
            &css_provider,
            settings.is_gtk_application_prefer_dark_theme(),
        );
    });
}
fn update_css(provider: &gtk::CssProvider, is_dark: bool) {
    let light = format_resource!("styles", "style.css");
    let dark = format_resource!("styles", "style-dark.css");

    let css = if is_dark { dark } else { light };
    provider.load_from_resource(css);
}

pub fn space_camelcase(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.char_indices() {
        if c.is_uppercase() && i != 0 {
            result.push(' ');
        }
        result.push(c);
    }
    result
}

pub mod text_tags {
    pub const BOLD: &str = "bold";
    pub const ITALIC: &str = "italic";
    pub const UNDERLINE: &str = "underline";
}

pub mod buffer_markup {

    use gtk::{
        TextIter,
        glib::{
            self,
            object::{Cast, IsA, ObjectExt},
        },
        prelude::TextBufferExt,
    };

    use crate::utils::RGBExtra;

    pub fn buffer_to_markup(buffer: &gtk::TextBuffer) -> String {
        let mut markup = String::new();
        let start = buffer.start_iter();
        let end = buffer.end_iter();

        let mut iter = start;
        while iter < end {
            // Collect tags turning ON at this position
            for tag in iter.toggled_tags(true) {
                // println!("tag: {:?}", get_truthy_properties(&tag));
                let attrs = get_truthy_properties(&tag)
                    .iter()
                    .filter_map(|(n, v)| Some(format!("{n}=\"{v}\"")))
                    .collect::<Vec<_>>()
                    .join(" ");

                markup.push_str(&format!("<span {attrs}>"));
            }

            // Append the character (escaped for Pango markup)
            let ch = iter.char();
            match ch {
                '<' => markup.push_str("&lt;"),
                '>' => markup.push_str("&gt;"),
                '&' => markup.push_str("&amp;"),
                _ => markup.push(ch),
            }

            // Collect tags turning OFF after this character
            iter.forward_char();
            for _tag in iter.toggled_tags(false) {
                markup.push_str("</span>");
            }
        }

        markup
    }

    fn get_truthy_properties<T: glib::object::IsA<glib::Object>>(obj: &T) -> Vec<(String, String)> {
        let object = obj.as_ref();

        object
            .list_properties()
            .iter()
            .filter(|p| p.flags().contains(glib::ParamFlags::READABLE))
            .filter(|p| p.name().ends_with("-set"))
            .filter(|p| obj.property_value(p.name()).get::<bool>().unwrap_or(false))
            .filter_map(|p| {
                let name = p.name().trim_end_matches("-set").to_string();
                let readable_name = match name.as_str() {
                    "foreground" => "foreground-rgba",
                    "background" => "background-rgba",
                    "paragraph-background" => "paragraph-background-rgba",
                    _ => &name,
                };

                let raw = obj.property_value(&readable_name);

                match cast_value(&raw) {
                    Some(pv) => Some((name, pv)),
                    None => None,
                }
            })
            .filter_map(|(n, v)| is_truthy(&v).then(|| (n, v.to_display_string())))
            .collect()
    }

    #[derive(Debug)]
    enum PropValue {
        Bool(bool),
        I32(i32),
        U32(u32),
        I64(i64),
        U64(u64),
        F32(f32),
        F64(f64),
        String(String),
        Object(glib::Object),
        Other(String), // fallback: type name as string
    }
    impl PropValue {
        pub fn to_display_string(&self) -> String {
            match self {
                PropValue::Bool(v) => v.to_string(),
                PropValue::I32(v) => v.to_string(),
                PropValue::U32(v) => v.to_string(),
                PropValue::I64(v) => v.to_string(),
                PropValue::U64(v) => v.to_string(),
                PropValue::F32(v) => v.to_string(),
                PropValue::F64(v) => v.to_string(),
                PropValue::String(v) => v.clone(),
                PropValue::Object(_) => "".to_string(),
                PropValue::Other(_) => "".to_string(),
            }
        }
    }
    fn is_truthy(value: &PropValue) -> bool {
        match value {
            PropValue::Bool(b) => *b != false,
            PropValue::I32(i) => *i != 0,
            PropValue::U32(i) => *i != 0,
            PropValue::I64(i) => *i != 0,
            PropValue::U64(i) => *i != 0,
            PropValue::F32(i) => *i != 0.0,
            PropValue::F64(i) => *i != 0.0,
            PropValue::String(s) => !s.is_empty(),
            PropValue::Object(_) => false,
            PropValue::Other(_) => false,
        }
    }

    fn cast_value(value: &glib::Value) -> Option<PropValue> {
        use glib::types::Type;
        match value.type_() {
            Type::BOOL => Some(PropValue::Bool(value.get::<bool>().ok()?)),
            Type::I32 => Some(PropValue::I32(value.get::<i32>().ok()?)),
            Type::U32 => Some(PropValue::U32(value.get::<u32>().ok()?)),
            Type::I64 => Some(PropValue::I64(value.get::<i64>().ok()?)),
            Type::U64 => Some(PropValue::U64(value.get::<u64>().ok()?)),
            Type::F32 => Some(PropValue::F32(value.get::<f32>().ok()?)),
            Type::F64 => Some(PropValue::F64(value.get::<f64>().ok()?)),
            Type::STRING => Some(PropValue::String(value.get::<String>().ok()?)),

            _ => {
                if let Ok(val) = value.clone().get::<gtk::pango::Style>() {
                    let val = format!("{:?}", val).to_lowercase();
                    return Some(PropValue::String(val));
                };
                if let Ok(val) = value.clone().get::<gtk::pango::Underline>() {
                    let val = format!("{:?}", val).to_lowercase();
                    return Some(PropValue::String(val));
                };
                if let Ok(val) = value.clone().get::<gtk::gdk::RGBA>() {
                    return Some(PropValue::String(val.to_hex()));
                };
                println!("cast = {:?}", value.type_());
                return None;
            }
        }
    }

    pub trait TextBufferExtra: IsA<gtk::TextBuffer> {
        fn markup(&self) -> String {
            let buff = self.upcast_ref::<gtk::TextBuffer>();
            buffer_to_markup(&buff)
        }
        fn cursor_is_between(&self, start: &gtk::TextIter, end: &gtk::TextIter) -> bool {
            let buf = self.upcast_ref::<gtk::TextBuffer>();
            let cursor = buf.iter_at_offset(buf.cursor_position());
            println!("cursor {}, end {:?} ", cursor.offset(), end.offset());
            cursor.offset() >= start.offset() && cursor.offset() <= end.offset()
            // cursor.in_range(start, end)
        }

        fn get_tags_by<F: Fn(&gtk::TextTag) -> bool>(&self, f: F) -> Vec<gtk::TextTag> {
            let buffer = self.upcast_ref::<gtk::TextBuffer>();

            let mut v = Vec::new();
            buffer.tag_table().foreach(|i| v.push(i.clone()));
            v.into_iter().filter(|t| f(t)).collect::<Vec<_>>()
        }
        fn remove_tags_by<F: Fn(&gtk::TextTag) -> bool>(
            &self,
            f: F,
            start: &gtk::TextIter,
            end: &TextIter,
        ) {
            let buffer = self.upcast_ref::<gtk::TextBuffer>();

            let tags = buffer.get_tags_by(f);
            tags.iter()
                // .filter(|v| v.is_weight_set())
                .for_each(|tag| buffer.remove_tag(tag, &start, &end));
        }
    }

    impl<O: IsA<gtk::TextBuffer>> TextBufferExtra for O {}
}
