use qr_protocol::export::{export_registry_dart, export_registry_json};
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [] => {
            println!("{}", export_registry_json()?);
        }
        [flag, output] if flag == "--dart" => {
            let path = Path::new(output);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, export_registry_dart()?)?;
        }
        _ => {
            eprintln!("用法: export_registry [--dart <输出文件>]");
            std::process::exit(2);
        }
    }
    Ok(())
}
