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
    println! ("ERROR: error_no={:?}, detail_no={:?}", error_no,
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


pub fn add_hpdf_page( pdf: HpdfDoc )->HpdfPage{
    unsafe{
        HpdfPage( HPDF_AddPage (pdf.0) )
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
    pub fn set_page_dimensions(&self, width: f32, height: f32){}

    pub fn render_text(&self, text: &str, size: f32, font: HpdfFont, color: &[f32;3], cursor: &[f32;2])->(f32, f32){
        unsafe{
            HPDF_Page_SetFontAndSize (self.0, font.0, size as HPDF_REAL);
            HPDF_Page_SetRGBFill  (self.0, color[0], color[1], color[2]);

            HPDF_Page_BeginText (self.0);

            HPDF_Page_MoveTextPos(self.0, cursor[0], cursor[1]);
            HPDF_Page_ShowText (self.0, cstring!(text).as_ptr());

            let pos = HPDF_Page_GetCurrentPos (self.0);
            HPDF_Page_EndText (self.0);

            (pos.x, pos.y)
        }
    }
}



/////////////////////////

pub enum Alignment{
    Right,
    Center,
    Left
}
pub struct DocumentSettings<'a>{
    slide_align: Alignment,
    slide_color: [f32;3],
    slide_height: f32,
    slide_width: f32,
    slide_font_size: f32,
    slide_font_color: [f32;3],
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

//TODO: Returning String or to return &str
//IDK
#[inline]
pub fn match_value_strdata(data: &str, valuetype: ValueType)->String{
    let rt_data = match valuetype{
        ValueType::Str(s) => s,
        _=> {
            println!("Wrong Type not String");
            data.to_string()
        }
    };
    return rt_data;
}


//TODO: What is this lifetime doing?
pub fn make_slide<'a>( document_settings: &DocumentSettings<'a>, slide_card: &SlideCard, &mut pdf: HpdfDoc ){
    let mut page = pdf.add_page();
    //Slide settings
    
    let mut font = document_settings.slide_font;
    let mut font_size = document_settings.slide_font_size;
    let mut font_color = document_settings.slide_font_c;
    let mut cursor = [0.0, 0.0];

        println!("Slides: ");
        match self.config{
            Some(ref config_card) => config_card.print(),
            _=>{}
        }
        for (i, iter) in self.slide_data.iter().enumerate(){
            println!("Slide Data {}", i);
            println!("\t Data type: {:?}\n\t Key: {:?}\n\t Row: {}", iter.data, iter.kwd, iter.text_row);  
            match iter.config{
                Some(ref config_card) => config_card.print(),
                _=>{}
                
            }
        }
    
    
      
    page.render_text(text: "temp", size: f32, font: HpdfFont, color: &[f32;3], cursor: &[f32;2])->(f32, f32){
}





//TODO:
//This function should return an error if it fails to render the slide
pub fn render(slides: Vec<Card>){
    let mut pdf = create_hpdf_pdf();

    let mut document_settings = DocumentSettings{   slide_align: Alignment::Center,
                                                    slide_color: [0.0, 0.0, 0.0],
                                                    slide_height: 800.0,
                                                    slide_width: 800.0,
                                                    slide_font_size: 16.0,
                                                    slide_font_color: [0.0, 0.0, 0.0],
                                                    slide_font: "Times", //Should prob be a handle of some kind.
                                                };


    for card in slides.iter(){

        match card{
            Card::SlideCard(slide)=>{
                slide.print()
            },
            Card::ConfigCard(config)=>{
                for (i, iter) in config.config_data.iter().enumerate(){
                    match iter.kwd{
                        slide_width => { 
                            document_settings.slide_width = match_value_f32data(&document_settings.slide_width, &iter.data);
                        },
                        slide_height => { 
                            document_settings.slide_height = match_value_f32data(&document_settings.slide_height, &iter.data);
                        },
                        _ => { println!("We ain't delt with that yet."); }
                    }
                }
            },
            _=>{ println!("Some error");}
        }

    }
    
}

