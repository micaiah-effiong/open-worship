use std::{collections::HashSet, sync::Mutex, usize};

use gtk::{
    glib::{
        self,
        object::{Cast, ObjectExt},
        subclass::types::ObjectSubclassIsExt,
    },
    prelude::WidgetExt,
    subclass::window,
};
use serde_json::Value as JsonValue;

use crate::{
    services::slide::{self, Slide},
    widgets::canvas::{
        CanvasItemType,
        canvas_item::CanvasItem,
        serialise::{SlideData, SlideManagerData},
        text_item::TextItem,
    },
};

// pub static ASPECT_RATIO_OVERRIDE: i32 = -1;
static ASPECT_RATIO_OVERRIDE: Mutex<i32> = Mutex::new(-1);

// signal

mod signals {
    // pub const ASPECT_RATIO_CHANGED: &str = "aspect-ratio-changed";
    pub const RESETED: &str = "reseted";
    pub const CURRENT_SLIDE_CHANGED: &str = "current-slide-changed";
    pub const ITEM_CLICKED: &str = "item-clicked";
    pub const NEW_SLIDE_CREATED: &str = "new-slide-created";
    pub const SLIDES_SORTED: &str = "slides-sorted";
}

mod imp {
    use std::cell::{Cell, RefCell};
    use std::sync::OnceLock;

    use glib::Object;
    use gtk::glib::subclass::Signal;
    use gtk::glib::{self, Properties, subclass::types::ObjectSubclass};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use super::*;
    use crate::services::slide::Slide;
    use crate::utils;
    use crate::widgets::canvas::canvas_item::CanvasItem;
    // use crate::services::utils::{self, AspectRatio};
    // use crate::spice_window::SpiceWindow;
    // use crate::widgets::canvas_item::base::CanvasItem;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type=super::SlideManager)]
    pub struct SlideManager {
        // #[property(get, construct_only)]
        // pub window: glib::WeakRef<SpiceWindow>,
        #[property(get, set)]
        pub slideshow: RefCell<gtk::Stack>,
        #[property(get, set)]
        pub animation: Cell<bool>,

        #[doc = "property setter is private "]
        pub(super) slides: RefCell<Vec<Slide>>,

        pub making_new_slide: Cell<bool>,

        /// private
        #[property(get=Self::get_current_slide_, set=Self::set_current_slide_, nullable)]
        pub current_slide: RefCell<Option<Slide>>,
        /// private
        // current_ratio: RefCell<AspectRatio>,
        pub end_presentation_slide: RefCell<Slide>,

        #[property(get, set=Self::set_current_item_, nullable)]
        pub current_item: RefCell<Option<CanvasItem>>,

        #[property(get=Self::get_preview_slide_, set, nullable)]
        pub preview_slide: RefCell<Option<Slide>>,

        propagating_ratio: Cell<bool>,
        pub checkpoint: RefCell<Option<Slide>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SlideManager {
        const NAME: &'static str = "OwSlideManager";
        type Type = super::SlideManager;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SlideManager {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    // Signal::builder(super::signals::ASPECT_RATIO_CHANGED)
                    //     .param_types([utils::AspectRatio::static_type()])
                    //     .build(),
                    Signal::builder(super::signals::RESETED).build(),
                    Signal::builder(super::signals::CURRENT_SLIDE_CHANGED)
                        .param_types([Slide::static_type()])
                        .build(),
                    Signal::builder(super::signals::ITEM_CLICKED)
                        .param_types([Option::<CanvasItem>::static_type()])
                        .build(),
                    Signal::builder(super::signals::NEW_SLIDE_CREATED)
                        .param_types([Slide::static_type()])
                        .build(),
                    Signal::builder(super::signals::SLIDES_SORTED).build(),
                ]
            })
        }
    }

    impl SlideManager {
        fn set_current_item_(&self, value: Option<CanvasItem>) {
            self.current_item.replace(value.clone());
            self.obj()
                .emit_item_clicked(self.current_item.borrow().as_ref());
        }

        fn get_preview_slide_(&self) -> Option<Slide> {
            let ps = self.preview_slide.borrow().clone();

            if let Some(ps) = ps.clone()
                && ps.visible()
            {
                return Some(ps);
            }

            return None;
        }

        // pub fn current_ratio(&self) -> AspectRatio {
        //     self.current_ratio.borrow().clone()
        // }
        // pub fn set_current_ratio(&self, value: AspectRatio) {
        //     self.current_ratio.replace(value);
        // }

        pub fn propagating_ratio(&self) -> bool {
            self.propagating_ratio.get()
        }
        pub fn set_propagating_ratio(&self, value: bool) {
            self.propagating_ratio.set(value);
        }

        /// current_slide
        #[doc(alias = "current_slide")]
        pub fn get_current_slide_(&self) -> Option<Slide> {
            self.current_slide.borrow().clone()
        }

        /// set_current_slide
        #[doc(alias = "set_current_slide")]
        pub fn set_current_slide_(&self, value: Option<Slide>) {
            let obj = self.obj();
            if let Some(slide) = self.current_slide.borrow().clone()
                && let Some(canvas) = slide.imp().canvas.borrow().clone()
            {
                canvas.unselect_all(None);
            }

            let Some(val) = value else { return };
            if obj.animation() {
                obj.slideshow().set_transition_type(val.transition());
                obj.slideshow().set_transition_duration(500);
            }

            if self.slides.borrow().contains(&val) {
                obj.set_current_item(None::<CanvasItem>);
            } else if val == self.end_presentation_slide.borrow().clone() {
                val.set_visible(true);
                val.set_presentation_mode(true);
            }

            val.load_slide();
            self.current_slide.replace(Some(val.clone()));
            if let Some(canvas) = val.canvas() {
                obj.slideshow().set_visible_child(&canvas);
            }
            obj.emit_current_slide_changed(&val);
        }
    }
}

