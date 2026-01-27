use gtk::{
    ToggleButton,
    glib::{
        self,
        object::{Cast, IsA, ObjectExt},
        subclass::types::ObjectSubclassIsExt,
    },
    prelude::{BoxExt, ButtonExt, ToggleButtonExt, WidgetExt},
};

mod signals {
    pub(super) const MODE_ADDED: &str = "mode-added";
    pub(super) const MODE_CHANGED: &str = "mode-changed";
    pub(super) const MODE_REMOVED: &str = "mode-removed";
}

mod imp {
    use std::{
        cell::{Cell, RefCell},
        sync::OnceLock,
    };

    use gtk::{
        glib::{
            Properties,
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::StaticType,
        subclass::{box_::BoxImpl, prelude::DerivedObjectProperties, widget::WidgetImpl},
    };

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::GroupToggleButton)]
    pub struct GroupToggleButton {
        #[property(get=Self::get_n_items)]
        pub n_items: Cell<u32>,

        #[property(get, set=Self::set_selected_)]
        pub selected: Cell<i32>,

        pub(super) children: RefCell<Vec<gtk::ToggleButton>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupToggleButton {
        const NAME: &'static str = "GroupToggleButton";
        type Type = super::GroupToggleButton;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for GroupToggleButton {
        fn constructed(&self) {
            self.parent_constructed();
            self.selected.set(-1);
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::MODE_ADDED)
                        .param_types([u32::static_type(), gtk::ToggleButton::static_type()])
                        .build(),
                    Signal::builder(signals::MODE_CHANGED)
                        .param_types([gtk::ToggleButton::static_type()])
                        .build(),
                    Signal::builder(signals::MODE_REMOVED)
                        .param_types([u32::static_type(), gtk::ToggleButton::static_type()])
                        .build(),
                ]
            })
        }
    }
    impl WidgetImpl for GroupToggleButton {}
    impl BoxImpl for GroupToggleButton {}
    impl GroupToggleButton {
        fn get_n_items(&self) -> u32 {
            self.children.borrow().len() as u32
        }

        pub fn set_selected_(&self, index: i32) {
            if index == -1 {
                for c in self.children.borrow().clone() {
                    c.set_active(false);
                }
                return;
            };
            if index < 0 {
                return;
            };
            self.obj().set_active(index as u32);
        }
    }
}

