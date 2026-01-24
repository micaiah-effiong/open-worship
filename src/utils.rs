use gtk::gdk_pixbuf::prelude::PixbufLoaderExt;
use gtk::gio::prelude::ListModelExt;
use gtk::glib::object::{Cast, CastNone, IsA};
use gtk::prelude::{SelectionModelExt, StyleContextExt, TextBufferExt, WidgetExt};
use gtk::{CssProvider, gio, glib};

use crate::widgets::canvas::canvas::Canvas;
use crate::widgets::canvas::canvas_item::CanvasItem;
use crate::widgets::canvas::serialise::{
    CanvasData, CanvasItemData, CanvasItemType, SlideData, TextItemData,
};
use crate::widgets::canvas::text_item::TextItem;

/// Returns a vector of widgets
/// useful for iterating over childern in listview
pub fn widget_to_vec(_w: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut v = Vec::new();

    let mut w = _w.clone();

    loop {
        v.push(w.clone());

        if let Some(next_s) = w.next_sibling() {
            w = next_s;
        } else {
            break;
        }
    }

    v
}
// pub trait WidgetExtrasExt: IsA<gtk::Widget> {
//     fn set_margin(&self, value: i32) {
//         self.set_margin_start(value);
//         self.set_margin_end(value);
//         self.set_margin_top(value);
//         self.set_margin_bottom(value);
//     }
//
//     fn set_expand(&self, value: bool) {
//         self.set_hexpand(value);
//         self.set_vexpand(value);
//     }
// }
// impl<O: IsA<gtk::Widget>> WidgetExtrasExt for O {}

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
    fn get_all_children_iter<W: IsA<gtk::Widget>>(widget: &gtk::Widget) -> Vec<W>;
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
    fn get_all_children_iter<W: IsA<gtk::Widget>>(widget: &gtk::Widget) -> Vec<W> {
        let mut result = Vec::new();
        let mut stack = vec![widget.clone()];

        while let Some(current) = stack.pop() {
            let mut child = current.first_child();

            while let Some(current_child) = child {
                if let Ok(c) = current_child.clone().downcast::<W>()
                    && c.is_drawable()
                {
                    result.push(c);
                }
                stack.push(current_child.clone());
                child = current_child.next_sibling();
            }
        }

        result
    }
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

    if let Ok(single) = selection_model.clone().downcast::<gtk::SingleSelection>() {
        single
            .model()
            .and_then(|m| m.downcast::<gtk::gio::ListStore>().ok())
    } else if let Ok(multi) = selection_model.clone().downcast::<gtk::MultiSelection>() {
        multi
            .model()
            .and_then(|m| m.downcast::<gtk::gio::ListStore>().ok())
    } else if let Ok(none) = selection_model.clone().downcast::<gtk::NoSelection>() {
        none.model()
            .and_then(|m| m.downcast::<gtk::gio::ListStore>().ok())
    } else {
        None
    }
}

pub trait ListViewExtra: IsA<gtk::ListView> {
    fn append_item(&self, item: &impl IsA<glib::Object>) {
        let list_view = self.upcast_ref::<gtk::ListView>();
        let Some(list_store) = get_list_store(&list_view) else {
            return;
        };

        list_store.append(item);
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
        // glib::dgettext("myapp", $msg)
    }
}
impl<O: IsA<gtk::Widget>> WidgetExtrasExt for O {}

//
macro_rules! tr {
    ($msg:expr) => {
        glib::dgettext(Some("myapp"), $msg)
    };
}
