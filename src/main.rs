mod decryption;
mod error;
mod save;

use save::{Save, SaveInfo, SavesData};

use anyhow::Result;
use iced::alignment::{Horizontal, Vertical};
use iced::executor;
use iced::theme;
use iced::widget::{column, container, radio, row, scrollable, text, Button, TextInput};
use iced::window::icon;
use iced::Length;
use iced::{
    Application, Background, Border, Color, Command, Element, Padding, Settings, Shadow, Theme,
};

const DEBUG: bool = false;

pub enum AppColor {
    SaveBorder,
    SaveBackground,
}

impl AppColor {
    pub fn color(self) -> Color {
        match self {
            Self::SaveBorder => Color::from_rgb8(232, 201, 90),
            Self::SaveBackground => Color::from_rgb8(33, 28, 51),
        }
    }
}

fn main() -> iced::Result {
    NineSaves::run(Settings {
        window: iced::window::Settings {
            icon: Some(
                icon::from_file_data(
                    include_bytes!("../assets/tmp_logo.png"),
                    Some(image::ImageFormat::Png),
                )
                .expect("failed to load icon"),
            ),
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    SaveSlotToNewExternal,
    WriteExternalToSlot,
    WriteSlotToExternal,
    DeleteExternal,
}

#[derive(Debug, Default)]
struct NineSaves {
    data: SavesData,
    slot_selected: Option<usize>,
    external_selected: Option<usize>,
    action_selected: Option<Action>,
    new_save_name: String,
    error_status: Option<String>,
}

impl NineSaves {
    pub fn new() -> Result<Self> {
        Ok(Self {
            data: SavesData::new()?,
            ..Default::default()
        })
    }
    pub fn action_ready(&self) -> bool {
        match self.action_selected {
            Some(Action::SaveSlotToNewExternal) => {
                self.slot_selected.is_some()
                    && !self.new_save_name.is_empty()
                    && !self
                        .data
                        .saves
                        .iter()
                        .any(|s| s.name.as_str() == self.new_save_name.as_str())
            }
            Some(Action::WriteExternalToSlot | Action::WriteSlotToExternal) => {
                self.slot_selected.is_some() && self.external_selected.is_some()
            }
            Some(Action::DeleteExternal) => self.external_selected.is_some(),
            _ => false,
        }
    }
}

impl SaveInfo {
    fn formatted_time(&self) -> String {
        let hours = (self.playtime / 3600.).trunc() as u32;
        let minutes = ((self.playtime / 60.) % 60.).trunc() as u8;
        let h_disp = match hours {
            0 => "".to_string(),
            _ => format!("{}h ", hours),
        };
        let m_disp = format!("{}m", minutes);
        format!("{}{}", h_disp, m_disp)
    }
}

#[derive(Debug, Clone)]
enum Message {
    SlotPicked(usize),
    SavePicked(usize),
    ActionPicked(Action),
    NewSaveNameChanged(String),
    Refresh,
    PerformAction,
}

#[derive(Copy, Clone)]
enum SaveListKind {
    Slots,
    Saves,
}

impl NineSaves {
    fn action_radio(&self, action: Action) -> Element<Message> {
        radio("", action, self.action_selected, Message::ActionPicked).into()
    }
    fn save_box(&self, kind: SaveListKind, list: &[Save], i: usize) -> Element<Message> {
        let save = &list[i];
        let box_appearance = container::Appearance {
            text_color: Some(Color::WHITE),
            background: Some(Background::Color(AppColor::SaveBackground.color())),
            border: Border {
                color: AppColor::SaveBorder.color(),
                width: 2.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
        };
        let info = save.info.as_ref().unwrap();
        container(row![
            container(radio(
                "",
                i,
                match kind {
                    SaveListKind::Slots => self.slot_selected,
                    SaveListKind::Saves => self.external_selected,
                },
                match kind {
                    SaveListKind::Slots => Message::SlotPicked,
                    SaveListKind::Saves => Message::SavePicked,
                }
            ))
            .center_y()
            .height(Length::Shrink),
            row![
                container(text(&save.name).size(20)).width(Length::Fill),
                container(row![
                    container(text(format!("Level {}", info.level)))
                        .width(Length::Fill)
                        .center_x(),
                    container(text(info.formatted_time()))
                        .width(Length::Fill)
                        .center_x(),
                ])
                .padding(Padding::from([10, 0, 0, 0]))
            ],
        ])
        .style(box_appearance)
        .padding(10)
        .center_x()
        .width(Length::Fill)
        .into()
    }
    fn selected_slot_display(&self) -> Element<Message> {
        container(match self.slot_selected {
            Some(slot) => &self.data.slots[slot].name,
            None => "selected slot",
        })
        .style(theme::Container::Box)
        .into()
    }
    fn selected_save_display(&self) -> Element<Message> {
        container(match self.external_selected {
            Some(slot) => &self.data.saves[slot].name,
            None => "selected save",
        })
        .style(theme::Container::Box)
        .into()
    }
}

impl Application for NineSaves {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let app = match NineSaves::new() {
            Ok(mut nine_saves) => {
                nine_saves.try_refresh();
                nine_saves
            }
            Err(e) => {
                NineSaves {
                    error_status: Some(format!("{:?}", e)),
                    ..Default::default()
                }
                
            }
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("Nine Saves")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SlotPicked(i) => self.slot_selected = Some(i),
            Message::SavePicked(i) => self.external_selected = Some(i),
            Message::ActionPicked(action) => self.action_selected = Some(action),
            Message::NewSaveNameChanged(s) => self.new_save_name = s.clone(),
            Message::Refresh => self.try_refresh(),
            Message::PerformAction => match self.action_selected {
                Some(Action::SaveSlotToNewExternal) => {
                    let destination = self.data.external_saves_dir.join(&self.new_save_name);
                    self.data.slots[self.slot_selected.expect("must exist")]
                        .copy(&destination)
                        .unwrap();
                    self.try_refresh();
                }
                Some(Action::WriteExternalToSlot) => {
                    let slot = &self.data.slots[self.slot_selected.expect("must exist")];
                    let source = &self.data.saves[self.external_selected.expect("must exist")];
                    self.data.backup_and_overwrite(source, slot).unwrap();
                    self.try_refresh();
                }
                Some(Action::WriteSlotToExternal) => {
                    let slot = &self.data.slots[self.slot_selected.expect("must exist")];
                    let save = &self.data.saves[self.external_selected.expect("must exist")];
                    self.data.backup_and_overwrite(slot, save).unwrap();
                    self.try_refresh();
                }
                Some(Action::DeleteExternal) => {
                    let save = &self.data.saves[self.external_selected.expect("must exist")];
                    self.data.backup_and_delete(save).unwrap();
                    save.delete_dir().unwrap();
                    self.external_selected = None;
                    self.try_refresh();
                }
                _ => todo!(),
            },
        };
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let content = match &self.error_status {
            Some(e) => container(column![
                text("Nine Saves encountered an error").size(20),
                text(e)
            ])
            .padding(20)
            .into(),
            None => {
                let game_slots = container(column![
                    container(text("Game Slots").size(25))
                        .center_x()
                        .width(Length::Fill)
                        .padding(10),
                    container(scrollable(
                        column(self.data.slots.iter().enumerate().map(|(i, _)| {
                            self.save_box(SaveListKind::Slots, &self.data.slots, i)
                        }))
                        .spacing(5)
                    ))
                    .height(Length::Shrink),
                ])
                .height(Length::Shrink);

                let external_saves: Element<_> = column![
                    container(text("External Saves").size(25))
                        .center_x()
                        .width(Length::Fill)
                        .padding(10),
                    scrollable(
                        column(self.data.saves.iter().enumerate().map(|(i, _)| {
                            self.save_box(SaveListKind::Saves, &self.data.saves, i)
                        }))
                        .spacing(5)
                    )
                    .height(Length::Fill)
                ]
                .into();

                let save_slot_to_external = row![
                    self.action_radio(Action::SaveSlotToNewExternal),
                    row![
                        text("Save "),
                        self.selected_slot_display(),
                        text(" to new "),
                        container(
                            TextInput::new("save name", &self.new_save_name)
                                .on_input(Message::NewSaveNameChanged)
                        )
                        .max_width(100)
                    ],
                ];

                let write_slot_to_external = row![
                    self.action_radio(Action::WriteSlotToExternal),
                    row![
                        text("Write "),
                        self.selected_slot_display(),
                        text(" to "),
                        self.selected_save_display()
                    ]
                ];

                let write_external_to_slot = row![
                    self.action_radio(Action::WriteExternalToSlot),
                    row![
                        text("Write "),
                        self.selected_save_display(),
                        text(" to "),
                        self.selected_slot_display(),
                    ]
                ];

                let delete_external = row![
                    self.action_radio(Action::DeleteExternal),
                    row![text("Delete "), self.selected_save_display(),]
                ];

                let actions: iced::widget::Container<Message> = container(column![
                    container(text("Actions").size(25))
                        .center_x()
                        .padding(10)
                        .width(Length::Fill),
                    row![
                        container(
                            column![save_slot_to_external, write_slot_to_external].spacing(5)
                        )
                        .width(Length::Fill),
                        container(column![write_external_to_slot, delete_external].spacing(5))
                            .width(Length::Fill)
                    ]
                    .spacing(20),
                    row![
                        container(Button::new("Refresh").on_press(Message::Refresh))
                            .align_x(Horizontal::Left),
                        container({
                            let button = Button::new("Perform Action");
                            match self.action_ready() {
                                true => button.on_press(Message::PerformAction),
                                false => button,
                            }
                        })
                        .align_x(Horizontal::Right)
                        .width(Length::Fill)
                        .padding(10),
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(10)
                ])
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(Vertical::Bottom);

                let content: Element<_> = container(column![
                    container(text("Nine Saves").size(30))
                        .center_x()
                        .align_y(Vertical::Top)
                        .width(Length::Fill),
                    container(row![game_slots, external_saves].spacing(40))
                        .height(Length::FillPortion(2)),
                    actions
                ])
                .padding(20)
                .into();
                content
            }
        };

        if DEBUG {
            content.explain(Color::WHITE)
        } else {
            content
        }
    }
    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
