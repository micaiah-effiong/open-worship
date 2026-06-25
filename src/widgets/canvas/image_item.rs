use gtk::gio;
use gtk::glib;
use gtk::glib::subclass::prelude::*;

use crate::widgets::canvas::canvas::Canvas;
use crate::widgets::canvas::canvas_item::CanvasItem;
use crate::widgets::canvas::canvas_item::CanvasItemExt;
use crate::widgets::canvas::serialise::CanvasItemData;
use crate::widgets::canvas::serialise::ImageItemData;

pub(super) const IMAGE_MISSING_CSS: &str = "
.missing-image {
   border: 4px dashed #000000;
   border-color: #c92e34;
}";

mod imp {

    use std::cell::RefCell;

    use gtk::{
        Picture, gdk,
        gio::prelude::FileExt,
        glib::{
            Properties,
            object::{Cast, CastNone},
        },
        prelude::{ObjectExt, WidgetExt},
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use crate::{
        services::file_manager::FileManager,
        utils::{self, WidgetExtrasExt},
        widgets::canvas::{
            canvas_item::{CanvasItem, CanvasItemImpl},
            serialise::CanvasItemType,
        },
    };

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ImageItem)]
    pub struct ImageItem {
        pub image: RefCell<Picture>,
        #[property(get, set, nullable)]
        paintable: RefCell<Option<gtk::gdk::Paintable>>,
        pub(super) image_url: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImageItem {
        const NAME: &'static str = "ImageItem";
        type Type = super::ImageItem;
        type ParentType = CanvasItem;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ImageItem {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj().clone();

            utils::set_style(&obj, IMAGE_MISSING_CSS);

            let img = self.image.borrow().clone();
            img.set_sensitive(false);
            img.set_margin_all(6);
            obj.add_overlay_child(&img);
        }

        fn dispose(&self) {
            self.image.borrow().unparent();
            self.image
                .borrow()
                .set_paintable(None::<&gtk::gdk::Paintable>);
            self.paintable.replace(None::<gdk::Paintable>);
            println!("[DISPOSE] image item");
        }
    }
    impl WidgetImpl for ImageItem {}
    impl BoxImpl for ImageItem {}

    impl CanvasItemImpl for ImageItem {
        fn load_item_data(&self) {
            let ci = self.obj().clone().upcast::<CanvasItem>();
            let Some(data) = ci.imp().save_data.borrow().clone() else {
                return;
            };

            let json_data = match serde_json::from_str::<CanvasItemData>(&data) {
                Ok(d) => d,
                Err(e) => {
                    glib::g_log!(
                        "ImageItem",
                        glib::LogLevel::Error,
                        "Could not parse loaded data: {:?}",
                        e
                    );
                    return;
                }
            };

            let image_item_data = match &json_data.item_type {
                CanvasItemType::Image(image_item_data) => image_item_data,
                _ => return,
            };

            self.image_url.replace(image_item_data.url.clone());
            self.set_image_url(image_item_data.url.clone(), || {});
            self.connect_image();
        }

        fn serialise_item(&self) -> CanvasItemType {
            CanvasItemType::Image(ImageItemData {
                url: self.image_url.borrow().clone(),
            })
        }

        fn style(&self) {
            let image = self.image.borrow().clone();
            let obj = self.obj().clone();
            let paintable = obj.paintable();

            if paintable.is_some() {
                image.set_paintable(paintable.as_ref());
                self.obj().remove_css_class("missing-image");
            } else {
                self.unstyle();
            }
        }
    }

    impl ImageItem {
        fn unstyle(&self) {
            self.obj().add_css_class("missing-image");
        }

        pub(super) fn connect_image(&self) {
            self.obj().connect_paintable_notify(glib::clone!(
                #[weak(rename_to=img_item)]
                self,
                move |_| {
                    img_item.unstyle();
                    img_item.style();
                }
            ));
        }

        pub(super) fn register_change_image(&self) {
            let obj = self.obj().clone();

            obj.connect_double_clicked(move |obj| {
                let Some(obj) = obj.downcast_ref::<super::ImageItem>() else {
                    return;
                };
                let Some(file) = FileManager::open_image(obj.toplevel_window().as_ref()) else {
                    return;
                };

                let Some(file_path) = file
                    .path()
                    .and_then(|v| v.into_os_string().into_string().ok())
                else {
                    return;
                };

                obj.imp().set_image_url(file_path.into(), {
                    let obj = obj.clone();
                    move || obj.style()
                });
            });
        }

        fn set_image_url<T: FnOnce() + 'static>(&self, value: String, cb: T) {
            self.image_url.replace(value.clone());

            if value.is_empty() {
                return;
            }

            FileManager::get_background_image(
                std::path::Path::new(&value),
                None,
                glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |v| {
                        imp.obj().set_paintable(v.and_upcast::<gdk::Paintable>());
                        cb();
                    }
                ),
            );
        }
    }
}

glib::wrapper! {
    pub struct ImageItem(ObjectSubclass<imp::ImageItem>)
        @extends CanvasItem, gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ImageItem {
    fn init(canvas: Option<&Canvas>, save_data: Option<CanvasItemData>) -> Self {
        let data = match save_data {
            Some(d) => serde_json::to_string(&d).ok(),
            None => None,
        };

        let obj: Self = glib::Object::builder()
            .property("canvas", canvas)
            .property("save-data", data)
            .build();

        obj.imp().register_change_image();

        obj
    }

    pub fn new(canvas: Option<&Canvas>, save_data: Option<CanvasItemData>) -> Self {
        let obj = Self::init(canvas, save_data);
        obj.load_data();

        if canvas.is_some() {
            obj.style();
        }

        obj
    }

    pub fn from_file(canvas: Option<&Canvas>, file: &gio::File) -> Self {
        let obj = Self::init(canvas, None);

        obj.imp().image.replace(gtk::Picture::for_file(file));
        obj.imp().connect_image();

        if canvas.is_some() {
            obj.style();
        }

        obj
    }
}
