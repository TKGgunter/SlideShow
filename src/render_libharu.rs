//Thoth Gunter
//
//TODO: 
//+Handle error codes!

#![allow(dead_code)]

extern crate libharu_sys;
extern crate libc;

use parser::*;

use std::ptr;
use std::ffi::CString;
use self::libharu_sys::*;
use std::path::Path;

macro_rules! cstring{
    ($fmt:expr) => {
        CString::new($fmt).unwrap()
    }
}


extern fn error_handler  (error_no: HPDF_STATUS,
                detail_no: HPDF_STATUS,
                user_data : HPDF_HANDLE)
{
    println! ("ERROR: error_no={:X?}, detail_no={:?}", error_no,
                detail_no);
}



pub struct HpdfDoc(HPDF_Doc);
pub struct HpdfPage(HPDF_Page);
pub struct HpdfFont(HPDF_Font);



pub fn create_hpdf_pdf()->HpdfDoc{
    unsafe{
        HpdfDoc(HPDF_New(error_handler, ptr::null_mut()))
    }
}


pub fn free_hpdf_pdf( pdf: HpdfDoc ) {
    unsafe{
        HPDF_Free (pdf.0);
    }
}





impl HpdfDoc{
    pub fn load_ttf_from_file(&self, file_path: &str )->HpdfFont{
        unsafe{
            let detail_font_name = HPDF_LoadTTFontFromFile (self.0, cstring!(file_path).as_ptr(), HPDF_TRUE);
            HpdfFont( HPDF_GetFont (self.0, detail_font_name, ptr::null_mut()) )
        }
    }

    pub fn save(&self, file_name: &str){
        unsafe{
            HPDF_SaveToFile(self.0, cstring!(file_name).as_ptr());
        }
    }
    pub fn add_page(&self )->HpdfPage{
        unsafe{
            HpdfPage( HPDF_AddPage (self.0) )
        }
    }
}

impl HpdfPage{
    pub fn set_page_dimensions(&self, width: &f32, height: &f32){
        unsafe{
            HPDF_Page_SetWidth (self.0, *width);
            HPDF_Page_SetHeight (self.0, *height);
        }
    }
    pub fn set_page_color(&self, color: [f32; 3]){
        unsafe{
            let width = HPDF_Page_GetWidth  (self.0);
            let height = HPDF_Page_GetHeight (self.0);

            HPDF_Page_SetRGBFill (self.0, color[0], color[1],color[2]);
            HPDF_Page_Rectangle (self.0, 0.0, 0.0, width, height);
            HPDF_Page_Fill (self.0);
        }
    }

    pub fn render_text(&self, text: &str, size: &f32, font: &HpdfFont, color: &[f32;3], cursor: &[f32;2])->[f32;2]{
        unsafe{
            HPDF_Page_SetFontAndSize (self.0, font.0, *size as HPDF_REAL);
            HPDF_Page_SetRGBFill  (self.0, color[0], color[1], color[2]);

            HPDF_Page_BeginText (self.0);

            HPDF_Page_MoveTextPos(self.0, cursor[0], cursor[1]);
            HPDF_Page_ShowText (self.0, cstring!(text).as_ptr());

            let pos = HPDF_Page_GetCurrentTextPos (self.0);
            HPDF_Page_EndText (self.0);

            [pos.x, pos.y]
        }
    }
}

impl HpdfFont{
    pub fn get_font_handle(pdf: &HpdfDoc, font_name: &str)->HpdfFont{unsafe{
        HpdfFont(HPDF_GetFont (pdf.0, cstring!(font_name).as_ptr(), ptr::null_mut()))

    }}
}



fn calc_text_width(pdf: &HpdfDoc, page: &HpdfPage, text: &str, font: &str, size: &f32)->f32{ unsafe{
    let font = HpdfFont(HPDF_GetFont (pdf.0, cstring!(font).as_ptr(), ptr::null_mut()));
    HPDF_Page_SetFontAndSize (page.0, font.0, *size);
    HPDF_Page_TextWidth( page.0,  cstring!(text).as_ptr() )

}}


