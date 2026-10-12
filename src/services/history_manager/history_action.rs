use gtk::glib::{
    self,
    object::{Cast, CastNone, ObjectExt},
    subclass::types::ObjectSubclassIsExt,
};

use crate::{
    utils,
    widgets::canvas::{
        canvas::Canvas,
        canvas_item::{CanvasItem, CanvasItemExt},
    },
};

mod signals {
    pub(super) const UNDO: &str = "undo";
    pub(super) const REDO: &str = "redo";
    pub(super) const ACTION_CALLED: &str = "action-called";
}

pub enum HistoryActionType {
    ItemDepthChanged(HistoryWithCanvas),
    ItemChanged(HistoryNoCanvas),
    ItemMoved(HistoryNoCanvas),
    SlideChanged(HistoryNoCanvas),
    CanvasChanged(HistoryNoCanvas),
}

impl HistoryActionType {
    fn item(&self) -> Option<glib::Object> {
        match self {
            HistoryActionType::ItemDepthChanged(history) => history.item.clone(),
            HistoryActionType::ItemChanged(history) => history.item.clone(),
            HistoryActionType::ItemMoved(history) => history.item.clone(),
            HistoryActionType::SlideChanged(history) => history.item.clone(),
            HistoryActionType::CanvasChanged(history) => history.item.clone(),
        }
        .upgrade()
    }
    fn property(&self) -> String {
        match self {
            HistoryActionType::ItemDepthChanged(history) => history.property.clone(),
            HistoryActionType::ItemChanged(history) => history.property.clone(),
            HistoryActionType::ItemMoved(history) => history.property.clone(),
            HistoryActionType::SlideChanged(history) => history.property.clone(),
            HistoryActionType::CanvasChanged(history) => history.property.clone(),
        }
    }
    fn value(&self) -> glib::Value {
        match self {
            HistoryActionType::ItemDepthChanged(history) => history.value.clone(),
            HistoryActionType::ItemChanged(history) => history.value.clone(),
            HistoryActionType::ItemMoved(history) => history.value.clone(),
            HistoryActionType::SlideChanged(history) => history.value.clone(),
            HistoryActionType::CanvasChanged(history) => history.value.clone(),
        }
    }

    fn run_action(&mut self) {
        let Some(item) = self.item() else {
            return;
        };
        let property_name = self.property();
        let property_value = self.value();

        match self {
            HistoryActionType::ItemDepthChanged(action) => {
                let Some(item) = item.downcast_ref::<CanvasItem>() else {
                    return;
                };
                let Some(parent) = action.parent.upgrade() else {
                    return;
                };

                let value = if let Ok(true) = property_value.get::<bool>() {
                    parent.move_down(item, false);
                    true
                } else {
                    parent.move_up(item, false);
                    false
                };

                action.value = value.into();
            }
            HistoryActionType::ItemChanged(action)
            | HistoryActionType::ItemMoved(action)
            | HistoryActionType::SlideChanged(action)
            | HistoryActionType::CanvasChanged(action) => {
                let new_value = item.property_value(&property_name);
                item.set_property(&property_name, property_value);
                action.value = new_value;
            }
        }

        match self {
            HistoryActionType::ItemChanged(_) => {
                if let Some(ci) = item.downcast_ref::<CanvasItem>() {
                    ci.style();
                };
            }
            HistoryActionType::CanvasChanged(_) => {
                if let Some(c) = item.downcast_ref::<Canvas>() {
                    c.style();
                };
            }
            _ => (),
        }
    }
}

// struct HistoryAction {
//     item: RefCell<glib::Object>,
//     item_property: RefCell<String>,
//     value: RefCell<glib::Value>,
//     parent: glib::WeakRef<Canvas>,
//     action: RefCell<HistoryActionType>,
// }

pub struct HistoryNoCanvas {
    item: glib::WeakRef<glib::Object>,
    property: String,
    value: glib::Value,
}

pub struct HistoryWithCanvas {
    item: glib::WeakRef<glib::Object>,
    property: String,
    value: glib::Value,
    parent: glib::WeakRef<Canvas>,
}

mod imp {
    use std::{cell::RefCell, collections::VecDeque, sync::OnceLock};

    use gtk::glib::{
        self,
        subclass::{Signal, object::ObjectImpl, types::ObjectSubclass},
    };
    use gtk::prelude::StaticType;

    use crate::{
        services::history_manager::history_action::{HistoryActionType, signals},
        widgets::canvas::canvas_item::CanvasItem,
    };

    #[derive(Default)]
    pub struct HistoryManager {
        pub(super) undo_list: RefCell<VecDeque<HistoryActionType>>,
        pub(super) redo_list: RefCell<VecDeque<HistoryActionType>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HistoryManager {
        const NAME: &'static str = "HistoryManager";
        type Type = super::HistoryManager;
    }

    impl ObjectImpl for HistoryManager {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::UNDO)
                        .param_types([bool::static_type()])
                        .build(),
                    Signal::builder(signals::REDO)
                        .param_types([bool::static_type()])
                        .build(),
                    Signal::builder(signals::ACTION_CALLED)
                        .param_types([Option::<CanvasItem>::static_type()])
                        .build(),
                ]
            })
        }
    }
}

glib::wrapper! {
    pub struct HistoryManager (ObjectSubclass<imp::HistoryManager>);
}

impl Default for HistoryManager {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl HistoryManager {
    const MAX_CAPACITY: usize = 50;
    pub fn new() -> Self {
        glib::Object::new()
    }