glib::wrapper! {
    pub struct SlideManager(ObjectSubclass<imp::SlideManager>);
}

impl Default for SlideManager {
    fn default() -> Self {
        let slide_manager = glib::Object::builder::<SlideManager>()
            // .property("window", window.clone())
            .build();

        let stack = gtk::Stack::builder()
            .hhomogeneous(false)
            .vhomogeneous(false)
            .build();
        slide_manager.set_slideshow(stack);

        slide_manager.imp().slides.replace(Vec::new());

        let empty_slide = Slide::empty(/* &window */ );
        slide_manager
            .imp()
            .end_presentation_slide
            .replace(empty_slide.clone());

        if let Some(canvas) = empty_slide.canvas() {
            canvas.connect_next_slide(glib::clone!(
                #[weak]
                slide_manager,
                move || slide_manager.next_slide()
            ));
            canvas.connect_previous_slide(glib::clone!(
                #[weak]
                slide_manager,
                move || slide_manager.previous_slide()
            ));
            slide_manager.slideshow().add_child(&canvas);
        }

        slide_manager
    }
}

impl SlideManager {
    pub fn emit_item_clicked(&self, item: Option<&CanvasItem>) {
        self.emit_by_name::<()>(signals::ITEM_CLICKED, &[&item]);
    }

    pub fn emit_current_slide_changed(&self, item: &Slide) {
        self.emit_by_name::<()>(signals::CURRENT_SLIDE_CHANGED, &[item]);
    }

    pub fn emit_reseted(&self) {
        self.emit_by_name::<()>(signals::RESETED, &[]);
    }

    pub fn emit_new_slide_created(&self, item: &Slide) {
        self.emit_by_name::<()>(signals::NEW_SLIDE_CREATED, &[item]);
    }

    // pub fn emit_aspect_ratio_changed(&self, item: &AspectRatio) {
    //     self.emit_by_name::<()>(signals::ASPECT_RATIO_CHANGED, &[item]);
    // }

    pub fn emit_slides_sorted(&self) {
        self.emit_by_name::<()>(signals::SLIDES_SORTED, &[]);
    }