/////////////////////////
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Alignment{
    Right,
    Center,
    Left
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VAlignment{
    Top,
    Center,
    Bottom
}
#[derive(Debug, Copy, Clone)]
pub struct DocumentSettings<'a>{
    slide_align: Alignment,
    slide_valign: VAlignment,
    slide_color: [f32;3],
    slide_height: f32,
    slide_width: f32,
    slide_font_size: f32,
    slide_font_color: [f32;3],
    slide_text_pos: [f32; 2],
    slide_text_margin: f32,
    slide_font: &'a str,
}


#[inline]
pub fn match_value_f32data(data: &f32, valuetype: &ValueType)->f32{
    let rt_data = match valuetype{
        ValueType::Num(n) => *n as f32,
        _=> {
            println!("Wrong Type Not Num"); //TODO: make proper error format!
            *data
        }
    };
    return rt_data;
}


pub fn match_value_arrf32data(data: &mut [f32], valuetype: &ValueType){
     
    match valuetype{
        ValueType::Arr(arr) => {
            if data.len() != arr.len(){
                return;
            }
            for (i, i_valuetype) in arr.iter().enumerate(){
                data[i] = match_value_f32data(&data[i], i_valuetype);
            }
        },
        _=>{}
    }
}

#[inline]
pub fn match_value_strdata(data: &str, valuetype: &ValueType)->String{
    let rt_data = match valuetype{
        ValueType::Str(s) => s[0..].to_string(),
        _=> {
            println!("Wrong Type not String");
            data.to_string()
        }
    };
    return rt_data;
}


pub fn get_slide_settings( document_settings : &mut DocumentSettings, config : &ConfigCard ){
    for iter in config.config_data.iter(){
        match iter.kwd{
            ConfigKwds::slide_width => { 
                document_settings.slide_width = match_value_f32data(&document_settings.slide_width, &iter.data);
            },
            ConfigKwds::slide_height => { 
                document_settings.slide_height = match_value_f32data(&document_settings.slide_height, &iter.data);
            },
            ConfigKwds::slide_background_color => { 
                match_value_arrf32data(&mut document_settings.slide_color, &iter.data);
                for i in 0..document_settings.slide_color.len(){
                    if document_settings.slide_color[i] > 1.0{
                        document_settings.slide_color[i] = document_settings.slide_color[i] / 255.0
                    }
                }
            },
            ConfigKwds::font_color => { 
                match_value_arrf32data(&mut document_settings.slide_font_color, &iter.data);
                for i in 0..document_settings.slide_font_color.len(){
                    if document_settings.slide_font_color[i] > 1.0{
                        document_settings.slide_font_color[i] = document_settings.slide_font_color[i] / 255.0
                    }
                }
            },
            ConfigKwds::align => { 
                let temp_align = match_value_strdata("", &iter.data).to_lowercase();
                if temp_align == "left"{
                    document_settings.slide_align = Alignment::Left;
                }
                else if temp_align == "right"{
                    document_settings.slide_align = Alignment::Right;
                }
                else if temp_align == "center"{
                    document_settings.slide_align = Alignment::Center;
                }
                else {
                    print!("Error: Unknown alignment {}", temp_align);
                }
            },
            ConfigKwds::valign => { 
                let temp_align = match_value_strdata("", &iter.data).to_lowercase();
                if temp_align == "top"{
                    document_settings.slide_valign = VAlignment::Top;
                }
                else if temp_align == "bottom"{
                    document_settings.slide_valign = VAlignment::Bottom;
                }
                else if temp_align == "center"{
                    document_settings.slide_valign = VAlignment::Center;
                }
                else {
                    print!("Error: Unknown vertical alignment {}", temp_align);
                }
            },
            //ConfigKwds::font => { 
            //    font = pdf.load_ttf_from_file("temp_shit" );
            //},
            ConfigKwds::font_size => { 
                document_settings.slide_font_size = match_value_f32data(&0.0, &iter.data);
            },
            ConfigKwds::text_position=>{
                match_value_arrf32data(&mut document_settings.slide_text_pos, &iter.data);
            },
            ConfigKwds::text_margin=>{
                document_settings.slide_text_margin = match_value_f32data(&1.1, &iter.data);
            },
            _ => { println!("We ain't delt with that yet."); }
        }
    }

}




