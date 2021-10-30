use std::{env, fs};

fn image_transfer(mut argv: env::Args) -> Result<i32, String> {
    let source_dir = argv.nth(1).ok_or_else(|| "What is the source".to_owned())?;
    let target_dir = argv.next().ok_or_else(|| "What is the target".to_owned())?;



    let mut count = 0;
    if let Ok(entry) = fs::read_dir(&source_dir).map_err(|_| source_dir + " is not exists") {
        count = 1;
        if fs::create_dir(&target_dir).is_err() {
            return Err(target_dir + " create target dir failed");
        }

        for img_dir in entry
            .filter_map(Result::ok)
            .filter(|dir| dir.file_type().map_or(false, |ty| ty.is_dir()))
            .filter_map(|dir| fs::read_dir(dir.path()).ok())
        {
            let mut paths: Vec<_> = img_dir
                .filter_map(Result::ok)
                .filter(|r| {
                    let path = r.path();
                    let ext = path.extension().unwrap_or_else(|| "".as_ref());
                    ext == "jpg" || ext == "jpeg" || ext == "png"
                })
                .collect();
            paths.sort_by_cached_key(|dir| {
                let file_name = dir
                    .file_name()
                    .into_string()
                    .unwrap_or_else(|_| String::new());
                if let Some((name, _)) = file_name.split_once(".") {
                    name.parse::<i32>().unwrap_or(1)
                } else {
                    1
                }
            });

            for f in paths {
                let path = f.path();
                let ext = path
                    .extension()
                    .unwrap_or_else(|| "".as_ref())
                    .to_str()
                    .unwrap_or("");

                let str_name = format!(
                    "{}{}{}.{}",
                    &target_dir,
                    std::path::MAIN_SEPARATOR,
                    count,
                    &ext
                );

                match fs::copy(path, str_name) {
                    Ok(_) => count += 1,
                    Err(e) => println!("{:?}", e),
                }
            }
        }
    }

    Ok(count)
}

fn main() {
    match image_transfer(env::args()) {
        Ok(n) => println!("suc copy  {} files", n),
        Err(err) => println!("Error: {}", err),
    }
}
