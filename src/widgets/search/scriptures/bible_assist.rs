use gtk::glib::{self, object::ObjectExt, subclass::types::ObjectSubclassIsExt};

use crate::{
    utils::ListViewExtra,
    widgets::{canvas::serialise::SlideManagerData, search::scriptures::ScriptureVerseRangeObject},
};

// const URL: &str = "https://us.aws.cdn.hf.co/xet-bridge-us/641ab5d15d107c5c5f346372/edd29d67e70b000132af65205b99bb774b77abc13d10103e14f80ce2242913e1?user_id=public&response-content-disposition=attachment%3B+filename*%3DUTF-8%27%27ggml-small.bin%3B+filename%3D%22ggml-small.bin%22%3B&X-Xet-Cas-Uid=public&response-content-type=application%2Foctet-stream&Expires=1787920820&Policy=eyJTdGF0ZW1lbnQiOlt7IlJlc291cmNlIjoiaHR0cHM6Ly91cy5hd3MuY2RuLmhmLmNvL3hldC1icmlkZ2UtdXMvNjQxYWI1ZDE1ZDEwN2M1YzVmMzQ2MzcyL2VkZDI5ZDY3ZTcwYjAwMDEzMmFmNjUyMDViOTliYjc3NGI3N2FiYzEzZDEwMTAzZTE0ZjgwY2UyMjQyOTEzZTFcXD91c2VyX2lkPXB1YmxpYyZyZXNwb25zZS1jb250ZW50LWRpc3Bvc2l0aW9uPWF0dGFjaG1lbnQlM0IrZmlsZW5hbWUlMkElM0RVVEYtOCUyNyUyN2dnbWwtc21hbGwuYmluJTNCK2ZpbGVuYW1lJTNEJTIyZ2dtbC1zbWFsbC5iaW4lMjIlM0ImWC1YZXQtQ2FzLVVpZD1wdWJsaWMmcmVzcG9uc2UtY29udGVudC10eXBlPWFwcGxpY2F0aW9uJTJGb2N0ZXQtc3RyZWFtIiwiQ29uZGl0aW9uIjp7IkRhdGVMZXNzVGhhbiI6eyJFcG9jaFRpbWUiOjE3ODc5MjA4MjB9fX1dfQ__&Signature=MEUCIQC7w23hR3uKMtGliFpW9DRDqTKgOISQTeGHiMUMO7waEQIgChi6QylNO6acf2lHVNLRkdfoVV9-yoaPpBQgP608Qf0_&Key-Pair-Id=01KXEF4KZ1B6FV465MAWR4M21F";
const URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin?download=true";

mod signals {
    pub(super) const SEND_TO_PREVIEW: &str = "send-to-preview";
}

mod imp {
    use std::{
        cell::RefCell,
        collections::HashMap,
        fs,
        sync::{OnceLock, mpsc},
    };

