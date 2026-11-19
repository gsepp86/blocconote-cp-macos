use iced::Alignment;
use iced::Length;
use iced::Settings;
use iced::Size;
use iced::Theme;
use iced::cloud::run;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_rule;
use iced::widget::row;
use iced::widget::text;
use iced::widget::{TextEditor, TextInput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Note {
    id: String,
    title: String,
    content: String,
    created: u64,
    updated: u64,
}

#[derive(Debug, Clone)]
enum Message {
    SelectedTab(usize),
    TitleChanged(String),
    ContentChanged(String),
    NewNote,
    DeleteNote,
    PointerMove,
    PointerLeave,
    HoveredTab(usize),
}

struct App {
    notes: Vec<Note>,
    order: Vec<String>,
    selected: Option<usize>,
    title_input: String,
    content_input: String,
    save_pending: bool,
    load_error: Option<String>,
    hover_tab: Option<usize>,
    mouse_pos: Option<Position>,
    dirty: bool,
}

impl Default for App {
    fn default() -> Self {
        let files_dir = {
            let mut p = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
            p.push("BloccoNoteCP");
            p
        };
        fs::create_dir_all(&files_dir).ok();
        let id = format!("blocco-note-cp-{}", std::process::id());
        let data_file = files_dir.join(format!("{id}.json"));
        let mut notes: Vec<Note>;
        let mut order: Vec<String>;
        if data_file.exists() {
            match fs::read_to_string(&data_file) {
                Ok(s) => match serde_json::from_str::<HashMap<String, Note>>(&s) {
                    Ok(m) => {
                        let mut order_tmp: Vec<String> = m.values().map(|n| n.id.clone()).collect();
                        order_tmp.sort_by_key(|id| m[id].created);
                        notes = order_tmp.iter().map(|id| m[id].clone()).collect();
                        order = order_tmp;
                        if notes.is_empty() {
                            let n = Note { id: new_id(), title: String::new(), content: String::new(), created: now(), updated: now() };
                            notes.push(n.clone());
                            order.push(n.id.clone());
                        }
                    }
                    Err(_) => {
                        let n = Note { id: new_id(), title: String::new(), content: String::new(), created: now(), updated: now() };
                        notes = vec![n.clone()];
                        order = vec![n.id.clone()];
                    }
                },
                Err(_) => {
                    let n = Note { id: new_id(), title: String::new(), content: String::new(), created: now(), updated: now() };
                    notes = vec![n.clone()];
                    order = vec![n.id.clone()];
                }
            }
        } else {
            let n = Note { id: new_id(), title: String::new(), content: String::new(), created: now(), updated: now() };
            notes = vec![n.clone()];
            order = vec![n.id.clone()];
        }
        let selected = Some(0);
        let title_input = notes[0].title.clone();
        let content_input = notes[0].content.clone();
        let mut app = Self {
            notes,
            order,
            selected,
            title_input,
            content_input,
            save_pending: false,
            load_error: None,
            hover_tab: None,
            mouse_pos: None,
            dirty: false,
        };
        app.title_input = app.notes[0].title.clone();
        app.content_input = app.notes[0].content.clone();
        app
    }
}

fn new_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    format!("{:x}-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos())
}

fn now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

impl App {
    fn save(&mut self) {
        let id = format!("blocco-note-cp-{}", std::process::id());
        let files_dir = {
            let mut p = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
            p.push("BloccoNoteCP");
            p
        };
        let data_file = files_dir.join(format!("{id}.json"));
        let map: HashMap<String, Note> = self.notes.iter().map(|n| (n.id.clone(), n.clone())).collect();
        match fs::write(&data_file, serde_json::to_string_pretty(&map).unwrap_or_default()) {
            Ok(()) => {
                self.save_pending = false;
                self.load_error = None;
            }
            Err(e) => self.load_error = Some(e.to_string()),
        }
    }

    fn ensure_note(&mut self, idx: usize) {
        if idx < self.notes.len() {
            let n = &mut self.notes[idx];
            n.title = self.title_input.clone();
            n.content = self.content_input.clone();
            n.updated = now();
        }
    }
}

impl iced::Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        "Blocco Note CP".into()
    }

    fn update(&mut self, message: Message) -> Option<Message> {
        match message {
            Message::SelectedTab(idx) => {
                if let Some(current) = self.selected {
                    self.ensure_note(current);
                }
                self.save();
                if let Some(n) = self.notes.get(idx) {
                    self.selected = Some(idx);
                    self.title_input = n.title.clone();
                    self.content_input = n.content.clone();
                }
                None
            }
            Message::TitleChanged(t) => {
                self.title_input = t;
                self.dirty = true;
                None
            }
            Message::ContentChanged(c) => {
                self.content_input = c;
                self.dirty = true;
                None
            }
            Message::NewNote => {
                let n = Note { id: new_id(), title: String::new(), content: String::new(), created: now(), updated: now() };
                self.notes.push(n.clone());
                self.order.push(n.id.clone());
                self.save();
                self.selected = Some(self.notes.len() - 1);
                self.title_input = n.title.clone();
                self.content_input = n.content.clone();
                self.dirty = true;
                None
            }
            Message::DeleteNote => {
                if let Some(idx) = self.selected {
                    self.ensure_note(idx);
                    let id = self.notes[idx].id.clone();
                    self.notes.remove(idx);
                    self.order.retain(|o| o != &id);
                    self.save();
                    if self.notes.is_empty() {
                        let n = Note { id: new_id(), title: String::new(), content: String::new(), created: now(), updated: now() };
                        self.notes.push(n.clone());
                        self.order.push(n.id.clone());
                        self.selected = Some(0);
                        self.title_input = n.title.clone();
                        self.content_input = n.content.clone();
                    } else if idx >= self.notes.len() {
                        self.selected = Some(self.notes.len() - 1);
                    } else {
                        self.selected = Some(idx);
                    }
                    self.dirty = true;
                }
                None
            }
            Message::PointerMove => {
                self.mouse_pos = Some(Position { x: 0.0, y: 0.0 });
                None
            }
            Message::PointerLeave => {
                self.mouse_pos = None;
                None
            }
            Message::HoveredTab(i) => {
                self.hover_tab = Some(i);
                None
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::subscription::run(|s| {
            let events = iced::subscription::events().map(|event| match event {
                iced::Event::Mouse(m) => match m {
                    iced::mouse::Event::CursorMoved { x: _, y: _, .. } => Message::PointerMove,
                    iced::mouse::Event::MouseDown(_) | iced::mouse::Event::MouseUp(_) | iced::mouse::Event::MouseWheel(_) => Message::PointerMove,
                    iced::mouse::Event::MouseLeave => Message::PointerLeave,
                    _ => Message::PointerLeave,
                },
                _ => Message::PointerLeave,
            });
            s.batch(vec![events])
        })
    }

    fn view(&self) -> iced::Element<Message> {
        let title_bar = title_bar(&self.notes, self.selected, self.hover_tab);
        let tab_bar = tab_bar(&self.notes, self.selected, self.hover_tab, self.mouse_pos);
        let content = content_area(&self.title_input, &self.content_input);
        let status = status_bar(self.dirty, &self.load_error);

        let mut col = column![title_bar, tab_bar, content, status];
        col = col.spacing(0);
        col.padding(0).into()
    }

    fn theme(&self) -> Theme {
        Theme::Custom(Box::new(Windows10Theme))
    }
}

struct Windows10Theme;

impl iced::Theme for Windows10Theme {
    type Styles = iced::theme::Theme;

    fn global(&self) -> iced::theme::GlobalStyles {
        iced::theme::GlobalStyles::default()
    }
}

fn title_bar(notes: &[Note], selected: Option<usize>, hover_tab: Option<usize>) -> Container<Message> {
    let (t, right) = if let Some(idx) = selected {
        let title = if notes[idx].title.is_empty() { "Senza titolo".into() } else { notes[idx].title.clone() };
        let title = format!("{} — Blocco Note CP", title);
        let right = window_buttons();
        (title, right)
    } else {
        ("Blocco Note CP".into(), iced::widget::space(Length::Fill))
    };
    container(
        row![
            text(&t)
                .size(13)
                .style(iced::theme::Text::Color(iced::Color::WHITE))
                .width(Length::Fill)
                .horizontal_alignment(Alignment::Center),
            right,
        ]
        .spacing(0)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(32.0)
    .style(
        iced::theme::container::Appearance {
            background: Some(iced::Color::from_rgb(0.168, 0.243, 0.365).into()),
            ..Default::default()
        },
    )
}

fn window_buttons() -> Row<Message> {
    row![
        psuedo_button(" _ ", Message::PointerLeave),
        psuedo_button(" □ ", Message::PointerLeave),
        psuedo_button(" ✕ ", Message::PointerLeave),
    ]
    .spacing(0)
    .width(128.0)
}

fn psuedo_button(label: &str, _msg: Message) -> Container<Message> {
    container(
        text(label)
            .size(12)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.85, 0.85, 0.85))),
    )
    .width(32.0)
    .height(24.0)
    .style(
        iced::theme::container::Appearance {
            background: Some(iced::Color::TRANSPARENT.into()),
            ..Default::default()
        },
    )
}

