use crate::dto::{SongData, SongVerse};
use crate::services::slide::Slide;
use crate::utils::{ListViewExtra, WidgetChildrenExt};
use crate::widgets::canvas::serialise::{CanvasItemType, SlideData};
use crate::widgets::canvas::text_item::TextItem;
use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::*;

use crate::{
    db::query::Query, dto::SongObject, utils::TextBufferExtraExt,
    widgets::search::songs::list_item::SongListItemModel,
};

const WIDTH: i32 = 900;

mod signals {
    pub const SAVE: &str = "save";
}
mod imp {
    use std::{
        cell::{Cell, RefCell},
        sync::OnceLock,
    };

    use gtk::{
        glib::{
            self, Properties,
            object::{Cast, CastNone},
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::{
            AccessibleExt, BoxExt, EntryExt, FrameExt, GtkWindowExt, ListItemExt, TextViewExt,
            WidgetExt,
        },
        subclass::{widget::WidgetImpl, window::WindowImpl},
    };

    use super::*;
    use crate::{
        services::slide_manager::SlideManager, utils::WidgetExtrasExt,
        widgets::search::songs::song_editor_toolbar::SongEditorToolbar,
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::SongEditWindow)]
    pub struct SongEditWindow {
        pub is_new_song: Cell<bool>,
        pub song: RefCell<Option<SongObject>>,
        pub screen: RefCell<gtk::Stack>,
        pub slide_manager: RefCell<SlideManager>,
        pub title_entry: RefCell<gtk::Entry>,

        // EditSongModalListItem
        pub list_view: RefCell<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SongEditWindow {
        const NAME: &'static str = "SongEditWindow";
        type Type = super::SongEditWindow;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for SongEditWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.set_default_width(WIDTH);
            obj.set_default_height(506);
            obj.set_modal(true);
            obj.set_focus_visible(true);
            obj.set_resizable(false);
            obj.set_accessible_role(gtk::AccessibleRole::Dialog);
            obj.add_css_class("dialog");
            //TODO: bind "is_active" to window "visible"

            let model = gtk::gio::ListStore::new::<Slide>();
            let selection_model = gtk::SingleSelection::new(Some(model));
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let tv = gtk::TextView::new();
                tv.set_margin_all(8);
                tv.set_height_request(40);
                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");

                // let group = gtk::Box::new(gtk::Orientation::Horizontal, 2);
                // let index_group = gtk::Box::new(gtk::Orientation::Horizontal, 2);
                // group.append(&index_group);
                // group.append(&tv);
                //
                // let pos = 1;
                // println!("pos={:?}", pos);
                // let label = gtk::Label::new(Some(&pos.to_string()));
                // index_group.append(&label);
                // index_group.append(&gtk::Image::from_icon_name("screen-symbolic"));
                li.set_child(Some(&tv));
            });

            factory.connect_bind(move |_, list_item| {
                let slide = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem")
                    .item()
                    .and_downcast::<Slide>()
                    .expect("The item has to be an `Slide`.");

                let textview = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem")
                    .child()
                    .and_downcast::<gtk::TextView>()
                    .expect("The child has to be a `TextView`.");

                if let Some(buf) = slide.entry_buffer() {
                    textview.set_buffer(Some(&buf));
                }
            });

            let listview = gtk::ListView::builder()
                .vexpand(true)
                .factory(&factory)
                .model(&selection_model)
                .css_name("blue_box")
                .build();
            self.list_view.replace(listview.clone());

            let box_ui = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .hexpand(true)
                .vexpand(true)
                .homogeneous(false)
                .build();
            obj.set_child(Some(&box_ui));

            let box_header = {
                let box_header = gtk::Box::builder()
                    .height_request(80)
                    .name("header-box")
                    .css_classes(["brown_box"])
                    .build();

                let entry_box = gtk::Box::builder()
                    .height_request(80)
                    .margin_top(13)
                    .margin_bottom(13)
                    .margin_start(6)
                    .margin_end(6)
                    .build();
                box_header.append(&entry_box);
                box_header.append(&gtk::Box::builder().hexpand(true).build());

                let title_label = gtk::Label::builder().label("Title").margin_end(6).build();

                let title_entry = gtk::Entry::new();
                self.title_entry.replace(title_entry.clone());
                title_entry.set_placeholder_text(Some("Enter song title"));
                // title_entry.connect_insert_text(move |l, m, n| {
                //     println!("song_title: \n{:?} \n{:?} \n{:?}", l, m, n);
                // });
                entry_box.append(&title_label);
                entry_box.append(&title_entry);

                box_header
            };
            box_ui.append(&box_header);

            box_ui.append(&SongEditorToolbar::new(&self.slide_manager.borrow()));

