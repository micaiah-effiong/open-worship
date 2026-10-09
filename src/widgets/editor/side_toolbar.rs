use gtk::glib;

use crate::services::slide_manager::SlideManager;

mod canvas_tool;
mod frame_tool;
mod text_tool;

mod imp {
    use super::*;
    use std::cell::RefCell;

    use crate::{
        services::slide_manager::SlideManager,
        widgets::{
            canvas::canvas_item::CanvasItem,
            editor::side_toolbar::{
                canvas_tool::CanvasTool, frame_tool::FrameTool, text_tool::TextTool,
            },
        },
    };

    use gtk::{
        glib::{
            Properties,
            object::CastNone,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::{BoxExt, ObjectExt, OrientableExt, WidgetExt},
        subclass::{box_::BoxImpl, prelude::DerivedObjectProperties, widget::WidgetImpl},
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::SideToolBar)]
    pub struct SideToolBar {
        #[property(get, construct_only)]
        pub(super) slide_manager: RefCell<SlideManager>,
        // pub(super) slide_manager: glib::WeakRef<SlideManager>,
        pub(super) frame_tool: RefCell<FrameTool>,
        pub(super) text_tool: RefCell<TextTool>,
        pub(super) canvas_tool: RefCell<CanvasTool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SideToolBar {
        const NAME: &'static str = "SideToolBar";
        type Type = super::SideToolBar;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SideToolBar {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj().clone();
            obj.set_orientation(gtk::Orientation::Vertical);
            obj.set_css_classes(&["small-pad"]);
            obj.set_spacing(7);

            let sm = self.slide_manager.borrow().clone();

            let a = [
                Self::build_frame_section(self, &sm),
                Self::build_text_section(self, &sm),
                Self::build_canvas_section(self, &sm),
            ];
            for (i, widget) in a.iter().enumerate() {
                if i == 0 {
                    widget.set_expanded(true);
                }
                obj.append(widget);
            }

            self.register_update_props(&sm);
        }
    }
    impl WidgetImpl for SideToolBar {}
    impl BoxImpl for SideToolBar {}

    impl SideToolBar {
        fn build_text_section(imp: &Self, slide_manager: &SlideManager) -> gtk::Expander {
            let ex = gtk::Expander::new(Some("Text"));
            let text_tool = TextTool::new(slide_manager);
            ex.set_child(Some(&text_tool));
            imp.text_tool.replace(text_tool);

            ex
        }

        fn build_canvas_section(imp: &Self, slide_manager: &SlideManager) -> gtk::Expander {
            let ex = gtk::Expander::new(Some("Canvas"));
            let canvas_tool = CanvasTool::new(slide_manager);
            ex.set_child(Some(&canvas_tool));
            imp.canvas_tool.replace(canvas_tool);

            ex
        }

        fn build_frame_section(imp: &Self, slide_manager: &SlideManager) -> gtk::Expander {
            let ex = gtk::Expander::new(Some("Frame"));
            let frame_tool = FrameTool::new(slide_manager);
            ex.set_child(Some(&frame_tool));
            imp.frame_tool.replace(frame_tool);

            ex
        }

        fn register_update_props(&self, sm: &SlideManager) {
            sm.connect_item_clicked(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |sm, _| {
                    let ci = sm.current_item().and_downcast::<CanvasItem>();

                    imp.frame_tool.borrow().update_props(&ci, sm);
                    imp.text_tool.borrow().update_props(&ci, sm);
                }
            ));

            sm.connect_current_slide_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |sm, _| {
                    let ci = sm.current_item().and_downcast::<CanvasItem>();
                    imp.frame_tool.borrow().update_props(&ci, sm);
                    imp.canvas_tool.borrow().update_props(&ci, sm);
                }
            ));

            // self.frame_tool.borrow().update_props(&ci, &sm);
            // self.text_tool.borrow().update_props(&ci, &sm);
            // self.canvas_tool.borrow().update_props(&ci, &sm);
        }
    }
}

glib::wrapper! {
pub struct SideToolBar(ObjectSubclass<imp::SideToolBar>)
@extends gtk::Box, gtk::Widget,
@implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable
    ;
}

impl SideToolBar {
    pub fn new(sm: &SlideManager) -> Self {
        let obj: Self = glib::Object::builder()
            .property("slide-manager", sm)
            .build();

        obj
    }
}
