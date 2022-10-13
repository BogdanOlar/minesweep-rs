use eframe::{
    egui::{PointerButton, self, Layout, Label, RichText, Button, Context, TextStyle, Ui, CentralPanel, Sense, Direction, TopBottomPanel, Window},
    epaint::{Color32, Vec2},
    Frame, NativeOptions, App,
};
use egui_extras::{TableBuilder, Size};
use minefield::{Minefield, SpotState, StepResult, SpotKind};
use std::sync::mpsc::{channel, Receiver};
use chrono::Duration;
use timer::{Timer, Guard};

mod minefield;

fn main() {
    let mut options = NativeOptions::default();
    let app = MinesweepRsApp::default();

    // FIXME: Solve auto resizing
    let size_x = 38.0;
    let size_y = 44.0;
    options.initial_window_size = Some(
        Vec2::new(
            size_x * app.minefield.width() as f32,
            size_y * app.minefield.height() as f32
        )
    );
    options.resizable = false;

    eframe::run_native(
        "Minesweep-Rs",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}

struct MinesweepRsApp {
    minefield: Minefield,
    placed_flags: i32,
    timer: AppTimer,
    seconds_lapsed: i32,
    game_state: GameState,
    game_config: GameConfig,
    ui_toolbar_group: UiToolbarGroup,
}
enum UiToolbarGroup {
    None,
    About,
    Settings,
}

impl Default for UiToolbarGroup {
    fn default() -> Self {
        Self::None
    }
}

struct GameConfig {
    width: u16,
    height: u16,
    mines: u16,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { width: 10, height: 10, mines: 10 }
    }
}

impl App for MinesweepRsApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.request_repaint();
        ctx.set_debug_on_hover(false);

        self.render_top_panel(ctx, frame);
        self.render_bottom_panel(ctx, frame);
        self.render_toolbar_group(ctx, frame);
        self.render_minefield(ctx, frame);
    }
}

impl MinesweepRsApp {
    const MINE_CAHR: &str = "☢";
    const MINE_COLOR: Color32 = Color32::RED;
    const MINE_EXPLODED_CHAR: &str = "💥";
    const MINE_EPLODED_COLOR: Color32 = Color32::RED;
    const FLAG_CHAR: &str = "⚐";
    const FLAG_COLOR_CORRECT: Color32 = Color32::GREEN;
    const FLAG_COLOR_WRONG: Color32 = Color32::RED;
    const EMPTY_SPOT_CHARS: [&str; 9] = [" ", "1", "2", "3", "4", "5", "6", "7", "8"];
    const EMPTY_SPOT_COLORS: [Color32; Self::EMPTY_SPOT_CHARS.len()] = [
        Color32::WHITE, Color32::WHITE, Color32::WHITE,
        Color32::WHITE, Color32::WHITE, Color32::WHITE,
        Color32::WHITE, Color32::WHITE, Color32::WHITE
    ];
    const HIDDEN_SPOT_CHAR: &str = " ";
    const HIDDEN_SPOT_COLOR: Color32 = Color32::GRAY;
    const WON_COLOR: Color32 = Color32::GREEN;
    const LOST_COLOR: Color32 = Color32::RED;
    const READY_COLOR: Color32 = Color32::GRAY;
    const FLAG_COUNT_OK_COLOR: Color32 = Color32::GRAY;
    const FLAG_COUNT_ERR_COLOR: Color32 = Color32::LIGHT_RED;

