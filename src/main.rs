use iced::{
    widget::{
        button, container, scrollable, text, text_input, Column, Container,
        Row, Text, image::Handle,
    },
    Application, Color, Command, Element, Length, Settings, Subscription, Theme,
    theme, executor, time::every, window::{self, Position, icon}, Vector,
};
use sysinfo::{Pid, System, SystemExt, ProcessExt, PidExt};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetIconInfo, ICONINFO, HICON,
};
use windows::Win32::Graphics::Gdi::{
    GetDIBits, BITMAPINFOHEADER, BITMAPINFO, GetDC, ReleaseDC,
    BI_RGB, DIB_RGB_COLORS, RGBQUAD,
};
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::Foundation::{HWND, HANDLE, CloseHandle};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess, PROCESS_ACCESS_RIGHTS};
use windows::core::PCWSTR;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use image::{DynamicImage, ImageBuffer, Rgba};
use enigo::{Enigo, Key, KeyboardControllable};
use chrono::{Local, DateTime};

// Constants for colors and styling
const DARK_BG: Color = Color::from_rgb(0.15, 0.15, 0.15);
const DARK_BG_LIGHTER: Color = Color::from_rgb(0.2, 0.2, 0.2);
const DARK_TEXT: Color = Color::from_rgb(0.9, 0.9, 0.9);
const DARK_SECONDARY_TEXT: Color = Color::from_rgb(0.7, 0.7, 0.7);
const BORDER_COLOR: Color = Color::from_rgb(0.3, 0.3, 0.3);
const ROW_HOVER: Color = Color::from_rgb(0.25, 0.25, 0.25);
const SUCCESS_COLOR: Color = Color::from_rgb(0.2, 0.8, 0.2);
const WARNING_COLOR: Color = Color::from_rgb(0.8, 0.3, 0.3);
const WARNING_COLOR_HOVER: Color = Color::from_rgb(0.9, 0.4, 0.4);
const ACCENT_COLOR: Color = Color::from_rgb(0.2, 0.5, 0.8);
const ACCENT_BLUE: Color = Color::from_rgb(0.0, 0.6, 1.0);
const ACCENT_BLUE_HOVER: Color = Color::from_rgb(0.1, 0.7, 1.0);

// Application categories and their executable names
const OFFICE_APPS: &[&str] = &[
    "WINWORD.EXE",      // Microsoft Word
    "EXCEL.EXE",        // Microsoft Excel
    "POWERPNT.EXE",     // Microsoft PowerPoint
    "ONENOTE.EXE",      // Microsoft OneNote
    "OUTLOOK.EXE",      // Microsoft Outlook
    "PUBLISHER.EXE",    // Microsoft Publisher
    "MSACCESS.EXE",     // Microsoft Access
    "swriter.exe",      // LibreOffice Writer
    "scalc.exe",        // LibreOffice Calc
    "simpress.exe",     // LibreOffice Impress
];

const TEXT_EDITORS: &[&str] = &[
    "notepad.exe",      // Notepad
    "notepad++.exe",    // Notepad++
    "sublime_text.exe", // Sublime Text
    "Code.exe",         // VS Code
    "atom.exe",         // Atom
    "vim.exe",          // Vim
    "gvim.exe",         // GVim
    "emacs.exe",        // Emacs
    "wordpad.exe",      // WordPad
];

const IDES: &[&str] = &[
    "devenv.exe",       // Visual Studio
    "idea64.exe",       // IntelliJ IDEA
    "pycharm64.exe",    // PyCharm
    "webstorm64.exe",   // WebStorm
    "rider64.exe",      // Rider
    "eclipse.exe",      // Eclipse
    "android studio.exe", // Android Studio
    "netbeans64.exe",   // NetBeans
];

const DESIGN_APPS: &[&str] = &[
    "photoshop.exe",    // Adobe Photoshop
    "illustrator.exe",  // Adobe Illustrator
    "gimp-2.10.exe",    // GIMP
    "inkscape.exe",     // Inkscape
    "figma.exe",        // Figma
    "xd.exe",           // Adobe XD
    "krita.exe",        // Krita
    "paint.net.exe",    // Paint.NET
    "designer.exe",     // Qt Designer
];

