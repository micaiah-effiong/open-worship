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

const WIDTH: i32 = 1000;

mod signals {
    pub const SAVE: &str = "save";
}
mod imp {
    use std::{
        cell::{Cell, RefCell},
        sync::OnceLock,
    };

    use gtk::{
        gdk,
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
            AccessibleExt, BoxExt, EntryExt, GtkWindowExt, ListItemExt, TextViewExt, WidgetExt,
        },
        subclass::{widget::WidgetImpl, window::WindowImpl},
    };

    use super::*;
    use crate::{
        config::AppConfig, services::slide_manager::SlideManager, utils::WidgetExtrasExt,
        widgets::search::songs::toolbar::song_editor_toolbar::SongEditorToolbar,
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
            obj.set_default_height(562);
            obj.set_modal(true);
            obj.set_focus_visible(true);
            obj.set_resizable(false);
            obj.set_accessible_role(gtk::AccessibleRole::Dialog);
            obj.add_css_class("dialog");
            //TODO: bind "is_active" to window "visible"

            let model = gtk::gio::ListStore::new::<Slide>();
            let selection_model = gtk::SingleSelection::new(Some(model));
            let factory = gtk::SignalListItemFactory::new();

            let listview = gtk::ListView::builder()
                .vexpand(true)
                .factory(&factory)
                .model(&selection_model)
                .show_separators(true)
                .build();

            factory.connect_setup({
                let listview = listview.clone();
                let obj = obj.downgrade();
                move |_, list_item| {
                    let tv = gtk::TextView::new();
                    tv.set_margin_all(8);
                    tv.set_height_request(40);
                    let li = list_item
                        .downcast_ref::<gtk::ListItem>()
                        .expect("Needs to be ListItem");

                    li.set_child(Some(&tv));

                    if let Some(obj) = obj.upgrade() {
                        obj.imp().setup_key_controller(&tv, &listview);
                    };
                }
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

                textview.set_margin_all(0);

                if let Some(buf) = slide.entry_buffer() {
                    textview.set_buffer(Some(&buf));
                }
            });

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
                    .height_request(30)
                    .name("header-box")
                    .build();

                let entry_box = gtk::Box::builder()
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
                entry_box.append(&title_label);
                entry_box.append(&title_entry);

                box_header
            };
            box_ui.append(&box_header);
            box_ui.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
            box_ui.append(&SongEditorToolbar::new(&self.slide_manager.borrow()));
            box_ui.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

            let editor_box = {
                // EDITOR SECTION
                let editor_box = gtk::Box::builder().hexpand(true).vexpand(true).build();

                let pane = gtk::Paned::builder()
                    .position(WIDTH / 2)
                    .shrink_start_child(false)
                    .shrink_end_child(false)
                    .build();
                editor_box.append(&pane);

                let editor_frame = gtk::Box::builder().vexpand(true).build();
                editor_frame.set_size_request(300, -1);
                pane.set_start_child(Some(&editor_frame));

                let screen = self.slide_manager.borrow().slideshow();
                self.screen.replace(screen.clone());
                screen.set_margin_all(4);
                let aspect_frame = gtk::AspectFrame::builder()
                    .css_name("pink_box")
                    .ratio(AppConfig::aspect_ratio())
                    .obey_child(false)
                    .child(&screen)
                    .build();
                aspect_frame.set_size_request(300, -1);
                pane.set_end_child(Some(&aspect_frame));

                let frame_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .hexpand(true)
                    .build();
                editor_frame.append(&frame_box);

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
                    .height_request(36)
                    .margin_top(3)
                    .margin_bottom(3)
                    .margin_start(3)
                    .margin_end(3)
                    .spacing(3)
                    .build();

                let footer_spacer_box = gtk::Box::builder().hexpand(true).build();
                footer_box.append(&footer_spacer_box);

                let footer_action_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                footer_box.append(&footer_action_box);

                let text_entry = obj.imp().title_entry.borrow().clone();

                let notify_no_title = move |title: &gtk::Entry| {
                    if title.buffer().text().is_empty() {
                        let label = gtk::Label::builder().label("Title cannot be empty").build();
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
                };

                let save_change_btn = gtk::Button::with_label("Apply");
                save_change_btn.connect_clicked(glib::clone!(
                    #[weak]
                    obj,
                    #[strong]
                    text_entry,
                    move |_| {
                        notify_no_title(&text_entry);
                        obj.save_changes();
                    }
                ));

                let ok_btn = gtk::Button::with_label("Ok");
                ok_btn.connect_clicked(glib::clone!(
                    #[weak]
                    obj,
                    #[strong]
                    text_entry,
                    move |_| {
                        notify_no_title(&text_entry);
                        obj.ok_reponse();
                    }
                ));

                let cancel_btn = gtk::Button::with_label("Cancel");
                cancel_btn.connect_clicked(glib::clone!(
                    #[weak]
                    obj,
                    move |_| obj.cancel_reponse()
                ));
                footer_box.append(&save_change_btn);
                footer_box.append(&ok_btn);
                footer_box.append(&cancel_btn);

                footer_box
            };
            box_ui.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
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

    impl SongEditWindow {
        fn setup_key_controller(&self, tv: &gtk::TextView, listview: &gtk::ListView) {
            let obj = self.obj();

            let ctl_ent = gtk::EventControllerKey::new();
            // ctl_ent.set_propagation_phase(gtk::PropagationPhase::Bubble);
            ctl_ent.connect_key_pressed(glib::clone!(
                #[weak]
                obj,
                #[weak]
                listview,
                #[strong]
                tv,
                #[upgrade_or]
                glib::Propagation::Proceed,
                move |_, k, _, m| {
                    if k == gdk::Key::Return && m == gdk::ModifierType::SHIFT_MASK {
                        return Self::shift_enter_action(&obj, &listview);
                    }

                    if k == gdk::Key::Up || k == gdk::Key::Down {
                        let is_up = k == gdk::Key::Up;
                        return Self::direction_action(&tv, &listview, is_up);
                    }

                    return glib::Propagation::Proceed;
                }
            ));
            tv.add_controller(ctl_ent);
        }

        fn shift_enter_action(
            obj: &super::SongEditWindow,
            listview: &gtk::ListView,
        ) -> glib::Propagation {
            let Some(model) = listview.model().and_downcast::<gtk::SingleSelection>() else {
                return glib::Propagation::Proceed;
            };

            if model.selected() + 1 == model.n_items() {
                obj.add_new_verse();
            } else {
                Self::select_nth(listview, &model, model.selected() + 1);
            }

            glib::Propagation::Stop
        }

        fn direction_action(
            tv: &gtk::TextView,
            listview: &gtk::ListView,
            is_up: bool,
        ) -> glib::Propagation {
            let Some(model) = listview.model().and_downcast::<gtk::SingleSelection>() else {
                return glib::Propagation::Proceed;
            };

            let buffer = tv.buffer();
            let cursor_mark = buffer.get_insert();
            let cursor_iter = buffer.iter_at_mark(&cursor_mark);
            let end_iter = buffer.end_iter();

            let top = cursor_iter.line() == 0;
            let start = cursor_iter.offset() == 0;
            let bottom = cursor_iter.line() == end_iter.line();
            let end = cursor_iter.offset() == end_iter.offset();

            let at_top = top && start;
            let at_bottom = bottom && end;

            let selected = model.selected();

            if at_top && selected != 0 && is_up {
                Self::select_nth(listview, &model, selected.saturating_sub(1));
                return glib::Propagation::Stop;
            }

            if at_bottom && selected != model.n_items().saturating_sub(1) && !is_up {
                Self::select_nth(listview, &model, selected.saturating_add(1));
                return glib::Propagation::Stop;
            }

            glib::Propagation::Proceed
        }

        fn select_nth(list: &gtk::ListView, model: &impl IsA<gtk::SelectionModel>, position: u32) {
            model.select_item(position, true);
            if let Some(child) = list.children().nth(position as usize) {
                child.grab_focus();
            }
        }
    }
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
        self.save_changes();
        self.imp().list_view.borrow_mut().remove_all();
        self.close();
    }

    fn save_changes(&self) {
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

            let slide_str_data = (s_data != SlideData::from_default()).then_some(slide_str);

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
            Ok(()) => imp.is_new_song.set(false),
            Err(x) => println!("SQL ERROR: {:?}", x),
        };

        self.emit_save(&song_model.song);
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
