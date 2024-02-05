use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use eframe::{App, Frame};
use eframe::egui::{CentralPanel, Context, TopBottomPanel};
use egui_file::FileDialog;
use crate::modules::create_update::CreateUpdateUI;

#[derive(Default, PartialEq)]
pub enum TabBar {
    #[default]
    CreateUpdate,
}
#[derive(Default)]
pub struct RedAltSteamUpdateCreator {
    tab_bar: TabBar,
    create_update_ui: CreateUpdateUI,
}

impl App for RedAltSteamUpdateCreator {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
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

            });
        });
    }

    fn display_central_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            match &self.tab_bar {
                TabBar::CreateUpdate => CreateUpdateUI::display(ctx, ui, &mut self.create_update_ui),
            }
        });
    }
}