extern crate printpdf;
extern crate freetype;

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

mod parser;
use parser::{Card, ConfigCard, SlideCard, ConfigKwds, ValueType, example};


/// Calculates the lower left of a centered string, to position it correctly in the document
/// 
/// Parameters:
/// 
/// - text: The actual text that will be embedded in the document
/// - font_scale: The current scale of the font
/// - parent_width: the width of the element in which relation the text should be centered
/// - font_face: The font-face of the text
/// From printpdf author 


pub fn calc_lower_left_for_centered_text(text: &String, font_scale: i64, parent_width: f64, font_face: &freetype::Face)
-> f64
{
    let s_w = calc_text_width_pt(text, font_scale, font_face);
    ((mm_to_pt!(parent_width) / 2.0) - (s_w / 2.0)) * 0.352778
}



#[allow(unused_variables)]
pub fn calc_text_width_pt(text: &String, font_scale: i64, font_face: &freetype::Face)
-> f64
{
    // vertical scale for the space character
    let vert_scale = {
        if let Ok(ch) = font_face.load_char(0x0020, freetype::face::LoadFlag::NO_SCALE) {
            font_face.glyph().metrics().vertAdvance
        } else {
            1000
        }
    };

    // calculate the width of the text in unscaled units
    let sum_width = text.chars().fold(0, |acc, ch| 
        if let Ok(ch) = font_face.load_char(ch as usize, freetype::face::LoadFlag::NO_SCALE) {
            let glyph_w = font_face.glyph().metrics().horiAdvance;
            acc + glyph_w
        } else { acc }
    );

    sum_width as f64 / (vert_scale as f64 / font_scale as f64)
}


fn render_centered_text( current_layer: &PdfLayerReference, text: &String, font_scale: i64, canvas_width: f64, y_mm: f64, face: &freetype::Face, font: &IndirectFontRef){

    let centered = calc_lower_left_for_centered_text(text, font_scale, canvas_width, face);
    current_layer.use_text(&text[..], font_scale, centered, y_mm, font);
  
}







//Need to take care of \n
fn main() {
    //DEFAULT SETINGS
    //16:9
    let mut dimensions = (Mm(338.7), Mm(190.5));
    //let default_font_data = File::open("/home/tgunter/Rust/SlideShow/assets/Roboto-Medium.ttf").unwrap();
    let default_font_data = File::open("/home/gunter/Rust/Projects/SlideShow/assets/lmroman6-regular.otf").unwrap();
    let mut default_slide_color = [256.0, 256.0, 256.0];
    let default_font_color = [0.0, 0.0, 0.0];


    let text1 = "Testing string one";
    //test image


    let document = example();

    let (doc, page1, layer1) = PdfDocument::new("PDF_Document_title", dimensions.0, dimensions.1, "Layer 1");
    let font = doc.add_external_font(default_font_data).unwrap();

    for i in 0..document.len(){
    
        if let Card::ConfigCard(ref card) = document[i] {

            println!("{:?}", card);

            for config_data in card.config_data.iter(){
                if config_data.kwd == ConfigKwds::background_color{
                    
                    let mut temp_array = Vec::new();
                    if let ValueType::Arr(ref array) = config_data.data{
                        for (it, element) in array.iter().enumerate(){
                            if let &ValueType::Num(ref number) = element{
                                temp_array.push(number + 0.0);
                            }
                            else{
                                println!("Unexpected Value type in Configuration");
                            }
                        }
                    }
                    else{
                        println!("Unexpected Value type in Configuration");
                    }

                    println!("Background color was {:?}", default_slide_color);
                    if temp_array.len() == 3{
                        default_slide_color[0] = temp_array[0];
                        default_slide_color[1] = temp_array[1];
                        default_slide_color[2] = temp_array[2];
                    }
                    println!("Now it is {:?}", default_slide_color);
                } 
            }
        };



        if i == 0{
            let mut current_layer = doc.get_page(page1).get_layer(layer1);
            let points = vec![(Point::new(dimensions.0, Mm(0.0)), false),
                                (Point::new(Mm(0.0), Mm(0.0)), false),
                                (Point::new(Mm(0.0), dimensions.1), false),
                                (Point::new(dimensions.0, dimensions.1), false)];

            let line1 = Line{points: points, is_closed: true, has_fill: true, has_stroke: true, is_clipping_path: false};
            let mut fill_color = Color::Rgb(Rgb::new(default_slide_color[0] / 256.0,
                                             default_slide_color[1] / 256.0,
                                             default_slide_color[2] / 256.0, None));
            current_layer.set_fill_color(fill_color);

            current_layer.add_shape(line1);

            fill_color = Color::Rgb(Rgb::new(default_font_color[0] / 256.0,
                                             default_font_color[1] / 256.0,
                                             default_font_color[2] / 256.0, None));
            current_layer.set_fill_color(fill_color);
            current_layer.use_text(text1, 32, Mm(20.0), Mm(168.0), &font);

        }
        else{
            let (page2, layer1) = doc.add_page(dimensions.0, dimensions.1,"Page 2, Layer 1");
            let mut current_layer = doc.get_page(page2).get_layer(layer1);
            let points = vec![(Point::new(dimensions.0, Mm(0.0)), false),
                                (Point::new(Mm(0.0), Mm(0.0)), false),
                                (Point::new(Mm(0.0), dimensions.1), false),
                                (Point::new(dimensions.0, dimensions.1), false)];
            let line1 = Line{points: points, is_closed: true, has_fill: true, has_stroke: true, is_clipping_path: false};
            let mut fill_color = Color::Rgb(Rgb::new(0.0, 0.2, 0.4, None));
            current_layer.set_fill_color(fill_color);
            current_layer.add_shape(line1);
        }
    }
    doc.save(&mut BufWriter::new(File::create("test_working.pdf").unwrap())).unwrap();
}

