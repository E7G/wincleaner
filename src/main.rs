#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use freya::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

// Include the window icon
const WINDOW_ICON: &[u8] = include_bytes!("../assets/wincleaner_icon.png");

// 环形日志缓冲区 - 恒定大小，保留最近100条日志
use std::collections::VecDeque;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static LOG_RING: Lazy<Mutex<VecDeque<String>>> = Lazy::new(|| {
    Mutex::new(VecDeque::with_capacity(100))
});

fn log(message: &str) {
    const LOG_FILE: &str = "wincleaner.log";
    const MAX_LOGS: usize = 100;
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let entry = format!("[{}] {}\n", timestamp, message);
    
    let mut ring = LOG_RING.lock().unwrap();
    
    // 环形缓冲区：满了就移除最旧的
    if ring.len() >= MAX_LOGS {
        ring.pop_front();
    }
    ring.push_back(entry);
    
    // 原子化文件写入，失败时报告错误
    let content = ring.iter().cloned().collect::<String>();
    if let Err(e) = std::fs::write(LOG_FILE, content) {
        eprintln!("日志写入失败: {}", e);
    }
}

// 加载自定义清理规则
fn load_custom_tasks() -> Vec<CleanTask> {
    const CONFIG_FILE: &str = "wincleaner-config.toml";
    
    match std::fs::read_to_string(CONFIG_FILE) {
        Ok(content) => {
            // 定义配置结构体来匹配 TOML 格式
            #[derive(Deserialize)]
            struct Config {
                task: Vec<CleanTask>,
            }
            
            // 解析为配置结构体
            match toml::from_str::<Config>(&content) {
                Ok(config) => {
                    log(&format!("加载了 {} 个自定义清理规则", config.task.len()));
                    config.task
                }
                Err(e) => {
                    log(&format!("配置文件格式错误: {}", e));
                    Vec::new()
                }
            }
        },
        Err(_) => {
            // 配置文件不存在，创建示例配置
            let example_tasks = vec![CleanTask {
                name: "示例: 清理临时文件".to_string(),
                description: "清理用户临时文件夹".to_string(),
                category: CleanCategory::Custom,
                command: "del /q %TEMP%\\*.tmp".to_string(),
                path_check: Some("%TEMP%".to_string()),
                requires_confirmation: true,
                dangerous: false,
                estimated_size: Some("~100MB".to_string()),
                icon: Some("📝".to_string()),
            }];
            
            // 创建符合 TOML 格式的配置内容
            let config_str = format!(
                "# WinCleaner 自定义清理规则配置\n# 警告：请谨慎配置，错误的命令可能导致系统问题\n\n[[task]]\n{}\n[[task]]\nname = \"清理 VSCode 工作区缓存\"\ndescription = \"清理 VSCode 工作区缓存文件\"\ncategory = \"Custom\"\ncommand = \"rmdir /s /q %APPDATA%\\\\Code\\\\User\\\\workspaceStorage\"\npath_check = \"%APPDATA%\\\\Code\\\\User\\\\workspaceStorage\"\nrequires_confirmation = true\ndangerous = false\nestimated_size = \"auto\"\nicon = \"💻\"",
                example_tasks.iter().map(|task| toml::to_string_pretty(task).unwrap()).collect::<Vec<_>>().join("\n").replace("[", "").replace("]", "")
            );
            
            let _ = std::fs::write(CONFIG_FILE, &config_str);
            log(&format!("创建示例配置文件"));
            Vec::new()
        }
    }
}

// Apple设计系统色彩方案 - 语义化命名
#[derive(PartialEq)]
struct AppTheme {
    // 背景层次 - macOS风格
    background_primary: &'static str,
    background_secondary: &'static str,
    background_tertiary: &'static str,

    // 前景内容
    label_primary: &'static str,
    label_secondary: &'static str,
    label_tertiary: &'static str,

    // 交互元素
    accent: &'static str,
    accent_hover: &'static str,
    danger: &'static str,
    danger_hover: &'static str,

    // 边框和分隔线
    separator: &'static str,
    grid: &'static str,
}

// 浅色主题 - 参考macOS浅色模式
const LIGHT_THEME: AppTheme = AppTheme {
    background_primary: "rgb(255, 255, 255)",
    background_secondary: "rgb(247, 247, 247)",
    background_tertiary: "rgb(242, 242, 247)",

    label_primary: "rgb(0, 0, 0)",
    label_secondary: "rgb(99, 99, 102)",
    label_tertiary: "rgb(142, 142, 147)",

    accent: "rgb(0, 122, 255)",
    accent_hover: "rgb(0, 105, 220)",
    danger: "rgb(255, 59, 48)",
    danger_hover: "rgb(230, 35, 25)",

    separator: "rgb(224, 224, 224)",
    grid: "rgb(229, 229, 234)",
};

