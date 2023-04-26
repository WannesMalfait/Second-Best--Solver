use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use second_best::position;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .init_resource::<UiState>()
        .init_resource::<SolverOutput>()
        .add_startup_system(launch_solver)
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_system(draw_board)
        .add_system(log_solver_output)
        .run();
}

#[derive(Default, Resource)]
struct SolverOutput {
    msgs: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

fn log_solver_output(reader: Res<SolverOutput>) {
    let mut msgs = reader.msgs.lock().unwrap();
    for msg in msgs.iter() {
        println!("{}", msg);
    }
    msgs.clear();
}

#[derive(Resource)]
struct Channel {
    stdin: std::process::ChildStdin,
}

impl Drop for Channel {
    fn drop(&mut self) {
        use std::io::Write;
        // Tell the solver to quit as well!
        writeln!(self.stdin, "quit").unwrap();
    }
}

#[derive(Resource)]
struct UiState {
    pos: position::Position,
    /// Index of the current move we are looking at.
    curr_move: Option<usize>,
    /// List of all the moves played.
    moves_played: Vec<position::PlayerMove>,
    /// Spot on the board where we started dragging from.
    drag_start: Option<(usize, usize)>,
    /// The depth to search the analyis in.
    depth: usize,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            pos: position::Position::default(),
            curr_move: None,
            moves_played: vec![],
            drag_start: None,
            depth: 10,
        }
    }
}

impl UiState {
    fn make_move(&mut self, pmove: position::PlayerMove) {
        if let Some(curr_move) = self.curr_move {
            // Shorten the moves list to the current move,
            // since the move to be played is in that position.
            self.moves_played.truncate(curr_move + 1);
        }
        self.pos
            .try_make_move(pmove)
            .expect("Move given should be valid");
        self.moves_played.push(pmove);
        self.curr_move = match self.curr_move {
            None => Some(0),
            Some(n) => Some(n + 1),
        };
    }

    fn set_curr_move(&mut self, move_index: Option<usize>) {
        self.curr_move = move_index;
        self.pos = position::Position::default();
        if let Some(move_i) = move_index {
            for pmove in &self.moves_played[0..=move_i] {
                self.pos
                    .try_make_move(*pmove)
                    .expect("Move given should be valid");
            }
        }
    }
}

struct SpotColor {}
impl SpotColor {
    const EMPTY: egui::Color32 = egui::Color32::from_rgb(130, 96, 37);
    const FROM: egui::Color32 = egui::Color32::from_rgb(100, 70, 20);
    const BLACK: egui::Color32 = egui::Color32::from_gray(10);
    const WHITE: egui::Color32 = egui::Color32::from_gray(210);
}

fn launch_solver(solver_output: Res<SolverOutput>, mut commands: Commands) {
    let path = "./target/release/second-best";
    let mut child = std::process::Command::new(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let stdin = child.stdin.take().unwrap();
    commands.insert_resource(Channel { stdin });
    let msgs = solver_output.msgs.clone();
    std::thread::spawn(move || {
        use std::io::BufRead;
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            msgs.lock().unwrap().push(line.unwrap());
        }
    });
}

