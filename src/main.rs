extern crate printpdf;
extern crate freetype;

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

mod parser;
use parser::{Card, ConfigCard, SlideCard, ConfigKwds, ValueType, example};



static PT_MM : f64 = 0.352778;
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
    ((mm_to_pt!(parent_width) / 2.0) - (s_w / 2.0)) * PT_MM
}

pub fn calc_lower_left_for_right_aligned_text(text: &String, font_scale: i64, parent_width: f64, font_face: &freetype::Face)
-> f64
{
    let s_w = calc_text_width_pt(text, font_scale, font_face);
    (mm_to_pt!(parent_width) - s_w ) * PT_MM
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


fn render_right_aligned_text( current_layer: &PdfLayerReference, text: &String, font_scale: i64, canvas_width: f64, y_mm: f64, face: &freetype::Face, font: &IndirectFontRef){

    let centered = calc_lower_left_for_right_aligned_text(text, font_scale, canvas_width, face);
    current_layer.use_text(&text[..], font_scale, centered, y_mm, font);
  
}







fn set_settings( card: &ConfigCard, 
                 dimensions: &mut (f64, f64), 
                 background_color: &mut [f64;3],
                 font_color: &mut [f64;3],
                 alignment: &mut Align,
                 ){

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

            if temp_array.len() == 3{
                background_color[0] = temp_array[0];
                background_color[1] = temp_array[1];
                background_color[2] = temp_array[2];
            }
        } 
        else if config_data.kwd == ConfigKwds::font_color{
            
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

            if temp_array.len() == 3{
                font_color[0] = temp_array[0];
                font_color[1] = temp_array[1];
                font_color[2] = temp_array[2];
            }
        } 
        else if config_data.kwd == ConfigKwds::align{
            if let ValueType::Str(ref s) = config_data.data{
                if s.to_lowercase()      == "right"{    alignment.data = Alignment::right;}
                else if s.to_lowercase() == "left"{     alignment.data = Alignment::left;}
                else if s.to_lowercase() == "center"{   alignment.data = Alignment::center;}
            };
        }
    }
}



#[derive(PartialEq)]
enum Alignment{
    right,
    left,
    center,
    default,
}
#[derive(PartialEq)]
struct Align{ data: Alignment}




