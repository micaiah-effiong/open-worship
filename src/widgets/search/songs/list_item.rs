use gtk::glib::value::ToValue;
use gtk::{glib, prelude::WidgetExt};
use relm4::{
    gtk::{self},
    typed_view::list::RelmListItem,
    view,
};

use crate::{
    dto::SongObject,
    widgets::canvas::serialise::{SlideData, SlideManagerData},
};

/// song search list item
#[derive(Debug, Clone)]
pub struct SongListItemModel {
    pub song: SongObject,
}

impl Into<SlideManagerData> for SongListItemModel {
    fn into(self) -> SlideManagerData {
        let slide_list = self
            .song
            .verses()
            .into_iter()
            .map(|s| {
                s.slide
                    .as_ref()
                    .and_then(|val| serde_json::from_str(val).ok())
                    .unwrap_or_else(SlideData::from_default)
            })
            .collect::<Vec<_>>();

        let mut sm_data = SlideManagerData::new(0, 0, slide_list);
        sm_data.title = self.song.title();
        sm_data
    }
}

impl SongListItemModel {
    pub fn new(song: SongObject) -> Self {
        SongListItemModel { song }
    }
}

pub struct SongListItemWidget {
    text: gtk::Label,
}

impl Drop for SongListItemWidget {
    fn drop(&mut self) {
        self.text.label();
    }
}

impl RelmListItem for SongListItemModel {
    type Root = gtk::Box;
    type Widgets = SongListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            list_box = gtk::Box {
                #[name="text"]
                gtk::Label {
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                }
            }
        }

        let widgets = SongListItemWidget { text };

        (list_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, root: &mut Self::Root) {
        let text = self.song.title().to_string();
        widgets.text.set_label(&text);

        {
            let drag_source = gtk::DragSource::new();
            drag_source.set_actions(gtk::gdk::DragAction::COPY);
            drag_source.connect_prepare(glib::clone!(
                #[strong(rename_to=obj)]
                self,
                move |_, _, _| {
                    let li = obj.clone();
                    let sm_data: SlideManagerData = li.into();
                    let content = gtk::gdk::ContentProvider::for_value(&sm_data.to_value());

                    return Some(content);
                }
            ));

            drag_source.connect_drag_begin(move |_, _| {
                // let item_text = item_text.to_string();
                // drag.set_icon_name(Some("document-properties"), 0, 0);
            });

            root.add_controller(drag_source);
        }
    }
}