const DEVELOPMENT_TOOLS: &[&str] = &[
    "ssms.exe",         // SQL Server Management Studio
    "pgadmin4.exe",     // pgAdmin
    "dbeaver.exe",      // DBeaver
    "postman.exe",      // Postman
    "insomnia.exe",     // Insomnia
    "sourcetree.exe",   // SourceTree
    "github desktop.exe", // GitHub Desktop
];

const CREATIVE_TOOLS: &[&str] = &[
    "premiere.exe",     // Adobe Premiere
    "aftereffects.exe", // Adobe After Effects
    "audition.exe",     // Adobe Audition
    "vegas.exe",        // Vegas Pro
    "resolve.exe",      // DaVinci Resolve
    "blender.exe",      // Blender
    "maya.exe",         // Maya
    "3dsmax.exe",       // 3ds Max
];

const BROWSER_APPS: &[&str] = &[
    "chrome.exe",
    "firefox.exe",
    "msedge.exe",
    "opera.exe",
    "brave.exe",
];

const PRODUCTIVITY_APPS: &[&str] = &[
    "winword.exe",
    "excel.exe",
    "powerpnt.exe",
    "onenote.exe",
    "outlook.exe",
    "publisher.exe",
    "msaccess.exe",
    "notepad.exe",
    "notepad++.exe",
    "code.exe",
    "sublime_text.exe",
    "atom.exe",
];

const DEVELOPMENT_APPS: &[&str] = &[
    "devenv.exe",
    "idea64.exe",
    "pycharm64.exe",
    "webstorm64.exe",
    "androidstudio64.exe",
    "eclipse.exe",
    "netbeans64.exe",
    "vscode.exe",
];

#[derive(Debug, Clone)]
struct Task {
    name: String,
    pid: u32,
    cpu_usage: f32,
    memory_usage: u64,
    icon: Option<ProcessIcon>,
    deadline: Option<DateTime<Local>>,
    status: ProcessStatus,
}

impl Task {
    fn new(name: String, pid: u32, cpu_usage: f32, memory_usage: u64, icon: Option<ProcessIcon>) -> Self {
        Self {
            name,
            pid,
            cpu_usage,
            memory_usage,
            icon,
            deadline: None,
            status: ProcessStatus::Running,
        }
    }

    fn get_status(&self) -> ProcessStatus {
        if let Some(deadline) = self.deadline {
            let now = Local::now();
            if now > deadline {
                ProcessStatus::DeadlineReached
            } else {
                ProcessStatus::Running
            }
        } else {
            ProcessStatus::Running
        }
    }

