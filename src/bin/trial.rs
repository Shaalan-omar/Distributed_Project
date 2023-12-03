use image::{DynamicImage, GenericImageView, Rgba};
use show_image::{create_window, ImageInfo, ImageView};
use std::fs;
use std::thread;
use std::time::Duration;

fn delete_image(image_path: &str) {
    fs::remove_file(image_path);
}

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace "path/to/your/image.jpg" with the actual path to your image file
    let image_path = "mypic.png";

    // Load the image file
    let img = image::open(image_path)?;

    // Convert the image to RGBA format
    let rgba_image = img.to_rgba8();

    // Get image dimensions
    let (width, height) = rgba_image.dimensions();

    // Convert the image to a flat vector of u8 pixel data
    let pixel_data: Vec<u8> = rgba_image.into_raw();

    // Create an ImageView with the loaded image data
    let image = ImageView::new(ImageInfo::rgba8(width, height), &pixel_data);

    // Create a window with default options and display the image
    let window = create_window("image", Default::default())?;
    window.set_image("image-001", image)?;

    thread::sleep(Duration::from_secs(4));

    delete_image(image_path);
    Ok(())
}