// 深色主题 - 参考macOS深色模式
const DARK_THEME: AppTheme = AppTheme {
    background_primary: "rgb(28, 28, 30)",
    background_secondary: "rgb(44, 44, 46)",
    background_tertiary: "rgb(58, 58, 60)",

    label_primary: "rgb(255, 255, 255)",
    label_secondary: "rgb(174, 174, 178)",
    label_tertiary: "rgb(99, 99, 102)",

    accent: "rgb(10, 132, 255)",
    accent_hover: "rgb(20, 122, 255)",
    danger: "rgb(255, 69, 58)",
    danger_hover: "rgb(235, 49, 38)",

    separator: "rgb(84, 84, 88)",
    grid: "rgb(58, 58, 62)",
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
enum CleanCategory {
    #[default]
    DevTools,
    AppCache,
    System,
    Custom, // 用户自定义分类
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
struct CleanTask {
    name: String,
    description: String,
    category: CleanCategory,
    command: String,
    path_check: Option<String>,
    requires_confirmation: bool,
    dangerous: bool,
    estimated_size: Option<String>,
    icon: Option<String>,
}

impl CleanTask {
    // 获取展开后的路径检查
    fn get_expanded_path(&self) -> Option<String> {
        self.path_check.as_ref().map(|path| expand_environment_variables(path))
    }
    
    // 获取实际大小，支持自动检测
    fn get_actual_size(&self) -> Option<String> {
        if let Some(ref size_str) = self.estimated_size {
            if size_str == "auto" {
                // 自动检测模式 - 使用展开后的路径
                if let Some(ref path) = self.get_expanded_path() {
                    return get_directory_size(path).map(format_size);
                }
            }
        }
        self.estimated_size.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CleanupStats {
    total_tasks: usize,
    successful_tasks: usize,
    failed_tasks: usize,
    total_space_freed: Option<u64>, // in bytes
    errors: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum AppState {
    Idle,
    Running(String),
    Success,
    SuccessWithStats(CleanupStats),
    Error(String),
}

// 主题管理 - 支持动态切换
#[derive(Clone, Copy, Debug, PartialEq)]
enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    fn current_theme(&self) -> &'static AppTheme {
        match self {
            ThemeMode::Light => &LIGHT_THEME,
            ThemeMode::Dark => &DARK_THEME,
        }
    }
}

// 获取目录大小（递归计算）
fn get_directory_size(path: &str) -> Option<u64> {
    let expanded_path = expand_environment_variables(path);
    let path = Path::new(&expanded_path);

    if !path.exists() {
        return None;
    }

    fn dir_size(dir: &Path) -> std::io::Result<u64> {
        let mut size = 0;
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    size += dir_size(&path)?;
                } else {
                    size += entry.metadata()?.len();
                }
            }
        }
        Ok(size)
    }

    match dir_size(path) {
        Ok(size) => Some(size),
        Err(_) => None,
    }
}

// 格式化文件大小为可读格式
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

// 扩展环境变量
fn expand_environment_variables(path: &str) -> String {
    if !path.contains('%') {
        return path.to_string();
    }
    
    // 获取所有常用Windows环境变量
    let env_vars = [
        ("%USERPROFILE%", std::env::var("USERPROFILE").unwrap_or_default()),
        ("%APPDATA%", std::env::var("APPDATA").unwrap_or_default()),
        ("%LOCALAPPDATA%", std::env::var("LOCALAPPDATA").unwrap_or_default()),
        ("%TEMP%", std::env::var("TEMP").unwrap_or_default()),
        ("%TMP%", std::env::var("TMP").unwrap_or_default()),
        ("%PROGRAMFILES%", std::env::var("PROGRAMFILES").unwrap_or_default()),
        ("%PROGRAMFILES(X86)%", std::env::var("PROGRAMFILES(X86)").unwrap_or_default()),
        ("%SYSTEMDRIVE%", std::env::var("SYSTEMDRIVE").unwrap_or_default()),
        ("%WINDIR%", std::env::var("WINDIR").unwrap_or_default()),
        ("%PUBLIC%", std::env::var("PUBLIC").unwrap_or_default()),
    ];
    
    let mut result = path.to_string();
    for (var_name, var_value) in &env_vars {
        result = result.replace(var_name, var_value);
    }
    
    result
}

fn main() {
    let window_icon = LaunchConfig::load_icon(WINDOW_ICON);

    launch_cfg(
        app,
        LaunchConfig::<()>::new()
            .with_size(900.0, 700.0)
            .with_decorations(true)
            .with_transparency(false)
            .with_title("WinCleaner - Windows系统清理工具")
            .with_background("rgb(28, 28, 30)")
            .with_icon(window_icon),
    );
}