    fn format_deadline(&self) -> String {
        if let Some(deadline) = self.deadline {
            let now = Local::now();
            if now > deadline {
                "Expired".to_string()
            } else {
                let remaining = deadline.signed_duration_since(now);
                let minutes = remaining.num_minutes();
                let seconds = remaining.num_seconds() % 60;
                if minutes > 0 {
                    format!("{}m {}s left", minutes, seconds)
                } else {
                    format!("{}s left", seconds)
                }
            }
        } else {
            "None".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TaskSelected(u32),
    TerminateTask(u32),
    SetDeadline(u32, TimeInterval),
    ClearDeadline(u32),
    Tick,
    SearchInput(String),
    CheckDeadlines,
    CustomDeadlineInput(String),
}

#[derive(Debug, Clone)]
pub enum TimeInterval {
    ThirtyMinutes,
    OneHour,
    TwoHours,
    Custom(DateTime<Local>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Terminated,
    DeadlineReached,
}

pub struct TaskManager {
    system: System,
    tasks: HashMap<u32, Task>,
    selected_task: Option<u32>,
    search_query: String,
    custom_deadline: String,
}

impl TaskManager {
    fn update_tasks(&mut self) {
        self.system.refresh_all();
        
        let mut updated_tasks = HashMap::new();
        for process in self.system.processes().values() {
            let pid = process.pid().as_u32();
            let name = process.name().to_string();
            
            // Get CPU usage with proper refresh
            let cpu_usage = process.cpu_usage();
            let memory_usage = process.memory();

            // Filter based on search query
            if !self.search_query.is_empty() {
                if !name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                    continue;
                }
            }

            // Get process icon
            let icon = if let Some(exe_path) = process.exe().to_str() {
                ProcessIcon::from_exe_path(exe_path)
            } else {
                None
            };

            // Update or create task
            if let Some(existing_task) = self.tasks.get(&pid) {
                updated_tasks.insert(pid, Task {
                    name: existing_task.name.clone(),
                    pid,
                    cpu_usage,
                    memory_usage,
                    icon: icon.or_else(|| existing_task.icon.clone()),
                    deadline: existing_task.deadline,
                    status: existing_task.get_status(),
                });
            } else {
                updated_tasks.insert(pid, Task::new(name, pid, cpu_usage, memory_usage, icon));
            }
        }

        // Replace tasks with filtered and updated list
        self.tasks = updated_tasks;

        // Check for deadline reached
        let now = Local::now();
        let mut to_terminate = Vec::new();
        for (&pid, task) in &self.tasks {
            if let Some(deadline) = task.deadline {
                if now > deadline {
                    to_terminate.push(pid);
                }
            }
        }

        // Terminate tasks that reached their deadline
        for pid in to_terminate {
            self.terminate_process(pid);
        }
    }

    fn view(&self) -> Element<Message> {
        let total_cpu: f32 = self.tasks.values().map(|t| t.cpu_usage).sum();
        let total_memory_mb: f32 = self.tasks.values().map(|t| t.memory_usage as f32).sum();
        let _system_memory_mb = self.system.total_memory() as f32;

        // Create a sorted list of tasks that keeps selected task in place
        let mut sorted_tasks: Vec<(&u32, &Task)> = self.tasks.iter().collect();
        
        if let Some(selected_pid) = self.selected_task {
            // First find the selected task's position
            if let Some(selected_idx) = sorted_tasks.iter().position(|(pid, _)| **pid == selected_pid) {
                // Remove the selected task temporarily
                let selected_task = sorted_tasks.remove(selected_idx);
                
                // Sort the remaining tasks
                sorted_tasks.sort_by(|a, b| {
                    a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())
                });
                
                // Reinsert the selected task at its original position
                sorted_tasks.insert(selected_idx, selected_task);
            } else {
                // Selected task not found, sort normally
                sorted_tasks.sort_by(|a, b| {
                    a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())
                });
            }
        } else {
            // No selected task, sort normally
            sorted_tasks.sort_by(|a, b| {
                a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())
            });
        }

        let header = Container::new(
            Column::new()
                .spacing(5)
                .push(text("Task Manager").size(28))
                .push(
                    Row::new()
                        .spacing(20)
                        .push(text(&format!("Total Tasks: {}", self.tasks.len())).size(16))
                        .push(text(&format!("CPU Usage: {:.1}%", total_cpu)).size(16))
                        .push(text(&format!("Memory Usage: {:.1} GB", total_memory_mb / 1024.0 / 1024.0)).size(16))
                )
        )
        .style(theme::Container::Custom(Box::new(CustomStyle {
            background: DARK_BG,
            text: DARK_TEXT,
            border_radius: 12.0,
            border_width: 1.0,
            border_color: BORDER_COLOR,
        })))
        .padding(20)
        .width(Length::Fill);

        let search_bar = Container::new(
            text_input("Search processes...", &self.search_query)
                .on_input(Message::SearchInput)
                .padding(10)
                .size(16)
        )
        .style(theme::Container::Custom(Box::new(CustomStyle {
            background: DARK_BG_LIGHTER,
            text: DARK_TEXT,
            border_radius: 8.0,
            border_width: 1.0,
            border_color: BORDER_COLOR,
        })))
        .padding(10)
        .width(Length::Fill);

