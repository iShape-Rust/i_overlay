mod square;
mod fill_view;
mod data;
mod app;
mod shape;
mod point_editor;
mod sheet;
mod geom;

use crate::app::main::EditorApp;


fn main() -> iced::Result {
    iced::run("iOverlay", EditorApp::update, EditorApp::view)
}