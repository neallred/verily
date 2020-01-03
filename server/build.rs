use std::{
    error::Error, fs::{self, File}, io::Write, path::Path,
};

const SOURCE_DIR: &str = "../client/web/dist";

fn print_index(f: &std::fs::DirEntry) -> String {
    let file_name = f.file_name();
    let bob = file_name.to_str().unwrap();
    if bob == "index.html" {
        String::from("")
    } else {
        bob.to_string()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = "../server/src";
    let dest_path = Path::new(&out_dir).join("main.rs");
    {
        match std::fs::remove_file(&dest_path) {
            Ok(_) => {
                println!("removed existing main.rs server file");
            }
            Err(_err) => {
                println!("no main.rs server file to remove");
            }
        };
    }

    let mut main_rs = File::create(&dest_path)?;

    let header = r##"#[macro_use]
extern crate rouille;

use rouille::Response;

fn main() {
    rouille::start_server("0.0.0.0:5000", move |request| {
        let response = router!(request,"##;

    let footer = r##"            _ => {
                Response::empty_404()
            }
        );

        response
    });
}
"##;

    writeln!(&mut main_rs, "{}", header).expect("Expected to be able to write to main.rs");

    for f in fs::read_dir(SOURCE_DIR).expect("expected to find client files for bundling") {
        let f = f?;
 
        if !f.file_type()?.is_file() {
            continue;
        }
 
        if f.file_name().to_str().unwrap().contains("wasm") {
            writeln!(
                &mut main_rs,
                r#"            (GET) ["/{short_name}"] => {{
                Response::from_data("application/wasm", include_bytes!("../{name}").to_vec())
            }},"#,
                name = f.path().display(),
                short_name = f.file_name().to_str().unwrap(),
            )?;
        } else {
            writeln!(
                &mut main_rs,
                r#"            (GET) ["/{short_name}"] => {{
                Response::{content_type}(include_str!("../{name}"))
            }},"#,
                name = f.path().display(),
                short_name = print_index(&f),
                content_type = if f.file_name().to_str().unwrap().contains("html") { "html" } else { "text" },
            )?;
        }
    }
    writeln!(&mut main_rs, "{}", footer)?;
    Ok(())
}