fn draw_board(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut channel: ResMut<Channel>,
) {
    let ctx = contexts.ctx_mut();
    egui::SidePanel::right("Move list").show(ctx, |ui| {
        ui.heading("Moves Panel");
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() / 2.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                egui::Grid::new("Moves list").striped(true).show(ui, |ui| {
                    let mut selected_move = None;
                    // Labels for the columns.
                    ui.label("Turn Number");
                    ui.label("Black Move");
                    ui.label("White Move");
                    ui.end_row();
                    for (i, pmove) in ui_state.moves_played.iter().enumerate() {
                        if i % 2 == 0 {
                            // Every row, show the turn number.
                            ui.label(format!("{}.", i / 2 + 1));
                        }
                        if ui
                            .add_sized(
                                ui.available_size(),
                                egui::SelectableLabel::new(
                                    Some(i) == ui_state.curr_move,
                                    pmove.to_string(),
                                ),
                            )
                            .clicked()
                        {
                            selected_move = Some(i);
                        };
                        if i % 2 == 1 {
                            // Every two moves we have a new row.
                            ui.end_row();
                        }
                    }
                    if selected_move.is_some() {
                        ui_state.set_curr_move(selected_move);
                    }
                });
            });
        ui.separator();
        egui::TopBottomPanel::bottom("Position Information").show_inside(ui, |ui| {
            ui.heading("Position Information");
            if let Some(bmove) = ui_state.pos.banned_move() {
                ui.label(format!(
                    "Banned move: {}",
                    position::BitboardMove::StoneMove(bmove).to_string(&ui_state.pos),
                ));
            } else {
                ui.label("No banned move");
            }
            if ui_state.pos.has_alignment(false) {
                ui.label(format!(
                    "{} has an alignment",
                    match ui_state.pos.current_player().other() {
                        position::Color::Black => "Black",
                        position::Color::White => "White",
                    }
                ));
            } else {
                ui.label("No alignments");
            }
            if ui_state.pos.game_over() {
                ui.label(egui::RichText::new("Game Over!").font(egui::FontId::proportional(40.0)));
            }
        });
        egui::TopBottomPanel::top("Solver Controls").show_inside(ui, |ui| {
            ui.heading("Solver Controls");
            ui.horizontal(|ui| {
                ui.label("Solver Depth");
                let response =
                    ui.add(egui::DragValue::new(&mut ui_state.depth).clamp_range(1..=25));
                response.on_hover_text("The depth to search to");
            });
            if ui
                .button("eval")
                .on_hover_text("Get the evaluation for the current position")
                .clicked()
            {
                use std::io::Write;
                // Stop any previous searches.
                writeln!(channel.stdin, "stop").unwrap();
                // Set to the correct position.
                if let Some(curr_move) = ui_state.curr_move {
                    writeln!(
                        channel.stdin,
                        "set-pos {}",
                        ui_state.moves_played[..=curr_move]
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                    .unwrap();
                } else {
                    writeln!(channel.stdin, "set-pos").unwrap();
                }
                writeln!(channel.stdin, "eval {}", ui_state.depth).unwrap();
            }
            if ui
                .button("Stop")
                .on_hover_text("Stop any currently running searches")
                .clicked()
            {
                use std::io::Write;
                writeln!(channel.stdin, "stop").unwrap();
            }
        });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        let clicked_pos = response.interact_pointer_pos();

        // Fill the whole space.
        let board_radius = rect.width().min(rect.height()) / 2.0;
        // Background board.
        painter.circle_filled(
            rect.center(),
            board_radius,
            // Light brown color:
            egui::Color32::from_rgb(188, 138, 50),
        );

        // Second Best! button.
        let button =
            egui::Button::new("Second Best!").rounding(egui::Rounding::default().at_least(10.0));
        ui.set_enabled(ui_state.pos.can_second_best());
        if ui
            .put(
                egui::Rect::from_center_size(
                    rect.center(),
                    egui::vec2(board_radius / 5.0, board_radius / 6.0),
                ),
                button,
            )
            .clicked()
        {
            ui_state.make_move(position::PlayerMove::SecondBest);
        }

        let mut available_spots = match ui_state.drag_start {
            None => {
                if ui_state.pos.is_second_phase() {
                    ui_state.pos.from_spots(true)
                } else if let Some(banned_move) = ui_state.pos.banned_move() {
                    ui_state.pos.free_spots() & !banned_move
                } else {
                    ui_state.pos.free_spots()
                }
            }
            Some((stack, _)) => {
                let left = position::Position::column_mask(stack + position::Position::LEFT);
                let right = position::Position::column_mask(stack + position::Position::RIGHT);
                let opposite =
                    position::Position::column_mask(stack + position::Position::OPPOSITE);
                let possible_to = left | right | opposite;
                if let Some(banned_move) = ui_state.pos.banned_move() {
                    if (banned_move & position::Position::column_mask(stack)) != 0 {
                        // Banned move starts at the current "from" stack, so ensure that
                        // we don't play the banned move.
                        ui_state.pos.free_spots() & possible_to & !banned_move
                    } else {
                        ui_state.pos.free_spots() & possible_to
                    }
                } else {
                    ui_state.pos.free_spots() & possible_to
                }
            }
        };
        if ui_state.pos.has_alignment(false) {
            // No legal moves
            available_spots = 0;
        }
        // Calculate where all the spots are.
        let spot_radius = board_radius / 10.0;
        for stack_index in 0..8 {
            let direction = egui::Vec2::angled(
                // Each index increases the angle by 45°. We want to start at the top,
                // so we subtract 90° from the angle.
                stack_index as f32 * std::f32::consts::FRAC_PI_4 - std::f32::consts::FRAC_PI_2,
            );
            painter.text(
                rect.center() + (direction * spot_radius * 3.0),
                egui::Align2::CENTER_CENTER,
                stack_index.to_string(),
                egui::FontId::default(),
                egui::Color32::BLACK,
            );
            for offset in 0..3 {
                let spot_bb = position::Position::bb_of_spot(stack_index, offset);
                let spot_available = (spot_bb & available_spots) != 0;
                let spot_center =
                    rect.center() + (direction * (spot_radius * 1.1) * (2.0 + offset as f32) * 2.0);
                let mut color = match ui_state.pos.stone_at(stack_index, offset) {
                    None => SpotColor::EMPTY,
                    Some(color) => {
                        if Some((stack_index, offset)) == ui_state.drag_start {
                            // Don't show the stone here since we are dragging it somewhere else.
                            SpotColor::FROM
                        } else {
                            match color {
                                position::Color::Black => SpotColor::BLACK,
                                position::Color::White => SpotColor::WHITE,
                            }
                        }
                    }
                };
                let stroke = if spot_available {
                    match ui_state.pos.current_player() {
                        position::Color::White => {
                            // Need more contrast in this case.
                            egui::Stroke::new(2.0, egui::Color32::RED)
                        }
                        position::Color::Black => egui::Stroke::new(2.0, egui::Color32::GREEN),
                    }
                } else {
                    egui::Stroke::NONE
                };
                if let Some(clicked_pos) = clicked_pos {
                    if (clicked_pos - spot_center).length_sq() < spot_radius.powi(2)
                        && spot_available
                    {
                        // Indicate that this is a possible place to drop the stone.
                        color = color.gamma_multiply(1.5);
                        if response.drag_started() {
                            if ui_state.pos.is_second_phase() {
                                ui_state.drag_start = Some((stack_index, offset));
                            } else {
                                ui_state.make_move(position::PlayerMove::StoneMove {
                                    from: None,
                                    to: stack_index,
                                });
                            }
                        }
                        if response.drag_released() {
                            if let Some((from_stack, _)) = ui_state.drag_start {
                                ui_state.make_move(position::PlayerMove::StoneMove {
                                    from: Some(from_stack),
                                    to: stack_index,
                                });
                            }
                        }
                    }
                } else {
                    ui_state.drag_start = None;
                }
                painter.circle(spot_center, spot_radius, color, stroke);
            }
        }

        if let Some(drag_start) = ui_state.drag_start {
            let direction = egui::Vec2::angled(
                drag_start.0 as f32 * std::f32::consts::FRAC_PI_4 - std::f32::consts::FRAC_PI_2,
            );
            let spot_center = rect.center()
                + (direction * (spot_radius * 1.1) * (2.0 + drag_start.1 as f32) * 2.0);

            painter.line_segment(
                [spot_center, clicked_pos.unwrap()],
                egui::Stroke::new(2.0, egui::Color32::DARK_BLUE),
            );
            painter.circle_filled(
                clicked_pos.unwrap(),
                spot_radius,
                match ui_state.pos.current_player() {
                    position::Color::Black => SpotColor::BLACK,
                    position::Color::White => SpotColor::WHITE,
                },
            );
        }
    });
}
