use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclass;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::path::Path;

use std::cell::RefCell;
use std::str;

use crate::utils;
use crate::widgets::canvas::canvas::Canvas;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ImpCanvasGrid {
        pub canvas: glib::WeakRef<Canvas>,
        pub grid: RefCell<gtk::Grid>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImpCanvasGrid {
        const NAME: &'static str = "CanvasGrid";
        type Type = super::CanvasGrid;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ImpCanvasGrid {}
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
        let cg = glib::Object::new::<Self>();
        cg.imp().canvas.set(Some(&canvas));

        let clicked = gtk::GestureClick::new();
        clicked.set_propagation_phase(gtk::PropagationPhase::Bubble);
        clicked.connect_pressed(glib::clone!(
            #[weak(rename_to=cg)]
            cg,
            move |g, _, _, _| {
                println!("canvas_grid pressed");
                if let Some(c) = cg.imp().canvas.upgrade() {
                    c.emit_clicked(g);
                }
                g.set_state(gtk::EventSequenceState::Claimed);
            }
        ));

        cg.add_controller(clicked);

        let grid = gtk::Grid::new();
        grid.add_css_class("ow-pattern");
        cg.imp().grid.replace(grid.clone());

        cg.set_homogeneous(true);
        cg.set_css_classes(&["canvas", "view", "ow-canvas-grid"]);
        cg.set_vexpand(true);
        cg.set_hexpand(true);

        cg.append(&grid);

        cg
    }

    pub fn style(&self, pattern: String) {
        let grid = self.imp().grid.borrow().clone();

        let has_pattern =
            !pattern.is_empty() && !(pattern.starts_with("/") && !Path::new(&pattern).exists());

        let res = match has_pattern {
            true => Self::pattern_css(&pattern),
            false => Self::no_pattern_css(),
        };

        utils::set_style(&grid, &res);
    }

    fn pattern_css(path: &str) -> String {
        let url = if path.starts_with("/") { "file://" } else { "" };
        format!(
            r##".ow-pattern {{
                background-size: cover;
                background-position: center center;
                box-shadow: inset 0 0 0 2px alpha(#fff, 0.05);
                background-image: url("{url}{path}");
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
