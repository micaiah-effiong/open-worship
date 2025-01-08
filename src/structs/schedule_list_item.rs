use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, typed_view::list::RelmListItem, view};

#[derive(Debug, Clone)]
pub struct ScheduleListItemModel {
    pub list: Vec<String>,
    pub title: String,
    pub backgound_image: Option<String>,
}

impl ScheduleListItemModel {
    pub fn new(title: String, list: Vec<String>, bg: Option<String>) -> Self {
        return ScheduleListItemModel {
            title,
            list,
            backgound_image: bg,
        };
    }

    fn format_bg_style(image: &str) -> String {
        let mut style = format!(
            "background-size: cover; background-position: center center; background-color: black;",
        );

        if !image.is_empty() {
            let bg_image_style = format!("background-image: url(\"file://{}\");", image);
            style = style + &bg_image_style;
        }

        return style;
    }
}

pub struct ScheduleListItemWidget {
    pub label: gtk::Label,
    pub preview_box: gtk::Box,
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

                #[name="preview_box"]
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

        let widgets = ScheduleListItemWidget { label, preview_box };

        return (list_item_box, widgets);
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.label.set_label(&self.title);

        if let Some(bg) = &self.backgound_image {
            widgets.preview_box.inline_css(&Self::format_bg_style(&bg));
        }
    }
}