//Need to take care of \n
fn main() {
    let default_font_path = "/home/gunter/Rust/Projects/SlideShow/assets/fonts/ofl/salsa/Salsa-Regular.ttf";
    //let default_font_path = "/home/tgunter/Rust/SlideShow/assets/Roboto-Medium.ttf";
    let ft_lib = match freetype::Library::init(){ Ok(lib)=>{lib}, Err(e)=>{panic!("FreeType could not load: {:?}", e)}};
    let ft_default_face = match ft_lib.new_face(default_font_path, 0) { Ok(face) => {face}, Err(e) => {panic!("Ft face could not be loaded {:?}", e)}};
    
    //DEFAULT SETINGS
    //16:9
    let mut dimensions = (338.7,190.5);
    let default_font_data = File::open(default_font_path).unwrap();
    //let default_font_data = File::open("/home/gunter/Rust/Projects/SlideShow/assets/lmroman6-regular.otf").unwrap();
    let mut default_slide_color = [256.0, 256.0, 256.0];
    let mut default_font_color = [0.0, 0.0, 0.0];
    let mut default_font_size = 16; 
    let mut default_alignment = Align{data: Alignment::right}; 

    let mut some_slide_color : Option<[f64; 3]> = None;
    let mut some_font_color  : Option<[f64; 3]> = None;
    let mut some_font_size   : Option<f64> = None;
    let mut some_alignment   : Option<Alignment>= None;
    
    let text1 = "Testing string one";
    //test image


    let document = example();
    println!("CARD GENERATION COMPLETE.\n\nSTARTING PDF GENERATION");

    if document.len() > 0{
        if let Card::ConfigCard(ref card) = document[0]{
            for config_data in card.config_data.iter(){
                if config_data.kwd == ConfigKwds::width{
                    if let ValueType::Num(ref num) = config_data.data{
                        dimensions.0 = num + 0.0;
                    }
                }
                else if config_data.kwd == ConfigKwds::height{
                    if let ValueType::Num(ref num) = config_data.data{
                        dimensions.1 = num + 0.0;
                    }
                }
            }
        }
    }

    let (doc, page1, layer1) = PdfDocument::new("PDF_Document_title", dimensions.0, dimensions.1, "Layer 1");
    let font = doc.add_external_font(default_font_data).unwrap();
    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut add_new_slide = true;

    for i in 0..document.len(){
    
        if let Card::ConfigCard(ref card) = document[i] {

            add_new_slide = false;
            //println!("{:?}", card);

            set_settings(card, &mut dimensions, &mut default_slide_color,
                         &mut default_font_color, &mut default_alignment);
        };

        let mut string_arr = Vec::<String>::new();
        if let Card::SlideCard(ref slide) = document[i]{
            add_new_slide = true;


            if let Some(ref config) = slide.config {

                let mut temp_dimensions = (-1.0, -1.0);
                let mut temp_slide_color = [-1.0, -1.0, -1.0];
                let mut temp_font_color = [-1.0, -1.0, -1.0];
                let mut temp_font_size = -1; 
                let mut temp_alignment = Align{data: Alignment::right}; 

                set_settings(config, &mut temp_dimensions,
                             &mut temp_slide_color,
                             &mut temp_font_color,
                             &mut temp_alignment);
                if temp_slide_color[0] != -1.0{
                    some_slide_color = Some(temp_slide_color);
                }
            };  

            for element in slide.slide_data.iter(){
                if let ValueType::Str(ref string) = element.data{
                    if calc_text_width_pt(&string.to_string(), default_font_size, &ft_default_face) * PT_MM <  dimensions.0{
                        string_arr.push(string.to_string());
                    }
                    else{
                        //Needed for text wrapping
                        let mut temp_string = String::from("");
                        let mut string_iter = string.split_whitespace().peekable();
                        loop{
                            match string_iter.next() {
                                Some(string_ele)=>{
                                    temp_string.push_str(&format!("{} ",string_ele));
                                },
                                None => {
                                    string_arr.push(temp_string);
                                    break;}
                            }
                            if let Some(string_peek) = string_iter.peek() { 
                                if calc_text_width_pt(&format!("{} {}", temp_string, string_peek), default_font_size, &ft_default_face) * PT_MM >  dimensions.0{
                                    string_arr.push(temp_string);
                                    temp_string = String::new();
                                }
                            };
                        }
                    }
                }
            }
        };
        println!("{:?}", string_arr);

        ////////////////////////////////////////////////
        //Background_color
        let points = vec![(Point::new(dimensions.0, 0.0), false),
                            (Point::new(0.0, 0.0), false),
                            (Point::new(0.0, dimensions.1), false),
                            (Point::new(dimensions.0, dimensions.1), false)];

        let line1 = Line{points: points, is_closed: true, has_fill: true, has_stroke: true, };
        let mut fill_color = Color::Rgb(Rgb::new(default_slide_color[0] / 256.0,
                                                 default_slide_color[1] / 256.0,
                                                 default_slide_color[2] / 256.0, None));
        if let Some(color) = some_slide_color {
            fill_color = Color::Rgb(Rgb::new(color[0] / 256.0,
                                             color[1] / 256.0,
                                             color[2] / 256.0, None));
        }
        some_slide_color = None;


        current_layer.set_fill_color(fill_color);
        current_layer.add_shape(line1);
        ////////////////////////////////////////////////


        for (it, string_ele) in string_arr.iter().enumerate(){
            fill_color = Color::Rgb(Rgb::new(default_font_color[0] / 256.0,
                                             default_font_color[1] / 256.0,
                                             default_font_color[2] / 256.0, None));
            current_layer.set_fill_color(fill_color);


            //Render default text
            if default_alignment.data == Alignment::left{
                current_layer.use_text(string_ele.to_string(), default_font_size, 20.0, 168.0 - (it as f64 * 0.02 * 25.4* default_font_size as f64), &font);
            }
            else if default_alignment.data == Alignment::right{
                render_right_aligned_text( &current_layer, &string_ele.to_string(), default_font_size, dimensions.0, 168.0 - (it as f64 * 0.02 * 25.4* default_font_size as f64), &ft_default_face, &font);
            }
            else if default_alignment.data == Alignment::center{
                render_centered_text( &current_layer, &string_ele.to_string(), default_font_size, dimensions.0, 168.0 - (it as f64 * 0.02 * 25.4* default_font_size as f64), &ft_default_face, &font);
            }


        }
        if add_new_slide{
            let (page_n, layer1) = doc.add_page(dimensions.0, dimensions.1,"Page 2, Layer 1");
            current_layer = doc.get_page(page_n).get_layer(layer1);
        }
    }
    doc.save(&mut BufWriter::new(File::create("test_working.pdf").unwrap())).unwrap();
}