    use futures_util::stream::AbortHandle;
    use gtk::{
        gio::{self, prelude::ListModelExt},
        glib::{
            self,
            object::{Cast, CastNone},
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt},
            },
            types::StaticType,
        },
        prelude::{BoxExt, ButtonExt, GtkWindowExt, ListItemExt, WidgetExt},
        subclass::{
            widget::WidgetImpl,
            window::{WindowImpl, WindowImplExt},
        },
    };

    use crate::{
        app_config::AppConfigDir,
        db::query::Query,
        dto::ScriptureVerseRange,
        parser::parser::Expression,
        services::audio::{
            AudioSignal,
            transcibe::{Stt, TranscriptParserEvent},
        },
        utils::ListViewExtra,
        widgets::{
            canvas::serialise::SlideManagerData,
            search::scriptures::{
                ScriptureVerseRangeObject,
                bible_assist::signals,
                download::{self, utils::DownloadStatus},
            },
        },
    };

    mod assist_pages {
        pub(super) const LIST: &str = "list";
        pub(super) const EMPTY: &str = "empty";
    }

    #[derive(Default)]
    pub struct BibleAssist {
        pub(super) list_view: RefCell<gtk::ListView>,
        pub(super) stack: RefCell<gtk::Stack>,
        pub(super) progress_bar: RefCell<gtk::ProgressBar>,
        pub(super) lable: RefCell<gtk::Label>,
        pub(super) cancel_hanlder: RefCell<Option<AbortHandle>>,
        pub(super) audio_assist_channel: RefCell<Option<mpsc::Sender<AudioSignal>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BibleAssist {
        const NAME: &'static str = "BibleAssist";
        type Type = super::BibleAssist;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for BibleAssist {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj().clone();
            obj.set_deletable(true);
            obj.set_default_size(300, 300);

            let list_view = {
                let model = gio::ListStore::new::<ScriptureVerseRangeObject>();
                let selection = gtk::SingleSelection::new(Some(model.clone()));
                let factory = gtk::SignalListItemFactory::new();

                factory.connect_setup(|_, list_item| {
                    let list_item = list_item
                        .downcast_ref::<gtk::ListItem>()
                        .expect("Expected ListItem");

                    let label = gtk::Label::builder().build();
                    list_item.set_child(Some(&label));
                });

                factory.connect_bind(|_, list_item| {
                    let list_item = list_item
                        .downcast_ref::<gtk::ListItem>()
                        .expect("Expected ListItem");

                    let item = list_item
                        .item()
                        .and_downcast::<ScriptureVerseRangeObject>()
                        .expect("Expected ScriptureVerseRangeObject");
                    let child = list_item
                        .child()
                        .and_downcast::<gtk::Label>()
                        .expect("Expected Label");

                    child.set_label(&item.note());
                });

                gtk::ListView::new(Some(selection), Some(factory))
            };

            let scroll_box = {
                let scroll = gtk::ScrolledWindow::new();
                scroll.set_hexpand(true);
                scroll.set_vexpand(true);
                scroll.set_child(Some(&list_view));
                self.list_view.replace(list_view);
                let bbox = gtk::Box::new(gtk::Orientation::Vertical, 2);

                bbox
            };

            let stack = gtk::Stack::new();
            let frame = {
                let frame = gtk::Box::new(gtk::Orientation::Vertical, 4);
                frame.set_vexpand(true);
                frame.set_valign(gtk::Align::Center);

                let label = self.lable.borrow().clone();
                label.set_text("No model available");
                frame.append(&label);

                let progress = self.progress_bar.borrow().clone();
                progress.set_visible(false);
                progress.set_margin_start(20);
                progress.set_margin_end(20);
                frame.append(&progress);

                let download_btn = gtk::Button::with_label("Download");
                download_btn.set_halign(gtk::Align::Center);
                frame.append(&download_btn);

                download_btn.connect_clicked(glib::clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |btn| imp.download_model(btn)
                ));

                frame
            };

            stack.add_named(&frame, Some(assist_pages::EMPTY));
            stack.add_named(&scroll_box, Some(assist_pages::LIST));

            obj.set_child(Some(&stack));
            self.stack.replace(stack);

            self.register_listview_activate();
            self.check_update_page();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::SEND_TO_PREVIEW)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl WidgetImpl for BibleAssist {}
    impl WindowImpl for BibleAssist {
        fn close_request(&self) -> glib::Propagation {
            self.parent_close_request();

            if let Some(tx) = self.audio_assist_channel.take() {
                let _ = tx.send(AudioSignal::Stop);
            }
            glib::Propagation::Proceed
        }
    }

    impl BibleAssist {
        pub(super) fn register_check(&self) -> bool {
            let model_dir = AppConfigDir::dir_path(AppConfigDir::Models).join("whisper.bin");

            fs::exists(model_dir).unwrap_or(false)
        }

        fn check_update_page(&self) {
            let page = match self.obj().has_model() {
                true => assist_pages::LIST,
                false => assist_pages::EMPTY,
            };
            self.stack.borrow().set_visible_child_name(page);
        }

        fn download_model(&self, btn: &gtk::Button) {
            btn.set_sensitive(false);
            btn.set_visible(false);
            let filepath = AppConfigDir::dir_path(AppConfigDir::Models).join("whisper.bin");

            let (fut, handler) =
                download::utils::download_something(super::URL.to_string(), filepath, {
                    let progress = self.progress_bar.borrow().clone();
                    let label = self.lable.borrow().clone();
                    move |res| {
                        progress.set_visible(true);
                        let text = match res {
                            DownloadStatus::Init => "Connecting...",
                            DownloadStatus::Progress(p, d, t) => {
                                progress.set_fraction(p as f64 / 100.0);
                                &format!("Downloading {d}/{t} {p}%")
                            }
                            DownloadStatus::Done(_path) => "Done",
                        };

                        label.set_label(text);
                    }
                });

            self.cancel_hanlder.replace(Some(handler));

            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                #[strong]
                btn,
                async move {
                    let res = match fut.await {
                        Ok(Ok(_)) => true,
                        // NOTE: This errors should be shown to the user
                        // on the UI
                        Ok(Err(e)) => {
                            eprintln!("[Err] {e:?}");
                            false
                        }
                        Err(e) => {
                            eprintln!("Error - {e:?}");
                            false
                        }
                    };

                    match res {
                        true => {
                            imp.check_update_page();
                            // TODO: start audio
                        }
                        false => {
                            btn.set_label("Download");
                            btn.set_sensitive(true);
                            btn.set_visible(true);
                        }
                    };

                    imp.progress_bar.borrow().set_visible(false);
                    imp.progress_bar.borrow().set_fraction(0.0);
                    imp.cancel_hanlder.replace(None);
                }
            ));
        }

        fn register_listview_activate(&self) {
            let listview = self.list_view.borrow().clone();

            listview.connect_activate(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |lv, pos| {
                    let model = lv.get_list_store().expect("Expecte gio::ListStore");
                    let item = model
                        .item(pos)
                        .and_downcast::<ScriptureVerseRangeObject>()
                        .expect("Expected ScriptureVerseRangeObject");
                    let range_obj = item.imp().item.borrow().clone();
                    let sm_data: SlideManagerData = range_obj.into();

                    imp.obj().emit_send_to_preveiew(sm_data);
                }
            ));
        }

        pub(super) fn start(&self, translation: &str) {
            let translation = translation.to_owned();
            if !self.obj().has_model() {
                return;
            };

            if let Some(tx) = self.audio_assist_channel.take() {
                let _ = tx.send(AudioSignal::Stop);
                return;
            }

            let (tx, rx) = mpsc::channel();
            self.audio_assist_channel.replace(Some(tx.clone()));
            let (gtk_tx, gtk_rx) = async_channel::unbounded();

            let anyhow::Result::Ok(audio) = Stt::new() else {
                return;
            };

            std::thread::spawn({
                let gtk_tx = gtk_tx.clone();
                let tx = tx.clone();
                move || {
                    let _ = audio
                        .start(gtk_tx, tx, rx)
                        .inspect_err(|e| eprintln!("[error] {e:?} "));
                }
            });

            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                async move {
                    let mut cache = HashMap::new();
                    while let Ok(event) = gtk_rx.recv().await {
                        match event {
                            TranscriptParserEvent::End => {
                                imp.obj().close();
                                imp.audio_assist_channel.replace(None);
                                break;
                            }
                            TranscriptParserEvent::Error(_) => {}
                            TranscriptParserEvent::Data(data) => {
                                if data.is_empty() {
                                    continue;
                                };

                                data.iter().for_each(|v| {
                                    let content = v.inspect();
                                    let bible_ref = v.eval();

                                    if cache.contains_key(&content) {
                                        return;
                                    }

                                    let Ok(passages) = Query::search_by_verses_query(
                                        translation.clone(),
                                        bible_ref.book.clone(),
                                        bible_ref.chapter,
                                        bible_ref.verses,
                                    ) else {
                                        return;
                                    };

                                    if passages.is_empty() {
                                        return;
                                    }

                                    let range_obj: ScriptureVerseRangeObject = {
                                        let chapter = bible_ref.chapter;
                                        let book = bible_ref.book;

                                        let passages = passages
                                            .iter()
                                            .map(|v| (v.verse, v.text.clone()))
                                            .collect::<Vec<_>>();
                                        ScriptureVerseRange::new(
                                            book,
                                            chapter,
                                            passages,
                                            translation.clone(),
                                        )
                                        .into()
                                    };
                                    range_obj.set_note(content.clone());

                                    imp.obj().append_item(&range_obj);
                                    cache.insert(content, range_obj);
                                });
                            }
                        };
                    }
                }
            ));
        }

        pub(super) fn stop(&self) {
            if let Some(tx) = self.audio_assist_channel.take() {
                let _ = tx.send(AudioSignal::Stop);
            }
        }
    }
}

