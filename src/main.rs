mod mips_int;

use iced::{Align, Application, Button, button, Clipboard, Color, Column, Command, Container, container, Element, executor, Font, Length, Row, Scrollable, scrollable, Settings, Text, VerticalAlignment};

use crate::mips_int::{MipsError, MipsInterpreter};
use crate::mips_int::register::RegNames;

const ASM_FILEPATH: &str = "data/game.asm";

#[derive(Debug, Clone, Copy)]
enum Message {
    BtnClick,
    LoadASM,
    NextStep,
    PreviousStep,
    CloseFocused,
}

/*
Window contains the core GUI work
 */
struct MipsWindow {
    backend: MipsInterpreter,
    output: String,
    load_button: button::State,
    next_button: button::State,
    go_button: button::State,
}

impl Application for MipsWindow {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut backend = MipsInterpreter::new();
        if let Err(MipsError::SyntaxError(e)) = backend.load_program(ASM_FILEPATH) {
            println!("Error parsing ASM on line: {}", e);
        };

        (MipsWindow {
            backend,
            load_button: button::State::new(),
            next_button: button::State::new(),
            go_button: button::State::new(),
            output: String::from("Output..."),
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("MIPS Interpreter")
    }

    fn update(&mut self, message: Self::Message, clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::BtnClick => {}
            Message::CloseFocused => {}
            Message::LoadASM => {
                self.backend.load_program(ASM_FILEPATH);
                self.output.clear();
            }
            _ => {}
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let col_reg_labels = Column::new()
            .align_items(Align::End)
            .width(Length::Units(115))
            .padding(15)
            .push(Text::new("Registers:").size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R0)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R1)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R2)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R3)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R4)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R5)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R6)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R7)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R8)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R9)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R10)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R11)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R12)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R13)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R14)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R15)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R16)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R17)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R18)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R19)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R20)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R21)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R22)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R23)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R24)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R25)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R26)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R27)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R28)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R29)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R30)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::R31)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::HI)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::LO)).size(15))
            .push(Text::new(self.backend.display_register(&RegNames::PC)).size(15));

        // Fonts
        let terminal_font = Font::External {
            name: "FixedFont",
            bytes: include_bytes!("../src/courier.ttf"),
        };

        let display = Column::new()
            .padding(15)
            .width(Length::Fill)
            .push(Text::new(&self.output).font(terminal_font));

        // All the buttons to run the system
        let b = Button::new(&mut self.load_button, Text::new("Load"))
            .on_press(Message::LoadASM);

        let buttons = Column::new()
            .push(b);

        let row = Row::new()
            .align_items(Align::Start)
            .push(col_reg_labels)
            .push(display)
            .push(buttons);

        Container::new(row)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            //.center_y()
            .into()
    }
}

fn main() {
    MipsWindow::run(Settings::default());
}
