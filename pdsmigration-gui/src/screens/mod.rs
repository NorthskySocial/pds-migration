use crate::ScreenType;
use egui::Ui;

pub mod advanced_home;
pub mod basic_home;
pub mod create_account;
pub mod deactivate_and_activate;
pub mod does_pds_exist;
pub mod export_blobs;
pub mod export_repo;
pub mod import_blobs;
pub mod import_repo;
pub mod migrate_plc;
pub mod migrate_preferences;
pub mod new_login;
pub mod old_login;
pub mod success;

pub trait Screen {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context);
    fn name(&self) -> ScreenType;
}
