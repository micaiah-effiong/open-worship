fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res\\windows\\openworship.ico");
        res.compile().unwrap();
    }

    glib_build_tools::compile_resources(
        &["data/resources/"],
        "data/resources/resources.gresource.xml",
        "resources.gresource",
    );

    relm4_icons_build::bundle_icons("icon_names.rs", None, None, None::<&str>, ["plus", "minus"]);
}
