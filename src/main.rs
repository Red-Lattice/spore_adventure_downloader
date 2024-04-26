#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

extern crate reqwest;
use eframe::{egui, egui::Visuals};
use std::{str, io::Write, path::Path};


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Error's Adventure Downloader",
        options,
        Box::new(|_cc| {Box::<App>::default()}),
    )
}

struct App {
    id: String,
    error_text: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            id: "".to_owned(),
            error_text: "".to_owned(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_visuals(Visuals::dark());
            ui.heading("Error's Spore Adventure Downloader\n");
            ui.horizontal(|ui| {
                let name_label = ui.label("Adventure ID: ");
                ui.text_edit_singleline(&mut self.id).labelled_by(name_label.id);
            });
            if ui.button("Download Adventure").clicked() {
                let res = get_adventure(&self.id);
                self.error_text = res.error;
            }
            ui.label(format!("{}",self.error_text));
        });
    }
}

#[allow(dead_code)] // The bool and error code are currently unused but may be used at some point so I don't want to get rid of them.
struct AdventureGetResult {
    success: bool,
    error: String,
    error_code: u8,
}

/// Downloads an adventure at a given ID.
/// Saves the adventure to a png, then calls the function to get the creations required for it.
fn get_adventure(input_id: &str) -> AdventureGetResult
{
    if !input_id.chars().all(char::is_numeric) {
        return AdventureGetResult {
            success: false,
            error: "Given ID contains non-digit chararcters.".to_string(),
            error_code: 2,
        }
    }

    if input_id.len() < 12 {
        return AdventureGetResult {
            success: false,
            error: "Given ID is too short | Creation ID's are 12 digits long".to_string(),
            error_code: 3,
        }
    }

    if input_id.len() > 12 {
        return AdventureGetResult {
            success: false,
            error: "Given ID is too long | Creation ID's are 12 digits long".to_string(),
            error_code: 4,
        }
    }

    let id_slice_1 = &input_id[0..3];
    let id_slice_2 = &input_id[3..6];
    let id_slice_3 = &input_id[6..9];

    let url = format!("http://static.spore.com/static/thumb/{id_slice_1}/{id_slice_2}/{id_slice_3}/{input_id}.png");
    let xml_url = format!("http://static.spore.com/static/model/{id_slice_1}/{id_slice_2}/{id_slice_3}/{input_id}.xml");

    let xml_result = req_data_from_server(&xml_url);

    if !String::from_utf8(xml_result.clone()).unwrap().contains("Scenario") {
        return AdventureGetResult {
            success: false,
            error: "ID provided is not the ID of an adventure".to_string(),
            error_code: 1,
        }
    }

    let buffer = req_data_from_server(&url);

    let file_name = format!("{input_id}.png");
    let file_path = Path::new(&file_name);
    let mut file = std::fs::File::create(file_path).unwrap();
    file.write_all(&buffer).unwrap();

    let xml_data = str::from_utf8(&xml_result).unwrap();

    get_adventure_creations(xml_data);

    return AdventureGetResult {
        success: true,
        error: "Successfully downloaded!".to_string(),
        error_code: 0,
    }
}

/// Sends a get request to the given URL. If an error is returned by the request it just repeatedly requests until it works again.
fn req_data_from_server(url: &str) -> Vec<u8> {
    let mut buffer = vec![];
    let result = reqwest::blocking::get(url);
            
    let mut result = match result {
        Ok(result) => result,
        Err(_) => {
            loop {
                let result = reqwest::blocking::get(url);
                if let Ok(result) = result {break result}
            }
        }
    };
    let _ = result.copy_to(&mut buffer);
    return buffer
}

/// Gets all of the creations required for an adventure.
/// Takes in the adventure's xml data as a string as a parameter
fn get_adventure_creations(xml_data: &str) {
    let pos = xml_data.find("<assets><asset>").unwrap() + 15; // 15 = length of <assets><asset>
    let end = xml_data.find("<cScenarioResource>").unwrap() - 17;
    
    let ids = xml_data[pos..end].split("</asset><asset>");
    for id in ids {
        let id_slice_1 = &id[0..3];
        let id_slice_2 = &id[3..6];
        let id_slice_3 = &id[6..9];

        let file_name = format!("{id}.png");
        let file_path = Path::new(&file_name);

        let url = format!("http://static.spore.com/static/thumb/{id_slice_1}/{id_slice_2}/{id_slice_3}/{id}.png");

        let result = req_data_from_server(&url);

        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(&result).unwrap();
    }
}