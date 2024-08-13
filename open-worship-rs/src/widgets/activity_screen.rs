use gtk::prelude::*;
use relm4::prelude::*;

// actrivity screen
#[derive(Debug)]
pub enum ActivityScreenInput {}
pub struct ActivityScreenModel {}

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
                    set_label: PREVIEW_SCREEN_LABEL_STR,
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
        // let screen_box = gtk::Box::builder()
        //     .homogeneous(true)
        //     .height_request(MIN_GRID_HEIGHT)
        //     .build();
        // screen_box.set_css_classes(&["brown_box", "black_bg_box"]);
        // screen_box.set_overflow(gtk::Overflow::Hidden);
        //
        // let live_screen_label = gtk::Label::builder()
        //     .label(PREVIEW_SCREEN_LABEL_STR)
        //     .justify(gtk::Justification::Center)
        //     .wrap(true)
        //     .wrap_mode(gtk::pango::WrapMode::Word)
        //     .build();
        //
        // live_screen_label.set_css_classes(&["red_box", "white", "yellow_box"]);
        // screen_box.append(&live_screen_label);
        //
        // root.set_child(Some(&screen_box));

        let model = ActivityScreenModel {};
        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }
}

const  PREVIEW_SCREEN_LABEL_STR: &str = "
Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet.
Nisi anim cupidatat excepteur officia.
Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident.
Nostrud officia pariatur ut officia.
Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate.
Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod.
Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim.
Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis.
";