fn tab_bar(notes: &[Note], selected: Option<usize>, hover_tab: Option<usize>, mouse_pos: Option<Position>) -> Container<Message> {
    let tabs: Vec<Container<Message>> = notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let selected = selected == Some(i);
            let hovered = hover_tab == Some(i);
            let bg = if selected {
                iced::Color::from_rgb(0.0, 0.5, 0.85)
            } else if hovered {
                iced::Color::from_rgb(0.9, 0.9, 0.9)
            } else {
                iced::Color::from_rgb(0.95, 0.95, 0.95)
            };
            let label_color = if selected { iced::Color::WHITE } else { iced::Color::BLACK };
            let title = if n.title.is_empty() { "Senza titolo".into() } else { n.title.clone() };
            container(
                text(&title)
                    .size(12)
                    .style(iced::theme::Text::Color(label_color))
                    .horizontal_alignment(Alignment::Center),
            )
            .width(120.0)
            .height(32.0)
            .style(iced::theme::container::Appearance {
                background: Some(bg.into()),
                ..Default::default()
            })
            .on_press(Message::SelectedTab(i))
        })
        .collect();

    let add_tab = container(
        row![
            text("+")
                .size(16)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.5, 0.85))),
            text(" Nuovo")
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.5, 0.85))),
        ]
        .spacing(4.0)
        .horizontal_alignment(Alignment::Center),
    )
    .width(120.0)
    .height(32.0)
    .style(iced::theme::container::Appearance {
        background: Some(iced::Color::from_rgb(0.95, 0.95, 0.97).into()),
        ..Default::default()
    })
    .on_press(Message::NewNote);

    let tabs_width = 120.0 * (notes.len() + 1) as f32;
    container(
        row![tabs].spacing(0).width(tabs_width),
    )
    .width(Length::Fill)
    .height(32.0)
    .style(iced::theme::container::Appearance {
        background: Some(iced::Color::from_rgb(0.95, 0.95, 0.97).into()),
        ..Default::default()
    })
}