        let selected_controls = if let Some(selected_pid) = self.selected_task {
            Container::new(
                Column::new()
                    .spacing(10)
                    .push(
                        Row::new()
                            .spacing(10)
                            .push(Text::new("Set Deadline:").size(14))
                            .push(
                                button(Text::new("30m").size(14))
                                    .on_press(Message::SetDeadline(
                                        selected_pid,
                                        TimeInterval::ThirtyMinutes,
                                    ))
                                    .style(theme::Button::Custom(Box::new(CustomButtonStyle {
                                        background: ACCENT_BLUE,
                                        hover_background: ACCENT_BLUE_HOVER,
                                        text_color: Color::WHITE,
                                        border_radius: 6.0,
                                        border_width: 0.0,
                                        border_color: Color::TRANSPARENT,
                                    })))
                                    .padding(8)
                            )
                            .push(
                                button(Text::new("1h").size(14))
                                    .on_press(Message::SetDeadline(
                                        selected_pid,
                                        TimeInterval::OneHour,
                                    ))
                                    .style(theme::Button::Custom(Box::new(CustomButtonStyle {
                                        background: ACCENT_BLUE,
                                        hover_background: ACCENT_BLUE_HOVER,
                                        text_color: Color::WHITE,
                                        border_radius: 6.0,
                                        border_width: 0.0,
                                        border_color: Color::TRANSPARENT,
                                    })))
                                    .padding(8)
                            )
                            .push(
                                button(Text::new("2h").size(14))
                                    .on_press(Message::SetDeadline(
                                        selected_pid,
                                        TimeInterval::TwoHours,
                                    ))
                                    .style(theme::Button::Custom(Box::new(CustomButtonStyle {
                                        background: ACCENT_BLUE,
                                        hover_background: ACCENT_BLUE_HOVER,
                                        text_color: Color::WHITE,
                                        border_radius: 6.0,
                                        border_width: 0.0,
                                        border_color: Color::TRANSPARENT,
                                    })))
                                    .padding(8)
                            )
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(
                                        text_input("Minutes", &self.custom_deadline)
                                            .on_input(Message::CustomDeadlineInput)
                                            .padding(8)
                                            .size(14)
                                    )
                                    .push(
                                        button(Text::new("Set Custom").size(14))
                                            .on_press({
                                                let minutes = self.custom_deadline.parse::<i64>().unwrap_or(30);
                                                Message::SetDeadline(
                                                    selected_pid,
                                                    TimeInterval::Custom(Local::now() + chrono::Duration::minutes(minutes)),
                                                )
                                            })
                                            .style(theme::Button::Custom(Box::new(CustomButtonStyle {
                                                background: ACCENT_BLUE,
                                                hover_background: ACCENT_BLUE_HOVER,
                                                text_color: Color::WHITE,
                                                border_radius: 6.0,
                                                border_width: 0.0,
                                                border_color: Color::TRANSPARENT,
                                            })))
                                            .padding(8)
                                    )
                            )
                            .push(
                                button(Text::new("Clear").size(14))
                                    .on_press(Message::ClearDeadline(selected_pid))
                                    .style(theme::Button::Custom(Box::new(CustomButtonStyle {
                                        background: WARNING_COLOR,
                                        hover_background: WARNING_COLOR_HOVER,
                                        text_color: Color::WHITE,
                                        border_radius: 6.0,
                                        border_width: 0.0,
                                        border_color: Color::TRANSPARENT,
                                    })))
                                    .padding(8)
                            )
                    )
                    .push(
                        button(Text::new("End Task").size(14))
                            .on_press(Message::TerminateTask(selected_pid))
                            .style(theme::Button::Custom(Box::new(CustomButtonStyle {
                                background: WARNING_COLOR,
                                hover_background: WARNING_COLOR_HOVER,
                                text_color: Color::WHITE,
                                border_radius: 6.0,
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            })))
                            .padding(8)
                    )
            )
            .padding(15)
            .style(theme::Container::Custom(Box::new(CustomStyle {
                background: DARK_BG_LIGHTER,
                text: DARK_TEXT,
                border_radius: 8.0,
                border_width: 1.0,
                border_color: BORDER_COLOR,
            })))
        } else {
            Container::new(
                text("Select a task to manage")
                    .size(14)
            )
            .padding(15)
            .style(theme::Container::Custom(Box::new(CustomStyle {
                background: DARK_BG_LIGHTER,
                text: DARK_TEXT,
                border_radius: 8.0,
                border_width: 1.0,
                border_color: BORDER_COLOR,
            })))
        };

        let table_header = Row::new()
            .spacing(10)
            .push(text("Name").width(Length::FillPortion(4)).size(14))
            .push(text("CPU").width(Length::Fixed(100.0)).size(14))
            .push(text("Memory").width(Length::Fixed(100.0)).size(14))
            .push(text("Deadline").width(Length::Fixed(150.0)).size(14));

        let process_list = {
            let mut rows = Vec::new();
            for (pid, task) in sorted_tasks {
                let status = task.get_status();
                let row_color = match status {
                    ProcessStatus::DeadlineReached => WARNING_COLOR,
                    _ => DARK_BG,
                };

                let is_selected = self.selected_task == Some(*pid);

                // Create row with icon
                let task_row = button(
                    Container::new(
                        Row::new()
                            .spacing(10)
                            .push(
                                if let Some(icon) = &task.icon {
                                    Container::new(
                                        iced::widget::image::Image::new(icon.handle.clone())
                                            .width(Length::Fixed(16.0))
                                            .height(Length::Fixed(16.0))
                                    )
                                    .width(Length::Fixed(20.0))
                                    .center_y()
                                } else {
                                    Container::new(text("").width(Length::Fixed(20.0)))
                                }
                            )
                            .push(text(&task.name).width(Length::FillPortion(4)))
                            .push(text(&format!("{:.1}%", task.cpu_usage)).width(Length::Fixed(100.0)))
                            .push(text(&format!("{:.1} MB", task.memory_usage as f64 / 1024.0 / 1024.0)).width(Length::Fixed(100.0)))
                            .push(text(&task.format_deadline()).width(Length::Fixed(150.0)))
                    )
                    .width(Length::Fill)
                    .padding(10)
                )
                .on_press(Message::TaskSelected(*pid))
                .style(theme::Button::Custom(Box::new(CustomButtonStyle {
                    background: if is_selected {
                        ROW_HOVER
                    } else {
                        row_color
                    },
                    hover_background: ROW_HOVER,
                    text_color: if status == ProcessStatus::DeadlineReached {
                        Color::WHITE
                    } else {
                        DARK_TEXT
                    },
                    border_radius: 6.0,
                    border_width: 1.0,
                    border_color: BORDER_COLOR,
                })));;

                rows.push(task_row.into());
            }
            rows
        };

        let content = Column::new()
            .spacing(20)
            .push(header)
            .push(search_bar)
            .push(
                Container::new(
                    Column::new()
                        .spacing(2)
                        .push(
                            Container::new(table_header)
                                .padding(10)
                                .style(theme::Container::Custom(Box::new(CustomStyle {
                                    background: Color { a: 0.1, ..DARK_BG },
                                    text: DARK_TEXT,
                                    border_radius: 6.0,
                                    border_width: 1.0,
                                    border_color: BORDER_COLOR,
                                })))
                        )
                        .push(
                            scrollable(
                                Column::with_children(process_list)
                            ).height(Length::Fill)
                        )
                )
                .height(Length::Fill)
            )
            .push(selected_controls);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(theme::Container::Custom(Box::new(CustomStyle {
                background: DARK_BG,
                text: DARK_TEXT,
                border_radius: 12.0,
                border_width: 1.0,
                border_color: BORDER_COLOR,
            })))
            .into()
    }

    fn try_save_application_work(&self, process_name: &str) -> bool {
        let window_class = match process_name.to_uppercase().as_str() {
            "WINWORD.EXE" => "OpusApp",
            "EXCEL.EXE" => "XLMAIN",
            "POWERPNT.EXE" => "PPTFrameClass",
            "NOTEPAD.EXE" => "Notepad",
            "NOTEPAD++.EXE" => "Notepad++",
            _ => "",
        };

        unsafe {
            let mut window = if !window_class.is_empty() {
                let wide_class: Vec<u16> = OsString::from(window_class)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                find_window_by_class_and_name(Some(window_class), None)
            } else {
                find_window_by_class_and_name(None, Some(process_name))
            };

            if window.is_none() {
                window = find_window_by_class_and_name(None, Some(process_name));
            }

            if window.is_none() {
                use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW};
                use windows::Win32::Foundation::{BOOL, LPARAM};
                use std::sync::Mutex;
                
                let found_window = Arc::new(Mutex::new(None));
                let target_name = process_name.to_lowercase();
                
                unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
                    let found_window = (lparam.0 as *mut Mutex<Option<HWND>>)
                        .as_ref()
                        .unwrap();
                    
                    let mut title: [u16; 512] = [0; 512];
                    let len = GetWindowTextW(hwnd, &mut title);
                    let window_title = String::from_utf16_lossy(&title[..len as usize])
                        .to_lowercase();
                    
                    if window_title.contains(&(lparam.0 as *const String)
                        .as_ref()
                        .unwrap()
                        .to_lowercase()
                    ) {
                        *found_window.lock().unwrap() = Some(hwnd);
                        BOOL(0)
                    } else {
                        BOOL(1)
                    }
                }
                
                EnumWindows(
                    Some(enum_callback),
                    LPARAM(&target_name as *const String as isize)
                );
                
                window = *found_window.lock().unwrap();
            }

            if window.is_none() {
                println!("❌ Could not find window for process: {}", process_name);
                return false;
            }

            let mut enigo = Enigo::new();
            enigo.key_down(Key::Control);
            enigo.key_click(Key::Layout('s'));
            enigo.key_up(Key::Control);

            true
        }
    }

    fn should_try_save(&self, process_name: &str) -> bool {
        let process_upper = process_name.to_uppercase();
        
        // Add more application categories that might need special save handling
        OFFICE_APPS.iter().any(|&app| process_upper.contains(&app.to_uppercase())) ||
        TEXT_EDITORS.iter().any(|&app| process_upper.contains(&app.to_uppercase())) ||
        IDES.iter().any(|&app| process_upper.contains(&app.to_uppercase())) ||
        DESIGN_APPS.iter().any(|&app| process_upper.contains(&app.to_uppercase())) ||
        DEVELOPMENT_TOOLS.iter().any(|&app| process_upper.contains(&app.to_uppercase())) ||
        CREATIVE_TOOLS.iter().any(|&app| process_upper.contains(&app.to_uppercase())) ||
        BROWSER_APPS.iter().any(|&app| process_upper.contains(&app.to_uppercase())) ||
        PRODUCTIVITY_APPS.iter().any(|&app| process_upper.contains(&app.to_uppercase())) ||
        DEVELOPMENT_APPS.iter().any(|&app| process_upper.contains(&app.to_uppercase()))
    }

    fn terminate_process(&mut self, pid: u32) {
        if let Some(process) = self.system.process(Pid::from_u32(pid)) {
            let name = process.name();
            println!("📝 Process name: {}", name);

            // Try to save work if it's a supported application
            if self.should_try_save(name) {
                println!("💾 Attempting to save work before termination...");
                if self.try_save_application_work(name) {
                    println!("✅ Save attempt completed");
                    // Give the application more time to finish saving
                    thread::sleep(Duration::from_secs(2));
                } else {
                    println!("⚠️ Could not attempt save");
                }
            }

            unsafe {
                let process_handle = OpenProcess(
                    PROCESS_ACCESS_RIGHTS(PROCESS_TERMINATE.0),
                    false,
                    pid
                );

                match process_handle {
                    Ok(handle) => {
                        if handle.is_invalid() {
                            println!("❌ Failed to get process handle - Access Denied");
                            return;
                        }

                        // Try to save one more time before terminating
                        if self.should_try_save(name) {
                            self.try_save_application_work(name);
                            thread::sleep(Duration::from_secs(1));
                        }

                        let result = TerminateProcess(handle, 1);
                        if result.as_bool() {
                            println!("✅ Process terminated successfully");
                            self.tasks.remove(&pid);
                            self.selected_task = None;
                        } else {
                            println!("❌ Failed to terminate process - Operation Failed");
                        }

                        let _ = CloseHandle(HANDLE(handle.0));
                    }
                    Err(_) => {
                        println!("❌ Failed to open process - Access Denied");
                    }
                }
            }
        }
    }
}

