mod decryption;
mod save;

use save::{Save, SaveInfo, SavesData};

use anyhow::Result;
use iced::alignment::{Horizontal, Vertical};
use iced::executor;
use iced::widget::{column, container, row, text};
use iced::Length;
use iced::{
    Application, Background, Border, Color, Command, Element, Settings, Shadow, Theme,
};
use iced::window::icon;

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

#[derive(Debug, Default)]
struct NineSaves {
    data: SavesData,
}

impl NineSaves {
    pub fn new() -> Result<Self> {
        Ok(Self {
            data: SavesData::new()?,
        })
    }
}

impl SaveInfo {
    fn formatted_time(&self) -> String {
        let hours = (self.playtime / 3600.).trunc() as u32;
        let minutes = ((self.playtime / 60.) % 60.).trunc() as u8;
        let h_disp = match hours {
            0 => "".to_string(),
            1 => "1 hour ".to_string(),
            _ => format!("{} hours ", hours),
        };
        let m_disp = match minutes {
            1 => "1 minute".to_string(),
            _ => format!("{} minutes", minutes),
        };
        format!("{}{}", h_disp, m_disp)
    }
}

enum Message {}

impl Application for NineSaves {
    type Executor = executor::Default;
    type Flags = ();
    type Message = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut nine_saves = NineSaves::new().unwrap();
        nine_saves.data.refresh().unwrap();
        nine_saves.data.slots.sort_by(|a, b| a.name.cmp(&b.name));
        (nine_saves, Command::none())
    }

    fn title(&self) -> String {
        String::from("Nine Saves")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
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
        let game_slots = container(column![
            container(text("Game Slots").size(25))
                .center_x()
                .width(Length::Fill)
                .padding(20),
            column(self.data.slots.iter().map(|s| {
                let info = s.info.as_ref().unwrap();
                container(column![
                    container(text(&s.name).size(20))
                        .center_x()
                        .width(Length::Fill),
                    container(text(format!("Level {}", info.level)))
                        .center_x()
                        .width(Length::Fill),
                    container(text(info.formatted_time()))
                        .center_x()
                        .width(Length::Fill),
                ])
                .style(box_appearance)
                .padding(20)
                .center_x()
                .width(Length::Fill)
                .into()
            }))
            .spacing(10),
        ]);

        column![
            container(text("Nine Saves").size(30))
                .center_x()
                .align_y(Vertical::Top)
                .width(Length::Fill),
            row![
                game_slots,
                container(column![]).width(Length::FillPortion(1))
            ]
            .spacing(40)
            .padding(20),
        ]
        .into()
    }
    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
