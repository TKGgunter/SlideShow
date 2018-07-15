//Thoth Gunter
//
//TODO: Handle error codes!

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
    //TODO:
    //We should be able to pass a string, &str and maybe also a PATH. 
    //Should loading and getting font name be in one step? There doesn't really seem to be a reason
    //to do one and not the other.  I'm going to do that now and maybe its not the right thing
    //todo.
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
    //TODO
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
pub struct DocumentSettings<'a>{
    slide_align: Alignment,
    slide_valign: VAlignment,
    slide_color: [f32;3],
    slide_height: f32,
    slide_width: f32,
    slide_font_size: f32,
    slide_font_color: [f32;3],
    slide_text_pos: [f32; 2],
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

//TODO: Returning String or to return &str
//IDK
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


//TODO: What is this lifetime doing?
pub fn make_slide<'a>( document_settings: &DocumentSettings<'a>, slide_card: &SlideCard, pdf: &mut HpdfDoc ){
    let mut page = pdf.add_page();
    //Slide settings
    
    let mut font = document_settings.slide_font;
    let mut font_size = document_settings.slide_font_size;
    let mut font_color = document_settings.slide_font_color;
    let mut slide_width = document_settings.slide_width;
    let mut slide_height = document_settings.slide_height;
    let mut slide_color = document_settings.slide_color;
    let mut text_align = document_settings.slide_align;
    let mut text_valign = document_settings.slide_valign;
    let mut cursor = [0.0, 0.0];



    match slide_card.config{
        Some(ref config_card) => {
            for (i, iter) in config_card.config_data.iter().enumerate(){
                match iter.kwd{
                    ConfigKwds::slide_width => { 
                        slide_width = match_value_f32data(&document_settings.slide_width, &iter.data);
                    },
                    ConfigKwds::slide_height => { 
                        slide_height = match_value_f32data(&document_settings.slide_height, &iter.data);
                    },
                    ConfigKwds::align => { 
                        let temp_align = match_value_strdata("", &iter.data).to_lowercase();
                        if temp_align == "left"{
                            text_align = Alignment::Left;
                        }
                        else if temp_align == "right"{
                            text_align = Alignment::Right;
                        }
                        else if temp_align == "center"{
                            text_align = Alignment::Center;
                        }
                        else {
                            print!("Error: Unknown alignment {}", temp_align);
                        }
                    },
                    ConfigKwds::valign => { 
                        let temp_align = match_value_strdata("", &iter.data).to_lowercase();
                        if temp_align == "top"{
                            text_valign = VAlignment::Top;
                        }
                        else if temp_align == "bottom"{
                            text_valign = VAlignment::Bottom;
                        }
                        else if temp_align == "center"{
                            text_valign = VAlignment::Center;
                        }
                        else {
                            print!("Error: Unknown vertical alignment {}", temp_align);
                        }
                    },
                    ConfigKwds::slide_background_color => { 
                        match_value_arrf32data(&mut slide_color, &iter.data);
                        for i in 0..slide_color.len(){
                            if slide_color[i] > 1.0{
                                slide_color[i] = slide_color[i] / 255.0
                            }
                        }
                    },
                    _ => { println!("We ain't delt with that yet."); }
                }
            }
        },
        _=>{}
    }

    
    page.set_page_dimensions( &slide_width, &slide_height);
    page.set_page_color(slide_color);
    cursor[0] = document_settings.slide_text_pos[0] * slide_width;
    let mut cursor_fixed = false; 

    //Vertical Alignment code
    //Should think about moveing out at some point
    {
        if text_valign != VAlignment::Top{
            let mut delta_row = 1;
            for (i, iter) in slide_card.slide_data.iter().enumerate(){
                match iter.kwd{
                    ConfigKwds::text=>{
                        let temp_string = match_value_strdata("", &iter.data);
                        if temp_string == "".to_string(){}
                        else{
                            delta_row += 1;
                        }
                    },
                    _=>{}
                }
            }
       
            if text_valign == VAlignment::Center{
                //TODO: 
                //Peek and look at all the text in the slide to determine where to start our cursor vertically
                //This is all down to do vertical center
                //Currently assuming all rows have the same font and font size
                
                cursor[1] = document_settings.slide_text_pos[1] * slide_height + font_size * delta_row as f32 / 2.0;
            }
            else if text_valign == VAlignment::Bottom{
                cursor[1] = document_settings.slide_text_pos[1] * slide_height + font_size * delta_row as f32;
            }
        }
        else{
                cursor[1] = document_settings.slide_text_pos[1] * slide_height;
        }
    }
    //

    let mut temp_cursor = None;
    let mut temp_default_cursor_x = 0.0;
    for iter in slide_card.slide_data.iter(){
        match iter.config{
            Some(ref config_card) => {
                for iter_config in config_card.config_data.iter(){

                    match iter_config.kwd{
                        ConfigKwds::text_position=>{
                            let mut _cursor = [0.0f32;2];
                            match_value_arrf32data(&mut _cursor, &iter_config.data);
                            if _cursor[0] < 1.0 && _cursor[1] < 1.0 {
                                _cursor[0] *= slide_width;
                                temp_default_cursor_x = _cursor[0];
                                _cursor[1] *= slide_height;
                            }
                            temp_cursor = Some(_cursor);
                        },
                        ConfigKwds::font_size=>{
                            println!("font size {:?}", iter_config.data);
                        },
                        _=>{
                            println!("What ever you want we don't do!");
                        }
                    }

                }

            },
            _=>{}
            
        }
        match iter.kwd{
            ConfigKwds::text=>{

                let temp_string = match_value_strdata("", &iter.data);
                if temp_string == "".to_string(){
                    temp_cursor = None;
                }
                else{
                    unsafe{
                        //TODO: This is temparary we only want fonts from ttf files
                        let font = HpdfFont(HPDF_GetFont (pdf.0, CString::new("Helvetica").unwrap().as_ptr(), ptr::null_mut()));



                        let mut delta = 0.0;
                        if text_align == Alignment::Center{
                            delta = 0.5; 
                            unsafe{
                                HPDF_Page_SetFontAndSize (page.0, font.0, font_size*1.0);
                                delta *= HPDF_Page_TextWidth( page.0,  cstring!(temp_string.as_str()).as_ptr() );
                            }
                        }
                        else if text_align == Alignment::Right{
                            unsafe{
                                HPDF_Page_SetFontAndSize (page.0, font.0, font_size*1.0);
                                delta = HPDF_Page_TextWidth( page.0,  cstring!(temp_string.as_str()).as_ptr() );
                            }
                        }


                        //TODO: 
                        //I can't believe this works with font example...
                        //Why does this work
                        cursor[0] = document_settings.slide_text_pos[0] * slide_width;
                        cursor[0] = cursor[0] - delta;
                        cursor[1] -= font_size; //TODO: Use correct next line thing

                        //TODO:
                        //Make all options swappable like this!
                        if temp_cursor == None{
                            cursor = page.render_text( &temp_string, &font_size, &font, &font_color, &cursor);
                        }
                        else {
                            temp_cursor = Some( [temp_default_cursor_x - delta, temp_cursor.unwrap()[1] - font_size]); //TODO: Use correct next line thing
                            temp_cursor = Some(page.render_text( &temp_string, &font_size, &font, &font_color, &temp_cursor.unwrap()));
                        }
                    }
                }
            },
            _=>{
                println!("Not implemented {:?}", iter.data);
            }
        }
    }
    
    
}





