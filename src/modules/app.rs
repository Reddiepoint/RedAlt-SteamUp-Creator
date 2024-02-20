use crate::modules::create_update::CreateUpdateUI;
use crate::modules::settings::SettingsUI;
use eframe::egui::{CentralPanel, Context, menu, TopBottomPanel, Ui};
use eframe::{App, Frame};
use crate::modules::help::HelpUI;

#[derive(Default, PartialEq)]
pub enum TabBar {
    #[default]
    CreateUpdate,
    Settings,
}

#[derive(Default)]
pub struct RedAltSteamUpCreator {
    pub tab_bar: TabBar,
    pub create_update_ui: CreateUpdateUI,
    pub settings_ui: SettingsUI,
    pub help_ui: HelpUI,
}

impl App for RedAltSteamUpCreator {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.settings_ui.read_settings();
        self.display_top_bar(ctx);
        self.display_central_panel(ctx);

    }
}

impl RedAltSteamUpCreator {
    fn display_top_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("Tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Add tabs for each function
                ui.selectable_value(&mut self.tab_bar, TabBar::CreateUpdate, "Create Update");
                ui.selectable_value(&mut self.tab_bar, TabBar::Settings, "Settings");

                // Add menu bar
                self.display_menu_bar(ctx, ui);
                // Display other windows
                self.help_ui.show_help_window(ctx);
                self.help_ui.show_update_window(ctx);
            });
        });
    }

    fn display_central_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| match &self.tab_bar {
            TabBar::CreateUpdate => CreateUpdateUI::display(
                ctx,
                ui,
                &mut self.create_update_ui,
                &mut self.settings_ui.depot_downloader_settings,
                &mut self.settings_ui.compression_settings,
                &mut self.tab_bar,
            ),
            TabBar::Settings => SettingsUI::display(ctx, ui, &mut self.settings_ui),
        });
    }

    fn display_menu_bar(&mut self, ctx: &Context, ui: &mut Ui) {
        menu::bar(ui, |ui| {
            ui.menu_button("Help", |ui| {
                if ui.button("Show help").clicked() {
                    self.help_ui.show_help = true;
                    ui.close_menu();
                };

                ui.separator();

                if ui.button("Check for updates").clicked() {
                    self.help_ui.show_update = true;
                    ui.close_menu();
                }


                ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            })
        });
    }
}
