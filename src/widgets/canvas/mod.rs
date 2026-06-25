pub mod canvas;
pub mod canvas_grid;
pub mod canvas_item;
mod grabber;
pub mod image_item;
pub mod serialise;
pub mod text_item;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum CanvasItemType {
    #[default]
    TEXT,
    IMAGE,
    // SHAPE,
}