glib::wrapper! {
pub struct GroupToggleButton(ObjectSubclass<imp::GroupToggleButton>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for GroupToggleButton {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl GroupToggleButton {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn append(&self, widget: &impl IsA<gtk::Widget>) -> u32 {
        let child = gtk::ToggleButton::new();
        child.set_child(Some(widget));

        self._append(&child)
    }

    pub fn append_toggle_button(&self, widget: &impl IsA<gtk::ToggleButton>) -> u32 {
        self._append(widget)
    }

    pub fn append_text(&self, text: &str) -> u32 {
        let child = gtk::ToggleButton::new();
        child.set_label(text);

        self._append(&child)
    }

    pub fn append_pixbuf(&self, pixbuf: &gtk::gdk_pixbuf::Pixbuf) -> u32 {
        let texture = gtk::gdk::Texture::for_pixbuf(pixbuf);
        let img = gtk::Image::from_paintable(Some(&texture));

        let child = gtk::ToggleButton::new();
        child.set_child(Some(&img));

        self._append(&child)
    }

    pub fn append_from_icon_name(&self, icon_name: &str) -> u32 {
        let child = gtk::ToggleButton::new();
        child.set_icon_name(icon_name);

        self._append(&child)
    }

    pub fn clear_children(&self) {
        for item in &self.imp().children.borrow().clone() {
            self.upcast_ref::<gtk::Box>().remove(item);
            item.unparent();
        }

        self.imp().children.borrow_mut().clear();
        self.imp().selected.set(-1);
    }

    pub fn remove(&self, index: u32) {
        let mut children = self.imp().children.borrow_mut();

        if index as usize >= children.len() {
            return;
        }

        let child = children.remove(index as usize);

        self.upcast_ref::<gtk::Box>().remove(&child);
        self.emit_mode_removed(index, &child);

        if children.is_empty() {
            self.imp().selected.set(-1);
        }
    }

    pub fn set_active(&self, index: u32) {
        let imp = self.imp();

        let list = imp.children.borrow();
        if index >= list.len() as u32 {
            return;
        }

        let Some(item) = list.get(index as usize) else {
            return;
        };

        if !item.is_active() {
            // TODO: check that this triggers a signal
            // item.set_active(true);
            item.emit_clicked();
            return;
        }
        imp.selected.set(index as i32);
        self.emit_mode_changed(item);
    }

    pub fn set_item_visible(&self, index: u32, value: bool) {
        if let Some(item) = self.imp().children.borrow().get(index as usize) {
            item.set_visible(value);
        }
    }

    fn emit_mode_added(&self, index: u32, child: &impl IsA<gtk::ToggleButton>) {
        self.emit_by_name::<()>(signals::MODE_ADDED, &[&index, child.as_ref()]);
    }

    fn emit_mode_changed(&self, child: &impl IsA<gtk::ToggleButton>) {
        self.emit_by_name::<()>(signals::MODE_CHANGED, &[child]);
    }

    fn emit_mode_removed(&self, index: u32, child: &impl IsA<gtk::ToggleButton>) {
        self.emit_by_name::<()>(signals::MODE_REMOVED, &[&index, child.as_ref()]);
    }

    fn _append(&self, child: &impl IsA<gtk::ToggleButton>) -> u32 {
        let child = child.as_ref();
        self.upcast_ref::<gtk::Box>().append(child);

        let imp = self.imp();
        imp.children.borrow_mut().push(child.clone());

        let index = imp.children.borrow().len().saturating_sub(1) as u32;
        if let Some(first_child) = self.imp().children.borrow().first()
            && index > 0
        {
            child.set_group(Some(first_child));
        };

        child.connect_toggled(glib::clone!(
            #[weak(rename_to=obj)]
            self,
            move |toggle| {
                let children_cache = obj.imp().children.borrow();
                let Some(index) = children_cache.iter().position(|c| c == toggle) else {
                    return;
                };

                if toggle.is_active() {
                    obj.imp().selected.set(index as i32);
                    obj.emit_mode_changed(toggle);
                }
            }
        ));

        self.emit_mode_added(index, child);

        index
    }

    pub fn connect_mode_added<F: Fn(&Self, u32, &ToggleButton) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::MODE_ADDED,
            false,
            glib::closure_local!(move |s: &Self, i: u32, widget: &ToggleButton| f(s, i, widget)),
        )
    }

    pub fn connect_mode_changed<F: Fn(&Self, &ToggleButton) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::MODE_CHANGED,
            false,
            glib::closure_local!(move |s: &Self, widget: &ToggleButton| f(s, widget)),
        )
    }

    pub fn connect_mode_removed<F: Fn(&Self, u32, &ToggleButton) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::MODE_REMOVED,
            false,
            glib::closure_local!(move |s: &Self, i: u32, widget: &ToggleButton| f(s, i, widget)),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{cell::Cell, rc::Rc};

    #[gtk::test]
    fn test_set_active() {
        let mode_btn = GroupToggleButton::new();

        mode_btn.append(&gtk::Button::with_label("0"));
        mode_btn.append(&gtk::Button::with_label("1"));
        assert_eq!(mode_btn.selected(), -1);

        let count = Rc::new(Cell::new(0));
        mode_btn.connect_mode_changed({
            let count = count.clone();
            move |_, _| count.set(count.get() + 1)
        });

        mode_btn.set_active(1);
        assert_eq!(mode_btn.selected(), 1);
        assert_eq!(count.get(), 1);
    }

    #[gtk::test]
    fn test_mode_added() {
        let mode_btn = GroupToggleButton::new();
        assert_eq!(mode_btn.n_items(), 0);

        let count = Rc::new(Cell::new(0));
        mode_btn.connect_mode_added({
            let count = count.clone();
            move |_, _, _| count.set(count.get() + 1)
        });

        mode_btn.append(&gtk::Button::with_label("0"));
        mode_btn.append(&gtk::Button::with_label("1"));

        assert_eq!(mode_btn.n_items(), 2);
        assert_eq!(count.get(), 2);
    }

    #[gtk::test]
    fn test_mode_removed() {
        let mode_btn = GroupToggleButton::new();
        assert_eq!(mode_btn.n_items(), 0);

        mode_btn.append(&gtk::Button::with_label("0"));
        mode_btn.append(&gtk::Button::with_label("1"));
        mode_btn.append(&gtk::Button::with_label("2"));
        assert_eq!(mode_btn.n_items(), 3);

        let count = Rc::new(Cell::new(0));
        mode_btn.connect_mode_removed({
            let count = count.clone();
            move |_, _, _| count.set(count.get() + 1)
        });

        mode_btn.remove(1);
        mode_btn.remove(1);
        assert_eq!(mode_btn.n_items(), 1);
        assert_eq!(count.get(), 2);

        mode_btn.remove(1); // invalid index
        assert_eq!(mode_btn.n_items(), 1);

        mode_btn.remove(0);
        assert_eq!(mode_btn.n_items(), 0);
        assert_eq!(mode_btn.selected(), -1);
    }

    #[gtk::test]
    fn test_clear_children() {
        let mode_btn = GroupToggleButton::new();

        mode_btn.append(&gtk::Button::with_label("0"));
        mode_btn.append(&gtk::Button::with_label("1"));
        mode_btn.append(&gtk::Button::with_label("2"));
        assert_eq!(mode_btn.n_items(), 3);

        mode_btn.clear_children();

        assert_eq!(mode_btn.n_items(), 0);
        assert_eq!(mode_btn.selected(), -1);
    }
}