fn content_area(title: &str, content: &str) -> Column<Message> {
    let title_field = TextInput::new("", title)
        .size(13)
        .on_input(Message::TitleChanged)
        .on_submit(Message::ContentChanged)
        .placeholder("Titolo")
        .width(Length::Fill)
        .map(Message::PointerLeave)
        .into();

    let line = horizontal_rule(1.0).style(iced::theme::palette::Background::Color(iced::Color::from_rgb(0.79, 0.79, 0.79)));
    let content_field = TextEditor::new()
        .on_input(Message::ContentChanged)
        .value(content)
        .map(Message::PointerLeave)
        .into();

    column![
        title_field,
        line,
        content_field,
    ]
    .spacing(6)
    .padding(8)
    .width(Length::Fill)
    .height(Length::Fill)
}

fn status_bar(dirty: bool, load_error: &Option<String>) -> Container<Message> {
    let left = text(if dirty { "Non salvato" } else { "Salvato" }).size(11);
    let right = if let Some(e) = load_error.as_ref() {
        text(format!("Errore: {}", e)).size(11).style(iced::theme::Text::Color(iced::Color::RED))
    } else {
        text("Blocco Note CP").size(11).style(iced::theme::Text::Color(iced::Color::from_rgb(0.45, 0.45, 0.45)))
    };
    container(
        row![left, iced::widget::space(Length::Fill), right]
            .spacing(0)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(22.0)
    .style(iced::theme::container::Appearance {
        background: Some(iced::Color::from_rgb(0.93, 0.93, 0.93).into()),
        ..Default::default()
    })
}

mod dirs {
    use std::path::PathBuf;
    pub fn data_local_dir() -> Option<PathBuf> {
        option_env!("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                option_env!("HOME")
                    .map(|h| PathBuf::from(h).join(".local/share"))
            })
    }
}

pub fn main() {
    let app = App::new();
    run(
        "Blocco Note CP",
        Settings {
            window: iced::window::Settings {
                size: Size::new(900.0, 600.0),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .run(app)
}
