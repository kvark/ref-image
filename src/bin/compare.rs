extern crate image;
extern crate ref_image;

use image::{DynamicImage, GenericImage, ImageFormat};
use ref_image::{ReftestImage, ReftestImageComparison};
use std::{cmp, env};
use std::fs::File;
use std::io::{BufReader, Write};


fn load_reftest(path: &str) -> DynamicImage {
    let format = if path.ends_with(".png") | path.ends_with(".PNG") {
        ImageFormat::PNG
    } else {
        panic!("Unsupported extension of {}", path);
    };
    let file = File::open(path)
        .expect(&format!("Unable to load {}", path));
    image::load(BufReader::new(file), format)
        .expect(&format!("Unable to parse {}", path))
}

fn main() {
    println!("Standalone image comparison tool");
    let (path_test, path_ref) = {
        let mut args = env::args();
        let _ = args.next().unwrap();
        let pt = args.next().unwrap();
        let pr = args.next().unwrap();
        (pt, pr)
    };

    println!("Loading images...");
    //TODO: more image formats
    let mut img_test = load_reftest(&path_test);
    let mut img_ref = load_reftest(&path_ref);

    let (rti_test, rti_ref) = {
        let (w0, h0) = img_test.dimensions();
        let (w1, h1) = img_ref.dimensions();
        let (w, h) = (cmp::min(w0, w1), cmp::min(h0, h1));
        if (w0, h0) != (w, h) {
            println!("\tCropping {} to {}x{}", path_test, w, h);
            img_test = img_test.crop(0, 0, w, h);
        }
        if (w1, h1) != (w, h) {
            println!("\tCropping {} to {}x{}", path_ref, w, h);
            img_ref = img_ref.crop(0, 0, w, h);
        }
        (ReftestImage::from(img_test), ReftestImage::from(img_ref))
    };

    match rti_test.compare(&rti_ref) {
        ReftestImageComparison::Equal => {
            println!("REFTEST TEST-PASS");
        }
        ReftestImageComparison::NotEqual {
            max_difference,
            count_different,
        } => {
            let log_name = "reftest.log";
            let mut output = File::create(log_name).unwrap();
            let name = format!("{} == {}", path_test, path_ref);
            let _ = writeln!(output,
                "{} | {} | {}: {}, {}: {}",
                "REFTEST TEST-UNEXPECTED-FAIL",
                name,
                "image comparison, max difference",
                max_difference,
                "number of differing pixels",
                count_different
            );
            let _ = writeln!(output, "REFTEST IMAGE 1 (TEST): {}", rti_test.into_data_uri());
            let _ = writeln!(output, "REFTEST IMAGE 2 (REFERENCE): {}", rti_ref.into_data_uri());
            let _ = writeln!(output, "REFTEST TEST-END | {}", name);
            println!("Difference log written to '{}'", log_name);
        }
    }
}