            let editor_box = {
                // EDITOR SECTION
                let editor_box = gtk::Box::builder().hexpand(true).vexpand(true).build();

                let pane = gtk::Paned::builder()
                    .position(WIDTH / 2)
                    .shrink_start_child(false)
                    .shrink_end_child(false)
                    .build();
                editor_box.append(&pane);

                let editor_frame = gtk::Frame::builder().vexpand(true).build();
                pane.set_start_child(Some(&editor_frame));

                let screen = self.slide_manager.borrow().slideshow();
                self.screen.replace(screen.clone());
                screen.set_margin_all(4);
                let aspect_frame = gtk::AspectFrame::builder()
                    .css_name("pink_box")
                    .ratio(16.0 / 9.0)
                    .obey_child(false)
                    .child(&screen)
                    .build();
                pane.set_end_child(Some(&aspect_frame));

                let frame_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .hexpand(true)
                    .build();
                editor_frame.set_child(Some(&frame_box));

                let slide_scrolled = gtk::ScrolledWindow::builder().vexpand(true).build();
                frame_box.append(&slide_scrolled);

                slide_scrolled.set_child(Some(&listview));

                let slide_footer_toolbox = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .build();
                frame_box.append(&slide_footer_toolbox);

                let add_slide_btn = gtk::Button::builder()
                    .tooltip_text("Add slide")
                    .icon_name("plus")
                    .build();
                add_slide_btn.add_css_class("flat");
                add_slide_btn.connect_clicked(glib::clone!(
                    #[weak]
                    obj,
                    move |_| obj.add_new_verse()
                ));

                let remove_slide_btn = gtk::Button::builder()
                    .tooltip_text("Remove slide")
                    .icon_name("minus")
                    .build();
                remove_slide_btn.add_css_class("flat");
                remove_slide_btn.connect_clicked(glib::clone!(
                    #[weak]
                    obj,
                    move |_| obj.remove_verse()
                ));

                slide_footer_toolbox.append(&add_slide_btn);
                slide_footer_toolbox.append(&remove_slide_btn);

                editor_box
            };
            box_ui.append(&editor_box);

            let footer_box = {
                // FOOTER
                let footer_box = gtk::Box::builder()
                    .height_request(50)
                    .css_name("brown_box")
                    .build();

                let footer_spacer_box = gtk::Box::builder().hexpand(true).build();
                footer_box.append(&footer_spacer_box);

                let footer_action_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                footer_box.append(&footer_action_box);

                let ok_btn = gtk::Button::with_label("Ok");
                let text_entry = obj.imp().title_entry.borrow().clone();
                ok_btn.connect_clicked(glib::clone!(
                    #[weak]
                    obj,
                    #[strong]
                    text_entry,
                    move |_| {
                        if text_entry.buffer().text().is_empty() {
                            let label =
                                gtk::Label::builder().label("Title cannot be empty").build();
                            let win = gtk::Window::builder()
                                .default_width(300)
                                .default_height(100)
                                .modal(true)
                                .focus_visible(true)
                                .child(&label)
                                .build();
                            win.set_visible(true);
                            return;
                        }

                        obj.ok_reponse();
                    }
                ));
                // TODO: implement ok

                let cancel_btn = gtk::Button::with_label("Cancel");
                cancel_btn.connect_clicked(glib::clone!(
                    #[weak]
                    obj,
                    move |_| {
                        // TODO:
                        // obj.cancel();
                        obj.close();
                    }
                ));
                footer_box.append(&ok_btn);
                footer_box.append(&cancel_btn);

                footer_box
            };
            box_ui.append(&footer_box);
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNAL: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNAL.get_or_init(|| {
                vec![
                    Signal::builder(signals::SAVE)
                        .param_types([SongObject::static_type()])
                        .build(),
                ]
            })
        }
    }
    impl WidgetImpl for SongEditWindow {}

    impl WindowImpl for SongEditWindow {}

    impl SongEditWindow {}
}

glib::wrapper! {
pub struct SongEditWindow(ObjectSubclass<imp::SongEditWindow>)
    @extends gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,gtk::Native,gtk::Root, gtk::ShortcutManager;
}

impl Default for SongEditWindow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SongEditWindow {
    pub fn new() -> Self {
        let obj: Self = glib::Object::new();
        obj.register_list_view_selection_changed();

        obj
    }

    pub fn show(&self, song: Option<SongObject>) {
        self.imp().song.replace(song.clone());

        if let Some(song) = song {
            self.load_song(&song);
            self.imp().is_new_song.set(false);
            self.set_title(Some("Edit Song"));
        } else {
            self.imp().is_new_song.set(true);
            self.add_new_verse();
            self.set_title(Some("Add Song"));
        }

        self.present();
    }

    pub fn hide(&self) {
        self.close();
    }