fn app() -> Element {
    // Apple风格主题管理
    let mut theme_mode = use_signal(|| ThemeMode::Dark); // 默认深色主题，更专业
    let theme = theme_mode().current_theme();

    let tasks = use_signal(|| {
        vec![
            CleanTask {
                name: "Go Module Cache".to_string(),
                description: "清理Go模块缓存".to_string(),
                category: CleanCategory::DevTools,
                command: "go clean -modcache".to_string(),
                path_check: None,
                requires_confirmation: false,
                dangerous: false,
                estimated_size: Some("~500MB".to_string()), // Go缓存大小相对稳定，保持估算
                icon: Some("🐹".to_string()),
            },
            CleanTask {
                name: "Gradle Cache".to_string(),
                description: "清理Gradle缓存".to_string(),
                category: CleanCategory::DevTools,
                command: "rmdir /s /q %USERPROFILE%\\.gradle\\caches".to_string(),
                path_check: Some("%USERPROFILE%\\.gradle\\caches".to_string()),
                requires_confirmation: false,
                dangerous: false,
                estimated_size: Some("auto".to_string()), // 自动检测实际大小
                icon: Some("🐘".to_string()),
            },
            CleanTask {
                name: "Cargo Cache".to_string(),
                description: "清理Cargo缓存（需要cargo-cache）".to_string(),
                category: CleanCategory::DevTools,
                command: "cargo cache --remove-dir all".to_string(),
                path_check: None,
                requires_confirmation: false,
                dangerous: false,
                estimated_size: Some("~2GB".to_string()),
                icon: Some("🦀".to_string()),
            },
            CleanTask {
                name: "npm Cache".to_string(),
                description: "清理npm缓存".to_string(),
                category: CleanCategory::DevTools,
                command: "npm cache clean --force".to_string(),
                path_check: None,
                requires_confirmation: false,
                dangerous: false,
                estimated_size: Some("~200MB".to_string()),
                icon: Some("📦".to_string()),
            },
            CleanTask {
                name: "Trae AI Chat Logs".to_string(),
                description: "清理Trae AI聊天记录（可能很大）".to_string(),
                category: CleanCategory::AppCache,
                command: "rmdir /s /q %USERPROFILE%\\.marscode\\ai-chat\\logs".to_string(),
                path_check: Some("%USERPROFILE%\\.marscode\\ai-chat\\logs".to_string()),
                requires_confirmation: true,
                dangerous: false,
                estimated_size: Some("auto".to_string()), // 自动检测实际大小
                icon: Some("🤖".to_string()),
            },
            CleanTask {
                name: "KuGou Image Cache".to_string(),
                description: "清理酷狗音乐图片缓存".to_string(),
                category: CleanCategory::AppCache,
                command: "rmdir /s /q %USERPROFILE%\\AppData\\Roaming\\KuGou8\\ImagesCache"
                    .to_string(),
                path_check: Some(
                    "%USERPROFILE%\\AppData\\Roaming\\KuGou8\\ImagesCache".to_string(),
                ),
                requires_confirmation: false,
                dangerous: false,
                estimated_size: Some("auto".to_string()), // 自动检测实际大小
                icon: Some("🎵".to_string()),
            },
            CleanTask {
                name: "VSCode Cpptools Cache".to_string(),
                description: "清理VSCode Cpptools缓存".to_string(),
                category: CleanCategory::AppCache,
                command: "rmdir /s /q %LocalAppData%\\Microsoft\\vscode-cpptools".to_string(),
                path_check: Some("%LocalAppData%\\Microsoft\\vscode-cpptools".to_string()),
                requires_confirmation: false,
                dangerous: false,
                estimated_size: Some("auto".to_string()), // 自动检测实际大小
                icon: Some("💻".to_string()),
            },
            CleanTask {
                name: "Office Updates".to_string(),
                description: "清理Office更新缓存".to_string(),
                category: CleanCategory::AppCache,
                command: "rmdir /s /q \"C:\\Program Files (x86)\\Microsoft Office\\Updates\""
                    .to_string(),
                path_check: Some("C:\\Program Files (x86)\\Microsoft Office\\Updates".to_string()),
                requires_confirmation: true,
                dangerous: true,
                estimated_size: Some("auto".to_string()), // 自动检测实际大小
                icon: Some("📊".to_string()),
            },
            CleanTask {
                name: "Gradle Wrapper Dists".to_string(),
                description: "清理Gradle Wrapper分发缓存".to_string(),
                category: CleanCategory::DevTools,
                command: "rmdir /s /q %USERPROFILE%\\.gradle\\wrapper\\dists".to_string(),
                path_check: Some("%USERPROFILE%\\.gradle\\wrapper\\dists".to_string()),
                requires_confirmation: false,
                dangerous: false,
                estimated_size: Some("auto".to_string()), // 自动检测实际大小
                icon: Some("🐘".to_string()),
            },
            CleanTask {
                name: "QQ MiniApp".to_string(),
                description: "清理QQ小程序缓存（未经测试）".to_string(),
                category: CleanCategory::AppCache,
                command: "rmdir /s /q %USERPROFILE%\\AppData\\Roaming\\QQ\\miniapp".to_string(),
                path_check: Some("%USERPROFILE%\\AppData\\Roaming\\QQ\\miniapp".to_string()),
                requires_confirmation: true,
                dangerous: true,
                estimated_size: Some("auto".to_string()), // 自动检测实际大小
                icon: Some("💬".to_string()),
            },
            CleanTask {
                name: "System Component Cleanup".to_string(),
                description: "系统组件清理（需要管理员权限）".to_string(),
                category: CleanCategory::System,
                command: "Dism.exe /online /Cleanup-Image /StartComponentCleanup /ResetBase"
                    .to_string(),
                path_check: None,
                requires_confirmation: true,
                dangerous: true,
                estimated_size: Some("~1-3GB".to_string()),
                icon: Some("⚙️".to_string()),
            },
            CleanTask {
                name: "Disk Cleanup".to_string(),
                description: "Windows自带磁盘清理工具".to_string(),
                category: CleanCategory::System,
                command: "cleanmgr".to_string(),
                path_check: None,
                requires_confirmation: false,
                dangerous: false,
                estimated_size: Some("~可变".to_string()),
                icon: Some("🧹".to_string()),
            },
            CleanTask {
                name: "Clear Recycle Bin".to_string(),
                description: "清空回收站".to_string(),
                category: CleanCategory::System,
                command: "powershell Clear-RecycleBin -Force".to_string(),
                path_check: None,
                requires_confirmation: true,
                dangerous: false,
                estimated_size: Some("~可变".to_string()),
                icon: Some("🗑️".to_string()),
            },
        ]
    });

    // 状态管理
    let mut selected_tasks = use_signal(|| HashSet::<String>::new());
    let mut progress = use_signal(|| 0.0f32);
    let mut show_batch_mode = use_signal(|| false);
    let mut selected_category = use_signal(|| CleanCategory::DevTools);
    let mut app_state = use_signal(|| AppState::Idle);
    
    // 加载自定义任务并合并到任务列表中
    let custom_tasks = load_custom_tasks();
    let all_tasks = {
        let mut all = tasks();
        all.extend(custom_tasks);
        all
    };

    // 批量清理功能已内联到按钮点击事件中
    let mut show_confirmation = use_signal(|| None::<CleanTask>);

    let theme_icon = if theme_mode() == ThemeMode::Dark {
        "🌙"
    } else {
        "☀️"
    };

    let categories = vec![
        ("开发工具", CleanCategory::DevTools),
        ("应用缓存", CleanCategory::AppCache),
        ("系统清理", CleanCategory::System),
        ("自定义规则", CleanCategory::Custom),
    ];

    let filtered_tasks = all_tasks
        .iter()
        .filter(|task| task.category == selected_category())
        .cloned()
        .collect::<Vec<_>>();

    rsx!(

        // Apple风格主界面
        rect {
            width: "100%",
            height: "100%",
            padding: "20",
            background: theme.background_primary,
            color: theme.label_primary,
            direction: "vertical",  // 垂直布局，让内容自动填充

            // 标题栏 - 类似macOS窗口标题
            rect {
                direction: "horizontal",
                width: "100%",
                height: "auto",
                main_align: "space_between",
                cross_align: "center",
                padding: "0 0 20 0",

                rect {
                    direction: "horizontal",
                    cross_align: "center",

                    label {
                        font_size: "24",
                        font_weight: "bold",
                        "WinCleaner"
                    }

                    rect {
                        width: "10"
                    }

                    label {
                        font_size: "16",
                        color: theme.label_secondary,
                        "系统清理工具"
                    }
                }

                // 主题切换按钮 - 类似macOS控制中心
                rect {
                    direction: "horizontal",
                    cross_align: "center",
                    padding: "8 12",
                    background: theme.background_tertiary,
                    corner_radius: "8",

                    label {
                        font_size: "14",
                        color: theme.label_secondary,
                        "主题"
                    }

                    rect {
                        width: "8"
                    }

                    Button {
                        onclick: move |_| {
                            let new_mode = match theme_mode() {
                                ThemeMode::Dark => ThemeMode::Light,
                                ThemeMode::Light => ThemeMode::Dark,
                            };
                            theme_mode.set(new_mode);
                        },
                        theme: theme_with!(ButtonTheme {
                            background: std::borrow::Cow::Borrowed("transparent"),
                            hover_background: std::borrow::Cow::Borrowed(theme.background_secondary),
                        }),
                        label {
                            font_size: "14",
                            "{theme_icon}"
                        }
                    }

                    rect {
                        width: "16"
                    }

                    label {
                        font_size: "14",
                        color: theme.label_secondary,
                        "批量模式"
                    }

                    rect {
                        width: "8"
                    }

                    Switch {
                        enabled: show_batch_mode(),
                        ontoggled: move |_| show_batch_mode.set(!show_batch_mode()),
                    }
                }
            }


            // 主内容区域 - 类似macOS侧边栏布局
            rect {
                direction: "horizontal",
                width: "100%",
                height: "fill",  // 使用fill填充剩余空间

                // 左侧边栏 - 分类和通知区域
                rect {
                    width: "200",
                    direction: "vertical",
                    height: "fill",

                    // 分类选择区域
                    rect {
                        width: "100%",
                        padding: "16",
                        background: theme.background_secondary,
                        corner_radius: "12",
                        margin: "0 0 12 0",

                        label {
                            font_size: "16",
                            font_weight: "semibold",
                            color: theme.label_primary,
                            margin: "0 0 16 0",
                            "清理分类"
                        }

                        for (name, category) in categories {
                            Button {
                                onclick: move |_| selected_category.set(category),
                                theme: theme_with!(ButtonTheme {
                                    background: if category == selected_category() {
                                        std::borrow::Cow::Borrowed(theme.accent)
                                    } else {
                                        std::borrow::Cow::Borrowed("transparent")
                                    },
                                    hover_background: if category == selected_category() {
                                        std::borrow::Cow::Borrowed(theme.accent_hover)
                                    } else {
                                        std::borrow::Cow::Borrowed(theme.background_tertiary)
                                    },
                                }),
                                label {
                                    font_size: "14",
                                    color: if category == selected_category() { "white" } else { theme.label_primary },
                                    "{name}"
                                }
                            }

                            rect {
                                height: "6"
                            }
                        }
                    }

                    // 通知气泡独立区域 - 放在分类下方但分隔开
                    NotificationBubble {
                        app_state: app_state(),
                        theme: theme
                    }

                    rect {
                        height: "16"
                    }

                    // 进度条（批量模式时显示）- Apple风格
                    if show_batch_mode() && matches!(app_state(), AppState::Running(_)) {
                        rect {
                            padding: "16",
                            background: theme.background_secondary,
                            corner_radius: "12",
                            margin: "0 0 20 0",
                            width: "100%",

                            rect {
                                direction: "horizontal",
                                main_align: "space_between",
                                cross_align: "center",
                                margin: "0 0 8 0",

                                label {
                                    font_size: "14",
                                    font_weight: "medium",
                                    "批量清理进度"
                                }

                            }

                            ProgressBar {
                                progress: (progress() * 100.0) as f32,
                                show_progress: true,
                                width: "100%",
                            }
                        }
                    }

                }

                rect {
                    width: "20"
                }

                // 右侧任务列表 - 类似macOS主内容区域
                rect {
                    width: "calc(100% - 220)",
                    padding: "16",
                    background: theme.background_secondary,
                    corner_radius: "12",
                    height: "fill",  // 确保占满父容器高度

                    ScrollView {
                        width: "100%",
                        height: "100%",

                        // 列表头部 - 类似Finder工具栏
                        rect {
                            direction: "horizontal",
                            width: "100%",
                            padding: "0 0 16 0",
                            main_align: "space_between",
                            cross_align: "center",
                            margin: "0 0 16 0",

                            label {
                                font_size: "18",
                                font_weight: "semibold",
                                color: theme.label_primary,
                                "{selected_category():?}"
                            }

                            if show_batch_mode() && !selected_tasks().is_empty() {
                                FilledButton {
                                    onclick: move |_| {
                                        let selected = selected_tasks();
                                        if !selected.is_empty() {
                                            app_state.set(AppState::Running(format!(
                                                "批量清理 {} 个任务",
                                                selected.len()
                                            )));
                                            progress.set(0.0);

                                            let mut app_state_clone = app_state;
                                            let mut progress_clone = progress;
                                            let mut selected_tasks_clone = selected_tasks;
                                            let all_tasks_clone = all_tasks.clone();

                                            spawn(async move {
                                                let total = selected.len();
                                                let mut completed = 0;
                                                let mut successful_tasks = 0;
                                                let mut failed_tasks = 0;
                                                let mut total_space_freed: u64 = 0;
                                                let mut errors = Vec::new();

                                                for task_name in selected {
                                                    if let Some(task) = all_tasks_clone.iter().find(|t| t.name == task_name) {
                                                        app_state_clone.set(AppState::Running(format!("正在清理: {}", task.name)));

                                                        let space_before = if let Some(ref path) = task.path_check {
                                                            get_directory_size(&expand_environment_variables(path))
                                                        } else {
                                                            None
                                                        };

                                                        let result = run_clean_task_impl(task.clone()).await;
                                                        completed += 1;
                                                        progress_clone.set(completed as f32 / total as f32);

                                                        match result {
                                                            Ok(_) => {
                                                                successful_tasks += 1;

                                                                if let Some(ref path) = task.path_check {
                                                                    let space_after = get_directory_size(&expand_environment_variables(path));
                                                                    if let (Some(before), Some(after)) = (space_before, space_after) {
                                                                        if before > after {
                                                                            total_space_freed += before - after;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                failed_tasks += 1;
                                                                errors.push(format!("{}: {}", task.name, e));
                                                            }
                                                        }
                                                    }
                                                }

                                                let stats = CleanupStats {
                                                    total_tasks: total,
                                                    successful_tasks,
                                                    failed_tasks,
                                                    total_space_freed: if total_space_freed > 0 {
                                                        Some(total_space_freed)
                                                    } else {
                                                        None
                                                    },
                                                    errors,
                                                };

                                                if failed_tasks > 0 {
                                                    app_state_clone.set(AppState::SuccessWithStats(stats));
                                                } else {
                                                    app_state_clone.set(AppState::Success);
                                                }
                                                selected_tasks_clone.set(HashSet::new());
                                            });
                                        }
                                    },

                                    label {
                                font_size: "14",
                                color: "white",
                                "清理选中 ({selected_tasks().len()})"
                            }
                                }
                            }
                        }

                        if filtered_tasks.is_empty() {
                            label {
                                font_size: "14",
                                color: theme.label_secondary,
                                "该分类下没有清理任务"
                            }
                        } else {
                            for task in filtered_tasks {
                                TaskCard {
                                    task: task.clone(),
                                    show_batch_mode: show_batch_mode(),
                                    selected_tasks: selected_tasks(),
                                    on_toggle: move |_| {
                                        let mut selected = selected_tasks();
                                        if selected.contains(&task.name) {
                                            selected.remove(&task.name);
                                        } else {
                                            selected.insert(task.name.clone());
                                        }
                                        selected_tasks.set(selected);
                                    },
                                    app_state: app_state.clone(),
                                    show_confirmation: show_confirmation.clone(),
                                    theme: theme,
                                }
                                rect {
                                    height: "12"
                                }
                            }
                        }
                    }
                }
            }

        }

        // 使用Freya内置Popup组件替代自定义对话框
        if let Some(task) = show_confirmation() {
            Popup {
                oncloserequest: move |_| show_confirmation.set(None),
                show_close_button: true,
                theme: theme_with!(PopupTheme {
                    background: std::borrow::Cow::Borrowed(theme.background_secondary),
                    color: std::borrow::Cow::Borrowed(theme.label_primary),
                    cross_fill: std::borrow::Cow::Borrowed(theme.label_secondary),
                    width: std::borrow::Cow::Borrowed("360"),
                    height: std::borrow::Cow::Borrowed("300"),
                }),

                PopupTitle {
                    label {
                        color: theme.label_primary,
                        "确认执行清理操作"
                    }
                }

                PopupContent {
                    // 内容区域使用ScrollView包裹，支持滚动
                    ScrollView {
                        height: "calc(100% - 60)",  // 为按钮区域预留空间

                        label {
                            color: theme.label_primary,
                            "您确定要执行以下清理操作吗？"
                        }

                        rect {
                            height: "10"
                        }

                        rect {
                            padding: "16",
                            background: theme.background_tertiary,
                            corner_radius: "8",

                            label {
                                font_weight: "bold",
                                color: theme.label_primary,
                                margin: "0 0 8 0",
                                "{task.name}"
                            }
                            label {
                                font_size: "14",
                                color: theme.label_secondary,
                                margin: "0 0 12 0",
                                "{task.description}"
                            }

                            if task.dangerous {
                                rect {
                                    padding: "12",
                                    background: if theme_mode() == ThemeMode::Dark { "rgb(60, 30, 30)" } else { "rgb(255, 240, 240)" },
                                    corner_radius: "6",
                                    border: "1 solid {theme.danger}",

                                    label {
                                        font_size: "13",
                                        color: theme.danger,
                                        "⚠️ 警告: 此操作可能影响系统稳定性！"
                                    }
                                }
                            }
                        }
                    }

                    // 按钮区域固定底部
                    rect {
                        height: "60",
                        padding: "12 0 0 0",
                        direction: "horizontal",
                        main_align: "end",

                        Button {
                            onclick: move |_| show_confirmation.set(None),
                            theme: theme_with!(ButtonTheme {
                                background: std::borrow::Cow::Borrowed(theme.background_tertiary),
                                hover_background: std::borrow::Cow::Borrowed(theme.background_primary),
                            }),
                            label {
                                color: theme.label_secondary,
                                "取消"
                            }
                        }

                        rect {
                            width: "20"
                        }

                        FilledButton {
                            theme: theme_with!(ButtonTheme {
                                background: std::borrow::Cow::Borrowed(if task.dangerous { theme.danger } else { theme.accent }),
                                hover_background: std::borrow::Cow::Borrowed(if task.dangerous { theme.danger_hover } else { theme.accent_hover }),
                            }),
                            onclick: move |_| {
                                let task_clone = task.clone();
                                show_confirmation.set(None);
                                spawn(async move {
                                    run_clean_task(task_clone, app_state).await;
                                });
                            },
                            label {
                                color: "white",
                                "确认"
                            }
                        }
                    }
                }
            }
        }
    )
}

#[component]
fn TaskCard(
    task: CleanTask,
    show_batch_mode: bool,
    selected_tasks: HashSet<String>,
    on_toggle: EventHandler<()>,
    mut app_state: Signal<AppState>,
    mut show_confirmation: Signal<Option<CleanTask>>,
    theme: &'static AppTheme,
) -> Element {
    let is_selected = selected_tasks.contains(&task.name);
    let is_dangerous = task.dangerous;
    let actual_size = task.get_actual_size();
    let estimated_size_text = actual_size.as_deref().unwrap_or("未知");
    let icon_text = task.icon.as_deref().unwrap_or("");

    rsx!(
        rect {
            width: "100%",
            padding: "16",
            background: if is_selected && show_batch_mode { theme.accent } else { theme.background_tertiary },
            corner_radius: "12",
            direction: "horizontal",
            main_align: "space_between",
            cross_align: "center",
            onclick: move |_| {
                if show_batch_mode {
                    on_toggle.call(());
                }
            },

            rect {
                direction: "horizontal",
                cross_align: "center",

                if show_batch_mode {
                    rect {
                        width: "20",
                        height: "20",
                        corner_radius: "6",
                        background: if is_selected { theme.accent } else { theme.background_secondary },
                        main_align: "center",
                        cross_align: "center",

                        if is_selected {
                            label {
                                font_size: "14",
                                font_weight: "bold",
                                color: "white",
                                "✓"
                            }
                        }
                    }

                    rect {
                        width: "12"
                    }
                }

                // 图标区域 - Apple风格
                rect {
                    width: "48",
                    height: "48",
                    corner_radius: "10",
                    background: theme.background_secondary,
                    main_align: "center",
                    cross_align: "center",

                    label {
                        font_size: "20",
                        color: theme.label_primary,
                        "{icon_text}"
                    }
                }

                rect {
                    width: "12"
                }

                // 文本内容区域
                rect {
                    width: "calc(100% - 180)",  // 为按钮区域预留足够空间

                    label {
                        font_size: "15",
                        font_weight: "medium",
                        color: theme.label_primary,
                        "{task.name.clone()}"
                    }

                    rect {
                        height: "4"
                    }

                    label {
                        font_size: "13",
                        color: theme.label_secondary,
                        "{task.description.clone()}"
                    }

                    rect {
                        height: "6"
                    }

                    label {
                        font_size: "12",
                        color: theme.label_tertiary,
                        "预估可清理: {estimated_size_text}"
                    }
                }
            }

            // 操作按钮区域
            rect {
                width: "120",  // 固定按钮区域宽度
                direction: "horizontal",
                main_align: "end",  // 按钮靠右对齐
                cross_align: "center",

                if !show_batch_mode {
                    Button {
                        onclick: move |_| {
                            let task_clone = task.clone();
                            if task.requires_confirmation {
                                show_confirmation.set(Some(task_clone));
                            } else {
                                spawn(async move {
                                    run_clean_task(task_clone, app_state).await;
                                });
                            }
                        },
                        theme: theme_with!(ButtonTheme {
                            background: std::borrow::Cow::Borrowed(if is_dangerous { theme.danger } else { theme.accent }),
                            hover_background: std::borrow::Cow::Borrowed(if is_dangerous { theme.danger } else { theme.accent_hover }),
                        }),
                        label {
                            font_size: "14",
                            font_weight: "medium",
                            color: "white",
                            "清理"
                        }
                    }
                }
            }

        }
    )
}

async fn run_clean_task_impl(task: CleanTask) -> Result<(), String> {
    log(&format!("检查任务: {} - 命令: {}", task.name, task.command));
    
    // 检查路径是否存在（如果有路径检查）
    if let Some(path_check) = &task.path_check {
        let expanded_path = expand_environment_variables(path_check);
        let path = Path::new(&expanded_path);

        if !path.exists() {
            let msg = format!("清理路径不存在: {}\n无需清理，跳过此任务", expanded_path);
            log(&format!("路径检查失败: {}", msg));
            return Err(msg);
        }

        if path.is_dir() {
            // 检查目录是否为空
            if let Ok(entries) = fs::read_dir(path) {
                let entry_count = entries.count();
                if entry_count == 0 {
                    let msg = format!("目录为空: {}\n无需清理，跳过此任务", expanded_path);
                    log(&format!("目录为空: {}", msg));
                    return Err(msg);
                }
            }
        }
        
        log(&format!("路径检查通过: {}", expanded_path));
    }

    // 执行命令
    let expanded_command = expand_environment_variables(&task.command);

    // 预处理命令，检查权限问题
    if expanded_command.contains("rmdir") || expanded_command.contains("del") {
        // 检查是否涉及系统保护目录
        let protected_paths = [
            "C:\\Windows",
            "C:\\Program Files",
            "C:\\Program Files (x86)",
        ];

        for protected in &protected_paths {
            if expanded_command.contains(protected) && !expanded_command.contains("\\Temp\\") {
                let msg = format!(
                    "尝试清理系统保护目录: {}\n出于安全考虑，此操作被拒绝",
                    protected
                );
                log(&format!("安全拦截: {}", msg));
                return Err(msg);
            }
        }
    }
    
    log(&format!("执行命令: {}", expanded_command));

    // 使用spawn方式执行命令，避免UI阻塞和命令窗口弹出
    let result = tokio::task::spawn_blocking(move || {
        let mut cmd = if task.command.starts_with("rmdir") {
            let mut cmd = Command::new("cmd");
            cmd.args(&["/C", &expanded_command]);
            cmd
        } else {
            let mut cmd = Command::new("cmd");
            cmd.args(&["/C", &expanded_command]);
            cmd
        };

        // 隐藏窗口，防止UI卡顿
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        cmd.output()
    })
    .await;

    match result {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(())
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                let stdout_msg = String::from_utf8_lossy(&output.stdout);

                // 提供更详细的错误信息
                let detailed_error = if error_msg.contains("拒绝访问") {
                    format!("权限不足: {}\n请尝试以管理员身份运行程序", error_msg.trim())
                } else if error_msg.contains("找不到文件") {
                    format!(
                        "文件或目录不存在: {}\n可能已被其他程序清理",
                        error_msg.trim()
                    )
                } else if error_msg.contains("正在使用") {
                    format!("文件正在被使用: {}\n请关闭相关程序后重试", error_msg.trim())
                } else if !stdout_msg.is_empty() {
                    format!(
                        "执行失败: {}\n详细信息: {}",
                        error_msg.trim(),
                        stdout_msg.trim()
                    )
                } else {
                    format!("执行失败: {}", error_msg.trim())
                };

                log(&format!("命令执行失败: {} - stderr: {} - stdout: {}", detailed_error, error_msg.trim(), stdout_msg.trim()));
                Err(detailed_error)
            }
        }
        Ok(Err(e)) => {
            // 区分不同类型的执行错误
            let error_detail = if e.to_string().contains("找不到指定的文件") {
                "系统命令执行失败: 找不到指定的命令或程序"
            } else if e.to_string().contains("拒绝访问") {
                "系统命令执行失败: 权限不足，请以管理员身份运行"
            } else {
                &format!("系统命令执行错误: {}", e)
            };

            log(&format!("命令创建失败: {} - {}", error_detail, e));
            Err(error_detail.to_string())
        }
        Err(e) => {
            // tokio任务执行错误
            let msg = format!("异步执行任务失败: {}", e);
            log(&format!("tokio任务失败: {}", msg));
            Err(msg)
        }
    }
}

#[component]
fn NotificationBubble(app_state: AppState, theme: &'static AppTheme) -> Element {
    // 预计算统计消息，避免生命周期问题
    let stats_message = if let AppState::SuccessWithStats(stats) = &app_state {
        let space_freed = stats
            .total_space_freed
            .map(|bytes| format_size(bytes))
            .unwrap_or_else(|| "0 B".to_string());

        if stats.failed_tasks > 0 {
            format!(
                "清理完成！成功: {}，失败: {}，释放空间: {}",
                stats.successful_tasks, stats.failed_tasks, space_freed
            )
        } else {
            format!(
                "清理完成！成功: {}，释放空间: {}",
                stats.successful_tasks, space_freed
            )
        }
    } else {
        String::new()
    };

    let (bg_color, text_color, icon, message, font_weight, icon_bg_color, icon_color) =
        match &app_state {
            AppState::Idle => (
                theme.background_tertiary,
                theme.label_secondary,
                "",
                "就绪",
                "normal",
                theme.background_primary,
                theme.label_secondary,
            ),
            AppState::Running(msg) => (
                theme.accent,
                "white",
                "⟳",
                msg.as_str(),
                "medium",
                "rgb(255, 255, 255)",
                theme.accent,
            ),
            AppState::Success => (
                "rgb(34, 197, 94)",
                "white",
                "✓",
                "清理完成！",
                "medium",
                "rgb(255, 255, 255)",
                "rgb(34, 197, 94)",
            ),
            AppState::SuccessWithStats(_) => (
                "rgb(34, 197, 94)",
                "white",
                "✓",
                stats_message.as_str(),
                "medium",
                "rgb(255, 255, 255)",
                "rgb(34, 197, 94)",
            ),
            AppState::Error(msg) => (
                "rgb(239, 68, 68)",
                "white",
                "✗",
                msg.as_str(),
                "medium",
                "rgb(255, 255, 255)",
                "rgb(239, 68, 68)",
            ),
        };

    rsx!(
        rect {
            width: "100%",
            padding: "16 20",
            background: bg_color,
            corner_radius: "12",
            margin: "16 0 0 0",
            direction: "horizontal",
            cross_align: "center",

            // 图标区域 - 增强对比度
            if !icon.is_empty() {
                rect {
                    width: "28",
                    height: "28",
                    corner_radius: "14",
                    background: icon_bg_color,
                    main_align: "center",
                    cross_align: "center",
                    margin: "0 12 0 0",
                    border: "2 solid {text_color}",

                    label {
                        font_size: "16",
                        font_weight: "bold",
                        color: icon_color,
                        "{icon}"
                    }
                }
            }

            // 文本内容
            label {
                font_size: "15",
                font_weight: font_weight,
                color: text_color,
                "{message}"
            }

            // 运行状态时的加载指示器 - 移除重复图标
            if matches!(app_state, AppState::Running(_)) && icon.is_empty() {
                label {
                    font_size: "16",
                    margin: "0 0 0 auto",
                    color: text_color,
                    "⟳"
                }
            }
            
        }
    )
}

async fn run_clean_task(task: CleanTask, mut app_state: Signal<AppState>) {
    log(&format!("开始执行任务: {}", task.name));
    app_state.set(AppState::Running(format!("正在执行: {}", task.name)));

    match run_clean_task_impl(task.clone()).await {
        Ok(_) => {
            log(&format!("任务成功: {}", task.name));
            app_state.set(AppState::Success);
        }
        Err(e) => {
            log(&format!("任务失败: {} - {}", task.name, e));
            app_state.set(AppState::Error(e));
        }
    }
}