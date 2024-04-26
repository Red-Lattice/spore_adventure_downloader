#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
extern crate reqwest;
use eframe::egui;
use eframe::egui::Visuals;
use std::io;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str;


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Error's Adventure Downloader",
        options,
        Box::new(|cc| {

            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    id: String,
    age: u32,
    error_text: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            id: "".to_owned(),
            age: 42,
            error_text: "".to_owned(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_visuals(Visuals::dark());
            ui.heading("Error's Spore Adventure Downloader");
            ui.horizontal(|ui| {
                let name_label = ui.label("Adventure ID: ");
                ui.text_edit_singleline(&mut self.id)
                    .labelled_by(name_label.id);
            });
            //ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Download Adventure").clicked() {
                let res = get_adventure(&self.id);
                self.error_text = res.error;
            }
            ui.label(format!("{}",self.error_text));
        });
    }
}

struct AdventureGetResult {
    success: bool,
    error: String,
    error_code: u32,
}

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

    let file_name = format!("{input_id}.png");
    let file_path = Path::new(&file_name);
    //This is the format the URL's follow: http://static.spore.com/static/thumb/123/456/789/123456789123.png

    let url = format!("http://static.spore.com/static/thumb/{id_slice_1}/{id_slice_2}/{id_slice_3}/{input_id}.png");
    let xml_url = format!("http://static.spore.com/static/model/{id_slice_1}/{id_slice_2}/{id_slice_3}/{input_id}.xml");

    let mut buffer = vec![];
    let mut xml_buffer = vec![];

    let xmlresult = reqwest::blocking::get(xml_url.clone());
    let mut xml_result = match xmlresult {
        Ok(xmlresult) => xmlresult,
        Err(_) => {
            loop {
                let xmlresult = reqwest::blocking::get(xml_url.clone());
                if let Ok(xmlresult) = xmlresult {break xmlresult}
            }
        }
    };
    let _ = xml_result.copy_to(&mut xml_buffer);

    if !String::from_utf8(xml_buffer.clone()).unwrap().contains("Scenario") {
        return AdventureGetResult {
            success: false,
            error: "ID provided is not the ID of an adventure".to_string(),
            error_code: 1,
        }
    }

    let result = reqwest::blocking::get(url.clone());
            
    let mut result = match result {
        Ok(result) => result,
        Err(_) => {
            loop {
                let result = reqwest::blocking::get(url.clone());
                if let Ok(result) = result {break result}
            }
        }
    };
    let _ = result.copy_to(&mut buffer);

    let mut file = std::fs::File::create(file_path).unwrap();
    file.write_all(&buffer).unwrap();

    let xml_data = match str::from_utf8(&xml_buffer) {
        Ok(v) => v,
        Err(e) => "XML ERROR",
    };
    if xml_data == "XML ERROR" {
        return AdventureGetResult {
            success: false,
            error: "Something has gone horribly wrong with the XML data".to_string(),
            error_code: 5,
        }
    }

    get_adventure_creations(xml_data);

    return AdventureGetResult {
        success: true,
        error: "Success".to_string(),
        error_code: 0,
    }
}

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

        let result = reqwest::blocking::get(url.clone());
        
        let mut result = match result {
            Ok(result) => result,
            Err(_) => {
                loop {
                    let result = reqwest::blocking::get(url.clone());
                    if let Ok(result) = result {break result}
                }
            }
        };
        let mut buffer = vec![];
        let _ = result.copy_to(&mut buffer);

        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(&buffer).unwrap();
    }
}

fn clean_id(input: u64) -> String
{
    if input > 99
    {
        return input.to_string();
    }
    if input > 9
    {
        return "0".to_owned() + &input.to_string();
    }
    return "00".to_owned() + &input.to_string();
}