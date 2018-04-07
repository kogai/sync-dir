use std::fs::read_dir;

fn main() {
    let my_dir = read_dir("/home/kogai/Downloads");
    match my_dir {
        Ok(_) => {
            println!("CAN READ");
        }
        Err(_) => {
            println!("CAN'T READ");
        }
    }
}