    pub fn connect_new_slide_created<F: Fn(&Slide) -> () + 'static>(&self, f: F) {
        self.connect_closure(
            signals::NEW_SLIDE_CREATED,
            false,
            glib::closure_local!(move |_: &Self, slide: &Slide| f(slide)),
        );
    }

    pub fn connect_slides_sorted<F: Fn() -> () + 'static>(&self, f: F) {
        self.connect_closure(
            signals::SLIDES_SORTED,
            false,
            glib::closure_local!(move |_: &Self| f()),
        );
    }

    pub fn connect_current_slide_changed<F: Fn(&Slide) -> () + 'static>(&self, f: F) {
        self.connect_closure(
            signals::CURRENT_SLIDE_CHANGED,
            false,
            glib::closure_local!(move |_: &Self, slide: &Slide| f(slide)),
        );
    }

    pub fn connect_reseted<F: Fn() -> () + 'static>(&self, f: F) {
        self.connect_closure(
            signals::RESETED,
            false,
            glib::closure_local!(move |_: &Self| f()),
        );
    }

    pub fn connect_item_clicked<F: Fn(&Self, Option<&CanvasItem>) -> () + 'static>(&self, f: F) {
        self.connect_closure(
            signals::ITEM_CLICKED,
            false,
            glib::closure_local!(move |sm: &Self, item: Option<&CanvasItem>| f(sm, item)),
        );
    }

    // pub fn connect_aspect_ratio_changed<F: Fn(AspectRatio) -> () + 'static>(&self, f: F) {
    //     self.connect_closure(
    //         signals::ASPECT_RATIO_CHANGED,
    //         false,
    //         glib::closure_local!(move |_: &Self, ratio: AspectRatio| {
    //             f(ratio);
    //         }),
    //     );
    // }

    pub fn set_making_new_slide(&self, value: bool) {
        self.imp().making_new_slide.set(value);
    }

    pub fn new(/* window: SpiceWindow */) -> Self {
        Self::default()
    }

    pub fn reset(&self) {
        self.set_current_slide(None::<Slide>);
        self.set_preview_slide(None::<Slide>);

        for slide in self.slides() {
            if let Some(canvas) = slide.imp().canvas.borrow().clone() {
                self.slideshow().remove(&canvas);
                slide.destroy();
            }
        }

        self.imp().slides.borrow_mut().clear();

        self.emit_reseted();
    }

    pub fn slide_count(&self) -> usize {
        let mut slide_count = 0;

        for slide in self.slides() {
            if slide.visible() {
                slide_count += 1;
            }
        }

        return slide_count;
    }

    pub fn serialise(&self) -> SlideManagerData {
        let mut data = Vec::new();

        for slide in self.slides() {
            if slide.visible() {
                data.push(slide.serialise());
            }
        }

        let current_slide_index = if let Some(current_slide) = self.current_slide()
            && let Some(pos) = self.slides().iter().position(|c| c.eq(&current_slide))
        {
            pos
        } else {
            0
        } as u32;
        // ? slides.index_of (current_slide) : 0;
        // var preview_slide_index = preview_slide != null ? slides.index_of (preview_slide) : 0;
        let preview_slide_index = if let Some(preview_slide) = self.preview_slide()
            && let Some(pos) = self.slides().iter().position(|c| c.eq(&preview_slide))
        {
            pos
        } else {
            0
        } as u32;

        SlideManagerData::new(current_slide_index, preview_slide_index, data)

        // return format!(
        //     "{{\"current-slide\":{}, \"preview-slide\":{}, \"aspect-ratio\":{}, \"slides\": [{}]}}",
        //     current_slide_index,
        //     preview_slide_index,
        //     self.imp().current_ratio() as u32,
        //     data
        // );
    }

    pub fn load_data(&self, data: SlideManagerData) {
        let slides_array = data.slides.clone();

        // let Some(mut ratio) = root
        //     .get("aspect-ratio")
        //     .and_then(|v| v.as_i64().map(|v| v as i32))
        // else {
        //     eprintln!("Error loading date: Could not get \"aspect-ratio\"");
        //     return;
        // };
        //
        // if let Ok(mut val) = ASPECT_RATIO_OVERRIDE.lock()
        //     && *val != -1
        // {
        //     ratio = *val as i32;
        //     *val = -1;
        // }

        // self.imp()
        //     .set_current_ratio(AspectRatio::get_mode(Some(ratio)));
        // self.emit_aspect_ratio_changed(&self.imp().current_ratio());

        for slide_object in slides_array {
            self.new_slide(Some(slide_object.clone()), false);
        }

        // if self.slides().len() > data.current_slide as usize {
        //     self.set_current_slide(self.slides().get(data.current_slide as usize).cloned());
        //     if let Some(current_slide) = self.current_slide() {
        //         current_slide.reload_preview_data();
        //     }
        // } else {
        self.set_current_slide(self.slides().get(0).cloned());
        // }

        if let Some(slide) = self.current_slide() {
            println!("Slide {:?}", slide.transition());
        }

        let slide = match self.slides().len() > data.preview_slide as usize {
            true => self.slides().get(data.preview_slide as usize).cloned(),
            false => self.slides().get(0).cloned(),
        };
        self.set_preview_slide(slide);
    }

    ///
    /// * `undoable_action` - default false
    pub fn new_slide(&self, save_data: Option<SlideData>, undoable_action: bool) -> Slide {
        // let win = self
        //     .imp()
        //     .window
        //     .upgrade()
        //     .expect("Error creating new slide: Window is not available in slide manager");
        let slide = Slide::new(/* &win, */ save_data);

        let canvas = slide
            .canvas()
            .expect("Error creating new slide: Could not get slide canvas");

        canvas.connect_item_clicked(glib::clone!(
            #[weak(rename_to=sm)]
            self,
            move |item| sm.set_current_item(item)
        ));

        canvas.connect_next_slide(glib::clone!(
            #[weak(rename_to=sm)]
            self,
            move || sm.next_slide()
        ));

        canvas.connect_previous_slide(glib::clone!(
            #[weak(rename_to=sm)]
            self,
            move || sm.previous_slide()
        ));

        canvas.connect_ratio_changed(glib::clone!(
            #[weak(rename_to=sm)]
            self,
            #[weak]
            canvas,
            move |ratio| {
                if sm.imp().propagating_ratio() {
                    return;
                }

                // let w = canvas.width();
                // let h = canvas.height();

                for s in sm.slides() {
                    if s.visible() {
                        let Some(s_canvas) = s.imp().canvas.borrow().clone() else {
                            return;
                        };
                        s_canvas.set_current_ratio(ratio);

                        // WARN: this breaks the aspect-ratio
                        // should be implementated to scale by current ratio
                        // Force size
                        // s_canvas.set_size_request(w, h);
                        // s_canvas.set_size_request(500, 380);
                    }
                }

                sm.imp().set_propagating_ratio(false);
            }
        ));

        // if undoable_action {
        //     slide.set_visible(false);
        //     let action = TypedHistoryAction::slide_changed(&slide, "visible");
        //     if let Some(window) = self.window() {
        //         window
        //             .history_manager()
        //             .add_undoable_action(action.into(), Some(true));
        //     }
        //     slide.set_visible(true);
        // }

        if let Some(current_slide) = self.current_slide()
            && let Some(index) = self.slides().iter().position(|v| v.eq(&current_slide))
        {
            self.set_slides(slide.clone(), Some(index + 1));
        } else {
            self.set_slides(slide.clone(), None);
        }

        self.slideshow().add_child(&canvas);
        self.emit_new_slide_created(&slide);

        if undoable_action {
            self.set_current_slide(Some(slide.clone()));
        }

        slide.connect_visible_notify(
            // slide.connect_visible_changed(
            glib::clone!(
                #[weak(rename_to=sm)]
                self,
                // #[weak]
                // slide,
                move |slide| {
                    let visible = slide.visible();
                    if visible {
                        sm.set_current_slide(Some(slide.clone()));
                    } else {
                        let mut next_slide = sm.get_next_slide(&slide);

                        if next_slide.is_none() {
                            next_slide = sm.get_previous_slide(&slide);
                        }

                        if let Some(next_slide) = next_slide {
                            sm.set_current_slide(Some(next_slide));
                        }
                    }
                }
            ),
        );

        //
        slide
    }

    pub fn make_new_slide(&self) -> Slide {
        self.set_making_new_slide(true);
        let slide = self.new_slide(None, true);
        slide.reload_preview_data();
        self.set_current_slide(Some(slide.clone()));
        self.set_making_new_slide(false);
        slide
    }

    pub fn previous_slide(&self) {
        if let Some(s) = self.current_slide() {
            let previous_slide = self.get_previous_slide(&s);
            self.set_current_slide(previous_slide);
        }
    }

    pub fn next_slide(&self) {
        let end_slide = self.imp().end_presentation_slide.borrow().clone();

        // if let Some(end_slide) = end_slide.clone().canvas().clone()
        //     && let Some(child) = self.slideshow().visible_child()
        //     && child.eq(&end_slide)
        // {
        //     if let Some(win) = self.window() {
        //         win.set_is_presenting(false);
        //     }
        //     return;
        // }

        if let Some(next_slide) = self.current_slide().and_then(|i| self.get_next_slide(&i)) {
            self.set_current_slide(Some(next_slide));
        };
    }

    pub fn get_next_slide(&self, current: &Slide) -> Option<Slide> {
        let slides = self.slides();

        if slides.is_empty() {
            return None;
        }

        let current_index = slides
            .iter()
            .position(|s| s == current)
            .unwrap_or(slides.len());

        // Find the next visible slide after the current one
        if let Some(slide) = slides.iter().skip(current_index + 1).find(|s| s.visible()) {
            return Some(slide.clone());
        }

        // Some(self.imp().end_presentation_slide.borrow().clone())
        None
    }

    fn get_previous_slide(&self, current: &Slide) -> Option<Slide> {
        let slides = self.slides();

        if slides.is_empty() {
            return None;
        }

        let current_index = slides
            .iter()
            .position(|s| s == current)
            .unwrap_or(slides.len());

        // Find the next visible slide after the current one
        if let Some(slide) = slides
            .iter()
            .take(current_index)
            .rev()
            .find(|s| s.visible())
        {
            return Some(slide.clone());
        }

        None
    }

    pub fn move_down(&self, slide: &Slide) {
        if let Some(index) = self.slides().iter().position(|v| v.eq(slide))
            && let Some(next_slide) = self.get_next_slide(&slide)
            && let Some(next_index) = self.slides().iter().position(|v| v.eq(&next_slide))
        {
            self.set_slides(slide.clone(), Some(next_index));
            self.set_slides(next_slide, Some(index));

            self.emit_slides_sorted();
        }
    }

    pub fn move_up(&self, slide: &Slide) {
        if let Some(index) = self.slides().iter().position(|v| v.eq(slide))
            && let Some(prev_slide) = self.get_previous_slide(&slide)
            && let Some(prev_index) = self.slides().iter().position(|v| v.eq(&prev_slide))
        {
            self.set_slides(slide.clone(), Some(prev_index));
            self.set_slides(prev_slide, Some(index));

            self.emit_slides_sorted();
        }
    }

    pub fn get_slide_pos(&self, current: &Slide) -> Option<usize> {
        self.slides().iter().position(|v| v.eq(current))
    }

    pub fn request_new_item(&self, item_type: CanvasItemType) -> Option<CanvasItem> {
        let mut item: Option<CanvasItem> = None;

        let (Some(_current_slide), Some(canvas)) = (
            self.current_slide(),
            self.current_slide().and_then(|v| v.canvas()),
        ) else {
            eprintln!("Error requesting new item: could not get current slide canvas");
            return None;
        };

        if item_type == CanvasItemType::TEXT {
            item = Some(TextItem::new(Some(&canvas), None).upcast::<CanvasItem>());
            // } else if item_type == CanvasItemType::IMAGE {
            //     let file = FileManager::open_image();
            //     if let Some(file) = file {
            //         item = Some(ImageItem::from_file(Some(&canvas), &file).upcast::<CanvasItem>());
            //     }
            // } else if item_type == CanvasItemType::SHAPE {
            //     item = Some(ColorItem::new(Some(&canvas), None).upcast::<CanvasItem>());
        }

        if let Some(item) = item.clone()
            && let Some(current_slide) = self.current_slide()
            && let Some(canvas) = current_slide.canvas()
        {
            canvas.add_item(item, true);
        }

        return item;
    }

    pub fn jump_to_checkpoint(&self) {
        // if let Some(win) = self.window()
        //     && !win.is_presenting()
        // {
        //     return;
        // }

        if let Some(checkpoint) = self.imp().checkpoint.borrow().clone() {
            let temp = checkpoint;
            self.imp().checkpoint.replace(self.current_slide());
            self.set_current_slide(Some(temp));
        }
    }

    pub fn set_checkpoint(&self) {
        // if let Some(win) = self.window()
        //     && !win.is_presenting()
        // {
        //     return;
        // }

        self.imp().checkpoint.replace(self.current_slide());
    }

    pub fn end_presentation(&self) {
        let imp = self.imp();
        if let Some(current_slide) = self.current_slide()
            && current_slide.eq(&imp.end_presentation_slide.borrow().clone())
        {
            self.set_current_slide(self.get_previous_slide(&current_slide));
        }
    }

    pub fn show_end_presentation_slide(&self) {
        let end_presentation_slide = self.imp().end_presentation_slide.borrow().clone();
        self.set_current_slide(Some(end_presentation_slide));
    }

    pub fn move_up_request(&self) {
        let Some(current_slide) = self.current_slide() else {
            eprintln!("Error on move-up-request: Could not current_slide");
            return;
        };

        if let (Some(current_slide_canvas), Some(current_item)) =
            (current_slide.canvas(), self.current_item())
        {
            current_slide_canvas.move_up(&current_item, None);
        } else {
            self.move_up(&current_slide);
        }
    }

    pub fn move_down_request(&self) {
        let Some(current_slide) = self.current_slide() else {
            eprintln!("Error on move-down-request: Could not current_slide");
            return;
        };

        if let (Some(current_slide_canvas), Some(current_item)) =
            (current_slide.canvas(), self.current_item())
        {
            current_slide_canvas.move_down(&current_item, None);
        } else {
            self.move_down(&current_slide);
        }
    }

    pub fn slides(&self) -> Vec<Slide> {
        self.imp().slides.borrow().clone()
    }
    fn set_slides(&self, slide: Slide, position: Option<usize>) {
        let mut slides = self.imp().slides.borrow_mut();

        if let Some(position) = position {
            slides.insert(position, slide.clone());
        } else {
            slides.push(slide);
        }
    }
}
