//! Macro Editor UI Module
//!
//! Provides the macro configuration interface for the Settings UI.
//! Layout:
//! - Left container: Macro list with Record/Stop button
//! - Middle container: Keys in macro with Insert Event dropdown
//! - Right side: Cycle settings and shortcut configuration

use crate::macro_config::{CycleMode, MacroAction, MacroDefinition, MacroShortcut, MouseButton};
use iced::{
    widget::{
        Button, Checkbox, Column, Container, Radio, Row, Scrollable, Space, Text,
        TextInput, Rule, mouse_area,
    },
    Alignment, Element, Length,
};

/// State for context menu (right-click)
#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenuType {
    None,
    MacroList { macro_index: usize },
    KeysList { action_index: usize },
}

/// State for the Insert Event dropdown
#[derive(Debug, Clone, PartialEq)]
pub enum InsertEventMenu {
    Closed,
    Open,
    InsertPrevious,
    InsertAfter,
    InsertXY { x: String, y: String },
    InsertDelay { ms: String },
}

/// Messages for macro editor interactions
#[derive(Debug, Clone)]
pub enum MacroMessage {
    // Macro list actions
    SelectMacro(usize),
    NewMacro,
    RenameMacro(String),
    DeleteMacro,
    ToggleMacroEnabled(bool),

    // Recording
    StartRecording,
    StopRecording,
    RecordedAction(MacroAction),

    // Key list actions
    SelectAction(usize),
    DeleteAction,
    ChangeActionParameter(String),

    // Insert event menu
    OpenInsertMenu,
    CloseInsertMenu,
    InsertMousePrevious(MouseButton, bool), // button, is_press
    InsertMouseAfter(MouseButton, bool),
    InsertXY(i32, i32),
    InsertDelay(u64),
    InsertXYInputX(String),
    InsertXYInputY(String),
    InsertDelayInput(String),
    ConfirmInsertXY,
    ConfirmInsertDelay,

    // Context menu
    ShowContextMenu(ContextMenuType),
    HideContextMenu,
    ContextMenuRename,
    ContextMenuDelete,
    ContextMenuChangeParam,

    // Shortcut settings
    ToggleCtrl(bool),
    ToggleAlt(bool),
    ToggleShift(bool),
    ToggleWin(bool),
    ShortcutKeyChanged(String),

    // Cycle settings
    SetCycleMode(CycleMode),
    CycleCountChanged(String),
    CycleUntilKeyChanged(String),
    
    // Warning popup
    DismissRecordingWarning,
}

/// State for the macro editor
#[derive(Debug, Clone)]
pub struct MacroEditorState {
    /// All macros for the current profile
    pub macros: Vec<MacroDefinition>,
    /// Currently selected macro index
    pub selected_macro: Option<usize>,
    /// Currently selected action index within the macro
    pub selected_action: Option<usize>,
    /// Whether we're currently recording
    pub is_recording: bool,
    /// Insert event menu state
    pub insert_menu: InsertEventMenu,
    /// Context menu state
    pub context_menu: ContextMenuType,
    /// Editing state for macro name (rename)
    pub edit_name: String,
    /// Editing state for cycle count
    pub edit_cycle_count: String,
    /// Editing state for cycle until key
    pub edit_cycle_key: String,
    /// Editing state for insert XY
    pub insert_x: String,
    pub insert_y: String,
    /// Editing state for insert delay
    pub insert_delay_ms: String,
    /// Editing state for shortcut key
    pub edit_shortcut_key: String,
    /// Show recording warning popup
    pub show_recording_warning: bool,
}

impl Default for MacroEditorState {
    fn default() -> Self {
        Self {
            macros: Vec::new(),
            selected_macro: None,
            selected_action: None,
            is_recording: false,
            insert_menu: InsertEventMenu::Closed,
            context_menu: ContextMenuType::None,
            edit_name: String::new(),
            edit_cycle_count: "1".to_string(),
            edit_cycle_key: String::new(),
            insert_x: "0".to_string(),
            insert_y: "0".to_string(),
            insert_delay_ms: "100".to_string(),
            edit_shortcut_key: String::new(),
            show_recording_warning: false,
        }
    }
}

impl MacroEditorState {
    /// Create new state from existing macros
    pub fn new(macros: Vec<MacroDefinition>) -> Self {
        Self {
            macros,
            ..Default::default()
        }
    }

