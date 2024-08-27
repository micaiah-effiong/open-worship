use gtk::prelude::{OrientableExt, WidgetExt};
use relm4::{gtk, typed_view::grid::RelmGridItem, view};

#[derive(Debug, Clone)]
pub struct BackgroundGridListItem {
    pub title: String,
    pub src: String,
}

pub struct BackgroundGridListItemWidget {
    pub title_label: gtk::Label,
    pub image: gtk::Picture, // will use image soon
}

impl Drop for BackgroundGridListItemWidget {
    fn drop(&mut self) {
        self.title_label.label();
    }
}

impl BackgroundGridListItem {
    pub fn new(src: String, title: Option<String>) -> Self {
        if let Some(title) = title {
            return BackgroundGridListItem { src, title };
        }

        let name = match std::path::Path::new(&src).file_name() {
            Some(name) => name.to_str(),
            None => panic!("Invalid file name"),
        };
        let title = match name {
            Some(name) => name.to_string(),
            None => panic!("Error converting file name to string"),
        };

        return BackgroundGridListItem { src, title };
    }
}

impl RelmGridItem for BackgroundGridListItem {
    type Root = gtk::Box;
    type Widgets = BackgroundGridListItemWidget;

    fn setup(_grid_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            bg_grid_li_view = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                // set_css_classes: &["border", "border-black"],

                #[name="bg_picture"]
                gtk::Picture {
                    add_css_class: "bg-preview-box",
                    set_height_request: 70,
                    set_width_request: 70,
                    set_hexpand: true,
                    set_vexpand: true,
                },

                #[name="title_label"]
                gtk::Label{
                    set_vexpand: false,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    // set_css_classes: &["border", "border-black"]
                }
            }
        }

        let widgets = BackgroundGridListItemWidget {
            title_label,
            image: bg_picture,
        };

        return (bg_grid_li_view, widgets);
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.title_label.set_label(&self.title);
        widgets.image.set_filename(Some(&self.src));
    }
}