//TODO: What is this lifetime doing?
pub fn make_slide<'a>( document_settings: &DocumentSettings<'a>, slide_card: &SlideCard, pdf: &mut HpdfDoc ){
    let mut page = pdf.add_page();
    //Slide settings
   
    let mut cloned_document_settings = document_settings.clone();




    match slide_card.config{
        Some(ref config_card) => {
            get_slide_settings( &mut cloned_document_settings, config_card );
        },
        _=>{}
    }

    let mut font_name   = cloned_document_settings.slide_font;
    let mut font_size   = cloned_document_settings.slide_font_size;
    let mut font_color  = cloned_document_settings.slide_font_color;
    let mut slide_width = cloned_document_settings.slide_width;
    let mut slide_height= cloned_document_settings.slide_height;
    let mut slide_color = cloned_document_settings.slide_color;
    let mut text_align  = cloned_document_settings.slide_align;
    let mut text_valign = cloned_document_settings.slide_valign;
    let mut text_margin = cloned_document_settings.slide_text_margin;
    let mut cursor      = cloned_document_settings.slide_text_pos;
    

    if font_size <= 1.0{
        font_size *= slide_height;
    }

    page.set_page_dimensions( &slide_width, &slide_height);
    page.set_page_color(slide_color);
    if cursor[0] < 1.0{
        cursor[0] *= slide_width;
    }
    if cursor[1] < 1.0{
        cursor[1] *= slide_height;
    }
    if text_margin <= 1.1001{
        text_margin *= slide_width;
    }
    
    
    
    //Vertical Alignment code
    //Should think about moved out at some point
    let mut counter_delta_horizontal = 0;

    #[derive(Debug)]
    struct TextAndProperties {
        align: Alignment,
        valign: VAlignment,
        size: f32,
        color: [f32;3],
        delta_horizontal_mm: f32,
        delta_vertical_lines: u32,
        cursor: Option<[f32;2]>,
        text: String,
        //font
        //style
    };
    let mut vec_text_and_properties = Vec::<TextAndProperties>::new();

    let mut delta_row = 1;
    let mut delta_horizontal_mm = 0.0;
    let mut delta_horizontal_mm_multiples = 0;
    let mut prev_row = 0;
    let mut temp_text_margin = text_margin.clone();
    for  i in 0..slide_card.slide_data.len(){
        let mut temp_cursor = None;
        if i == 0 {
            prev_row = slide_card.slide_data[i].text_row;
        }
        match slide_card.slide_data[i].kwd{
            ConfigKwds::text=>{
                let temp_string = match_value_strdata("", &slide_card.slide_data[i].data);
                if temp_string == "".to_string(){}
                else{

                    //TODO:: NASTY
                    let mut temp_font_size = font_size * 1.0;
                    let mut temp_font_color = font_color.clone();
                    let mut temp_text_align = text_align.clone();
                    match slide_card.slide_data[i].config{
                        Some(ref config) => { 


                            for iter_config in config.config_data.iter(){

                                match iter_config.kwd{
                                    ConfigKwds::font_size=>{
                                        let mut _font_size = match_value_f32data(&font_size, &iter_config.data);
                                        if _font_size <= 1.0{
                                            _font_size *= slide_height;
                                        }

                                        temp_font_size = _font_size;
                                    },
                                    ConfigKwds::font_color=>{
                                        match_value_arrf32data(&mut temp_font_color, &iter_config.data);
                                        for i in 0..temp_font_color.len(){
                                            if temp_font_color[i] > 1.0{
                                                temp_font_color[i] = temp_font_color[i] / 255.0;
                                            }
                                        }
                                    },
                                    ConfigKwds::text_position=>{
                                        let mut _cursor = [0.0f32; 2];
                                        match_value_arrf32data(&mut _cursor, &iter_config.data);
                                        if _cursor[0] < 1.0 {
                                            _cursor[0] *= slide_width;
                                        }
                                        if _cursor[1] < 1.0 {
                                             _cursor[1] *= slide_height;
                                        }
                                        temp_cursor = Some(_cursor);
                                    },
                                    ConfigKwds::text_margin=>{
                                        let mut _text_margin = match_value_f32data(&text_margin, &iter_config.data);
                                        if _text_margin <= 1.0{
                                            _text_margin *= slide_width;
                                        }

                                        temp_text_margin = _text_margin;
                                    },
                                    ConfigKwds::text_align=>{
                                        let temp_align = match_value_strdata("", &iter_config.data).to_lowercase();
                                        if temp_align == "left"{
                                            temp_text_align = Alignment::Left;
                                        }
                                        else if temp_align == "right"{
                                            temp_text_align = Alignment::Right;
                                        }
                                        else if temp_align == "center"{
                                            temp_text_align = Alignment::Center;
                                        }
                                        else {
                                            print!("Error: Unknown alignment {}", temp_align);
                                        }
                                    },
                                    //ConfigKwds::font_braced=>{
                                    //    temp_font_nth_line = Some( match_value_f32data(&0.0, &iter_config.data));
                                    //}
                                    NOT_HANDLED =>{
                                        println!("What ever you want we don't do! {:?}", NOT_HANDLED);
                                    }
                                }

                            }

                        },
                        _=>{}
                    }

                    let current_text_width = calc_text_width( &pdf, &page, &temp_string, font_name, &temp_font_size);
                    delta_horizontal_mm += current_text_width;
                    delta_horizontal_mm_multiples += 1;

                    let margin = temp_text_margin;
                    let mut margin_vertical_lines = 1;

                    //NOTE
                    //Word wrap by splitting on words
                    if delta_horizontal_mm > margin && temp_string.contains(" "){
                        delta_horizontal_mm -= current_text_width;
                        delta_horizontal_mm_multiples -= 1;
                        for word in temp_string.split_whitespace(){

                            let word_space = word.to_string() + " ";
                            delta_horizontal_mm += calc_text_width( &pdf, &page, &word_space, font_name, &temp_font_size);

                            vec_text_and_properties.push(
                                TextAndProperties{ 
                                    align: temp_text_align.clone(),
                                    valign: text_valign.clone(),
                                    size: temp_font_size,
                                    color:temp_font_color,
                                    delta_horizontal_mm: 0.0,
                                    delta_vertical_lines: margin_vertical_lines.clone(),
                                    cursor: temp_cursor.clone(),
                                    text: word_space.clone(),
                                });

                            if delta_horizontal_mm > margin{

                                delta_horizontal_mm -= calc_text_width( &pdf, &page, &word_space, font_name, &temp_font_size);
                                //loop over previous lines and set the delta_vertical
                                let offset = counter_delta_horizontal;
                                for i in 0..delta_horizontal_mm_multiples {
                                    counter_delta_horizontal += 1;
                                    vec_text_and_properties[offset + i].delta_horizontal_mm = delta_horizontal_mm;
                                }
                                delta_horizontal_mm_multiples = 0;
                                delta_row += 1;

                                delta_horizontal_mm = 0.0;
                                let offset = vec_text_and_properties.len();
                                margin_vertical_lines += 1;
                            }
                            else{
                                delta_horizontal_mm_multiples += 1;
                            }
                        }
                        {
                            let offset = counter_delta_horizontal;
                            let delta_vertical_lines = if offset == 0 { vec_text_and_properties[0].delta_vertical_lines+1} 
                                                       else {vec_text_and_properties[offset-1].delta_vertical_lines + 1};
                            for i in 0..delta_horizontal_mm_multiples {
                                counter_delta_horizontal += 1;
                                vec_text_and_properties[offset + i].delta_horizontal_mm = delta_horizontal_mm;
                            }
                            delta_horizontal_mm_multiples = 0;
                        }
                    }
                    else{
                        vec_text_and_properties.push(
                            TextAndProperties{ 
                                align: temp_text_align.clone(),
                                valign: text_valign.clone(),
                                size: temp_font_size,
                                color:temp_font_color,
                                delta_horizontal_mm: 0.0,
                                delta_vertical_lines: 0,
                                cursor: temp_cursor.clone(),
                                text: temp_string
                            });

                        if i+1 < slide_card.slide_data.len(){ 
                            if (slide_card.slide_data[i+1].text_row - slide_card.slide_data[i].text_row) > 1 {
                                let offset = counter_delta_horizontal;
                                let delta_vertical_lines = if offset == 0 { vec_text_and_properties[0].delta_vertical_lines+1} 
                                                           else {vec_text_and_properties[offset-1].delta_vertical_lines + 1};

                                for i in 0..delta_horizontal_mm_multiples {
                                    counter_delta_horizontal += 1;
                                    vec_text_and_properties[offset + i].delta_horizontal_mm = delta_horizontal_mm;
                                    vec_text_and_properties[offset + i].delta_vertical_lines = delta_vertical_lines;
                                }
                                delta_horizontal_mm_multiples = 0;
                                delta_horizontal_mm = 0.0;
                                delta_row += 1;
                            }
                        }
                        if (slide_card.slide_data[i].text_row - prev_row) > 1 {
                            let offset = counter_delta_horizontal;
                            let delta_vertical_lines = if offset == 0 { vec_text_and_properties[0].delta_vertical_lines+1}
                                                       else {vec_text_and_properties[offset-1].delta_vertical_lines + 1};

                            for i in 0..delta_horizontal_mm_multiples {
                                counter_delta_horizontal += 1;
                                vec_text_and_properties[offset + i].delta_horizontal_mm = delta_horizontal_mm;
                                vec_text_and_properties[offset + i].delta_vertical_lines = delta_vertical_lines;
                            }
                            delta_horizontal_mm_multiples = 0;
                            delta_horizontal_mm = 0.0;
                        }


                    }

                }
            },
            _=>{}
        }

        //TODO: 
        //Test these setting in regard to word wrap!
        if (slide_card.slide_data[i].text_row - prev_row) > 1 {
            if text_valign == VAlignment::Center{
                
                cursor[1] = cloned_document_settings.slide_text_pos[1] * slide_height + font_size * delta_row as f32 / 2.0;
            }
            else if text_valign == VAlignment::Bottom{
                cursor[1] = cloned_document_settings.slide_text_pos[1] * slide_height + font_size * delta_row as f32;
            }
            else{
                cursor[1] = cloned_document_settings.slide_text_pos[1] * slide_height;
            }
        }
        prev_row = slide_card.slide_data[i].text_row;

    }
    
    //TODO:
    //Alignment is wrong when word wrapping
    //Test right align
    //Text Bottom align
    let mut prev_line = 0;
    let mut text_position = cursor.clone();
    for iter in vec_text_and_properties.iter(){
        if prev_line != iter.delta_vertical_lines{
            if iter.cursor.is_some() == false {
                text_position = cursor.clone();
            }
            else{
                text_position = iter.cursor.unwrap();
            }
            if iter.align == Alignment::Center{
                text_position[0] -= iter.delta_horizontal_mm * 0.5;
            }
            else if iter.align == Alignment::Right{
                text_position[0] -= iter.delta_horizontal_mm;
            }
            prev_line = iter.delta_vertical_lines;

            text_position[1] -= iter.delta_vertical_lines as f32 * iter.size;
        } 


        //println!("ASDF {:?} {:?} {} {:?}", iter.text, iter.align, iter.delta_horizontal_mm, text_position);
        let font = HpdfFont::get_font_handle(&pdf, font_name);
        text_position = page.render_text( &iter.text, &iter.size, &font, &iter.color, &text_position);
    }
    
    
}





//TODO:
//This function should return an error if it fails to render the slide
//PDF save file name
pub fn render(slides: &Vec<Card>){
    let mut pdf = create_hpdf_pdf();

    let mut document_settings = DocumentSettings{   
                                                    slide_align: Alignment::Center,
                                                    slide_valign: VAlignment::Center,
                                                    slide_color: [0.0, 0.0, 0.0],
                                                    slide_height: 600.0,
                                                    slide_width:  800.0,
                                                    slide_font_size: 32.0,
                                                    slide_font_color: [1.0, 1.0, 1.0],
                                                    slide_text_pos: [0.5, 0.5],
                                                    slide_text_margin: 1.1, //Purposfully ackwardly long
                                                    slide_font: "Helvetica", //"Times", Should prob be a handle of some kind.
                                                };

    for card in slides.iter(){

        match card{
            Card::SlideCard(slide)=>{
                make_slide(&document_settings, slide, &mut pdf);
            },
            Card::ConfigCard(config)=>{
                get_slide_settings( &mut document_settings, config );
            },
            _=>{ println!("Some error"); }
        }

    }
    
    pdf.save("TEST.pdf");
}