    /// Get the currently selected macro (if any)
    pub fn current_macro(&self) -> Option<&MacroDefinition> {
        self.selected_macro.and_then(|i| self.macros.get(i))
    }

    /// Get the currently selected macro mutably
    pub fn current_macro_mut(&mut self) -> Option<&mut MacroDefinition> {
        self.selected_macro.and_then(|i| self.macros.get_mut(i))
    }

    /// Handle a macro editor message
    pub fn update(&mut self, message: MacroMessage) {
        match message {
            MacroMessage::SelectMacro(index) => {
                // Prevent switching macros during recording
                if self.is_recording {
                    self.show_recording_warning = true;
                    return;
                }
                self.selected_macro = Some(index);
                self.selected_action = None;
                if let Some(m) = self.macros.get(index) {
                    self.edit_name = m.name.clone();
                    if let Some(ref shortcut) = m.shortcut {
                        self.edit_shortcut_key = shortcut.key.clone();
                    } else {
                        self.edit_shortcut_key.clear();
                    }
                    match &m.cycle_mode {
                        CycleMode::Once => {
                            self.edit_cycle_count = "1".to_string();
                        }
                        CycleMode::Count(n) => {
                            self.edit_cycle_count = n.to_string();
                        }
                        CycleMode::UntilKeyPressed(key) => {
                            self.edit_cycle_key = key.clone();
                        }
                    }
                }
            }

            MacroMessage::NewMacro => {
                // Prevent creating new macros during recording
                if self.is_recording {
                    self.show_recording_warning = true;
                    return;
                }
                let name = format!("Macro {}", self.macros.len() + 1);
                let new_macro = MacroDefinition::new(name);
                self.macros.push(new_macro);
                self.selected_macro = Some(self.macros.len() - 1);
                self.selected_action = None;
                if let Some(m) = self.current_macro() {
                    self.edit_name = m.name.clone();
                }
            }

            MacroMessage::RenameMacro(name) => {
                self.edit_name = name.clone();
                if let Some(m) = self.current_macro_mut() {
                    m.name = name;
                }
            }

            MacroMessage::DeleteMacro => {
                if let Some(index) = self.selected_macro {
                    if index < self.macros.len() {
                        self.macros.remove(index);
                        self.selected_macro = if self.macros.is_empty() {
                            None
                        } else {
                            Some(index.saturating_sub(1).min(self.macros.len() - 1))
                        };
                        self.selected_action = None;
                    }
                }
            }

            MacroMessage::ToggleMacroEnabled(enabled) => {
                if let Some(m) = self.current_macro_mut() {
                    m.enabled = enabled;
                }
            }

            MacroMessage::StartRecording => {
                // Auto-create a new macro if none selected
                if self.selected_macro.is_none() {
                    let name = format!("Macro {}", self.macros.len() + 1);
                    let new_macro = MacroDefinition::new(name.clone());
                    self.macros.push(new_macro);
                    self.selected_macro = Some(self.macros.len() - 1);
                    self.edit_name = name;
                } else {
                    // Clear existing actions for clean slate recording
                    if let Some(m) = self.current_macro_mut() {
                        m.actions.clear();
                    }
                }
                self.selected_action = None; // Reset action selection
                self.is_recording = true;
            }

            MacroMessage::StopRecording => {
                self.is_recording = false;
            }

            MacroMessage::RecordedAction(action) => {
                if self.is_recording {
                    if let Some(m) = self.current_macro_mut() {
                        m.actions.push(action);
                    }
                }
            }

            MacroMessage::SelectAction(index) => {
                self.selected_action = Some(index);
            }

            MacroMessage::DeleteAction => {
                if let Some(action_idx) = self.selected_action {
                    if let Some(m) = self.current_macro_mut() {
                        if action_idx < m.actions.len() {
                            m.actions.remove(action_idx);
                            self.selected_action = if m.actions.is_empty() {
                                None
                            } else {
                                Some(action_idx.saturating_sub(1).min(m.actions.len() - 1))
                            };
                        }
                    }
                }
            }

            MacroMessage::ChangeActionParameter(_param) => {
                // Handle parameter change (e.g., delay value)
                // Implementation depends on action type
            }

            MacroMessage::OpenInsertMenu => {
                self.insert_menu = InsertEventMenu::Open;
            }

            MacroMessage::CloseInsertMenu => {
                self.insert_menu = InsertEventMenu::Closed;
            }

            MacroMessage::InsertMousePrevious(button, is_press) => {
                if let Some(action_idx) = self.selected_action {
                    if let Some(m) = self.current_macro_mut() {
                        let action = MacroAction::MouseClick {
                            button,
                            press: is_press,
                        };
                        m.actions.insert(action_idx, action);
                    }
                }
                self.insert_menu = InsertEventMenu::Closed;
            }

            MacroMessage::InsertMouseAfter(button, is_press) => {
                if let Some(action_idx) = self.selected_action {
                    if let Some(m) = self.current_macro_mut() {
                        let action = MacroAction::MouseClick {
                            button,
                            press: is_press,
                        };
                        let insert_at = (action_idx + 1).min(m.actions.len());
                        m.actions.insert(insert_at, action);
                    }
                } else if let Some(m) = self.current_macro_mut() {
                    // No selection, append to end
                    let action = MacroAction::MouseClick {
                        button,
                        press: is_press,
                    };
                    m.actions.push(action);
                }
                self.insert_menu = InsertEventMenu::Closed;
            }

            MacroMessage::InsertXYInputX(x) => {
                self.insert_x = x;
            }

            MacroMessage::InsertXYInputY(y) => {
                self.insert_y = y;
            }

            MacroMessage::ConfirmInsertXY => {
                let x: i32 = self.insert_x.parse().unwrap_or(0);
                let y: i32 = self.insert_y.parse().unwrap_or(0);
                if let Some(m) = self.current_macro_mut() {
                    m.actions.push(MacroAction::MouseMove { x, y });
                }
                self.insert_menu = InsertEventMenu::Closed;
            }

            MacroMessage::InsertXY(x, y) => {
                if let Some(m) = self.current_macro_mut() {
                    m.actions.push(MacroAction::MouseMove { x, y });
                }
                self.insert_menu = InsertEventMenu::Closed;
            }

            MacroMessage::InsertDelayInput(ms) => {
                self.insert_delay_ms = ms;
            }

            MacroMessage::ConfirmInsertDelay => {
                let ms: u64 = self.insert_delay_ms.parse().unwrap_or(100);
                if let Some(m) = self.current_macro_mut() {
                    m.actions.push(MacroAction::Delay { ms });
                }
                self.insert_menu = InsertEventMenu::Closed;
            }

            MacroMessage::InsertDelay(ms) => {
                if let Some(m) = self.current_macro_mut() {
                    m.actions.push(MacroAction::Delay { ms });
                }
                self.insert_menu = InsertEventMenu::Closed;
            }

            MacroMessage::ShowContextMenu(menu_type) => {
                self.context_menu = menu_type;
            }

            MacroMessage::HideContextMenu => {
                self.context_menu = ContextMenuType::None;
            }

            MacroMessage::ContextMenuRename => {
                // Handled by UI - focus on rename input
                self.context_menu = ContextMenuType::None;
            }

            MacroMessage::ContextMenuDelete => {
                match &self.context_menu {
                    ContextMenuType::MacroList { .. } => {
                        self.update(MacroMessage::DeleteMacro);
                    }
                    ContextMenuType::KeysList { .. } => {
                        self.update(MacroMessage::DeleteAction);
                    }
                    _ => {}
                }
                self.context_menu = ContextMenuType::None;
            }

            MacroMessage::ContextMenuChangeParam => {
                // Handled by UI - opens parameter editor
                self.context_menu = ContextMenuType::None;
            }

            MacroMessage::ToggleCtrl(enabled) => {
                if let Some(m) = self.current_macro_mut() {
                    let shortcut = m.shortcut.get_or_insert(MacroShortcut::default());
                    shortcut.ctrl = enabled;
                }
            }

            MacroMessage::ToggleAlt(enabled) => {
                if let Some(m) = self.current_macro_mut() {
                    let shortcut = m.shortcut.get_or_insert(MacroShortcut::default());
                    shortcut.alt = enabled;
                }
            }

            MacroMessage::ToggleShift(enabled) => {
                if let Some(m) = self.current_macro_mut() {
                    let shortcut = m.shortcut.get_or_insert(MacroShortcut::default());
                    shortcut.shift = enabled;
                }
            }

            MacroMessage::ToggleWin(enabled) => {
                if let Some(m) = self.current_macro_mut() {
                    let shortcut = m.shortcut.get_or_insert(MacroShortcut::default());
                    shortcut.win = enabled;
                }
            }

            MacroMessage::ShortcutKeyChanged(key) => {
                self.edit_shortcut_key = key.clone();
                if let Some(m) = self.current_macro_mut() {
                    let shortcut = m.shortcut.get_or_insert(MacroShortcut::default());
                    shortcut.key = key.to_uppercase();
                }
            }

            MacroMessage::SetCycleMode(mode) => {
                if let Some(m) = self.current_macro_mut() {
                    m.cycle_mode = mode;
                }
            }

            MacroMessage::CycleCountChanged(count) => {
                self.edit_cycle_count = count.clone();
                if let Ok(n) = count.parse::<u32>() {
                    if let Some(m) = self.current_macro_mut() {
                        m.cycle_mode = CycleMode::Count(n);
                    }
                }
            }

            MacroMessage::CycleUntilKeyChanged(key) => {
                self.edit_cycle_key = key.clone();
                if let Some(m) = self.current_macro_mut() {
                    m.cycle_mode = CycleMode::UntilKeyPressed(key.to_uppercase());
                }
            }
            
            MacroMessage::DismissRecordingWarning => {
                self.show_recording_warning = false;
            }
        }
    }

