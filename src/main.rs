mod decryption;
mod save;

use save::{SaveInfo, SavesData};

use anyhow::Result;
use iced::alignment::{Horizontal, Vertical};
use iced::executor;
use iced::widget::{column, container, row, text};
use iced::Length;
use iced::{Application, Border, Color, Command, Element, Settings, Shadow, Theme};

fn main() -> iced::Result {
    NineSaves::run(Settings::default())
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

impl Application for NineSaves {
    type Executor = executor::Default;
    type Flags = ();
    type Message = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut nine_saves = NineSaves::new().unwrap();
        nine_saves.data.refresh().unwrap();
        let info = SaveInfo::decrypt_from(&nine_saves.data.slots[2]).unwrap();
        println!("{:?}", info);
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
            text_color: Some(Color::BLACK),
            background: None,
            border: Border {
                color: Color::BLACK,
                width: 2.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
        };
        column![
            container(text("Nine Saves").size(30))
                .center_x()
                .align_y(Vertical::Top)
                .width(Length::Fill),
            row![
                container(
                    column(self.data.slots.iter().map(|s| {
                        let info = s.info.as_ref().unwrap();
                        let hours = (info.playtime / 3600.).trunc() as u32;
                        let minutes = ((info.playtime / 60.) % 60.).trunc() as u8;
                        let h_disp = match hours {
                            0 => "".to_string(),
                            1 => "1 hour ".to_string(),
                            _ => format!("{} hours ", hours),
                        };
                        let m_disp = match minutes {
                            1 => "1 minute".to_string(),
                            _ => format!("{} minutes", minutes)
                        };
                        container(column![
                            container(text(&s.name).size(20))
                                .center_x()
                                .width(Length::Fill),
                            container(text(format!("Level {}", info.level)))
                                .center_x()
                                .width(Length::Fill),
                            container(text(format!("{}{}", h_disp, m_disp)))
                                .center_x()
                                .width(Length::Fill),
                        ])
                        .style(box_appearance)
                        .padding(20)
                        .center_x()
                        .width(Length::FillPortion(2))
                        .into()
                    }))
                    .spacing(10),
                ),
                container(column![]).width(Length::FillPortion(1))
            ]
            .spacing(40)
            .padding(20),
        ]
        .into()
    }
}
