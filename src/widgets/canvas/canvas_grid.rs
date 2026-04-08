use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclass;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::path::Path;

use std::cell::RefCell;
use std::str;

use crate::services::file_manager::FileManager;
use crate::utils::{self, WidgetExtrasExt};
use crate::widgets::canvas::canvas::Canvas;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ImpCanvasGrid {
        pub canvas: glib::WeakRef<Canvas>,
        pub grid: RefCell<gtk::Grid>,

        //
        pub(super) stack: RefCell<gtk::Stack>,
        pub(super) picture: RefCell<gtk::Picture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImpCanvasGrid {
        const NAME: &'static str = "CanvasGrid";
        type Type = super::CanvasGrid;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ImpCanvasGrid {
        fn dispose(&self) {
            self.picture.borrow().unparent();
            self.stack.borrow().unparent();
            self.grid.borrow().unparent();
            self.canvas.set(None);

            self.picture
                .borrow()
                .set_paintable(None::<&gtk::gdk::Paintable>);
        }
    }
    impl WidgetImpl for ImpCanvasGrid {}
    impl BoxImpl for ImpCanvasGrid {}

    impl ImpCanvasGrid {}
}

glib::wrapper! {
    pub struct CanvasGrid(ObjectSubclass<imp::ImpCanvasGrid>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CanvasGrid {
    pub fn new(canvas: Canvas) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        imp.canvas.set(Some(&canvas));

        let clicked = gtk::GestureClick::new();
        clicked.set_propagation_phase(gtk::PropagationPhase::Bubble);
        clicked.connect_pressed(glib::clone!(
            #[weak(rename_to=obj)]
            obj,
            move |g, _, _, _| {
                println!("canvas_grid pressed");
                if let Some(c) = obj.imp().canvas.upgrade() {
                    c.emit_clicked(g);
                }
                g.set_state(gtk::EventSequenceState::Claimed);
            }
        ));

        obj.add_controller(clicked);

        let grid = imp.grid.borrow().clone();
        grid.add_css_class("ow-pattern");
        grid.set_row_homogeneous(true);
        grid.set_column_homogeneous(true);
        obj.imp().grid.replace(grid.clone());

        obj.set_homogeneous(true);
        obj.set_css_classes(&["canvas", "view", "ow-canvas-grid"]);
        obj.set_expand(true);

        let stack = imp.stack.borrow().clone();
        grid.attach(&stack, 0, 0, 1, 1);
        let picture = imp.picture.borrow().clone();
        picture.set_content_fit(gtk::ContentFit::Cover);

        stack.add_named(&picture, Some("image"));
        stack.set_visible_child_name("image");

        obj.append(&grid.clone());

        obj
    }

    pub fn style(&self, pattern: String) {
        let grid = self.imp().grid.borrow().clone();

        let has_pattern = !pattern.is_empty() && Path::new(&pattern).exists();

        let res = match has_pattern {
            true => {
                let path = std::path::PathBuf::from(pattern);
                let picture = self.imp().picture.borrow().clone();
                FileManager::get_background_image(&path.clone(), None, move |v| {
                    picture.set_paintable(v.clone().as_ref());
                });
                Self::pattern_css()
            }
            false => Self::no_pattern_css(),
        };

        utils::set_style(&grid, &res);
    }

    fn pattern_css() -> String {
        format!(
            r##".ow-pattern {{
                background-size: cover;
                background-position: center center;
                box-shadow: inset 0 0 0 2px alpha(#ffffffff, 0.05);
                border-radius: 6px;
            }}"##
        )
    }

    fn no_pattern_css() -> String {
        ".ow-pattern {
            background-image: none;
        }"
        .to_string()
    }
}
