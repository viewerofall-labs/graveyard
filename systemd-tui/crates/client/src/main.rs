use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Terminal,
};
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use shared::{Command, Group, Response, ServiceInfo, ServiceStatus};

const SOCKET_PATH: &str = "/tmp/systemd-tui.sock";

#[derive(Clone, PartialEq)]
enum FilterMode {
    All,
    Active,
    Inactive,
    Failed,
}

impl FilterMode {
    fn next(&self) -> Self {
        match self {
            FilterMode::All => FilterMode::Active,
            FilterMode::Active => FilterMode::Inactive,
            FilterMode::Inactive => FilterMode::Failed,
            FilterMode::Failed => FilterMode::All,
        }
    }

    fn label(&self) -> &str {
        match self {
            FilterMode::All => "all",
            FilterMode::Active => "active",
            FilterMode::Inactive => "inactive",
            FilterMode::Failed => "failed",
        }
    }
}

#[derive(Clone, PartialEq)]
enum ViewMode {
    List,
    Detail,
}

#[derive(Clone)]
enum ListEntry {
    GroupHeader {
        name: String,
        collapsed: bool,
        active: usize,
        inactive: usize,
        failed: usize,
    },
    Service {
        info: ServiceInfo,
        id: usize,
    },
}

struct App {
    services: Vec<ServiceInfo>,
    groups: Vec<Group>,
    collapsed: Vec<bool>,
    list_state: ListState,
    status_message: String,
    filter: FilterMode,
    view_mode: ViewMode,
    detail_lines: Vec<String>,
    detail_scroll: usize,
    detail_service_name: String,
}

