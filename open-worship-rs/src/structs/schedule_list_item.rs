use relm4::{gtk, typed_view::list::RelmListItem, view};

pub struct ScheduleListItem {
    pub list: Vec<String>,
    pub title: String,
}

impl ScheduleListItem {
    pub fn new(title: String, list: Vec<String>) -> Self {
        return ScheduleListItem { title, list };
    }
}

pub struct ScheduleListItemWidget {
    pub label: gtk::Label,
}

impl Drop for ScheduleListItemWidget {
    fn drop(&mut self) {
        dbg!(self.label.label());
    }
}

impl RelmListItem for ScheduleListItem {
    type Root = gtk::Box;
    type Widgets = ScheduleListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            list_item_box = gtk::Box{
                #[name="label"]
                gtk::Label,
            }
        }

        let widgets = ScheduleListItemWidget { label };

        (list_item_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.label.set_label(&self.title);
    }
}
