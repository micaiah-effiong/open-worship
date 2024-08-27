use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub enum SearchScriptureInput {}

#[derive(Debug)]
pub enum SearchScriptureOutput {}

#[derive(Debug)]
pub struct SearchScriptureModel {}

impl SearchScriptureModel {}

pub struct SearchScriptureInit {}

impl SearchScriptureModel {}

#[relm4::component(pub)]
impl SimpleComponent for SearchScriptureModel {
    type Init = SearchScriptureInit;
    type Output = SearchScriptureOutput;
    type Input = SearchScriptureInput;

    view! {
        #[root]
        gtk::Box{
            set_orientation:gtk::Orientation::Vertical,
            set_vexpand: true,
            add_css_class: "blue_box",

            // #[name="search_field"]
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 2,
                set_height_request: 48,
                add_css_class: "green_double_box",

                gtk::SearchEntry {
                    set_placeholder_text: Some("Genesis 1:1"),
                    set_hexpand: true
                }
            },

            gtk::ScrolledWindow {
                set_vexpand: true,

                #[wrap(Some)]
                set_child = &gtk::ListView {
                    #[wrap(Some)]
                    set_model = &gtk::SingleSelection{
                        set_model: Some(&(0..1000).map(|_| LIST_VEC[0]).collect::<gtk::StringList>()),
                    },

                    #[wrap(Some)]
                    set_factory = &gtk::SignalListItemFactory {
                        connect_setup => move |_, list_item|{
                            let label = gtk::Label::builder()
                                .ellipsize(gtk::pango::EllipsizeMode::End)
                                .single_line_mode(true)
                                .halign(gtk::Align::Start)
                                .justify(gtk::Justification::Fill).build();

                            list_item
                                .downcast_ref::<gtk::ListItem>()
                                .expect("Must be a list item")
                                .set_child(Some(&label));

                            list_item
                                .property_expression("item")
                                .chain_property::<gtk::StringObject>("string")
                                .bind(&label, "label", gtk::Widget::NONE);
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = SearchScriptureModel {};

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {};
    }
}

const LIST_VEC: [&str; 1] = [
    "Golden sun, a radiant masterpiece, paints the canvas of the morning sky. With hues of pink and softest blue, a breathtaking, ethereal sight,",
];
