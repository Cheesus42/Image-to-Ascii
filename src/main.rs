use std::env;
use std::fs::File;
use std::io::BufReader;

use ansi_term;
use png::{self, OutputInfo};
use termsize;

//const BRIGHTNESS: &[u8; 70] = b"$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ";
const BRIGHTNESS: &[u8] =
    b" .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";

fn extract_png(f: File) -> (Vec<[u8; 4]>, OutputInfo) {
    let mut reader = png::Decoder::new(BufReader::new(f)).read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];

    let pixels: Vec<[u8; 4]> = match info.color_type {
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .map(|byte| [byte[0], byte[1], byte[2], 255])
            .collect(),
        png::ColorType::Rgba => bytes
            .chunks_exact(4)
            .map(|byte| [byte[0], byte[1], byte[2], byte[3]])
            .collect(),
        png::ColorType::Grayscale => bytes.iter().map(|&byte| [byte, byte, byte, 255]).collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .map(|byte| [byte[0], byte[0], byte[0], byte[1]])
            .collect(),
        png::ColorType::Indexed => panic!("indexed color-type is not supported"),
    };
    return (pixels, info);
}

fn to_ascii(f: File) {
    let (pixels, info) = extract_png(f);
    //get terminal width
    let term_size = termsize::get().unwrap_or(termsize::Size { rows: 0, cols: 50 });
    //determine ratio
    let ratio = (info.width as f32 / info.height as f32) * 3.5;
    let h_pixels_per_char = (info.width as f32 / term_size.cols as f32).floor() as usize;
    let v_pixels_per_char = (h_pixels_per_char as f32 * ratio).floor().max(1.0) as u32;
    let rows = info.height as usize / v_pixels_per_char as usize;
    for dy in 0..rows {
        for dx in 0..term_size.cols as usize {
            let mut sum: [u32; 4] = [0; 4];
            let mut count = 0;

            for y in 0..v_pixels_per_char as usize {
                for x in 0..h_pixels_per_char as usize {
                    let px = (dx * h_pixels_per_char as usize + x).min(info.width as usize - 1);
                    let py = (dy * v_pixels_per_char as usize + y).min(info.height as usize - 1);
                    let pixel = pixels[py * info.width as usize + px];
                    sum[0] += pixel[0] as u32;
                    sum[1] += pixel[1] as u32;
                    sum[2] += pixel[2] as u32;
                    sum[3] += pixel[3] as u32;
                    count += 1;
                }
            }
            if count > 0 {
                sum[0] /= count;
                sum[1] /= count;
                sum[2] /= count;
                sum[3] /= count;
            }
            let color = ansi_term::Colour::RGB(sum[0] as u8, sum[1] as u8, sum[2] as u8);
            let brightnes_idx = (sum[3] as usize * (BRIGHTNESS.len() - 1)) / 255;
            let brightness = BRIGHTNESS[brightnes_idx] as char;
            print!("{}", color.paint(brightness.to_string()));
        }
        println!();
    }

    println!(
        "width: {}px,\nheight: {}px,\ncolor-type: {:?},\nbit-depth: {:?}",
        info.width, info.height, info.color_type, info.bit_depth
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please enter a filepath")
    } else {
        let path = &args[1].clone();
        let file = File::open(path);
        to_ascii(file.unwrap());
    }
}
