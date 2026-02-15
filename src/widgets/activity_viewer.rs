use gtk::{
    gio::prelude::ListModelExt,
    glib::{self, object::ObjectExt, subclass::types::ObjectSubclassIsExt},
    prelude::{SelectionModelExt, WidgetExt},
};

use crate::{
    utils::{ListViewExtra, WidgetChildrenExt},
    widgets::canvas::{serialise::SlideManagerData, text_item::TextItem},
};

const MIN_GRID_WIDTH: i32 = 300;
const MIN_GRID_HEIGHT: i32 = 300;

mod signals {
    pub const ACTIVATE_SLIDE: &str = "activate-slide";
    pub const SLIDE_CHANGE: &str = "slide-change";
}

mod imp {
    use std::{
        cell::{Cell, RefCell},
        sync::OnceLock,
    };

    use gtk::{
        glib::{
            self, Properties,
            object::{Cast, CastNone},
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt},
            },
            types::StaticType,
        },
        prelude::{BoxExt, ListItemExt, ObjectExt, OrientableExt, SelectionModelExt, WidgetExt},
        subclass::{box_::BoxImpl, prelude::DerivedObjectProperties, widget::WidgetImpl},
    };

    use crate::{
        app_config::AppConfig,
        services::{slide::Slide, slide_manager::SlideManager},
        utils::{TextBufferExtraExt, WidgetExtrasExt},
        widgets::{activity_viewer::signals, canvas::serialise::SlideManagerData},
    };

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type=super::ActivityViewer)]
    pub struct ActivityViewer {
        pub slide_manager: RefCell<SlideManager>,
        pub(super) listview: RefCell<gtk::ListView>,

        //
        pub title_label: RefCell<gtk::Label>,
        #[property(get, set, construct)]
        pub title: RefCell<String>,
        pub clear: Cell<bool>,

        #[property(get, set)]
        pub background_image: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ActivityViewer {
        const NAME: &'static str = "ActivityViewer";
        type Type = super::ActivityViewer;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ActivityViewer {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj().clone();
            //
            obj.set_homogeneous(true);
            obj.set_orientation(gtk::Orientation::Vertical);
            obj.set_vexpand(true);
            obj.set_width_request(super::MIN_GRID_WIDTH);

            let listview = {
                let store_model = gtk::gio::ListStore::new::<Slide>();
                let selection_model = gtk::SingleSelection::new(Some(store_model));
                let factory = gtk::SignalListItemFactory::new();

                factory.connect_setup(move |_, list_item| {
                    let label = gtk::Label::builder()
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .wrap_mode(gtk::pango::WrapMode::Word)
                        // .lines(2)
                        .margin_top(12)
                        .margin_bottom(12)
                        .halign(gtk::Align::Start)
                        .justify(gtk::Justification::Fill)
                        .build();
                    label.set_margin_all(8);
                    label.set_height_request(40);
                    let li = list_item
                        .downcast_ref::<gtk::ListItem>()
                        .expect("Needs to be ListItem");

                    let view = gtk::Box::default();
                    view.append(&label);

                    li.set_child(Some(&view));
                });

                factory.connect_bind(move |_, list_item| {
                    let slide = list_item
                        .downcast_ref::<gtk::ListItem>()
                        .expect("Needs to be ListItem")
                        .item()
                        .and_downcast::<Slide>()
                        .expect("The item has to be an `Slide`.");

                    let view = list_item
                        .downcast_ref::<gtk::ListItem>()
                        .expect("Needs to be ListItem")
                        .child()
                        .and_downcast::<gtk::Box>()
                        .expect("The child has to be a `Box`.");

                    let child = view.first_child();
                    let label = child
                        .and_downcast::<gtk::Label>()
                        .expect("The child has to be a `Label`.");

                    if let Some(buf) = slide.entry_buffer() {
                        label.set_label(&buf.full_text());
                    }
                });

                let listview = gtk::ListView::new(Some(selection_model), Some(factory));

                listview.set_show_separators(true);

                listview
            };
            self.listview.replace(listview.clone());

            listview.connect_activate(glib::clone!(
                #[weak]
                obj,
                move |_, pos| {
                    // TODO:
                    // move slide manager to position
                    // send payload
                    // activated event

                    let sm = obj.imp().slide_manager.borrow();

                    let slides = sm.slides();
                    let Some(slide) = slides.get(pos as usize) else {
                        return;
                    };
                    sm.set_current_slide(Some(slide.clone()));

                    obj.emit_activate_slide();
                }
            ));

            if let Some(model) = listview.model() {
                model.connect_selection_changed(glib::clone!(
                    #[weak]
                    obj,
                    move |model, _, _| {
                        let Some(model) = model.downcast_ref::<gtk::SingleSelection>() else {
                            return;
                        };

                        let pos = model.selected();

                        let sm = obj.imp().slide_manager.borrow();
                        let slides = sm.slides();
                        let Some(slide) = slides.get(pos as usize) else {
                            return;
                        };

                        sm.set_current_slide(Some(slide.clone()));
                        obj.emit_slide_change(pos);
                    }
                ));
            }

            let list_viewer = {
                let base = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .hexpand(true)
                    .height_request(super::MIN_GRID_HEIGHT)
                    .css_classes(["pink_box", "ow-listview"])
                    .build();

                let scrolled = gtk::ScrolledWindow::builder()
                    .vexpand(true)
                    .child(&listview)
                    .build();

                let title_label = self.title_label.borrow().clone();
                title_label.set_halign(gtk::Align::Start);
                title_label.set_label(&self.title.borrow());
                base.append(&title_label);
                base.append(&scrolled);
                base
            };

            let screen = {
                let base = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                let aspect_frame = gtk::AspectFrame::builder()
                    .css_name("pink_box")
                    .height_request(super::MIN_GRID_HEIGHT)
                    .ratio(AppConfig::aspect_ratio())
                    .obey_child(false)
                    .build();

                aspect_frame.set_child(Some(&self.slide_manager.borrow().slideshow()));

                let frame = gtk::Box::builder()
                    .height_request(super::MIN_GRID_HEIGHT)
                    .hexpand(true)
                    .vexpand(true)
                    .build();
                frame.append(&aspect_frame);

                base.add_css_class("gray_bg_box");
                base.append(&frame);

                base
            };

            let paned = gtk::Paned::builder()
                .orientation(gtk::Orientation::Vertical)
                .shrink_end_child(false)
                .shrink_start_child(false)
                .start_child(&list_viewer)
                .end_child(&screen)
                .build();

            obj.append(&paned);

            self.slide_manager.borrow().show_end_presentation_slide();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::ACTIVATE_SLIDE)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                    Signal::builder(signals::SLIDE_CHANGE)
                        .param_types([u32::static_type()])
                        .build(),
                ]
            })
        }
    }
    impl WidgetImpl for ActivityViewer {}
    impl BoxImpl for ActivityViewer {}
}

