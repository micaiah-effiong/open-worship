use relm4::{gtk, typed_view::list::RelmListItem, view};

#[derive(Debug, Clone)]
pub struct ScriptureListItem {
    pub book: String,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
}

impl ScriptureListItem {
    pub fn screen_display(&self) -> String {
        let text = format!(
            "{}\n{} {}:{}",
            self.text, self.book, self.chapter, self.verse,
        );
        return text;
    }
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
        let text = format!("{}:{} \t{}", self.chapter, self.verse, self.text);
        _widgets.text.set_label(&text);
    }
}
