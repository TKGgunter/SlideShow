extern crate printpdf;
extern crate freetype;
extern crate image;


use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::collections::HashMap;
use std::io::Read;
use std::env;

use image::GenericImage;


mod parser;
use parser::{Card, ConfigCard, SlideCard, ConfigKwds, ValueType, example};



static PT_MM : f64 = 0.352778;
static PX_MM : f64 = 0.02 * 25.4;
/// Calculates the lower left of a centered string, to position it correctly in the document
/// 
/// Parameters:
/// 
/// - text: The actual text that will be embedded in the document
/// - font_scale: The current scale of the font
/// - parent_width: the width of the element in which relation the text should be centered
/// - font_face: The font-face of the text
/// From printpdf author 
/// Slight alterations by Thoth Gunter
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
///  END
/// 
/// 
/// 







fn set_settings( card: &ConfigCard, 
                 dimensions: &mut (f64, f64), 
                 background_color: &mut [f64;3],
                 font_color: &mut [f64;3],
                 font_size: &mut i64,
                 font_family: &mut [String; 2],
                 font_current: &mut String,
                 font_style: &mut String,
                 font_position: &mut [f64; 2],
                 font_nth: &mut i64,
                 alignment: &mut Align,
                 image_path: &mut String,
                 image_position: &mut [f64;2],
                 image_width: &mut f64,
                 image_height: &mut f64,
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
        else if config_data.kwd == ConfigKwds::font_size{

             if let ValueType::Num(ref num) = config_data.data{
                 *font_size = (num + 0.0) as i64; 
             }
        }
        else if config_data.kwd == ConfigKwds::font{

            let mut temp_array = Vec::new();
            if let ValueType::Arr(ref array) = config_data.data{
                for element in array.iter(){
                    if let &ValueType::Str(ref string) = element{
                        temp_array.push(format!("{}", string));
                    }
                }
            }
            if temp_array.len() == 2{
               font_family[0] = format!("{}", temp_array[0]); 
               font_family[1] = format!("{}", temp_array[1]); 
            }
        }
        else if config_data.kwd == ConfigKwds::image_path{
            if let ValueType::Str(ref string) = config_data.data{
                image_path.push_str(string);
            }
        }
        else if config_data.kwd == ConfigKwds::image_position{
            let mut temp_arr = Vec::new(); 
            if let ValueType::Arr(ref array) = config_data.data{
                for element in array.iter(){
                    if let &ValueType::Num(ref num) = element{
                        temp_arr.push(num + 0.0);
                    }
                }
            }
            if temp_arr.len() == 2{
                image_position[0] = temp_arr[0];
                image_position[1] = temp_arr[1];
            }
        }
        else if config_data.kwd == ConfigKwds::image_width{

             if let ValueType::Num(ref num) = config_data.data{
                 *image_width = num + 0.0; 
             }
        }
        else if config_data.kwd == ConfigKwds::image_height{

             if let ValueType::Num(ref num) = config_data.data{
                 *image_height = num + 0.0; 
             }
        }
        else if config_data.kwd == ConfigKwds::font_position{
            let mut temp_arr = Vec::new(); 
            if let ValueType::Arr(ref array) = config_data.data{
                for element in array.iter(){
                    if let &ValueType::Num(ref num) = element{
                        temp_arr.push(num + 0.0);
                    }
                }
            }
            if temp_arr.len() == 2{
                font_position[0] = temp_arr[0];
                font_position[1] = temp_arr[1];
            }
        }
        else if config_data.kwd == ConfigKwds::font_current{
            let mut temp_str = String::new(); 
            if let ValueType::Str(ref s) = config_data.data{
                temp_str = s.to_lowercase();
            }
            *font_current = temp_str;
        }
        else if config_data.kwd == ConfigKwds::font_style{
            let mut temp_str = String::new(); 
            if let ValueType::Str(ref s) = config_data.data{
                temp_str = s.to_lowercase();
            }
            *font_style = temp_str;
        }
        else if config_data.kwd == ConfigKwds::font_nth{
            if let ValueType::Num(ref n) = config_data.data{
                *font_nth = n.clone() as i64;
            }
        }
        //END OF IFS//
    }
}



