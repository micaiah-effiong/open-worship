use gtk::prelude::*;
use relm4::prelude::*;

// actrivity screen
#[derive(Debug)]
pub enum ActivityScreenInput {
    DisplayUpdate(String),
}
pub struct ActivityScreenModel {
    display_data: String,
}

const MIN_GRID_HEIGHT: i32 = 300;

#[relm4::component(pub)]
impl SimpleComponent for ActivityScreenModel {
    type Init = ();
    type Input = ActivityScreenInput;
    type Output = ();

    view! {
        #[root]
        gtk::Frame {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_homogeneous: true,
                set_height_request: MIN_GRID_HEIGHT,
                set_css_classes: &["brown_box", "black_bg_box"],
                set_overflow: gtk::Overflow::Hidden,

                gtk::Label {
                    #[watch]
                    set_label: &model.display_data,
                    set_justify: gtk::Justification::Center,
                    set_wrap: true,
                    set_wrap_mode: gtk::pango::WrapMode::Word,
                    set_css_classes: &["red_box", "white", "yellow_box"]

                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = ActivityScreenModel {
            display_data: String::from(""),
        };
        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ActivityScreenInput::DisplayUpdate(display_data) => {
                self.display_data = display_data;
                ()
            }
        };
    }
}
