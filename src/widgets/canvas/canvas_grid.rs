use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclass;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use std::cell::RefCell;

use crate::widgets::canvas::canvas::Canvas;

const PATTERN_CSS: &str = "
.pattern {
    box-shadow: inset 0 0 0 2px alpha (#fff, 0.05);
    background-image: url(\"{}\");
    border-radius: 6px;
}
";

const NO_PATTERN_CSS: &str = "
.pattern {
    background-image: none;
}
";

mod imp {
    use std::path::Path;

    // use crate::services::utils;
    // use crate::widgets::canvas::Canvas;

    use crate::utils;

    use super::*;

    #[derive(Default)]
    pub struct ImpCanvasGrid {
        pub canvas: glib::WeakRef<Canvas>, // TODO: CanvasGrid
        pub grid: RefCell<gtk::Grid>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImpCanvasGrid {
        const NAME: &'static str = "CanvasGrid";
        type Type = super::CanvasGrid;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ImpCanvasGrid {
        // fn constructed(&self) {
        //     self.parent_constructed();
        //     let obj = self.obj();
        //
        // }
    }
    impl WidgetImpl for ImpCanvasGrid {}
    impl BoxImpl for ImpCanvasGrid {}

    impl ImpCanvasGrid {
        fn style(&self, pattern: String) {
            if pattern.is_empty() {
                utils::set_style(&self.grid.borrow().clone(), NO_PATTERN_CSS);
                return;
            }

            // TODO: Widgets.CanvasToolbar.PATTERNS_DIR
            if pattern.contains("") || Path::new(&pattern).exists() {
                utils::set_style(
                    &self.grid.borrow().clone(),
                    &PATTERN_CSS.replace("{}", &pattern),
                );
            } else {
                utils::set_style(&self.grid.borrow().clone(), NO_PATTERN_CSS);
            }
        }
    }
}

glib::wrapper! {
    pub struct CanvasGrid(ObjectSubclass<imp::ImpCanvasGrid>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

// impl Default for CanvasGrid {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl CanvasGrid {
    pub fn new(canvas: Canvas) -> Self {
        let cg = glib::Object::new::<Self>();
        cg.imp().canvas.set(Some(&canvas));

        let clicked = gtk::GestureClick::builder().name("CanvasGridClick").build();
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
        grid.add_css_class("pattern");

        cg.set_homogeneous(true);
        cg.set_css_classes(&["canvas", "view"]);
        cg.set_vexpand(true);
        cg.set_hexpand(true);

        cg.append(&grid);
        *cg.imp().grid.borrow_mut() = grid;

        cg
    }
}