#[derive(PartialEq, Debug, Copy, Clone)]
enum Alignment{
    right,
    left,
    center,
    default,
}
#[derive(PartialEq)]
struct Align{ data: Alignment}

#[derive(Debug)]
struct SpecialText{ 
    align:      Alignment,
    nth:        i64,
    font_size:  i64,
    position:   [f64;2],
    font_color: [f64;3],
    font:       String,
    string:     String,
} 

impl SpecialText{
    fn default()->SpecialText{
        SpecialText{align: Alignment::default, 
                    nth: 0, 
                    font_size: -1, 
                    position: [-1.0, -1.0], 
                    font_color: [-1.0, -1.0, -1.0], 
                    font: String::new(),
                    string: String::new(),}
    }
}



#[derive(Debug)]
struct SpecialImage{ 
    align:      Alignment,
    position:   [f64;2],
    dimensions: [f64;2],
    path:       String,
} 

impl SpecialImage{
    fn new()->SpecialImage{
        SpecialImage{
            align: Alignment::left,
            position: [0.0f64;2],
            dimensions: [-1.0f64;2],
            path: String::new(),
        }
    }
}


fn load_image(sp_img: &SpecialImage)->ImageXObject{
    let default_img = match image::open(&sp_img.path[..]){  Ok(img)=> img, 
                                                Err(e)=>{println!("Error: image not found!");
                                                         image::load_from_memory(DEFAULT_IMG).unwrap()}
                                            };

    let mut image_data = ImageXObject{
        width: default_img.dimensions().0 as i64,
        height: default_img.dimensions().1 as i64,
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: Vec::new(),
        image_filter: None,
        clipping_bbox: None,
    };

    for pixel in default_img.pixels(){
        image_data.image_data.push( pixel.2[0] as u8);
        image_data.image_data.push( pixel.2[1] as u8);
        image_data.image_data.push( pixel.2[2] as u8);
    }

    image_data
}



static DEFAULT_IMG:   &'static [u8]         =  include_bytes!("linux_peng.png");
static DEFAULT_FONTS: [&str; 3]             =  ["times","helvetica", "Courier"];
static FONT_TIMES:    &'static [u8]         =  include_bytes!("FreeSerif/FreeSerif.ttf");
static FONT_TIMES_BOLD:      &'static [u8]  =  include_bytes!("FreeSerif/FreeSerifBold.ttf");
static FONT_TIMES_ITALIC:   &'static [u8]  =  include_bytes!("FreeSerif/FreeSerifItalic.ttf");
static FONT_TIMES_BOLDITALIC: &'static [u8] =  include_bytes!("FreeSerif/FreeSerifBoldItalic.ttf");