impl Application for TaskManager {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            TaskManager {
                system: System::new_all(),
                tasks: HashMap::new(),
                selected_task: None,
                search_query: String::new(),
                custom_deadline: String::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("TaskTide - Modern Task Manager")
    }

    fn subscription(&self) -> Subscription<Message> {
        every(Duration::from_secs(1)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TaskSelected(pid) => {
                self.selected_task = Some(pid);
                Command::none()
            }
            Message::SearchInput(query) => {
                self.search_query = query;
                self.update_tasks();
                Command::none()
            }
            Message::CustomDeadlineInput(input) => {
                self.custom_deadline = input;
                Command::none()
            }
            Message::TerminateTask(pid) => {
                self.terminate_process(pid);
                Command::none()
            }
            Message::SetDeadline(pid, interval) => {
                if let Some(task) = self.tasks.get_mut(&pid) {
                    match interval {
                        TimeInterval::ThirtyMinutes => task.deadline = Some(Local::now() + chrono::Duration::minutes(30)),
                        TimeInterval::OneHour => task.deadline = Some(Local::now() + chrono::Duration::hours(1)),
                        TimeInterval::TwoHours => task.deadline = Some(Local::now() + chrono::Duration::hours(2)),
                        TimeInterval::Custom(deadline) => task.deadline = Some(deadline),
                    }
                }
                Command::none()
            }
            Message::ClearDeadline(pid) => {
                if let Some(task) = self.tasks.get_mut(&pid) {
                    task.deadline = None;
                }
                Command::none()
            }
            Message::Tick => {
                self.update_tasks();
                Command::none()
            }
            Message::CheckDeadlines => {
                self.update_tasks();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        self.view()
    }
}

fn find_window_by_class_and_name(class_name: Option<&str>, window_name: Option<&str>) -> Option<HWND> {
    unsafe {
        let wide_class: Option<Vec<u16>> = class_name.map(|s| {
            OsString::from(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        });
        
        let wide_name: Option<Vec<u16>> = window_name.map(|s| {
            OsString::from(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        });

        let class_ptr = wide_class
            .as_ref()
            .map(|v| PCWSTR::from_raw(v.as_ptr()))
            .unwrap_or(PCWSTR::null());

        let name_ptr = wide_name
            .as_ref()
            .map(|v| PCWSTR::from_raw(v.as_ptr()))
            .unwrap_or(PCWSTR::null());

        let hwnd = FindWindowW(class_ptr, name_ptr);
        if hwnd.0 == 0 {
            None
        } else {
            Some(hwnd)
        }
    }
}

fn main() -> iced::Result {
    let icon = icon::from_file_data(
        include_bytes!("../assets/logo.png"),
        Some(image::ImageFormat::Png),
    ).unwrap();

    TaskManager::run(Settings {
        window: window::Settings {
            size: (800, 600),
            position: Position::Centered,
            icon: Some(icon),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug, Clone)]
pub struct CustomStyle {
    background: Color,
    text: Color,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

impl From<CustomStyle> for iced::theme::Container {
    fn from(style: CustomStyle) -> Self {
        iced::theme::Container::Custom(Box::new(style))
    }
}

impl container::StyleSheet for CustomStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(self.text),
            background: Some(iced::Background::Color(self.background)),
            border_radius: self.border_radius.into(),
            border_width: self.border_width,
            border_color: self.border_color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CustomButtonStyle {
    background: Color,
    hover_background: Color,
    text_color: Color,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

impl button::StyleSheet for CustomButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::default(),
            background: Some(iced::Background::Color(self.background)),
            border_radius: self.border_radius.into(),
            border_width: self.border_width,
            border_color: self.border_color,
            text_color: self.text_color,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.hover_background)),
            ..self.active(style)
        }
    }
}

#[derive(Debug, Clone)]
struct ProcessIcon {
    handle: Handle,
}

impl ProcessIcon {
    fn new(handle: Handle) -> Self {
        ProcessIcon { handle }
    }

    pub fn from_exe_path(path: &str) -> Option<Self> {
        unsafe {
            let mut bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: 16,
                    biHeight: 16,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0 as u32,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default()],
            };

            let wide_path: Vec<u16> = OsString::from(path)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            
            let mut icon_large: HICON = HICON::default();
            let mut icon_small: HICON = HICON::default();
            
            let result = ExtractIconExW(
                PCWSTR::from_raw(wide_path.as_ptr()),
                0,
                Some(&mut icon_large as *mut _),
                Some(&mut icon_small as *mut _),
                1,
            );

            if result > 0 && !icon_small.is_invalid() {
                let mut icon_info = ICONINFO::default();
                if GetIconInfo(icon_small, &mut icon_info).as_bool() {
                    // Convert icon to image
                    let mut bits = vec![0u8; 16 * 16 * 4];
                    let hdc = GetDC(HWND(0));
                    
                    GetDIBits(
                        hdc,
                        icon_info.hbmColor,
                        0,
                        16,
                        Some(bits.as_mut_ptr() as *mut _),
                        &mut bi,
                        DIB_RGB_COLORS,
                    );

                    ReleaseDC(HWND(0), hdc);
                    
                    if !icon_large.is_invalid() {
                        windows::Win32::UI::WindowsAndMessaging::DestroyIcon(icon_large);
                    }
                    windows::Win32::UI::WindowsAndMessaging::DestroyIcon(icon_small);

                    // Convert to iced image handle
                    let img_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(16, 16, bits)
                        .unwrap_or_else(|| ImageBuffer::new(16, 16));
                    
                    let dynamic_image = DynamicImage::ImageRgba8(img_buffer);
                    let rgba_bytes = dynamic_image.to_rgba8().to_vec();
                    
                    return Some(ProcessIcon::new(Handle::from_pixels(16, 16, rgba_bytes)));
                }
            }
        }
        None
    }
}