    fn render_top_panel(&mut self, ctx: &Context, _: &mut Frame) {
        // Service app timer
        while self.timer.poll().is_some() {
            self.seconds_lapsed += 1;
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {

                // Config and game data
                ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
                    // refresh btn
                    let refresh_btn = ui.add(
                        Button::new(
                            RichText::new("🔄").text_style(TextStyle::Heading),
                        )
                    );

                    if refresh_btn.clicked() {
                        let minefield = Minefield::new(self.game_config.width, self.game_config.height).with_mines(self.game_config.mines);
                        *self = Self {
                            minefield,
                            ..Default::default()
                        };
                    }

                    ui.separator();

                    ui.allocate_ui_with_layout(Vec2::new(10.0, 10.0), Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add(
                            Label::new(
                            RichText::new("Mines").text_style(TextStyle::Body)
                        ));
                        ui.add(
                            Label::new(
                            RichText::new(format!("{}", self.minefield.mines())).monospace().text_style(TextStyle::Heading)
                        ));
                    });

                    ui.separator();

                    ui.allocate_ui_with_layout(Vec2::new(10.0, 10.0), Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add(
                            Label::new(
                            RichText::new("Flags").text_style(TextStyle::Body)
                        ));

                        let flag_count_color = if self.minefield.mines() as i32 >= self.placed_flags { Self::FLAG_COUNT_OK_COLOR } else { Self::FLAG_COUNT_ERR_COLOR };
                        ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.placed_flags))
                                .color(flag_count_color)
                                .monospace()
                                .text_style(TextStyle::Heading)
                        ));
                    });

                    ui.separator();

                    ui.allocate_ui_with_layout(Vec2::new(10.0, 10.0), Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add(
                            Label::new(
                            RichText::new("Time").text_style(TextStyle::Body)
                        ));
                        ui.add(
                            Label::new(
                            RichText::new(format!("{}", self.seconds_lapsed)).monospace().text_style(TextStyle::Heading)
                        ));
                    });

                    ui.separator();
                });

                // controls
                ui.with_layout(Layout::right_to_left(egui::Align::TOP), |ui| {

                    // settings button
                    let settings_btn = ui.add(
                        Button::new(
                            RichText::new("🛠").text_style(TextStyle::Heading),
                        )
                    );

                    if settings_btn.clicked() {
                        if let UiToolbarGroup::Settings = self.ui_toolbar_group {
                            self.ui_toolbar_group = UiToolbarGroup::None;
                        } else {
                            self.ui_toolbar_group = UiToolbarGroup::Settings;
                        }
                    }

                    // about button
                    let about_btn = ui.add(Button::new(RichText::new("ℹ").text_style(TextStyle::Heading)));

                    if about_btn.clicked() {
                        if let UiToolbarGroup::About = self.ui_toolbar_group {
                            self.ui_toolbar_group = UiToolbarGroup::None;
                        } else {
                            self.ui_toolbar_group = UiToolbarGroup::About;
                        }
                    }
                });
            });
            ui.add_space(10.);
        });
    }

    fn render_toolbar_group(&mut self, ctx: &Context, _: &mut Frame) {
        let mut open = true;

        match self.ui_toolbar_group {
            UiToolbarGroup::About => {
                Window::new("Settings").open(&mut open).show(ctx, |ui| {
                    let info = Label::new("TODO!");
                    ui.add(info);
                });
            },

            UiToolbarGroup::Settings => {
                let _ = Window::new("About Minesweep-Rs").open(&mut open).show(ctx, |ui| {
                    let info = Label::new("A Rust implementation of the popular game, using the `egui` library.");
                    ui.add(info);
                    ui.hyperlink("https://github.com/BogdanOlar/minesweep-rs");
                });
            },

            UiToolbarGroup::None => {},
        }

        if !open {
            self.ui_toolbar_group = UiToolbarGroup::None;
        }
    }

    fn render_bottom_panel(&mut self, ctx: &Context, _: &mut Frame) {
        // define a TopBottomPanel widget
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                match self.game_state {
                    GameState::Ready => {
                        ui.add(Label::new(
                            RichText::new("Ready")
                                .small()
                                .color(Self::READY_COLOR)
                                .text_style(TextStyle::Monospace),
                        ));
                    },
                    GameState::Running => {
                    },
                    GameState::Stopped(is_won) => {
                        if is_won {
                            ui.add(Label::new(
                                RichText::new("You WIN!")
                                    .color(Self::WON_COLOR)
                                    .text_style(TextStyle::Monospace),
                            ));
                        } else {
                            ui.add(Label::new(
                                RichText::new("You lost.")
                                    .color(Self::LOST_COLOR)
                                    .text_style(TextStyle::Monospace),
                            ));
                        }
                    },
                }
            })
        });
    }

    fn render_minefield(&mut self, ctx: &Context, _: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let size = 30.0;
            TableBuilder::new(ui)
                .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
                .columns(Size::Absolute { initial: size - 1.0, range: (size - 1.0, size - 1.0) }, self.minefield.width() as usize)
                .body(|mut body| {
                    for y in 0..self.minefield.height() {
                        body.row(size + 2.0, |mut row| {
                            for x in 0..self.minefield.width() {
                                row.col(|ui| {
                                    self.render_spot(x, y, size, ui);
                                });
                            }
                        });
                    }
                }
            );
        });
    }

    /// Render one spot/tile at the given field coordinates
    fn render_spot(&mut self, x: u16, y: u16, size: f32, ui: &mut Ui) {
        let spot = self.minefield.spot(x, y).unwrap();

        match self.game_state {
            GameState::Ready | GameState::Running => {
                match spot.state() {
                    SpotState::Hidden => {
                        let hidden_btn = Button::new(
                            RichText::new(Self::HIDDEN_SPOT_CHAR)
                            .color(Self::HIDDEN_SPOT_COLOR)
                            .monospace()
                            .size(size)
                        );
                        let hidden_btn = ui.add_enabled(true, hidden_btn);

                        if hidden_btn.clicked_by(PointerButton::Primary) {
                            self.check_ready_to_running();

                            if self.minefield.step(x, y) == StepResult::Boom {
                                self.game_over(false);
                            } else if self.minefield.is_cleared() {
                                self.game_over(true);
                            }
                        }

                        if hidden_btn.clicked_by(PointerButton::Secondary) {
                            self.check_ready_to_running();
                            self.placed_flags += self.minefield.toggle_flag(x, y);

                            if self.minefield.is_cleared() {
                                self.game_over(true);
                            }
                        }
                    },
                    SpotState::Revealed => {
                        if let SpotKind::Empty(n) = spot.kind() {
                            let empty_lbl = Label::new(
                                RichText::new(Self::EMPTY_SPOT_CHARS[n as usize])
                                .color(Self::EMPTY_SPOT_COLORS[n as usize])
                                .monospace()
                                .size(size)
                            );

                            let empty_lbl = ui.add_enabled(true, empty_lbl.sense(Sense::click()));

                            if empty_lbl.clicked_by(PointerButton::Middle) {
                                self.check_ready_to_running();

                                if self.minefield.try_resolve_step(x, y) == StepResult::Boom {
                                    self.game_over(false);
                                } else if self.minefield.is_cleared() {
                                    self.game_over(true);
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    },
                    SpotState::Flagged => {
                        let flag_btn = Button::new(
                            RichText::new(Self::FLAG_CHAR)
                            .color(Self::FLAG_COLOR_CORRECT)
                            .monospace()
                            .size(size)
                        );
                        let flag_btn = ui.add_enabled(true, flag_btn);

                        if flag_btn.clicked_by(PointerButton::Secondary) {
                            self.placed_flags += self.minefield.toggle_flag(x, y);

                            if self.minefield.is_cleared() {
                                self.game_over(true);
                            }
                        }
                    },
                    SpotState::Exploded => {
                        // Can't have exploded mine while gamestate is not `Stopped`
                        unreachable!()
                    },
                }
            },

            GameState::Stopped(is_won) => {
                match spot.state() {
                    SpotState::Hidden => {
                        match spot.kind() {
                            SpotKind::Mine => {
                                let mine_btn = Button::new(
                                    RichText::new(Self::MINE_CAHR)
                                    .color(Self::MINE_COLOR)
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(false, mine_btn);
                            },
                            SpotKind::Empty(_) => {
                                let hidden_btn = Button::new(
                                    RichText::new(Self::HIDDEN_SPOT_CHAR)
                                    .color(Self::HIDDEN_SPOT_COLOR)
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(false, hidden_btn);
                            },
                        }
                    },
                    SpotState::Revealed => {
                        match spot.kind() {
                            SpotKind::Mine => {
                                // Can't have a revealed spot of mine kind. If a mine is revealed then the spot's
                                // state becomes `Exploded`, not `Revealed`
                                unreachable!()
                            },
                            SpotKind::Empty(n) => {
                                let empty_lbl = Label::new(
                                    RichText::new(Self::EMPTY_SPOT_CHARS[n as usize])
                                    .color(Self::EMPTY_SPOT_COLORS[n as usize])
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(is_won, empty_lbl);
                            },
                        }
                    },
                    SpotState::Flagged => {
                        match spot.kind() {
                            SpotKind::Mine => {
                                let flag_btn = Button::new(
                                    RichText::new(Self::FLAG_CHAR)
                                    .color(Self::FLAG_COLOR_CORRECT)
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(false, flag_btn);
                            },
                            SpotKind::Empty(_) => {
                                let flag_btn = Button::new(
                                    RichText::new(Self::FLAG_CHAR)
                                    .color(Self::FLAG_COLOR_WRONG)
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(false, flag_btn);
                            },
                        }
                    },
                    SpotState::Exploded => {
                        match spot.kind() {
                            SpotKind::Mine => {
                                let mine_btn = Button::new(
                                    RichText::new(Self::MINE_EXPLODED_CHAR)
                                    .color(Self::MINE_EPLODED_COLOR)
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(false, mine_btn);
                            },
                            SpotKind::Empty(_) => {
                                // Only a spot of kind `Mine` can have the state `Exploded`. Anything else is a mistake
                                unreachable!()
                            },
                        }
                    },
                }
            },
        }
    }

    fn game_over(&mut self, is_won: bool) {
        self.game_state = GameState::Stopped(is_won);
        self.timer.stop();
    }

    fn check_ready_to_running(&mut self) {
        if self.game_state == GameState::Ready {
            self.game_state = GameState::Running;
            self.timer.start();
        }
    }
}

impl Default for MinesweepRsApp {
    fn default() -> Self {
        Self {
            minefield: Minefield::new(10, 10).with_mines(10),
            placed_flags: 0,
            seconds_lapsed: 0,
            timer: AppTimer::default(),
            game_state: GameState::Ready,
            game_config: GameConfig::default(),
            ui_toolbar_group: UiToolbarGroup::default(),
        }
    }
}

#[derive(Default)]
pub struct AppTimer {
    timer: Option<Timer>,
    guard: Option<Guard>,
    rx: Option<Receiver<()>>
}

impl AppTimer {
    pub fn stop(&mut self) {
        self.guard = None;
        self.timer = None;
        self.rx = None;
    }

    pub fn start(&mut self) {
        let (tx, rx) = channel();
        let timer = Timer::new();
        let guard = timer.schedule_repeating(Duration::seconds(1), move || {
                tx.send(()).unwrap();
        });

        self.timer = Some(timer);
        self.guard = Some(guard);
        self.rx = Some(rx);
    }

    pub fn poll(&self) -> Option<()> {
        if let Some(rx) = &self.rx {
            rx.try_iter().next()
        } else {
            None
        }
    }
}

/// Current state of the game
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GameState {
    /// Game is ready to start running
    Ready,

    /// Game is running
    Running,

    /// Game is stopped, and was either won (`true`), or lost (`false`)
    Stopped(bool)
}