glib::wrapper! {
pub struct ActivityViewer (ObjectSubclass<imp::ActivityViewer>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ActivityViewer {
    pub fn new(title: &str) -> Self {
        let obj: Self = glib::Object::builder().property("title", title).build();
        obj.imp()
            .slide_manager
            .borrow()
            .show_end_presentation_slide();

        obj.imp().title_label.borrow().set_label(title);
        obj.imp().title.replace(title.to_string());

        obj
    }

    pub fn load_data(&self, data: &SlideManagerData) {
        let imp = self.imp();
        let sm = imp.slide_manager.borrow();
        sm.set_title(data.title.clone());
        imp.title_label
            .borrow()
            .set_label(&format!("{} - {}", imp.title.borrow(), data.title));
        let listview = imp.listview.borrow();

        listview.remove_all();
        sm.reset();
        sm.load_data(data.clone());

        for slide in &sm.slides() {
            slide.load_slide();
            slide.set_presentation_mode(true);

            if let Some(canvas) = slide.canvas()
                && canvas.background_pattern().is_empty()
            {
                canvas.set_background_pattern(self.background_image());
                canvas.style();
            };

            listview.append_item(slide);
        }
        sm.set_current_slide(sm.slides().get(data.current_slide as usize));

        let Some(model) = listview.model() else {
            return;
        };

        if model.n_items() > 0 {
            model.select_item(data.current_slide, true);
            if let Some(child) = listview.children().nth(data.current_slide as usize) {
                listview.set_focus_child(Some(&child));
            }
        }

        self.clear_display(imp.clear.get());
    }

    pub fn connect_activate_slide<F: Fn(&Self, &SlideManagerData) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::ACTIVATE_SLIDE,
            false,
            glib::closure_local!(move |obj: &Self, slide_data: &SlideManagerData| {
                f(obj, slide_data);
            }),
        )
    }

    pub fn emit_activate_slide(&self) {
        let mut slide_data = self.imp().slide_manager.borrow().serialise();
        slide_data.title = self.imp().slide_manager.borrow().title();
        self.emit_by_name::<()>(signals::ACTIVATE_SLIDE, &[&slide_data]);
    }

    pub fn connect_slide_change<F: Fn(&Self, u32) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::SLIDE_CHANGE,
            false,
            glib::closure_local!(move |obj: &Self, position: u32| {
                f(obj, position);
            }),
        )
    }
    fn emit_slide_change(&self, position: u32) {
        self.emit_by_name::<()>(signals::SLIDE_CHANGE, &[&position]);
    }

    pub fn update_background(&self, img: String) {
        let imp = self.imp();
        let sm = imp.slide_manager.borrow();
        self.set_background_image(img);

        for slide in sm.slides() {
            let Some(canvas) = slide.canvas() else {
                continue;
            };
            canvas.set_background_pattern(self.background_image());
            canvas.style();
        }
    }

    pub fn clear_display(&self, clear: bool) {
        let imp = self.imp();
        let sm = imp.slide_manager.borrow();

        imp.clear.set(clear);
        for slide in sm.slides() {
            let Some(c) = slide.canvas() else {
                return;
            };

            for text in c.widget().get_children::<TextItem>() {
                text.set_visible(!clear);
            }
        }
    }
}
