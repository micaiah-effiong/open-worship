use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, typed_view::list::RelmListItem, view};

#[derive(Debug, Clone)]
pub struct ScheduleListItemModel {
    pub list: Vec<String>,
    pub title: String,
}

impl ScheduleListItemModel {
    pub fn new(title: String, list: Vec<String>) -> Self {
        return ScheduleListItemModel { title, list };
    }
}

pub struct ScheduleListItemWidget {
    pub label: gtk::Label,
    // pub note: Option<gtk::Label>,
}

impl Drop for ScheduleListItemWidget {
    fn drop(&mut self) {
        dbg!(self.label.label());
    }
}

impl RelmListItem for ScheduleListItemModel {
    type Root = gtk::Box;
    type Widgets = ScheduleListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            list_item_box = gtk::Box {
                set_margin_all: 6,

                gtk::Box {
                    set_height_request: 30,
                    set_width_request: 70,
                    set_margin_end: 10,
                    add_css_class: "schedule-list-item-preview"
                },

                gtk::Box{
                    set_orientation: gtk::Orientation::Vertical,

                    #[name="label"]
                    gtk::Label {
                        set_xalign: 0.0,
                    },

                    gtk::Box {
                        set_width_request: 50,
                        set_margin_start: 10,
                        add_css_class: "schedule-list-item-editable",

                        gtk::EditableLabel {
                            set_text: "No item"
                        }
                    }
                },
            }
        }

        let widgets = ScheduleListItemWidget { label };

        (list_item_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.label.set_label(&self.title);
    }
}
