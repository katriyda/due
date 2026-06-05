use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let assets_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");
    fs::create_dir_all(&assets_dir).unwrap();

    generate_icon(&assets_dir);

    println!("cargo:rerun-if-changed=ui/main.slint");
    slint_build::compile("ui/main.slint").unwrap();
}

fn generate_icon(assets_dir: &PathBuf) {
    let size = 32u32;
    let mut img = image::RgbaImage::new(size, size);

    for y in 0..size {
        for x in 0..size {
            let (r, g, b, a) = pixel_color(x, y, size);
            img.put_pixel(x, y, image::Rgba([r, g, b, a]));
        }
    }

    // 写入 RGBA 原始数据（供 tray_icon 使用）
    let rgba_path = assets_dir.join("icon.rgba");
    fs::write(&rgba_path, img.as_raw()).unwrap();

    // 写入 PNG（供 Slint 窗口使用）
    let png_path = assets_dir.join("icon.png");
    img.save(&png_path).unwrap();

    println!("cargo:rerun-if-changed={}", rgba_path.display());
    println!("cargo:rerun-if-changed={}", png_path.display());
}

fn pixel_color(x: u32, y: u32, size: u32) -> (u8, u8, u8, u8) {
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let r = size as f32 / 2.0 - 1.0;
    let dx = x as f32 - cx;
    let dy = y as f32 - cy;

    if dx * dx + dy * dy > r * r {
        return (0, 0, 0, 0); // 透明
    }

    // "d" 字母像素（6x8 网格，居中）
    let letter = [
        [0, 1, 1, 0, 0, 0],
        [1, 0, 0, 1, 0, 0],
        [1, 0, 0, 1, 0, 0],
        [1, 0, 0, 1, 0, 0],
        [1, 0, 0, 1, 0, 0],
        [1, 0, 0, 1, 0, 0],
        [1, 0, 0, 1, 0, 0],
        [0, 1, 1, 1, 0, 0],
    ];

    let grid_x = (x * 6) / size;
    let grid_y = (y * 8) / size;

    if grid_x < 6
        && grid_y < 8
        && letter[grid_y as usize][grid_x as usize] == 1
    {
        (255, 255, 255, 255) // 白色字母
    } else {
        (0, 120, 212, 255) // Fluent 蓝色背景
    }
}
