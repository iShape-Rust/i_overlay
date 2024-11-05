use crate::geom::camera::Camera;
use std::collections::HashMap;
use i_triangle::i_overlay::core::overlay::Overlay;
use i_triangle::i_overlay::i_shape::int::count::PointsCount;
use i_triangle::i_overlay::i_float::int::rect::IntRect;
use crate::app::boolean::workspace::WorkspaceState;
use crate::app::design::style_sidebar_background;
use iced::widget::scrollable;
use crate::app::boolean::control::ModeOption;
use crate::app::boolean::control::FillOption;
use crate::app::boolean::control::SolverOption;
use iced::{Alignment, Length, Padding, Size, Vector};
use iced::widget::{Button, Column, Container, Row, Space, Text};
use crate::app::design::{style_action_button, style_action_button_selected};
use crate::app::main::{EditorApp, AppMessage};
use crate::data::polygon::BooleanResource;
use crate::point_editor::point::PathsToEditorPoints;
use crate::point_editor::widget::PointEditUpdate;

pub(crate) struct BooleanState {
    pub(crate) test: usize,
    pub(crate) fill: FillOption,
    pub(crate) mode: ModeOption,
    pub(crate) solver: SolverOption,
    pub(crate) workspace: WorkspaceState,
    pub(crate) size: Size,
    pub(crate) cameras: HashMap<usize, Camera>,
}

#[derive(Debug, Clone)]
pub(crate) enum BooleanMessage {
    TestSelected(usize),
    FillSelected(FillOption),
    ModeSelected(ModeOption),
    SolverSelected(SolverOption),
    PointEdited(PointEditUpdate),
    WorkspaceSized(Size),
    WorkspaceZoomed(f32),
    WorkspaceDraged(Vector<f32>),
}

impl EditorApp {
    fn sidebar(&self) -> Column<AppMessage> {
        let count = self.app_resource.boolean.count;
        let mut column = Column::new().push(Space::new(Length::Fill, Length::Fixed(2.0)));
        for index in 0..count {
            let is_selected = self.state.boolean.test == index;

            column = column.push(
                Container::new(
                    Button::new(Text::new(format!("test_{}", index)))
                        .width(Length::Fill)
                        .on_press(AppMessage::Bool(BooleanMessage::TestSelected(index)))
                        .style(if is_selected { style_action_button_selected } else { style_action_button })
                ).padding(self.design.action_padding())
            );
        }

        column
    }

    pub(crate) fn boolean_content(&self) -> Row<AppMessage> {
        Row::new()
            .push(
                scrollable(
                    Container::new(self.sidebar())
                        .width(Length::Fixed(160.0))
                        .height(Length::Shrink)
                        .align_x(Alignment::Start)
                        .padding(Padding::new(0.0).right(8))
                        .style(style_sidebar_background)
                ).direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new()
                        .width(4)
                        .margin(0)
                        .scroller_width(4)
                        .anchor(scrollable::Anchor::Start),
                ))
            )
            .push(self.boolean_workspace())
    }

    pub(crate) fn update_boolean(&mut self, message: BooleanMessage) {
        match message {
            BooleanMessage::TestSelected(index) => self.set_test(index),
            BooleanMessage::SolverSelected(solver) => self.update_boolean_solver(solver),
            BooleanMessage::FillSelected(fill) => self.update_boolean_fill(fill),
            BooleanMessage::ModeSelected(mode) => self.update_boolean_mode(mode),
            BooleanMessage::PointEdited(update) => self.update_boolean_point(update),
            BooleanMessage::WorkspaceSized(size) => self.update_boolean_size(size),
            BooleanMessage::WorkspaceZoomed(zoom) => self.update_boolean_zoom(zoom),
            BooleanMessage::WorkspaceDraged(drag) => self.update_boolean_drag(drag),
        }
    }

    fn set_test(&mut self, index: usize) {
        self.state.boolean.set_test(index, &mut self.app_resource.boolean);
        self.state.boolean.update_boolean_solution();
    }

    fn update_boolean_size(&mut self, size: Size) {
        self.state.boolean.size = size;
        let points = &self.state.boolean.workspace.points;
        if self.state.boolean.workspace.camera.is_empty() && !points.is_empty() {
            let rect = IntRect::with_iter(points.iter().map(|p| &p.pos))
                .unwrap_or(IntRect::new(-10_000, 10_000, -10_000, 10_000));
            let camera = Camera::new(rect, size);
            self.state.boolean.workspace.camera = camera;
        }
    }

    fn update_boolean_solver(&mut self, solver: SolverOption) {
        self.state.boolean.solver = solver;
        self.state.boolean.update_boolean_solution();
    }

    fn update_boolean_fill(&mut self, fill: FillOption) {
        self.state.boolean.fill = fill;
        self.state.boolean.update_boolean_solution();
    }

    fn update_boolean_mode(&mut self, mode: ModeOption) {
        self.state.boolean.mode = mode;
        self.state.boolean.update_boolean_solution();
    }
}

impl BooleanState {
    pub(crate) fn new(resource: &mut BooleanResource) -> Self {
        let mut state = BooleanState {
            test: usize::MAX,
            fill: FillOption::NonZero,
            mode: ModeOption::Xor,
            solver: SolverOption::Auto,
            workspace: Default::default(),
            cameras: HashMap::with_capacity(resource.count),
            size: Size::ZERO,
        };

        state.set_test(0, resource);
        state.update_boolean_solution();
        state
    }

    fn set_test(&mut self, index: usize, resource: &mut BooleanResource) {
        if let Some(test) = resource.load(index) {
            let editor_points = &mut self.workspace.points;

            if editor_points.is_empty() {
                editor_points.reserve(test.clip_paths.points_count() + test.subj_paths.points_count())
            } else {
                editor_points.clear();
            }

            self.workspace.subj = test.subj_paths.clone();
            self.workspace.clip = test.clip_paths.clone();

            self.workspace.subj.feed_edit_points(0, editor_points);
            self.workspace.clip.feed_edit_points(1, editor_points);

            self.cameras.insert(self.test, self.workspace.camera);
            let mut camera = *self.cameras.get(&index).unwrap_or(&Camera::empty());
            if camera.is_empty() && self.size.width > 0.001 {
                let rect = IntRect::with_iter(editor_points.iter().map(|p| &p.pos))
                    .unwrap_or(IntRect::new(-10_000, 10_000, -10_000, 10_000));
                camera = Camera::new(rect, self.size);
            }

            self.workspace.camera = camera;

            self.test = index;
        }
    }

    fn update_boolean_solution(&mut self) {
        let subj = &self.workspace.subj;
        let clip = &self.workspace.clip;
        let fill_rule = self.fill.to_fill_rule();
        if let Some(overlay_rule) = self.mode.to_overlay_rule() {
            let solution = Overlay::with_contours(subj, clip)
                .into_graph(fill_rule)
                .extract_shapes_min_area(overlay_rule, 0);
            self.workspace.solution = solution;
        }
    }

    pub(super) fn update_boolean_point(&mut self, update: PointEditUpdate) {
        self.workspace.points[update.index] = update.point.clone();
        let m_index = update.point.index;
        if m_index.group_index == 0 {
            self.workspace.subj[m_index.path_index][m_index.point_index] = update.point.pos;
        } else {
            self.workspace.clip[m_index.path_index][m_index.point_index] = update.point.pos;
        }
        self.update_boolean_solution();
    }
}