    pub fn add_new_verse(&self) {
        let listview = self.imp().list_view.borrow().clone();
        let sm = self.imp().slide_manager.borrow();
        let slide = sm.new_slide(Some(SlideData::from_default()), true);
        listview.append_item(&slide);

        if let Some(model) = listview.model() {
            if model.n_items() > 0 {
                model.select_item(model.n_items().saturating_sub(1), true);
            }

            if let Some(child) = listview.last_child() {
                child.grab_focus();
            }
        }
    }

    pub fn remove_verse(&self) {
        let listview = self.imp().list_view.borrow().clone();

        listview.remove_selected_items();
        if let Some(model) = listview.model() {
            if model.n_items() > 0 {
                model.select_item(model.n_items().saturating_sub(1), true);
            }

            if let Some(child) = listview.last_child() {
                child.grab_focus();
            }
        }
    }

    pub fn ok_reponse(&self) {
        let mut verses: Vec<SongVerse> = Vec::new();
        let imp = self.imp();

        let list_view = imp.list_view.borrow_mut().clone();
        let list_items = list_view.get_items();

        for item in list_items {
            let Some(slide) = item.downcast::<Slide>().ok() else {
                return;
            };
            let mut s_data = slide.serialise();
            for item in &mut s_data.items {
                match &mut item.item_type {
                    CanvasItemType::Text(text_item) => {
                        text_item.text_data = "".into();
                        break;
                    }
                    _ => (),
                }
            }
            let Some(slide_str) = serde_json::to_string(&s_data).ok() else {
                return;
            };

            let slide_str_data = {
                let d = serde_json::to_string(&SlideData::from_default())
                    .ok()
                    .unwrap_or_default();

                (d != slide_str).then_some(slide_str)
            };

            let Some(buff) = slide.entry_buffer() else {
                return;
            };

            verses.push(SongVerse::new(
                buff.full_text().into(),
                None,
                slide_str_data,
            ));
        }

        let title = imp.title_entry.borrow_mut().buffer();
        let song = SongData::new(
            match imp.song.borrow().clone() {
                Some(s) => s.song_id(),
                None => 0,
            },
            title.text().into(),
            verses,
        );
        let song_model = SongListItemModel::new(song.clone().into());

        let res = match imp.is_new_song.get() {
            true => Query::insert_song(song.into()),
            false => Query::update_song(song.into()),
        };
        let _ = match res {
            Ok(()) => (),
            Err(x) => println!("SQL ERROR: {:?}", x),
        };

        // NOTE: implement save signal
        self.emit_save(&song_model.song);

        list_view.remove_all();

        self.close();
    }
    pub fn cancel_reponse(&self) {
        self.close();
    }

    fn load_song(&self, song: &SongObject) {
        let listview = self.imp().list_view.borrow().clone();

        //
        let sm = self.imp().slide_manager.borrow();
        sm.reset();

        for verse in song.verses() {
            let mut s = match verse
                .slide
                .and_then(|v| serde_json::from_str::<SlideData>(&v).ok())
            {
                Some(v) => v,
                None => SlideData::from_default(),
            };

            for item in &mut s.items {
                match &mut item.item_type {
                    CanvasItemType::Text(text_item) => {
                        text_item.text_data = verse.text.clone();
                        break;
                    }
                    _ => (),
                }
            }

            let slide = sm.new_slide(Some(s), true);

            if let Some(canvas) = slide.canvas() {
                for t in canvas.widget().get_children::<TextItem>() {
                    let buff = t.buffer();
                    buff.set_text(&verse.text);
                    break;
                }
            }

            listview.append_item(&slide);
        }

        let title = song.title();
        self.imp().title_entry.borrow().set_text(&title);

        let Some(model) = listview.model() else {
            return;
        };

        if model.n_items() > 0 {
            let position = model.n_items().saturating_sub(1);
            model.select_item(position, true);
            if let Some(child) = listview.last_child() {
                child.grab_focus();
            }
        }
    }

    fn register_list_view_selection_changed(&self) {
        let list_view = self.imp().list_view.borrow();
        let Some(list_model) = list_view.model() else {
            return;
        };

        list_model.connect_selection_changed(glib::clone!(
            #[weak(rename_to=obj)]
            self,
            #[strong]
            list_view,
            move |_, _, _| {
                let list = list_view.clone();
                let item = list.get_selected_items().first().cloned();

                let slide = item.and_downcast::<Slide>();
                obj.imp().slide_manager.borrow().set_current_slide(slide);
            }
        ));

        list_model.unselect_all();
        if list_model.n_items() > 0 {
            list_model.select_item(1, true);
        }
    }

    fn emit_save(&self, song: &SongObject) {
        self.emit_by_name::<()>(signals::SAVE, &[song]);
    }

    pub fn connect_save<F: Fn(&Self, &SongObject) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::SAVE,
            false,
            glib::closure_local!(move |obj: &Self, song: &SongObject| f(obj, song)),
        )
    }
}
