use gtk::glib::subclass::types::ObjectSubclassIsExt;
use std::cell::RefCell;

use gtk::{gio, glib, prelude::*};

mod signals {
    pub(super) const VALUE_CHANGED: &str = "value-changed";
}

mod imp {

    use std::sync::OnceLock;

    use gtk::{
        glib::subclass::{
            Signal,
            object::{ObjectImpl, ObjectImplExt},
            types::{ObjectSubclass, ObjectSubclassExt},
        },
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use super::*;

    #[derive(Debug, Default)]
    pub struct EntryCombo {
        pub(super) entry: RefCell<gtk::Entry>,
        pub(super) dropdown: RefCell<gtk::DropDown>,
        pub(super) dropdown_select_handlerid: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) entry_changer_handlerid: RefCell<Option<glib::SignalHandlerId>>,

        pub(super) value: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EntryCombo {
        const NAME: &'static str = "EntryCombo";
        type Type = super::EntryCombo;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for EntryCombo {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.add_css_class("entrycombo");

            let entry = self.entry.borrow().clone();
            entry.set_css_classes(&["flat"]);
            let entry_hanlderid = entry.connect_changed(glib::clone!(
                #[weak(rename_to = obj)]
                self,
                move |entry| {
                    let value: String = entry.text().into();
                    obj.value.replace(value.clone());
                    obj.obj().emit_connect_changed(value.clone());
                    //

                    let dd = obj.dropdown.borrow();

                    if let Some(m) = dd.model().and_downcast::<gtk::StringList>() {
                        let index = m.find(&value);
                        let index = match index == gtk::INVALID_LIST_POSITION {
                            true => gtk::INVALID_LIST_POSITION,
                            false => index,
                        };

                        let id = obj.dropdown_select_handlerid.take();
                        if let Some(signal_id) = id {
                            obj.dropdown.borrow().block_signal(&signal_id);
                            obj.dropdown_select_handlerid.replace(Some(signal_id));
                        }

                        dd.set_selected(index);

                        let id = obj.dropdown_select_handlerid.take();
                        if let Some(signal_id) = id {
                            obj.dropdown.borrow().unblock_signal(&signal_id);
                            obj.dropdown_select_handlerid.replace(Some(signal_id));
                        }
                    }
                }
            ));
            self.entry_changer_handlerid.replace(Some(entry_hanlderid));

            let dropdown = self.dropdown.borrow().clone();
            {
                if let Some(toggle) = dropdown.first_child().and_downcast::<gtk::ToggleButton>() {
                    toggle.set_css_classes(&["flat"]);

                    if let Some(stack) = toggle
                        .first_child()
                        .and_then(|v| v.first_child())
                        .and_downcast::<gtk::Stack>()
                    {
                        stack.set_visible(false);
                    }
                }
                let handler = dropdown.connect_selected_item_notify(glib::clone!(
                    #[weak(rename_to = obj)]
                    self,
                    #[strong]
                    entry,
                    move |d| {
                        let Some(item) = d.selected_item().and_downcast::<gtk::StringObject>()
                        else {
                            return;
                        };
                        let value: String = item.string().to_string();
                        obj.value.replace(value.clone());
                        obj.obj().emit_connect_changed(value.clone());

                        //

                        let id = obj.entry_changer_handlerid.take();
                        if let Some(signal_id) = id {
                            obj.entry.borrow().block_signal(&signal_id);
                            obj.entry_changer_handlerid.replace(Some(signal_id));
                        }

                        entry.set_text(&value);

                        let id = obj.entry_changer_handlerid.take();
                        if let Some(signal_id) = id {
                            obj.entry.borrow().unblock_signal(&signal_id);
                            obj.entry_changer_handlerid.replace(Some(signal_id));
                        }
                    }
                ));
                self.dropdown_select_handlerid.replace(Some(handler));
            }
            obj.append(&entry);
            obj.append(&dropdown);
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::VALUE_CHANGED)
                        .param_types([String::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl WidgetImpl for EntryCombo {}
    impl BoxImpl for EntryCombo {}
}

glib::wrapper! {
pub struct EntryCombo (ObjectSubclass<imp::EntryCombo>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for EntryCombo {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EntryCombo {
    pub fn new(model: Option<&impl IsA<gio::ListModel>>) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().dropdown.borrow().set_model(model);

        obj
    }

    pub fn set_input_purpose(&self, purpose: gtk::InputPurpose) {
        self.imp().entry.borrow().set_input_purpose(purpose);
    }

    fn emit_connect_changed(&self, value: String) {
        self.emit_by_name::<()>(signals::VALUE_CHANGED, &[&value]);
    }

    pub fn connect_changed<F: Fn(&Self, String) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::VALUE_CHANGED,
            true,
            glib::closure_local!(move |obj: &Self, value: String| f(obj, value)),
        )
    }

    pub fn set_text(&self, value: impl Into<String>) {
        let value: String = value.into();
        let dd = self.imp().dropdown.borrow().clone();

        if let Some(m) = dd.model().and_downcast::<gtk::StringList>() {
            let index = m.find(&value);
            let i = match index == u32::MAX {
                true => gtk::INVALID_LIST_POSITION,
                false => index,
            };
            dd.set_selected(i);
        }

        self.imp().entry.borrow().set_text(&value);
        self.imp().value.replace(value.clone());
        self.emit_connect_changed(value);
    }
}
