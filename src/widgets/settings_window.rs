use gtk::{gio, glib};

mod imp {
    use std::{cell::RefCell, collections::HashMap, usize};

    use gtk::{
        gdk::{
            self,
            prelude::{DisplayExt, MonitorExt},
        },
        gio::prelude::ListModelExtManual,
        glib::{
            self,
            object::{Cast, CastNone},
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt},
            },
            types::StaticTypeExt,
            value::ToValue,
            variant::ToVariant,
        },
        pango::{self, prelude::FontFamilyExt},
        prelude::{GtkWindowExt, ListItemExt, RangeExt, WidgetExt},
        subclass::{
            widget::{
                CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt,
                WidgetImpl,
            },
            window::WindowImpl,
        },
    };

    use super::*;
    use crate::{
        application::OwApplication,
        services::settings::ApplicationSettings,
        structs::integer_object::IntegerObject,
        utils::{self, WidgetChildrenExt},
        widgets::canvas::{canvas::Canvas, serialise::CanvasData},
    };

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/settings_window.ui")]
    pub struct SettingsWindow {
        #[template_child]
        sidebar: gtk::TemplateChild<gtk::StackSidebar>,
        #[template_child]
        stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        output_view: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        monitor_dropdowon: gtk::TemplateChild<gtk::DropDown>,

        #[template_child]
        monitor_width: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        monitor_height: gtk::TemplateChild<gtk::Label>,

        #[template_child]
        demo_screen: gtk::TemplateChild<Canvas>,
        #[template_child]
        screen_aspect_frame: gtk::TemplateChild<gtk::AspectFrame>,

        // song
        #[template_child]
        song_font_dropdown: gtk::TemplateChild<gtk::DropDown>,

        // scripture
        #[template_child]
        scripture_font_dropdown: gtk::TemplateChild<gtk::DropDown>,
        #[template_child]
        show_reference: gtk::TemplateChild<gtk::CheckButton>,
        #[template_child]
        show_verse_number: gtk::TemplateChild<gtk::CheckButton>,
        #[template_child]
        show_passage: gtk::TemplateChild<gtk::CheckButton>,
        #[template_child]
        show_only_reference: gtk::TemplateChild<gtk::CheckButton>,
        #[template_child]
        break_new_verse: gtk::TemplateChild<gtk::CheckButton>,

        // transition
        #[template_child]
        transition_dropdown: gtk::TemplateChild<gtk::DropDown>,
        /// for duration
        #[template_child]
        transition_scale: gtk::TemplateChild<gtk::Scale>,

        monitor_map: RefCell<HashMap<String, gdk::Monitor>>,

        fonts_map: RefCell<HashMap<String, pango::FontFamily>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsWindow {
        const NAME: &'static str = "SettingsWindow";
        type Type = super::SettingsWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Canvas::ensure_type();
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.register_fonts();
            self.register_transtions();
            self.register_song_fonts();
            self.register_scripture_fonts();

            self.sidebar.set_stack(&self.stack);

            self.demo_screen
                .imp()
                .sava_data
                .replace(Some(CanvasData::default()));
            self.demo_screen.imp().load_data();
            self.demo_screen.style();

            if let Some(display) = gtk::gdk::Display::default()
                && let Some(dropdown_model) = self
                    .monitor_dropdowon
                    .model()
                    .and_downcast::<gtk::StringList>()
            {
                let mut mon_hash = self.monitor_map.borrow_mut();
                let mon_list = display.monitors();

                for monitor in mon_list.iter::<gdk::Monitor>() {
                    let Ok(monitor) = monitor else {
                        return;
                    };

                    if let Some(name) = monitor.model() {
                        let name = name.to_string();
                        dropdown_model.append(&name);
                        mon_hash.insert(name.clone(), monitor);
                    }
                }
            }

            let fn_connect = glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |dropdown: &gtk::DropDown| {
                    let Some(item) = dropdown.selected_item().and_downcast::<gtk::StringObject>()
                    else {
                        return;
                    };
                    let item: String = item.into();

                    let monitor_map = imp.monitor_map.borrow();
                    let Some(monitor) = monitor_map.get(&item) else {
                        imp.monitor_width.set_label("0");
                        imp.monitor_height.set_label("0");
                        imp.screen_aspect_frame.set_ratio(1.0);
                        return;
                    };
                    let geo = monitor.geometry();
                    imp.monitor_width.set_label(&geo.width().to_string());
                    imp.monitor_height.set_label(&geo.height().to_string());

                    let ratio = geo.width() as f32 / geo.height() as f32;
                    imp.screen_aspect_frame.set_ratio(ratio);

                    let Some(app) = imp.obj().application().and_downcast::<OwApplication>() else {
                        return;
                    };

                    let ext_screen = app.main_window().extended_screen();
                    ext_screen.fullscreen_on_monitor(monitor);
                }
            );

            fn_connect(&self.monitor_dropdowon.clone());
            self.monitor_dropdowon
                .connect_selected_item_notify(fn_connect);

            // NOTE: this happens here to ensure that all models are set
            // before binding settings to values
            // this way default values from settings are not overwritten
            self.bind_settings_values();
        }
    }

    impl WidgetImpl for SettingsWindow {}
    impl WindowImpl for SettingsWindow {}

    impl SettingsWindow {
        fn get_fonts(&self) -> Vec<String> {
            let font_map = self.fonts_map.borrow().clone();
            let mut fonts = font_map.keys().map(|v| v.clone()).collect::<Vec<_>>();
            fonts.sort();
            fonts
        }
        fn register_scripture_fonts(&self) {
            let binding = self.get_fonts();
            let fonts = binding.iter().map(|v| v.as_str()).collect::<Vec<_>>();
            let model = gtk::StringList::new(&fonts);

            let model = gtk::SingleSelection::new(Some(model));
            let font_dropdown = self.scripture_font_dropdown.clone();

            let Some(factory) = font_dropdown
                .factory()
                .and_downcast::<gtk::SignalListItemFactory>()
            else {
                return;
            };

            factory.connect_bind(move |_, list_item| {
                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");

                let li_box = li.child().expect("Needs to be a box");
                let label = li_box
                    .get_children::<gtk::Label>()
                    .next()
                    .expect("Need to be a label");

                let item = li
                    .item()
                    .and_downcast::<gtk::StringObject>()
                    .expect("Needs to be a label");

                label.set_label(&item.string());
            });
            // font_dropdown.set_factory(Some(&factory));
            font_dropdown.set_model(Some(&model));
        }
        fn register_song_fonts(&self) {
            let binding = self.get_fonts();
            let fonts = binding.iter().map(|v| v.as_str()).collect::<Vec<_>>();
            let model = gtk::StringList::new(&fonts);

            let model = gtk::SingleSelection::new(Some(model));
            let font_dropdown = self.song_font_dropdown.clone();

            let Some(factory) = font_dropdown
                .factory()
                .and_downcast::<gtk::SignalListItemFactory>()
            else {
                return;
            };

            factory.connect_bind(move |_, list_item| {
                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");

                let li_box = li.child().expect("Needs to be a box");
                let label = li_box
                    .get_children::<gtk::Label>()
                    .next()
                    .expect("Need to be a label");

                let item = li
                    .item()
                    .and_downcast::<gtk::StringObject>()
                    .expect("Needs to be a label");

                label.set_label(&item.string());
            });
            // font_dropdown.set_factory(Some(&factory));
            font_dropdown.set_model(Some(&model));
        }
        fn register_transtions(&self) {
            let vector: Vec<IntegerObject> = (0..=22).map(IntegerObject::new).collect();

            let model = gio::ListStore::new::<IntegerObject>();
            model.extend_from_slice(&vector);
            let model = gtk::SingleSelection::new(Some(model));
            let transition_dropdown = self.transition_dropdown.clone();

            transition_dropdown.set_expression(Some(
                gtk::ClosureExpression::new::<Option<String>>(
                    gtk::Expression::NONE,
                    glib::closure_local!(move |item: glib::Object| {
                        match item.downcast_ref::<IntegerObject>() {
                            Some(item) => Some(item.number().to_string()),
                            None => None,
                        }
                    }),
                ),
            ));

            let Some(factory) = transition_dropdown
                .factory()
                .and_downcast::<gtk::SignalListItemFactory>()
            else {
                return;
            };

            factory.connect_bind(move |_, list_item| {
                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");

                let li_box = li.child().expect("Needs to be a box");
                let label = li_box
                    .get_children::<gtk::Label>()
                    .next()
                    .expect("Need to be a label");

                let item = li
                    .item()
                    .and_downcast::<IntegerObject>()
                    .expect("Needs to be a label");

                let n = utils::int_to_transition(item.number());
                let text = utils::space_camelcase(&format!("{:?}", n));
                label.set_label(&text);
            });
            transition_dropdown.set_model(Some(&model));
        }
        fn register_fonts(&self) {
            let fonts = self.obj().pango_context().list_families();
            let mut fonts_map = self.fonts_map.borrow_mut();

            for font in fonts {
                let name = font.name().to_string();
                fonts_map.insert(name, font);
            }
        }

        fn bind_settings_values(&self) {
            let settings = ApplicationSettings::get_instance();

            let show_reference = self.show_reference.clone();
            settings
                .bind_show_reference(&show_reference, "active")
                .build();

            let show_verse_number = self.show_verse_number.clone();
            settings
                .bind_show_verse_number(&show_verse_number, "active")
                .build();

            let show_passage = self.show_passage.clone();
            settings.bind_show_passage(&show_passage, "active").build();

            let show_only_reference = self.show_only_reference.clone();
            settings
                .bind_show_only_reference(&show_only_reference, "active")
                .build();

            let break_new_verse = self.break_new_verse.clone();
            settings
                .bind_break_new_verse(&break_new_verse, "active")
                .build();

            let transition_dropdown = self.transition_dropdown.clone();
            settings
                .bind_transition(&transition_dropdown, "selected")
                .build();

            let transition_scale = self.transition_scale.clone();
            settings
                .bind_transition_duration(&transition_scale.adjustment(), "value")
                .build();

            let font_names = self.get_fonts();
            let song_font_dropdown = self.song_font_dropdown.clone();
            settings
                .bind_song_font(&song_font_dropdown, "selected")
                .mapping(glib::clone!(
                    #[strong]
                    font_names,
                    move |font, _| {
                        let font: String = font
                            .get()
                            .expect("The variant needs to be of type `String`.");

                        let Some(found) = font_names.iter().position(|v| *v == font) else {
                            return None;
                        };
                        Some((found as u32).to_value())
                    }
                ))
                .set_mapping(glib::clone!(
                    #[strong]
                    font_names,
                    move |font, _| {
                        let selected: u32 =
                            font.get().expect("The variant needs to be of type `i32`.");

                        let Some(font) = font_names.get(selected as usize) else {
                            return None;
                        };
                        Some(font.to_variant())
                    }
                ))
                .build();

            let scripture_font_dropdown = self.scripture_font_dropdown.clone();
            settings
                .bind_scripture_font(&scripture_font_dropdown, "selected")
                .mapping(glib::clone!(
                    #[strong]
                    font_names,
                    move |font, _| {
                        let font: String = font
                            .get()
                            .expect("The variant needs to be of type `String`.");

                        let Some(found) = font_names.iter().position(|v| *v == font) else {
                            return None;
                        };
                        Some((found as u32).to_value())
                    }
                ))
                .set_mapping(glib::clone!(
                    #[strong]
                    font_names,
                    move |font, _| {
                        let selected: u32 =
                            font.get().expect("The variant needs to be of type `i32`.");

                        let Some(font) = font_names.get(selected as usize) else {
                            return None;
                        };
                        Some(font.to_variant())
                    }
                ))
                .build();
        }
    }
}

glib::wrapper! {
pub struct SettingsWindow(ObjectSubclass<imp::SettingsWindow>)
    @extends gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,gtk::Native,gtk::Root, gtk::ShortcutManager;
}

impl Default for SettingsWindow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SettingsWindow {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
