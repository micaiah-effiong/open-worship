use gtk::glib::subclass::types::ObjectSubclassIsExt;
use std::{cell::RefCell, rc::Rc};

use gtk::{gio, glib, prelude::*};
use relm4::{prelude::*, tokio::sync::watch::Ref};

mod imp {

    use gtk::{
        glib::subclass::{
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
                let handler = dropdown.connect_selected_item_notify({
                    let entry = entry.clone();
                    move |d| {
                        let Some(item) = d.selected_item().and_downcast::<gtk::StringObject>()
                        else {
                            return;
                        };
                        entry.set_text(&item.string());
                    }
                });
                self.dropdown_select_handlerid.replace(Some(handler));
            }
            obj.append(&entry);
            obj.append(&dropdown);
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

    pub fn connect_changed<F: Fn(&Self, String) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        let obj = self.downgrade();
        self.imp().entry.borrow().connect_changed(move |entry| {
            let Some(obj) = obj.upgrade() else {
                return;
            };

            let value: String = entry.text().into();
            let dd = obj.imp().dropdown.borrow().clone();

            if let Some(m) = dd.model().and_downcast::<gtk::StringList>() {
                let index = m.find(&value);
                let index = match index == u32::MAX {
                    true => gtk::INVALID_LIST_POSITION,
                    false => index,
                };

                let id = obj.imp().dropdown_select_handlerid.take();
                if let Some(signal_id) = id {
                    obj.imp().dropdown.borrow().block_signal(&signal_id);
                    obj.imp().dropdown_select_handlerid.replace(Some(signal_id));
                }

                dd.set_selected(index);
                let id = obj.imp().dropdown_select_handlerid.take();
                if let Some(signal_id) = id {
                    obj.imp().dropdown.borrow().unblock_signal(&signal_id);
                    obj.imp().dropdown_select_handlerid.replace(Some(signal_id));
                }
            }

            //
            f(&obj, entry.text().into());
        })
    }

    pub fn set_text(&self, value: impl Into<String>) {
        let value: String = value.into();
        let dd = self.imp().dropdown.borrow().clone();

        if let Some(m) = dd.model().and_downcast::<gtk::StringList>() {
            let index = m.find(&value);
            if index == u32::MAX {
                self.imp().entry.borrow().set_text(&value);
                dd.set_selected(gtk::INVALID_LIST_POSITION);
                return;
            }

            dd.set_selected(index);
        }
    }
}
