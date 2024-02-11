use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use eframe::{App, Frame};
use eframe::egui::{CentralPanel, Context, TopBottomPanel};
use egui_file::FileDialog;
use crate::modules::create_update::CreateUpdateUI;
use crate::modules::settings::SettingsUI;

#[derive(Default, PartialEq)]
pub enum TabBar {
    #[default]
    CreateUpdate,
    Settings,
}

#[derive(Default)]
pub struct RedAltSteamUpdateCreator {
    pub tab_bar: TabBar,
    pub create_update_ui: CreateUpdateUI,
    pub settings_ui: SettingsUI,
}

impl App for RedAltSteamUpdateCreator {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.settings_ui.read_settings();
        RedAltSteamUpdateCreator::display_top_bar(self, ctx);
        RedAltSteamUpdateCreator::display_central_panel(self, ctx);
    }
}

impl RedAltSteamUpdateCreator {
    fn display_top_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("Tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Add tabs for each function
                ui.selectable_value(&mut self.tab_bar, TabBar::CreateUpdate, "Create Update");
                ui.selectable_value(&mut self.tab_bar, TabBar::Settings, "Settings");
            });
        });
    }

    fn display_central_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            match &self.tab_bar {
                TabBar::CreateUpdate => CreateUpdateUI::display(ctx, ui, &mut self.create_update_ui,
                                                                &mut self.settings_ui.depot_downloader_settings,
                                                                &mut self.tab_bar),
                TabBar::Settings => SettingsUI::display(ctx, ui, &mut self.settings_ui),
            }
        });
    }
}