fn main() {

    let ft_lib = match freetype::Library::init(){ Ok(lib)=>{lib}, Err(e)=>{panic!("FreeType could not load: {:?}", e)}};
    let ft_default_face = match ft_lib.new_memory_face(FONT_TIMES, 0) { Ok(face) => {face}, Err(e) => {panic!("Ft face could not be loaded {:?}", e)}};


    //DEFAULT SETINGS
    //16:9
    let mut dimensions = (338.7,190.5);
    let mut default_slide_color = [256.0, 256.0, 256.0];
    let mut default_font_family = "times";
    let mut default_font_color = [0.0, 0.0, 0.0];
    let mut default_font_position = [20.0, 0.0];
    let mut default_font_size : i64 = 16; 
    let mut default_alignment = Align{data: Alignment::left}; 

    let mut some_slide_color : Option<[f64; 3]> = None;
    let mut some_font_color  : Option<[f64; 3]> = None;
    let mut some_font_size   : Option<f64> = None;
    let mut some_alignment   : Option<Alignment>= None;



 ///////////////////////////////////////////////////
    let image_data = load_image(&SpecialImage{ align: Alignment::default, 
                                              position: [0.0, 0.0],
                                              dimensions: [0.0, 0.0],
                                              path: String::from("/home/gunter/Rust/Projects/SlideShow/assets/linux_peng.png")});
 ///////////////////////////////////////////////////

    let mut file_name = String::new();
    for (it, argument) in env::args().enumerate(){
        println!("Arg {} {}", it, argument);
        if it == 1{
            file_name = argument;
        } 
    }

    
    let mut file_contents : Option<String> = None;
    match File::open(file_name.clone()){
        Ok(mut f)=> { let mut content = String::new(); 
                  f.read_to_string(&mut content);
                  file_contents = Some(content);}
        Err(e)=> println!("{} {}",e, file_name)
    }

    let document = example(file_contents);
    println!("CARD GENERATION COMPLETE.\n\nSTARTING PDF GENERATION");
    for card in document.iter(){
        //println!("{:?}\n\n", card);
    }




    //Load default default dimension settings
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


    
    //Setting up Pdf document
    let (doc, page1, layer1) = PdfDocument::new("PDF_Document_title", dimensions.0, dimensions.1, "Layer 1");
    let font = doc.add_external_font(FONT_TIMES).unwrap();
    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut add_new_slide = true;

    let mut font_book = HashMap::new();    
    font_book.insert("times",             doc.add_external_font(FONT_TIMES).unwrap());
    font_book.insert("times_bold",        doc.add_external_font(FONT_TIMES_BOLD).unwrap());
    font_book.insert("times_italic",      doc.add_external_font(FONT_TIMES_ITALIC).unwrap() );
    font_book.insert("times_bolditalic",  doc.add_external_font(FONT_TIMES_BOLDITALIC).unwrap());


    for i in 0..document.len(){
    
        ///////////
        //Default setting for the document as determined by the user
        if let Card::ConfigCard(ref card) = document[i] {
            add_new_slide = false;

            let mut temp_font_family = [String::new(), String::new()];
            let mut temp_font_current = String::new();
            let mut temp_font_style = String::new();
            let mut temp_font_pos = [-1.0, -1.0];
            let mut temp_font_nth = -1;
            let mut temp_image_path = String::new();
            let mut temp_image_pos = [0.0, 0.0];
            let mut temp_image_width = 0.0;
            let mut temp_image_height = 0.0;
            set_settings(card,  &mut dimensions, 
                                &mut default_slide_color,
                                &mut default_font_color, 
                                &mut default_font_size, 
                                &mut temp_font_family, 
                                &mut temp_font_current, 
                                &mut temp_font_style, 
                                &mut temp_font_pos, 
                                &mut temp_font_nth, 
                                &mut default_alignment,
                                &mut temp_image_path,
                                &mut temp_image_pos,
                                &mut temp_image_width,
                                &mut temp_image_height,
                                );
        };

        let mut text_arr = Vec::<SpecialText>::new();
        let mut img_arr = Vec::<SpecialImage>::new();


        
        if let Card::SlideCard(ref slide) = document[i]{
            add_new_slide = true;
            
            ///////////
            //Modified settings for a particular slide
            if let Some(ref config) = slide.config {

                let mut temp_dimensions = (-1.0, -1.0);
                let mut temp_slide_color = [-1.0, -1.0, -1.0];
                let mut temp_font_color = [-1.0, -1.0, -1.0];
                let mut temp_font_family = [String::new(), String::new()];
                let mut temp_font_current = String::new();
                let mut temp_font_style = String::new();
                let mut temp_font_size = -1; 
                let mut temp_font_pos = [-1.0, -1.0];
                let mut temp_font_nth = -1;
                let mut temp_alignment = Align{data: Alignment::left}; 
                let mut temp_image_path = String::new();
                let mut temp_image_pos = [0.0, 0.0];
                let mut temp_image_width = 0.0;
                let mut temp_image_height = 0.0;

                set_settings(config, &mut temp_dimensions,
                             &mut temp_slide_color,
                             &mut temp_font_color,
                             &mut temp_font_size,
                             &mut temp_font_family,
                             &mut temp_font_current,
                             &mut temp_font_style,
                             &mut temp_font_pos,
                             &mut temp_font_nth,
                             &mut temp_alignment,
                             &mut temp_image_path,
                             &mut temp_image_pos,
                             &mut temp_image_width,
                             &mut temp_image_height,
                             );

                //ToDo: Need to add everything else ... What is everything else?
                if temp_slide_color[0] != -1.0{
                    some_slide_color = Some(temp_slide_color);
                }
            };  

            let mut nth_text = 0; //I need a better name for this...
            for element in slide.slide_data.iter(){
                let mut temp_text = SpecialText::default();
                temp_text.nth = nth_text.clone();
                
                ///////////////
                //Load special text, images and other
                if let Some(ref config) = element.config{
                    let mut temp_dimensions = (-1.0, -1.0);
                    let mut temp_slide_color = [-1.0, -1.0, -1.0];
                    let mut temp_font_color = [-1.0, -1.0, -1.0];
                    let mut temp_font_family = [String::new(), String::new()];
                    let mut temp_font_current = String::new();
                    let mut temp_font_style = String::new();
                    let mut temp_font_pos = [-1.0, -1.0];
                    let mut temp_font_nth = -1;
                    let mut temp_font_size = -1; 
                    let mut temp_alignment = Align{data: Alignment::left}; 
                    let mut temp_image_path = String::new();
                    let mut temp_image_pos = [0.0, 0.0];
                    let mut temp_image_width = 0.0;
                    let mut temp_image_height = 0.0;

                    set_settings(config, &mut temp_dimensions,
                                 &mut temp_slide_color,
                                 &mut temp_font_color,
                                 &mut temp_font_size,
                                 &mut temp_font_family,
                                 &mut temp_font_current,
                                 &mut temp_font_style,
                                 &mut temp_font_pos,
                                 &mut temp_font_nth,
                                 &mut temp_alignment,
                                 &mut temp_image_path,
                                 &mut temp_image_pos,
                                 &mut temp_image_width,
                                 &mut temp_image_height,
                                 );

                    temp_text.align = temp_alignment.data;
                    temp_text.font_size = temp_font_size;
                    temp_text.font_color = temp_font_color;
                    temp_text.position = temp_font_pos;
                    temp_text.font = temp_font_current;
                    temp_text.nth = temp_font_nth;
                    if temp_font_style != ""{
                        temp_text.font.push_str("_");
                        temp_text.font.push_str(&temp_font_style[..]);
                    } 
                    if temp_image_path != ""{
                        let mut temp_image = SpecialImage::new();
                        temp_image.path = temp_image_path;
                        temp_image.position = temp_image_pos;
                        temp_image.dimensions[0] = temp_image_width;
                        temp_image.dimensions[1] = temp_image_height;
                        img_arr.push(temp_image);
                    }
                }


                if let ValueType::Str(ref string) = element.data{
                    if calc_text_width_pt(&string.to_string(), temp_text.font_size, &ft_default_face) * PT_MM <  dimensions.0{
                        temp_text.string = string.to_string();
                        text_arr.push(temp_text);
                        nth_text += 1;
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

                                    let temp_temp_str = temp_string[..].to_string();
                                    let return_text = SpecialText{  align: temp_text.align,
                                                                    nth: nth_text.clone(),
                                                                    font_size: temp_text.font_size,
                                                                    position: temp_text.position,
                                                                    font_color: temp_text.font_color,
                                                                    font: String::from(&temp_text.font[..]),
                                                                    string: temp_temp_str};
                                    text_arr.push(return_text);
                                    nth_text += 1;
                                    break;
                                }
                            }
                            if let Some(string_peek) = string_iter.peek() { 
                                if calc_text_width_pt(&format!("{} {}", temp_string, string_peek), temp_text.font_size, &ft_default_face) * PT_MM >  dimensions.0{
                                    let temp_temp_str = temp_string[..].to_string();
                                    let return_text = SpecialText{  align: temp_text.align,
                                                                    nth: nth_text.clone(),
                                                                    font_size: temp_text.font_size,
                                                                    position: temp_text.position,
                                                                    font_color: temp_text.font_color,
                                                                    font: String::from(&temp_text.font[..]),
                                                                    string: temp_temp_str};
                                    text_arr.push(return_text);
                                    temp_string = String::new();
                                    nth_text += 1;
                                }
                            };
                        }
                    }
                }
                ///////////////////
            }
        };

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


        for (it, text_ele) in text_arr.iter_mut().enumerate(){

            if text_ele.align == Alignment::default {text_ele.align = default_alignment.data;}
            if text_ele.font_size <= -1         {text_ele.font_size = default_font_size;}
            if text_ele.font_color[0] <= -1.0 || text_ele.font_color[1] <= -1.0 || text_ele.font_color[2] <= -1.0   {text_ele.font_color = default_font_color;}

            default_font_position = [20.0, dimensions.1 * 0.95 - (it as f64 * PX_MM * text_ele.font_size as f64)];
            default_font_position = [20.0, dimensions.1 * 0.95 - (text_ele.nth as f64 * PX_MM * text_ele.font_size as f64)];

            if text_ele.position[0] <= -1.0 || text_ele.position[1] <= -1.0 {text_ele.position = default_font_position;}
            else{
                if text_ele.position[0] < 1.0 && text_ele.position[1] < 1.0{
                    text_ele.position[0] = text_ele.position[0] * dimensions.0;
                    text_ele.position[1] = text_ele.position[1] * dimensions.1;
                }
                else{
                    text_ele.position[1] = text_ele.position[1] - (text_ele.nth as f64 * PX_MM * text_ele.font_size as f64) ;
                }
            }

            fill_color = Color::Rgb(Rgb::new(text_ele.font_color[0] / 256.0,
                                             text_ele.font_color[1] / 256.0,
                                             text_ele.font_color[2] / 256.0, None));
            current_layer.set_fill_color(fill_color);
           
            let mut current_font = default_font_family; 
            if font_book.contains_key(&text_ele.font[..]){
                current_font = &text_ele.font;
            }
            else{
                if text_ele.font != ""{
                    println!("Error: font {} not found.", text_ele.font);
                }
            }

            //Render default text
            if text_ele.align == Alignment::left{
                current_layer.use_text(&text_ele.string[..], text_ele.font_size, text_ele.position[0], text_ele.position[1], font_book.get(current_font).unwrap());
            }
            else if text_ele.align == Alignment::right{
                render_right_aligned_text( &current_layer, &text_ele.string, text_ele.font_size,
                                           dimensions.0, text_ele.position[1], &ft_default_face, font_book.get(current_font).unwrap());
            }
            else if text_ele.align == Alignment::center{
                render_centered_text( &current_layer, &text_ele.string, text_ele.font_size, 
                                      dimensions.0, text_ele.position[1], &ft_default_face, font_book.get(current_font).unwrap());
            }
        }

        for (it, img_ele) in img_arr.iter().enumerate(){
            let temp_img = load_image(&img_ele);

            let mut img_position_x = img_ele.position[0];
            let mut img_position_y = img_ele.position[1];
            if img_ele.position[0] < 1.0 && img_ele.position[1] < 1.0{
                img_position_x *= dimensions.0;
                img_position_y *= dimensions.1;
            }
            
            let mut img_width  : Option<f64> = None;
            let mut img_height : Option<f64> = None;
            if img_ele.dimensions[0] > 0.0 {
                if img_ele.dimensions[0] > 1.0{
                    img_width = Some(img_ele.dimensions[0] / temp_img.width as f64);
                }
                else{
                    img_width = Some(img_ele.dimensions[0]);
                }
            }
            if img_ele.dimensions[1] > 0.0 {
                if img_ele.dimensions[1] > 1.0{
                    img_height = Some(img_ele.dimensions[1] / temp_img.height as f64);
                }
                else{
                    img_height = Some(img_ele.dimensions[1]);
                }
            }
            Image::from(temp_img).add_to_layer(current_layer.clone(), Some(img_position_x),
                                                                      Some(img_position_y),
                                                                      None,
                                                                      img_width, img_height,None);
        }


        if add_new_slide{
            let (page_n, layer1) = doc.add_page(dimensions.0, dimensions.1,"Page 2, Layer 1");
            current_layer = doc.get_page(page_n).get_layer(layer1);
        }
    }


    //Testing area
    Image::from(image_data).add_to_layer(current_layer.clone(), Some(100.0), Some(100.0), None, Some(0.5), Some(0.5), None); //Defauct
    current_layer.use_text("ASDFADFAD", 32, 20.0, 20.0, font_book.get("times").unwrap());
    current_layer.begin_text_section();
    current_layer.set_font(font_book.get(&"times_italic").unwrap(), 32);
    current_layer.set_text_cursor(20.0,160.0);
    current_layer.write_text("RETEWRTW ", font_book.get(&"times_italic").unwrap());

    current_layer.set_font(font_book.get(&"times").unwrap(), 32);
    current_layer.write_text("wert ", font_book.get(&"times").unwrap());
    current_layer.end_text_section();
    //


    doc.save(&mut BufWriter::new(File::create("test_working.pdf").unwrap())).unwrap();
}