    /// Render the macro editor view
    pub fn view(&self) -> Element<'_, MacroMessage> {
        let macro_list = self.render_macro_list();
        let keys_list = self.render_keys_list();
        let settings_panel = self.render_settings_panel();

        // Main layout: three columns with visual separation
        let content = Row::new()
            .spacing(10)
            .push(
                Container::new(macro_list)
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(450.0))
                    .padding(10),
            )
            .push(Rule::vertical(2))
            .push(
                Container::new(keys_list)
                    .width(Length::Fixed(260.0))
                    .height(Length::Fixed(450.0))
                    .padding(10),
            )
            .push(Rule::vertical(2))
            .push(
                Container::new(settings_panel)
                    .width(Length::Fill)
                    .height(Length::Fixed(450.0))
                    .padding(10),
            );

        // Recording warning popup overlay
        let mut main_content: Element<'_, MacroMessage> = Container::new(content)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into();

        // Show warning popup if recording and user tried to switch
        if self.show_recording_warning {
            let warning_popup = Container::new(
                Column::new()
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .push(Text::new("🔴 Recording!").size(16))
                    .push(Text::new("Stop recording before\nswitching macros.").size(12))
                    .push(
                        Button::new(Text::new("OK").size(11))
                            .on_press(MacroMessage::DismissRecordingWarning)
                            .padding([4, 16]),
                    ),
            )
            .padding(15)
            .width(Length::Fixed(180.0));

            // Stack the popup over the content
            main_content = Column::new()
                .push(main_content)
                .push(
                    Container::new(warning_popup)
                        .width(Length::Fill)
                        .center_x(),
                )
                .into();
        }

        main_content
    }

    /// Render the macro list container
    fn render_macro_list(&self) -> Element<'_, MacroMessage> {
        let header = Row::new()
            .spacing(5)
            .align_items(Alignment::Center)
            .push(Text::new("📋 Macro List").size(15))
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(
                Button::new(Text::new("+").size(12))
                    .on_press(MacroMessage::NewMacro)
                    .padding(4),
            );

        // Macro items
        let mut macro_items = Column::new().spacing(2);

        if self.macros.is_empty() {
            macro_items = macro_items.push(
                Text::new("No macros.\nClick + or Record.").size(11)
            );
        } else {
            for (i, macro_def) in self.macros.iter().enumerate() {
                let is_selected = self.selected_macro == Some(i);
                let icon = if macro_def.enabled { "✓" } else { "○" };
                let prefix = if is_selected { "▶" } else { " " };
                let label = format!("{} {} {}", prefix, icon, macro_def.name);

                let select_button = Button::new(Text::new(label).size(11))
                    .on_press(MacroMessage::SelectMacro(i))
                    .width(Length::Fill)
                    .padding(4);

                // Wrap with mouse_area for right-click detection
                let item_with_right_click = mouse_area(select_button)
                    .on_right_press(MacroMessage::ShowContextMenu(ContextMenuType::MacroList { macro_index: i }));

                macro_items = macro_items.push(item_with_right_click);
            }
        }

        // Show context menu if active for macro list
        if let ContextMenuType::MacroList { macro_index: _ } = &self.context_menu {
            let ctx_menu = Container::new(
                Column::new()
                    .spacing(2)
                    .push(
                        Button::new(Text::new("✏ Rename").size(10))
                            .on_press(MacroMessage::ContextMenuRename)
                            .width(Length::Fill)
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("🗑 Delete").size(10))
                            .on_press(MacroMessage::ContextMenuDelete)
                            .width(Length::Fill)
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("✕ Close").size(10))
                            .on_press(MacroMessage::HideContextMenu)
                            .width(Length::Fill)
                            .padding(4),
                    ),
            )
            .padding(5)
            .width(Length::Fixed(100.0));
            
            macro_items = macro_items.push(ctx_menu);
        }

        // Wrap macro items in scrollable with fixed height
        let scrollable_macros = Scrollable::new(macro_items)
            .height(Length::Fixed(320.0))
            .width(Length::Fill);

        // Record/Stop button at bottom
        let record_button = if self.is_recording {
            Button::new(
                Row::new()
                    .spacing(6)
                    .align_items(Alignment::Center)
                    .push(Text::new("🔴").size(12))
                    .push(Text::new("Stop").size(12)),
            )
            .on_press(MacroMessage::StopRecording)
            .padding(8)
            .width(Length::Fill)
        } else {
            Button::new(
                Row::new()
                    .spacing(6)
                    .align_items(Alignment::Center)
                    .push(Text::new("🟢").size(12))
                    .push(Text::new("Record").size(12)),
            )
            .on_press(MacroMessage::StartRecording)
            .padding(8)
            .width(Length::Fill)
        };

        Column::new()
            .spacing(6)
            .push(header)
            .push(Rule::horizontal(1))
            .push(scrollable_macros)
            .push(Space::new(Length::Shrink, Length::Fixed(10.0)))
            .push(record_button)
            .into()
    }

    /// Render the keys in macro container
    fn render_keys_list(&self) -> Element<'_, MacroMessage> {
        let header = Row::new()
            .spacing(5)
            .align_items(Alignment::Center)
            .push(Text::new("⌨️ Keys in Macro").size(15))
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(
                if self.selected_action.is_some() {
                    Button::new(Text::new("🗑").size(10))
                        .on_press(MacroMessage::DeleteAction)
                        .padding(4)
                } else {
                    Button::new(Text::new("🗑").size(10)).padding(4)
                }
            );

        // Actions list - show ALL actions (scrollable)
        let mut action_items = Column::new().spacing(2);

        if let Some(macro_def) = self.current_macro() {
            if macro_def.actions.is_empty() {
                action_items = action_items.push(
                    Text::new("No actions.\nRecord or Insert.").size(11)
                );
            } else {
                // Show ALL actions - container is scrollable
                for (i, action) in macro_def.actions.iter().enumerate() {
                    let is_selected = self.selected_action == Some(i);
                    let prefix = if is_selected { "▶" } else { " " };
                    let label = format!("{} {}", prefix, action.display_text());

                    let select_button = Button::new(Text::new(label).size(10))
                        .on_press(MacroMessage::SelectAction(i))
                        .width(Length::Fill)
                        .padding(3);

                    // Wrap with mouse_area for right-click detection
                    let item_with_right_click = mouse_area(select_button)
                        .on_right_press(MacroMessage::ShowContextMenu(ContextMenuType::KeysList { action_index: i }));

                    action_items = action_items.push(item_with_right_click);
                }
            }
        } else {
            action_items = action_items.push(
                Text::new("Select a macro").size(11)
            );
        }

        // Show context menu if active for keys list
        if let ContextMenuType::KeysList { action_index: _ } = &self.context_menu {
            let ctx_menu = Container::new(
                Column::new()
                    .spacing(2)
                    .push(
                        Button::new(Text::new("📝 Change Param").size(10))
                            .on_press(MacroMessage::ContextMenuChangeParam)
                            .width(Length::Fill)
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("🗑 Delete").size(10))
                            .on_press(MacroMessage::ContextMenuDelete)
                            .width(Length::Fill)
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("✕ Close").size(10))
                            .on_press(MacroMessage::HideContextMenu)
                            .width(Length::Fill)
                            .padding(4),
                    ),
            )
            .padding(5)
            .width(Length::Fixed(120.0));
            
            action_items = action_items.push(ctx_menu);
        }

        // Wrap action items in scrollable with fixed height
        let scrollable_actions = Scrollable::new(action_items)
            .height(Length::Fixed(320.0))
            .width(Length::Fill);

        // Insert Event dropdown button (aligned with Record button)
        let insert_button = Button::new(
            Row::new()
                .spacing(6)
                .align_items(Alignment::Center)
                .push(Text::new("Insert Event").size(12))
                .push(Space::new(Length::Fill, Length::Shrink))
                .push(Text::new(if self.insert_menu == InsertEventMenu::Open { "▲" } else { "▼" }).size(10)),
        )
        .on_press(if self.insert_menu == InsertEventMenu::Open {
            MacroMessage::CloseInsertMenu
        } else {
            MacroMessage::OpenInsertMenu
        })
        .padding(8)
        .width(Length::Fill);

        let mut content = Column::new()
            .spacing(6)
            .push(header)
            .push(Rule::horizontal(1))
            .push(scrollable_actions)
            .push(Space::new(Length::Shrink, Length::Fixed(10.0)))
            .push(insert_button);

        // Show dropdown menu if open
        if self.insert_menu == InsertEventMenu::Open {
            content = content.push(self.render_insert_dropdown());
        }

        content.into()
    }

    /// Render the insert event dropdown content
    fn render_insert_dropdown(&self) -> Element<'_, MacroMessage> {
        let mouse_section = Column::new()
            .spacing(2)
            .push(Text::new("Mouse:").size(10))
            .push(
                Row::new()
                    .spacing(2)
                    .push(Button::new(Text::new("L↓").size(9)).on_press(MacroMessage::InsertMouseAfter(MouseButton::Left, true)).padding(3))
                    .push(Button::new(Text::new("L↑").size(9)).on_press(MacroMessage::InsertMouseAfter(MouseButton::Left, false)).padding(3))
                    .push(Button::new(Text::new("R↓").size(9)).on_press(MacroMessage::InsertMouseAfter(MouseButton::Right, true)).padding(3))
                    .push(Button::new(Text::new("R↑").size(9)).on_press(MacroMessage::InsertMouseAfter(MouseButton::Right, false)).padding(3))
                    .push(Button::new(Text::new("M↓").size(9)).on_press(MacroMessage::InsertMouseAfter(MouseButton::Middle, true)).padding(3))
                    .push(Button::new(Text::new("M↑").size(9)).on_press(MacroMessage::InsertMouseAfter(MouseButton::Middle, false)).padding(3)),
            );

        let xy_section = Row::new()
            .spacing(3)
            .align_items(Alignment::Center)
            .push(Text::new("XY:").size(9))
            .push(TextInput::new("X", &self.insert_x).on_input(MacroMessage::InsertXYInputX).width(Length::Fixed(35.0)).padding(2))
            .push(TextInput::new("Y", &self.insert_y).on_input(MacroMessage::InsertXYInputY).width(Length::Fixed(35.0)).padding(2))
            .push(Button::new(Text::new("+").size(9)).on_press(MacroMessage::ConfirmInsertXY).padding(3));

        let delay_section = Row::new()
            .spacing(3)
            .align_items(Alignment::Center)
            .push(Text::new("Delay:").size(9))
            .push(TextInput::new("ms", &self.insert_delay_ms).on_input(MacroMessage::InsertDelayInput).width(Length::Fixed(40.0)).padding(2))
            .push(Text::new("ms").size(9))
            .push(Button::new(Text::new("+").size(9)).on_press(MacroMessage::ConfirmInsertDelay).padding(3));

        Container::new(
            Column::new()
                .spacing(5)
                .padding(6)
                .push(mouse_section)
                .push(xy_section)
                .push(delay_section)
        )
        .width(Length::Fill)
        .into()
    }

    /// Render the insert event dropdown menu (legacy, kept for reference)
    #[allow(dead_code)]
    fn render_insert_menu(&self) -> Element<'_, MacroMessage> {
        let menu = Column::new()
            .spacing(2)
            .padding(5)
            .push(Text::new("Insert Previous ▶").size(12))
            .push(
                Row::new()
                    .spacing(2)
                    .push(
                        Button::new(Text::new("Left ↑").size(10))
                            .on_press(MacroMessage::InsertMousePrevious(MouseButton::Left, true))
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("Left ↓").size(10))
                            .on_press(MacroMessage::InsertMousePrevious(MouseButton::Left, false))
                            .padding(4),
                    ),
            )
            .push(
                Row::new()
                    .spacing(2)
                    .push(
                        Button::new(Text::new("Right ↑").size(10))
                            .on_press(MacroMessage::InsertMousePrevious(MouseButton::Right, true))
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("Right ↓").size(10))
                            .on_press(MacroMessage::InsertMousePrevious(MouseButton::Right, false))
                            .padding(4),
                    ),
            )
            .push(
                Row::new()
                    .spacing(2)
                    .push(
                        Button::new(Text::new("Middle ↑").size(10))
                            .on_press(MacroMessage::InsertMousePrevious(MouseButton::Middle, true))
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("Middle ↓").size(10))
                            .on_press(MacroMessage::InsertMousePrevious(
                                MouseButton::Middle,
                                false,
                            ))
                            .padding(4),
                    ),
            )
            .push(Space::new(Length::Fill, Length::Fixed(5.0)))
            .push(Text::new("Insert After ▶").size(12))
            .push(
                Row::new()
                    .spacing(2)
                    .push(
                        Button::new(Text::new("Left ↑").size(10))
                            .on_press(MacroMessage::InsertMouseAfter(MouseButton::Left, true))
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("Left ↓").size(10))
                            .on_press(MacroMessage::InsertMouseAfter(MouseButton::Left, false))
                            .padding(4),
                    ),
            )
            .push(
                Row::new()
                    .spacing(2)
                    .push(
                        Button::new(Text::new("Right ↑").size(10))
                            .on_press(MacroMessage::InsertMouseAfter(MouseButton::Right, true))
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("Right ↓").size(10))
                            .on_press(MacroMessage::InsertMouseAfter(MouseButton::Right, false))
                            .padding(4),
                    ),
            )
            .push(
                Row::new()
                    .spacing(2)
                    .push(
                        Button::new(Text::new("Middle ↑").size(10))
                            .on_press(MacroMessage::InsertMouseAfter(MouseButton::Middle, true))
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("Middle ↓").size(10))
                            .on_press(MacroMessage::InsertMouseAfter(MouseButton::Middle, false))
                            .padding(4),
                    ),
            )
            .push(Space::new(Length::Fill, Length::Fixed(5.0)))
            .push(Text::new("Insert XY").size(12))
            .push(
                Row::new()
                    .spacing(5)
                    .push(Text::new("X:").size(10))
                    .push(
                        TextInput::new("0", &self.insert_x)
                            .on_input(MacroMessage::InsertXYInputX)
                            .width(Length::Fixed(50.0))
                            .padding(3),
                    )
                    .push(Text::new("Y:").size(10))
                    .push(
                        TextInput::new("0", &self.insert_y)
                            .on_input(MacroMessage::InsertXYInputY)
                            .width(Length::Fixed(50.0))
                            .padding(3),
                    )
                    .push(
                        Button::new(Text::new("Add").size(10))
                            .on_press(MacroMessage::ConfirmInsertXY)
                            .padding(4),
                    ),
            )
            .push(Space::new(Length::Fill, Length::Fixed(5.0)))
            .push(Text::new("Insert Delay").size(12))
            .push(
                Row::new()
                    .spacing(5)
                    .push(
                        TextInput::new("100", &self.insert_delay_ms)
                            .on_input(MacroMessage::InsertDelayInput)
                            .width(Length::Fixed(70.0))
                            .padding(3),
                    )
                    .push(Text::new("ms").size(10))
                    .push(
                        Button::new(Text::new("Add").size(10))
                            .on_press(MacroMessage::ConfirmInsertDelay)
                            .padding(4),
                    ),
            );

        Container::new(menu).padding(5).into()
    }

    /// Render the settings panel (cycle mode + shortcut)
    fn render_settings_panel(&self) -> Element<'_, MacroMessage> {
        let current_macro = self.current_macro();

        // Cycle settings
        let cycle_mode = current_macro.map(|m| &m.cycle_mode);
        let is_count = matches!(cycle_mode, Some(CycleMode::Count(_)));
        let is_until_key = matches!(cycle_mode, Some(CycleMode::UntilKeyPressed(_)));

        let cycle_settings = Column::new()
            .spacing(10)
            .push(Text::new("🔁 Cycle Settings").size(16))
            .push(
                Row::new()
                    .spacing(10)
                    .push(Radio::new(
                        "Cycle until",
                        true,
                        Some(is_until_key),
                        |_| MacroMessage::SetCycleMode(CycleMode::UntilKeyPressed(String::new())),
                    ))
                    .push(
                        TextInput::new("Key", &self.edit_cycle_key)
                            .on_input(MacroMessage::CycleUntilKeyChanged)
                            .width(Length::Fixed(60.0))
                            .padding(5),
                    )
                    .push(Text::new("pressed").size(12)),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Radio::new(
                        "Specified cycle",
                        true,
                        Some(is_count),
                        |_| MacroMessage::SetCycleMode(CycleMode::Count(1)),
                    ))
                    .push(
                        TextInput::new("1", &self.edit_cycle_count)
                            .on_input(MacroMessage::CycleCountChanged)
                            .width(Length::Fixed(60.0))
                            .padding(5),
                    ),
            );

        // Shortcut settings
        let shortcut = current_macro.and_then(|m| m.shortcut.as_ref());
        let ctrl = shortcut.map(|s| s.ctrl).unwrap_or(false);
        let alt = shortcut.map(|s| s.alt).unwrap_or(false);
        let shift = shortcut.map(|s| s.shift).unwrap_or(false);
        let win = shortcut.map(|s| s.win).unwrap_or(false);

        let shortcut_settings = Column::new()
            .spacing(10)
            .push(Text::new("⌨️ Shortcut Keys").size(16))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Column::new()
                            .spacing(5)
                            .push(Checkbox::new("CTRL", ctrl).on_toggle(MacroMessage::ToggleCtrl))
                            .push(Checkbox::new("SHIFT", shift).on_toggle(MacroMessage::ToggleShift)),
                    )
                    .push(
                        Column::new()
                            .spacing(5)
                            .push(Checkbox::new("ALT", alt).on_toggle(MacroMessage::ToggleAlt))
                            .push(Checkbox::new("WIN", win).on_toggle(MacroMessage::ToggleWin)),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("+ Key:").size(12))
                    .push(
                        TextInput::new("A-Z, F1-F12, 0-9", &self.edit_shortcut_key)
                            .on_input(MacroMessage::ShortcutKeyChanged)
                            .width(Length::Fixed(120.0))
                            .padding(5),
                    ),
            )
            .push(
                Text::new(format!(
                    "Current: {}",
                    shortcut.map(|s| s.display_text()).unwrap_or("Not set".to_string())
                ))
                .size(12),
            );

        // Macro name (for selected macro)
        let name_section = if current_macro.is_some() {
            Column::new()
                .spacing(5)
                .push(Text::new("📝 Macro Name").size(16))
                .push(
                    TextInput::new("Macro name...", &self.edit_name)
                        .on_input(MacroMessage::RenameMacro)
                        .padding(8)
                        .width(Length::Fill),
                )
                .push(
                    Checkbox::new(
                        "Enabled",
                        current_macro.map(|m| m.enabled).unwrap_or(false),
                    )
                    .on_toggle(MacroMessage::ToggleMacroEnabled),
                )
        } else {
            Column::new().push(Text::new("Select a macro to edit").size(14))
        };

        // Delete button
        let delete_button = if current_macro.is_some() {
            Button::new(Text::new("🗑️ Delete Macro").size(12))
                .on_press(MacroMessage::DeleteMacro)
                .padding(6)
                .width(Length::Fill)
        } else {
            Button::new(Text::new("🗑️ Delete Macro").size(12))
                .padding(6)
                .width(Length::Fill)
        };

        Column::new()
            .spacing(10)
            .push(name_section)
            .push(Rule::horizontal(1))
            .push(cycle_settings)
            .push(Rule::horizontal(1))
            .push(shortcut_settings)
            .push(Space::new(Length::Fill, Length::Fill))
            .push(delete_button)
            .into()
    }
}
