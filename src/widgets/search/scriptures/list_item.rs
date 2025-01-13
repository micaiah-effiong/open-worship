use relm4::{gtk, typed_view::list::RelmListItem, view};

use crate::dto::Scripture;

#[derive(Debug, Clone)]
pub struct ScriptureListItem {
    pub data: Scripture,
}

pub struct ScriptureListItemWidget {
    text: gtk::Label,
}

impl Drop for ScriptureListItemWidget {
    fn drop(&mut self) {}
}

impl RelmListItem for ScriptureListItem {
    type Root = gtk::Box;
    type Widgets = ScriptureListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            list_box = gtk::Box {
                #[name="text"]
                gtk::Label {
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                }
            }
        }

        let widgets = ScriptureListItemWidget { text };

        return (list_box, widgets);
    }

    fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let text = format!(
            "{}:{} \t{}",
            self.data.chapter, self.data.verse, self.data.text
        );
        _widgets.text.set_label(&text);
    }
}
