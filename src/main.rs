#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

extern crate reqwest;
use eframe::{egui, egui::Visuals};
use std::{fs, str, io::Write, path::Path};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Error's Adventure Downloader",
        options,
        Box::new(|_cc| {Box::<App>::default()}),
    )
}

#[derive(PartialEq)]
enum Input {ID, URL}

struct App {
    user_input: String,
    error_text: String,
    input_style: Input,
    file_path: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            user_input: "".to_owned(),
            error_text: "".to_owned(),
            input_style: Input::URL,
            file_path: "".to_owned(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_visuals(Visuals::dark());
            ctx.set_pixels_per_point(1.33);
            ui.heading("Error's Spore Adventure Downloader\n");
            
            ui.horizontal(|ui| {
                ui.label("Input type: ");
                ui.radio_value(&mut self.input_style, Input::URL, "Url");
                ui.radio_value(&mut self.input_style, Input::ID, "ID");
            });
            ui.horizontal(|ui| {
                let name_label = ui.label("Adventure: ");
                ui.text_edit_singleline(&mut self.user_input).labelled_by(name_label.id);
            });
            ui.horizontal(|ui| {
                let name_label = ui.label("File path: ");
                ui.text_edit_singleline(&mut self.file_path).labelled_by(name_label.id);
            });
            if ui.button("Download Adventure").clicked() {
                let res = match self.input_style {
                    Input::ID => get_adventure(IDPackage {valid: true, id: self.user_input.clone(), error_message: "".to_string()}, clean_file_path(&self.file_path).as_str()),
                    Input::URL => get_adventure(pull_id_from_url(&self.user_input), clean_file_path(&self.file_path).as_str()),
                };
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
}

struct IDPackage {
    valid: bool,
    id: String,
    error_message: String,
}

fn clean_file_path(input_path: &str) -> String {
    let edit_str = input_path;

    if edit_str.len() == 0 {return edit_str.to_owned();}

    if edit_str.chars().last().unwrap() == '/' || edit_str.chars().last().unwrap() == '\\' {
        return edit_str.to_owned();
    }

    let end_appended = format!("{edit_str}//");
    
    return end_appended;
}

fn pull_id_from_url(url: &str) -> IDPackage {
    let id_start = url.find("sast-");
    if let Some(id_start) = id_start {
        let end = id_start + 17;
        let id = &url[id_start+5..end];
        if !id.chars().all(char::is_numeric) {
            return IDPackage {valid: false, id: "".to_string(), error_message: format!("Error: URL contains an invalid ID: {id}")}
        }
        return IDPackage {valid: true, id: id.to_string(), error_message: "".to_string()}
    };
    return IDPackage {valid: false, id: "".to_string(), error_message: format!("Error: URL is not in a valid format")}
}

fn check_for_file(path: &str) {let _ = fs::create_dir_all(path);}

fn get_adventure_name(id: &str) -> String {
    let url = format!("http://www.spore.com/rest/asset/{id}");
    let buff = req_data_from_server(&url);
    let data = String::from_utf8(buff.clone()).unwrap();
    let start = data.find("<name>");
    let end = data.find("</name>");
    
    if let Some(start) = start {
        let end_pos = end.unwrap();
        let adv_name = &data[start+6..end_pos];
        return format!("{adv_name}");
    };
    return format!("Adventure-{id}")
}

/// Downloads an adventure at a given ID.
/// Saves the adventure to a png, then calls the function to get the creations required for it.
fn get_adventure(package: IDPackage, path: &str) -> AdventureGetResult
{
    let input_id = match package.valid {
        true => package.id,
        false => return AdventureGetResult {
            success: false,
            error: package.error_message,
        }
    };
    if !input_id.chars().all(char::is_numeric) {
        return AdventureGetResult {
            success: false,
            error: "Given ID contains non-digit chararcters.".to_string(),
        }
    }

    if input_id.len() < 12 {
        return AdventureGetResult {
            success: false,
            error: format!("Given ID {input_id} is too short | Creation ID's are 12 digits long"),
        }
    }

    if input_id.len() > 12 {
        return AdventureGetResult {
            success: false,
            error: "Given ID is too long | Creation ID's are 12 digits long".to_string(),
        }
    }

    if !Path::exists(Path::new(path)) && path.len() > 0 {
        return AdventureGetResult {
            success: false,
            error: format!("Error: File path {path} does not exist!"),
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
        }
    }
    let adv_name = get_adventure_name(&input_id);

    let buffer = req_data_from_server(&url);

    check_for_file(&format!("{path}{adv_name}//"));
    let file_name = format!("{path}{adv_name}//{input_id}.png");
    let file_path = Path::new(&file_name);
    let mut file = std::fs::File::create(file_path).unwrap();
    file.write_all(&buffer).unwrap();

    let xml_data = str::from_utf8(&xml_result).unwrap();

    get_adventure_creations(xml_data, &format!("{path}{adv_name}//"));

    return AdventureGetResult {
        success: true,
        error: "Successfully downloaded!".to_string(),
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
fn get_adventure_creations(xml_data: &str, file_path: &str) {
    let pos = xml_data.find("<assets><asset>").unwrap() + 15; // 15 = length of <assets><asset>
    let end = xml_data.find("<cScenarioResource>").unwrap() - 17;
    
    let ids = xml_data[pos..end].split("</asset><asset>");
    for id in ids {
        let id_slice_1 = &id[0..3];
        let id_slice_2 = &id[3..6];
        let id_slice_3 = &id[6..9];

        let file_name = format!("{file_path}{id}.png");
        let file_path = Path::new(&file_name);

        let url = format!("http://static.spore.com/static/thumb/{id_slice_1}/{id_slice_2}/{id_slice_3}/{id}.png");

        let result = req_data_from_server(&url);

        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(&result).unwrap();
    }
}