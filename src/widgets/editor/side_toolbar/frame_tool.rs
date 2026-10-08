use gtk::glib::{self, object::ObjectExt, subclass::types::ObjectSubclassIsExt};

use crate::{
    services::slide_manager::SlideManager,
    widgets::canvas::canvas_item::{CanvasItem, CanvasItemExt},
};

mod imp {
    use super::*;
    use std::cell::{Cell, RefCell};

    use glib::{
        self, Properties,
        subclass::{object::ObjectImpl, types::ObjectSubclass},
    };
    use gtk::{
        glib::subclass::{object::ObjectImplExt, types::ObjectSubclassExt},
        prelude::{BoxExt, GridExt, ObjectExt, OrientableExt, WidgetExt},
        subclass::{box_::BoxImpl, prelude::DerivedObjectProperties, widget::WidgetImpl},
    };

    use crate::{
        services::slide_manager::SlideManager, utils, widgets::ow_spinbutton::OwSpinButton,
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::FrameTool)]
    pub struct FrameTool {
        #[property(get, construct_only)]
        slide_manager: glib::WeakRef<SlideManager>,

        pub(super) x_spin: RefCell<OwSpinButton>,
        pub(super) y_spin: RefCell<OwSpinButton>,
        pub(super) w_spin: RefCell<OwSpinButton>,
        pub(super) h_spin: RefCell<OwSpinButton>,
        pub(super) is_setting_position: Cell<bool>,

        pub(super) item_signal_id: RefCell<Option<(CanvasItem, glib::SignalHandlerId)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FrameTool {
        const NAME: &'static str = "FrameTool";
        type Type = super::FrameTool;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for FrameTool {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj().clone();
            obj.set_orientation(gtk::Orientation::Vertical);
            obj.set_spacing(5);

            let info_grid = gtk::Grid::builder()
                .css_classes(["small-pad"])
                .row_spacing(7)
                .column_spacing(3)
                .build();

            obj.append(&info_grid);

            let x_label = gtk::Label::new(Some("x: "));
            let y_label = gtk::Label::new(Some("y: "));
            let w_label = gtk::Label::new(Some("w: "));
            let h_label = gtk::Label::new(Some("h: "));

            let x_spin = OwSpinButton::with_range(f64::MIN, f64::MAX, 1.0);
            x_spin.connect_value_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |spin| imp.handle_spin_btn_change_value(spin, |r| &mut r.x)
            ));

            let y_spin = OwSpinButton::with_range(f64::MIN, f64::MAX, 1.0);
            y_spin.connect_value_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |spin| imp.handle_spin_btn_change_value(spin, |r| &mut r.y)
            ));

            let w_spin = OwSpinButton::with_range(f64::MIN, f64::MAX, 1.0);
            w_spin.connect_value_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |spin| imp.handle_spin_btn_change_value(spin, |r| &mut r.width)
            ));

            let h_spin = OwSpinButton::with_range(f64::MIN, f64::MAX, 1.0);
            h_spin.connect_value_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |spin| imp.handle_spin_btn_change_value(spin, |r| &mut r.height)
            ));

            {
                let spins = [&x_spin, &y_spin, &w_spin, &h_spin];
                for s in spins {
                    s.set_hexpand(true); // take the extra space
                    s.set_halign(gtk::Align::Fill); // default, stated for clarity
                }
            }

            y_label.set_margin_start(10);
            h_label.set_margin_start(10);

            // column row width height
            info_grid.attach(&x_label, 0, 0, 1, 1);
            info_grid.attach(&x_spin, 1, 0, 1, 1);
            info_grid.attach(&y_label, 2, 0, 1, 1);
            info_grid.attach(&y_spin, 3, 0, 1, 1);

            info_grid.attach(&w_label, 0, 1, 1, 1);
            info_grid.attach(&w_spin, 1, 1, 1, 1);
            info_grid.attach(&h_label, 2, 1, 1, 1);
            info_grid.attach(&h_spin, 3, 1, 1, 1);

            self.x_spin.replace(x_spin);
            self.y_spin.replace(y_spin);
            self.w_spin.replace(w_spin);
            self.h_spin.replace(h_spin);
        }
    }

    impl WidgetImpl for FrameTool {}
    impl BoxImpl for FrameTool {}

    impl FrameTool {
        fn handle_spin_btn_change_value<T: FnOnce(&mut utils::rect::Rect) -> &mut i32>(
            &self,
            spin: &OwSpinButton,
            callback: T,
        ) {
            let imp = self;

            if imp.is_setting_position.get() {
                return;
            }

            let Some(ci) = self.obj().slide_manager().and_then(|v| v.current_item()) else {
                return;
            };

            imp.is_setting_position.set(true);
            let mut r = ci.rectangle();
            *callback(&mut r) = spin.value() as i32;
            ci.set_rectangle(r);
            imp.is_setting_position.set(false);
        }
    }
}

glib::wrapper! {
pub struct FrameTool(ObjectSubclass<imp::FrameTool>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for FrameTool {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl FrameTool {
    pub fn new(sm: &SlideManager) -> Self {
        glib::Object::builder()
            .property("slide_manager", sm.clone())
            .build()
    }

    pub fn update_props(&self, ci: &CanvasItem, _: &SlideManager) {
        let imp = self.imp();

        if let Some((cci, signal)) = imp.item_signal_id.take() {
            cci.disconnect(signal);
        }

        let r = ci.rectangle();
        imp.x_spin.borrow().set_value(r.x as f64);
        imp.y_spin.borrow().set_value(r.y as f64);
        imp.w_spin.borrow().set_value(r.width as f64);
        imp.h_spin.borrow().set_value(r.height as f64);

        let signal = ci.connect_checkposition(glib::clone!(
            #[weak]
            imp,
            move |ci| {
                // let dx = ci.imp().delta_x.get();
                // let dy = ci.imp().delta_y.get();
                // println!("dx => {dx}, dy => {dy}",);

                if !imp.is_setting_position.get() {
                    let r = ci.rectangle();
                    // println!("MOVEING {r:?}");

                    imp.x_spin.borrow().set_value(r.x as f64);
                    imp.y_spin.borrow().set_value(r.y as f64);
                    imp.w_spin.borrow().set_value(r.width as f64);
                    imp.h_spin.borrow().set_value(r.height as f64);
                }
            }
        ));

        imp.item_signal_id.replace(Some((ci.clone(), signal)));
    }
}