    // ItemDepthChanged(HistoryActionWithCanvas),
    pub fn item_depth_changed(
        &self,
        item: &impl glib::object::IsA<glib::Object>,
        property: &str,
        parent: &Canvas,
        force_add: bool,
    ) {
        let value = item.property(property);
        let history = HistoryWithCanvas {
            item: item.as_ref().downgrade(),
            property: property.into(),
            value,
            parent: parent.downgrade(),
        };

        self.add_undo_action(
            HistoryActionType::ItemDepthChanged(history),
            item,
            force_add,
        );
    }

    // ItemChanged(HistoryActionNoCanvas),
    pub fn item_changed(
        &self,
        item: &impl glib::object::IsA<glib::Object>,
        property: &str,
        force_add: bool,
    ) {
        let value = item.property(property);
        let history = HistoryNoCanvas {
            item: item.as_ref().downgrade(),
            property: property.into(),
            value,
        };

        self.add_undo_action(HistoryActionType::ItemChanged(history), item, force_add);
    }

    // ItemMoved(HistoryActionNoCanvas),
    pub fn item_moved(
        &self,
        canvas_item: &impl glib::object::IsA<CanvasItem>,
        rect: &utils::rect::Rect,
        force_add: bool,
    ) {
        let property = "rectangle".to_string();
        let item = canvas_item.as_ref().upcast_ref::<glib::Object>();

        let history = HistoryNoCanvas {
            item: item.downgrade(),
            property,
            value: rect.into(),
        };

        self.add_undo_action(HistoryActionType::ItemMoved(history), item, force_add);
    }

    // SlideChanged(HistoryActionNoCanvas),
    pub fn slide_changed(
        &self,
        item: &impl glib::object::IsA<glib::Object>,
        property: &str,
        force_add: bool,
    ) {
        let value = item.property(property);
        let history = HistoryNoCanvas {
            item: item.as_ref().downgrade(),
            property: property.into(),
            value,
        };

        self.add_undo_action(HistoryActionType::SlideChanged(history), item, force_add);
    }

    // CanvasChanged(HistoryActionNoCanvas),
    pub fn canvas_changed(
        &self,
        item: &impl glib::object::IsA<glib::Object>,
        property: &str,
        force_add: bool,
    ) {
        let value = item.property(property);
        let history = HistoryNoCanvas {
            item: item.as_ref().downgrade(),
            property: property.into(),
            value,
        };

        self.add_undo_action(HistoryActionType::CanvasChanged(history), item, force_add);
    }

    /// pass item here or call action.item()
    /// which will upgrade the weak ref
    /// to prevent that pass item here
    fn add_undo_action(
        &self,
        action: HistoryActionType,
        item: &impl glib::object::IsA<glib::Object>,
        force_add: bool,
    ) {
        let mut undo_list = self.imp().undo_list.borrow_mut();

        // TODO: check cooldown time
        if let Some(last_action) = undo_list.front()
            && last_action.item().is_some_and(|v| v == *item.as_ref())
            && last_action.property() == action.property()
            && !force_add
        {
            return;
        }
        self.imp().redo_list.borrow_mut().clear();

        if undo_list.len() == Self::MAX_CAPACITY {
            undo_list.pop_back();
        }

        undo_list.push_front(action);
        drop(undo_list); // NOTE: drop here because send_signals also borrows undo_list

        self.send_signals();
    }

    pub fn undo(&self) {
        let action = self.imp().undo_list.borrow_mut().pop_front();
        if let Some(mut action) = action {
            action.run_action();
            self.emit_action_called(&action);

            self.imp().redo_list.borrow_mut().push_front(action);
            self.send_signals();
        };
    }
    pub fn redo(&self) {
        let action = self.imp().redo_list.borrow_mut().pop_front();
        if let Some(mut action) = action {
            action.run_action();
            self.emit_action_called(&action);

            self.imp().undo_list.borrow_mut().push_front(action);
            self.send_signals();
        };
    }

    pub fn clear_history(&self) {
        self.imp().undo_list.borrow_mut().clear();
        self.imp().redo_list.borrow_mut().clear();
    }

    fn send_signals(&self) {
        self.emit_undo(self.imp().undo_list.borrow().is_empty());
        self.emit_redo(self.imp().redo_list.borrow().is_empty());
    }
}

impl HistoryManager {
    fn emit_undo(&self, is_empty: bool) {
        self.emit_by_name(signals::UNDO, &[&is_empty])
    }
    fn emit_redo(&self, is_empty: bool) {
        self.emit_by_name(signals::REDO, &[&is_empty])
    }
    fn emit_action_called(&self, action: &HistoryActionType) {
        let canvas_item: Option<CanvasItem> = match &action {
            HistoryActionType::CanvasChanged(ci) => ci.item.upgrade().and_downcast::<CanvasItem>(),
            _ => None,
        };

        self.emit_by_name(signals::ACTION_CALLED, &[&canvas_item.as_ref()])
    }

    pub fn connect_undo<F: Fn(&Self, bool) + 'static>(&self, f: F) -> gtk::glib::SignalHandlerId {
        self.connect_closure(
            signals::UNDO,
            false,
            glib::closure_local!(|obj: &Self, is_empty: bool| f(obj, is_empty)),
        )
    }
    pub fn connect_redo<F: Fn(&Self, bool) + 'static>(&self, f: F) -> gtk::glib::SignalHandlerId {
        self.connect_closure(
            signals::REDO,
            false,
            glib::closure_local!(|obj: &Self, is_empty: bool| f(obj, is_empty)),
        )
    }
    pub fn connect_action_called<F: Fn(&Self, Option<&CanvasItem>) + 'static>(
        &self,
        f: F,
    ) -> gtk::glib::SignalHandlerId {
        self.connect_closure(
            signals::ACTION_CALLED,
            false,
            glib::closure_local!(|obj: &Self, canvas_item: Option<&CanvasItem>| f(
                obj,
                canvas_item
            )),
        )
    }
}
