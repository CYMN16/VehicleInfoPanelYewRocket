use std::fs;

use qrcode::QrCode;
use image::Luma;
const BASE_URL: &str = "local.cymn.com:8080/";

pub fn create_qr_for_id(id: i32) -> String {
    let url = BASE_URL.to_string() + &id.to_string();
    let qr = QrCode::new(url.clone()).unwrap();
    let img = qr.render::<Luma<u8>>().build();
    let path = format!("./results/qr_codes/id_{}.png",id);
    fs::create_dir_all("./results/qr_codes").unwrap();
    img.save(path.clone()).unwrap();
    // let char_img= qr.render::<char>().build();    
    // println!("data: {url}\nqr:\n{}", char_img);
    path
}   