use crate::config::GLOBAL_CONFIG;
use egui::{CentralPanel, Context, ScrollArea, SidePanel};
use file_browser::{FileBrowserSortingMethod, FileBrowserState};
use std::fmt::Display;
use std::path::PathBuf;
use strum::{EnumIter, IntoEnumIterator};
mod file_browser;

pub enum UiOutput {
    OpenGame { path: PathBuf },
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, EnumIter)]
pub enum MenuItem {
    #[default]
    Main,
    FileBrowser,
    Options,
    Database,
}

impl Display for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MenuItem::Main => "Main",
                MenuItem::FileBrowser => "File Browser",
                MenuItem::Options => "Options",
                MenuItem::Database => "Database",
            }
        )
    }
}

#[derive(Default, Clone, Debug)]
pub struct MenuState {
    open_menu_item: MenuItem,
    file_browser_state: FileBrowserState,
    pub egui_context: egui::Context,
    pub active: bool,
}

impl MenuState {
    /// TODO: barely does anything
    pub fn run_menu(&mut self, ctx: &Context) -> Option<UiOutput> {
        let mut output = None;

        SidePanel::left("options_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        for item in MenuItem::iter() {
                            if ui.button(format!("{}", item)).clicked() {
                                self.open_menu_item = item;
                            }
                        }
                    })
                })
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::LEFT),
                |ui| match self.open_menu_item {
                    MenuItem::Main => if ui.button("Resume").clicked() {},
                    MenuItem::FileBrowser => {
                        let mut new_dir = None;

                        ui.horizontal(|ui| {
                            // Iter over the path segments
                            for (index, path_segment) in
                                self.file_browser_state.directory().iter().enumerate()
                            {
                                if index != 0 {
                                    ui.label("/");
                                }

                                if ui.button(path_segment.to_str().unwrap()).clicked() {
                                    new_dir = Some(PathBuf::from_iter(
                                        self.file_browser_state.directory().iter().take(index + 1),
                                    ));
                                }
                            }

                            ui.separator();

                            if ui.button("🔄").clicked() {
                                self.file_browser_state.refresh_directory();
                            }

                            let mut selected_sorting = self.file_browser_state.get_sorting_method();
                            egui::ComboBox::from_label("Sorting")
                                .selected_text(format!("{:?}", selected_sorting))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut selected_sorting,
                                        FileBrowserSortingMethod::Name,
                                        "Name",
                                    );
                                    ui.selectable_value(
                                        &mut selected_sorting,
                                        FileBrowserSortingMethod::Date,
                                        "Date",
                                    );
                                });
                            self.file_browser_state.set_sorting_method(selected_sorting);
                        });

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for file_entry in self.file_browser_state.directory_contents() {
                                let file_name = file_entry.file_name().unwrap().to_str().unwrap();

                                if ui.button(file_name).clicked() {
                                    if file_entry.is_dir() {
                                        new_dir = Some(file_entry.to_path_buf());
                                    }

                                    if file_entry.is_file() {
                                        output = Some(UiOutput::OpenGame {
                                            path: file_entry.to_path_buf(),
                                        });
                                    }
                                }
                            }
                        });

                        if let Some(new_dir) = new_dir {
                            tracing::trace!("Changing directory to {:?}", new_dir);
                            self.file_browser_state.change_directory(new_dir);
                        }
                    }
                    MenuItem::Options => {
                        let mut global_config_guard = GLOBAL_CONFIG.write().unwrap();

                        ui.horizontal(|ui| {
                            if ui.button("Save Config").clicked() {
                                global_config_guard.save().unwrap();
                            }
                        });

                        ui.checkbox(
                            &mut global_config_guard.hardware_acceleration,
                            "Hardware Acceleration",
                        );

                        ui.checkbox(&mut global_config_guard.vsync, "VSync");
                    }
                    MenuItem::Database => {}
                },
            );
        });

        output
    }
}