impl App {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            services: vec![],
            groups: vec![],
            collapsed: vec![],
            list_state,
            status_message: String::new(),
            filter: FilterMode::All,
            view_mode: ViewMode::List,
            detail_lines: vec![],
            detail_scroll: 0,
            detail_service_name: String::new(),
        }
    }

    fn set_data(&mut self, services: Vec<ServiceInfo>, groups: Vec<Group>) {
        let group_count = groups.len() + 1;
        self.collapsed = vec![false; group_count];
        self.services = services;
        self.groups = groups;
    }

    fn build_entries(&self) -> Vec<ListEntry> {
        let mut entries = vec![];
        let mut id = 1usize;
        let mut grouped: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (gi, group) in self.groups.iter().enumerate() {
            let collapsed = self.collapsed.get(gi).copied().unwrap_or(false);
            let mut active = 0;
            let mut inactive = 0;
            let mut failed = 0;

            for svc_name in &group.services {
                if let Some(svc) = self.services.iter().find(|s| &s.name == svc_name) {
                    match svc.status {
                        ServiceStatus::Active => active += 1,
                        ServiceStatus::Inactive => inactive += 1,
                        ServiceStatus::Failed => failed += 1,
                        ServiceStatus::Unknown => {}
                    }
                    grouped.insert(svc_name.clone());
                }
            }

            entries.push(ListEntry::GroupHeader {
                name: group.name.clone(),
                collapsed,
                active,
                inactive,
                failed,
            });

            if !collapsed {
                for svc_name in &group.services {
                    if let Some(svc) = self.services.iter().find(|s| &s.name == svc_name) {
                        if self.filter_matches(svc) {
                            entries.push(ListEntry::Service {
                                info: svc.clone(),
                                id,
                            });
                            id += 1;
                        }
                    }
                }
            }
        }

        let other_index = self.groups.len();
        let other_collapsed = self.collapsed.get(other_index).copied().unwrap_or(false);
        let other_services: Vec<&ServiceInfo> = self
            .services
            .iter()
            .filter(|s| !grouped.contains(&s.name) && self.filter_matches(s))
            .collect();

        if !other_services.is_empty() {
            let active = other_services
                .iter()
                .filter(|s| s.status == ServiceStatus::Active)
                .count();
            let inactive = other_services
                .iter()
                .filter(|s| s.status == ServiceStatus::Inactive)
                .count();
            let failed = other_services
                .iter()
                .filter(|s| s.status == ServiceStatus::Failed)
                .count();

            entries.push(ListEntry::GroupHeader {
                name: "Other".into(),
                collapsed: other_collapsed,
                active,
                inactive,
                failed,
            });

            if !other_collapsed {
                for svc in other_services {
                    entries.push(ListEntry::Service {
                        info: svc.clone(),
                        id,
                    });
                    id += 1;
                }
            }
        }

        entries
    }

    fn filter_matches(&self, svc: &ServiceInfo) -> bool {
        match self.filter {
            FilterMode::All => true,
            FilterMode::Active => svc.status == ServiceStatus::Active,
            FilterMode::Inactive => svc.status == ServiceStatus::Inactive,
            FilterMode::Failed => svc.status == ServiceStatus::Failed,
        }
    }

    fn next(&mut self) {
        let entries = self.build_entries();
        if entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % entries.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let entries = self.build_entries();
        if entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_collapse(&mut self) {
        let entries = self.build_entries();
        if let Some(selected) = self.list_state.selected() {
            if let Some(ListEntry::GroupHeader { name, .. }) = entries.get(selected) {
                let name = name.clone();
                if name == "Other" {
                    let idx = self.groups.len();
                    if let Some(c) = self.collapsed.get_mut(idx) {
                        *c = !*c;
                    }
                } else {
                    for (gi, group) in self.groups.iter().enumerate() {
                        if group.name == name {
                            if let Some(c) = self.collapsed.get_mut(gi) {
                                *c = !*c;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    fn jump_to_id(&mut self, id: usize) {
        let entries = self.build_entries();
        for (i, entry) in entries.iter().enumerate() {
            if let ListEntry::Service { id: eid, .. } = entry {
                if *eid == id {
                    self.list_state.select(Some(i));
                    return;
                }
            }
        }
        self.status_message = format!("No service with id {}", id);
    }

    fn selected_service(&self) -> Option<ServiceInfo> {
        let entries = self.build_entries();
        if let Some(i) = self.list_state.selected() {
            if let Some(ListEntry::Service { info, .. }) = entries.get(i) {
                return Some(info.clone());
            }
        }
        None
    }

    fn open_detail(&mut self, name: String, output: String) {
        self.detail_service_name = name;
        self.detail_lines = output.lines().map(|l| l.to_string()).collect();
        self.detail_scroll = 0;
        self.view_mode = ViewMode::Detail;
    }

    fn detail_scroll_down(&mut self, visible_height: usize) {
        let max = self.detail_lines.len().saturating_sub(visible_height);
        if self.detail_scroll < max {
            self.detail_scroll += 1;
        }
    }

    fn detail_scroll_up(&mut self) {
        if self.detail_scroll > 0 {
            self.detail_scroll -= 1;
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut jump_buf = String::new();
    let mut detail_height: usize = 20;

    match send_command(Command::ListServices).await {
        Ok(Response::ServiceList { services, groups }) => {
            app.set_data(services, groups);
        }
        Ok(_) => app.status_message = "Unexpected response".into(),
        Err(e) => app.status_message = format!("Failed to connect: {}", e),
    }

    let (refresh_tx, mut refresh_rx) =
        tokio::sync::mpsc::channel::<(Vec<ServiceInfo>, Vec<Group>)>(1);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            match send_command(Command::ListServices).await {
                Ok(Response::ServiceList { services, groups }) => {
                    let _ = refresh_tx.send((services, groups)).await;
                }
                _ => {}
            }
        }
    });

    loop {
        if app.view_mode == ViewMode::List {
            if let Ok((services, groups)) = refresh_rx.try_recv() {
                let selected = app.list_state.selected();
                app.set_data(services, groups);
                app.list_state.select(selected);
            }
        }

        let entries = app.build_entries();
        let filter_label = app.filter.label().to_string();
        let status_msg = app.status_message.clone();
        let view_mode = app.view_mode.clone();
        let detail_lines = app.detail_lines.clone();
        let detail_scroll = app.detail_scroll;
        let detail_name = app.detail_service_name.clone();

        terminal.draw(|f| {
            let area = f.area();

            if view_mode == ViewMode::List {
                let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                             Constraint::Min(0),
                             Constraint::Length(3),
                ])
                .split(area);

                let keybinds = Paragraph::new(
                    "j/k move  space collapse  enter detail  f filter  r restart  s start  x stop  e enable  d disable  #id jump  q quit"
                )
                .block(Block::default().borders(Borders::ALL).title(" keys "));
                f.render_widget(keybinds, chunks[0]);

                let items: Vec<ListItem> = entries.iter().map(|entry| {
                    match entry {
                        ListEntry::GroupHeader { name, collapsed, active, inactive, failed } => {
                            let arrow = if *collapsed { "▶" } else { "▼" };
                            let summary = format!(
                                "{} {}  ●{} active  ●{} inactive  ●{} failed",
                                arrow, name, active, inactive, failed
                            );
                            ListItem::new(summary)
                            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                        }
                        ListEntry::Service { info, id } => {
                            let color = match info.status {
                                ServiceStatus::Active => Color::Green,
                                ServiceStatus::Failed => Color::Red,
                                ServiceStatus::Inactive => Color::Yellow,
                                ServiceStatus::Unknown => Color::Gray,
                            };
                            let boot_flag = if info.enabled { "E" } else { "D" };
                            ListItem::new(format!(
                                "  {:>3}  [{}] {} — {}",
                                id, boot_flag, info.name, info.status
                            ))
                            .style(Style::default().fg(color))
                        }
                    }
                }).collect();

                let title = format!(" systemd-tui  filter: {} ", filter_label);
                let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(
                    Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
                )
                .highlight_symbol("▶ ");

                f.render_stateful_widget(list, chunks[1], &mut app.list_state);

                let status_text = if jump_buf.is_empty() {
                    status_msg
                } else {
                    format!("jump to id: {}", jump_buf)
                };
                let status = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL).title(" status "));
                f.render_widget(status, chunks[2]);

            } else {
                // Detail view
                let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                             Constraint::Min(0),
                ])
                .split(area);

                let keybinds = Paragraph::new("j/k scroll  esc back to list")
                .block(Block::default().borders(Borders::ALL).title(" keys "));
                f.render_widget(keybinds, chunks[0]);

                let inner_height = chunks[1].height.saturating_sub(2) as usize;
                detail_height = inner_height;

                let visible: Vec<&str> = detail_lines
                .iter()
                .skip(detail_scroll)
                .take(inner_height)
                .map(|s| s.as_str())
                .collect();

                let text = visible.join("\n");
                let detail = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title(
                    format!(" {} ", detail_name)
                ));
                f.render_widget(detail, chunks[1]);

                // Scrollbar
                let mut scrollbar_state = ScrollbarState::new(detail_lines.len())
                .position(detail_scroll);
                f.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight),
                                         chunks[1],
                                         &mut scrollbar_state,
                );
            }
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                // Detail view input
                if app.view_mode == ViewMode::Detail {
                    match key.code {
                        KeyCode::Esc => app.view_mode = ViewMode::List,
                        KeyCode::Char('j') | KeyCode::Down => app.detail_scroll_down(detail_height),
                        KeyCode::Char('k') | KeyCode::Up => app.detail_scroll_up(),
                        _ => {}
                    }
                    continue;
                }

                // Jump buf input
                if !jump_buf.is_empty() {
                    match key.code {
                        KeyCode::Char(c) if c.is_ascii_digit() => jump_buf.push(c),
                        KeyCode::Enter => {
                            if let Ok(id) = jump_buf.parse::<usize>() {
                                app.jump_to_id(id);
                            }
                            jump_buf.clear();
                        }
                        KeyCode::Esc => jump_buf.clear(),
                        _ => {}
                    }
                    continue;
                }

                // Normal list input
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Char(' ') => app.toggle_collapse(),
                    KeyCode::Enter => {
                        if let Some(svc) = app.selected_service() {
                            match send_command(Command::GetDetailedStatus(svc.name.clone())).await {
                                Ok(Response::DetailedStatus(text)) => {
                                    app.open_detail(svc.name.clone(), text);
                                }
                                Ok(Response::Error(e)) => app.status_message = e,
                                _ => {}
                            }
                        } else {
                            app.toggle_collapse();
                        }
                    }
                    KeyCode::Char('f') => {
                        app.filter = app.filter.next();
                        app.status_message = format!("Filter: {}", app.filter.label());
                    }
                    KeyCode::Char('r') => {
                        if let Some(svc) = app.selected_service() {
                            match send_command(Command::RestartService(svc.name.clone())).await {
                                Ok(Response::Success(msg)) => app.status_message = msg,
                                Ok(Response::Error(e)) => app.status_message = e,
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        if let Some(svc) = app.selected_service() {
                            match send_command(Command::StartService(svc.name.clone())).await {
                                Ok(Response::Success(msg)) => app.status_message = msg,
                                Ok(Response::Error(e)) => app.status_message = e,
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('x') => {
                        if let Some(svc) = app.selected_service() {
                            match send_command(Command::StopService(svc.name.clone())).await {
                                Ok(Response::Success(msg)) => app.status_message = msg,
                                Ok(Response::Error(e)) => app.status_message = e,
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('e') => {
                        if let Some(svc) = app.selected_service() {
                            match send_command(Command::EnableService(svc.name.clone())).await {
                                Ok(Response::Success(msg)) => app.status_message = msg,
                                Ok(Response::Error(e)) => app.status_message = e,
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(svc) = app.selected_service() {
                            match send_command(Command::DisableService(svc.name.clone())).await {
                                Ok(Response::Success(msg)) => app.status_message = msg,
                                Ok(Response::Error(e)) => app.status_message = e,
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => jump_buf.push(c),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn send_command(cmd: Command) -> anyhow::Result<Response> {
    let stream = UnixStream::connect(SOCKET_PATH).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let mut json = serde_json::to_string(&cmd)?;
    json.push('\n');
    writer.write_all(json.as_bytes()).await?;

    let mut line = String::new();
    reader.read_line(&mut line).await?;

    Ok(serde_json::from_str::<Response>(line.trim())?)
}