glib::wrapper! {
    pub struct BibleAssist (ObjectSubclass<imp::BibleAssist>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Default for BibleAssist {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl BibleAssist {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn append_item(&self, item: &ScriptureVerseRangeObject) {
        let listview = self.imp().list_view.borrow().clone();
        let Some(store) = listview.get_list_store() else {
            return;
        };

        store.append(item);
    }

    pub fn list_view(&self) -> gtk::ListView {
        self.imp().list_view.borrow().clone()
    }

    pub fn has_model(&self) -> bool {
        self.imp().register_check()
    }

    pub fn start(&self, translation: &str) {
        if self.can_start_running() {
            self.imp().start(translation);
        }
    }

    pub fn stop(&self) {
        self.imp().stop();
    }

    pub fn can_start_running(&self) -> bool {
        let imp = self.imp();
        let is_running =
            imp.cancel_hanlder.borrow().is_some() || imp.audio_assist_channel.borrow().is_some();

        !is_running
    }

    pub fn emit_send_to_preveiew(&self, data: SlideManagerData) {
        self.emit_by_name::<()>(signals::SEND_TO_PREVIEW, &[&data])
    }

    pub fn connect_send_to_preveiew<F: Fn(&Self, &SlideManagerData) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::SEND_TO_PREVIEW,
            false,
            glib::closure_local!(|obj: &Self, data: &SlideManagerData| f(obj, data)),
        )
    }
}
