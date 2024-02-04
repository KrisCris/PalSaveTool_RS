use palsavetool_rs::PalSave;
use std::{env, fs, io};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <compress|decompress> <input.sav> <output.sav>",
            args[0]
        );
        std::process::exit(1);
    }

    let operation = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    match operation.as_str() {
        "compress" => {
            let palsav = PalSave::from_decompressed_file(input_path, '2')?;
            palsav.to_file(output_path)?;
        }
        "decompress" => {
            let palsav = PalSave::from_file(input_path)?;
            let pal_save_data = palsav.get_decompressed_body()?;
            fs::write(output_path, pal_save_data)?;
            println!("Decompressed {} to {}", input_path, output_path);
        }
        _ => {
            eprintln!("Invalid operation: {}", operation);
            eprintln!(
                "Usage: {} <compress|decompress> <input.sav> <output.sav>",
                args[0]
            );
            std::process::exit(1);
        }
    }

    Ok(())
}