//TODO:
//This function should return an error if it fails to render the slide
pub fn render(slides: &Vec<Card>){
    let mut pdf = create_hpdf_pdf();

    let mut document_settings = DocumentSettings{   
                                                    slide_align: Alignment::Center,
                                                    slide_valign: VAlignment::Center,
                                                    slide_color: [0.0, 0.0, 0.0],
                                                    slide_height: 600.0,
                                                    slide_width:  800.0,
                                                    slide_font_size: 32.0,
                                                    slide_font_color: [0.0, 0.0, 0.0],
                                                    slide_text_pos: [0.5, 0.50],
                                                    slide_font: "Times", //Should prob be a handle of some kind.
                                                };

    for card in slides.iter(){

        match card{
            Card::SlideCard(slide)=>{
                make_slide(&document_settings, slide, &mut pdf);
            },
            Card::ConfigCard(config)=>{
                //TODO: This set of commands and the set of commands for a given slide should maybe
                //be the same. Should sleep on it.
                for (i, iter) in config.config_data.iter().enumerate(){
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
                        ConfigKwds::slide_width => { 
                            document_settings.slide_width = match_value_f32data(&document_settings.slide_width, &iter.data);
                        },
                        ConfigKwds::slide_height => { 
                            document_settings.slide_height = match_value_f32data(&document_settings.slide_height, &iter.data);
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
                        _ => { println!("We ain't delt with that yet."); }
                    }
                }
            },
            _=>{ println!("Some error"); }
        }

    }
    
    pdf.save("TEST.pdf");
}

