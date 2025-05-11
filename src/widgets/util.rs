use gtk::prelude::WidgetExt;

/// Returns a vector of widgets
/// useful for iterating over childern in listview
pub fn widget_to_vec(_w: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut v = Vec::new();

    let mut w = _w.clone();

    loop {
        v.push(w.clone());

        if let Some(next_s) = w.next_sibling() {
            w = next_s;
        } else {
            break;
        }
    }

    v
}
