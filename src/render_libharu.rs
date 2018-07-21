//Thoth Gunter
//
//TODO: 
//+Handle error codes!
//+Fix Valignment

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

impl HpdfFont{
    //HpdfFont(HPDF_GetFont (pdf.0, CString::new("Helvetica").unwrap().as_ptr(), ptr::null_mut()));
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

    let mut font        = cloned_document_settings.slide_font;
    let mut font_size   = cloned_document_settings.slide_font_size;
    let mut font_color  = cloned_document_settings.slide_font_color;
    let mut slide_width = cloned_document_settings.slide_width;
    let mut slide_height= cloned_document_settings.slide_height;
    let mut slide_color = cloned_document_settings.slide_color;
    let mut text_align  = cloned_document_settings.slide_align;
    let mut text_valign = cloned_document_settings.slide_valign;
    let mut cursor = [0.0, 0.0];
    

    if font_size <= 1.0{
        font_size *= slide_height;
    }

    page.set_page_dimensions( &slide_width, &slide_height);
    page.set_page_color(slide_color);
    cursor[0] = cloned_document_settings.slide_text_pos[0] * slide_width;
    let mut cursor_fixed = false; 

    //Vertical Alignment code
    //Should think about moved out at some point
    let mut vec_delta_horizontal = Vec::new();

    #[derive(Debug)]
    struct TextAndProperties {
        size: f32,
        color: [f32;3],
        delta_horizontal: f32,
        lines_from_init: u32,
        //cursor: [f32;2],
        text: String,
        //font
        //style
    };
    let mut vec_text_and_properties = Vec::<TextAndProperties>::new();

    let mut delta_row = 1;
    let mut delta_horizontal = 0.0;
    let mut delta_horizontal_multiples = 0;
    let mut prev_row = 0;
    for  i in 0..slide_card.slide_data.len(){
        if i == 0 {
            prev_row = slide_card.slide_data[i].text_row;
        }
        match slide_card.slide_data[i].kwd{
            ConfigKwds::text=>{
                let temp_string = match_value_strdata("", &slide_card.slide_data[i].data);
                if temp_string == "".to_string(){}
                else{
                    //TODO We need to concate strings until we get to a new line then do the text
                    //width calculation

                    //TODO:: NASTY
                    let mut temp_font_size = font_size * 1.0;
                    let mut temp_font_color = font_color.clone();
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
                                    //ConfigKwds::text_position=>{
                                    //    let mut _cursor = [0.0f32;2];
                                    //    match_value_arrf32data(&mut _cursor, &iter_config.data);
                                    //    if _cursor[0] < 1.0 && _cursor[1] < 1.0 {
                                    //        _cursor[0] *= slide_width;
                                    //        temp_default_cursor_x = _cursor[0];
                                    //        _cursor[1] *= slide_height;
                                    //    }
                                    //    temp_cursor = Some(_cursor);
                                    //},
                                    //ConfigKwds::font_braced=>{
                                    //    temp_font_nth_line = Some( match_value_f32data(&0.0, &iter_config.data));
                                    //}
                                    _=>{
                                        println!("What ever you want we don't do!");
                                    }
                                }

                            }



                        },
                        _=>{}
                    }

                    let current_text_width = calc_text_width( &pdf, &page, &temp_string, "Helvetica", &temp_font_size);
                    delta_horizontal += current_text_width;
                    delta_horizontal_multiples += 1;

                    let margin = 5000.0;
                    if delta_horizontal > margin && temp_string.contains(" "){
                        delta_horizontal -= current_text_width;
                        for word in temp_string.split_whitespace(){
                            delta_horizontal += calc_text_width( &pdf, &page, word, "Helvetica", &temp_font_size);

                            vec_text_and_properties.push(
                                TextAndProperties{ size: temp_font_size,
                                color:temp_font_color,
                                delta_horizontal: 0.0,
                                lines_from_init: 0,
                                text: word.to_string()});

                            if delta_horizontal > margin{
                                delta_horizontal = 0.0;
                                let offset = vec_text_and_properties.len();
                                vec_text_and_properties[ offset - 1].lines_from_init += 1;

                            }
                            else{
                                delta_horizontal_multiples += 1;
                            }
                        }
                    }
                    else{
                        vec_text_and_properties.push(
                            TextAndProperties{ size: temp_font_size,
                            color:temp_font_color,
                            delta_horizontal: 0.0,
                            lines_from_init: 0,
                            text: temp_string});

                    }

                    if i+1 < slide_card.slide_data.len(){ 
                        if (slide_card.slide_data[i+1].text_row - slide_card.slide_data[i].text_row) > 1 {
                            let offset = vec_delta_horizontal.len();
                            let lines_from_init = if offset == 0 { vec_text_and_properties[0].lines_from_init+1} else {vec_text_and_properties[offset-1].lines_from_init + 1};
                            for i in 0..delta_horizontal_multiples {
                                vec_delta_horizontal.push(delta_horizontal);
                                //I think this caused a bug when offset is > 1
                                vec_text_and_properties[offset + i].delta_horizontal = delta_horizontal;
                                vec_text_and_properties[offset + i].lines_from_init = lines_from_init;
                            }
                            delta_horizontal_multiples = 0;
                            delta_horizontal = 0.0;
                            delta_row += 1;
                        }
                    }
                    if (slide_card.slide_data[i].text_row - prev_row) > 1 {
                        let offset = vec_delta_horizontal.len();
                        let lines_from_init = if offset == 0 { vec_text_and_properties[0].lines_from_init+1} else {vec_text_and_properties[offset-1].lines_from_init + 1};
                        for i in 0..delta_horizontal_multiples {
                            vec_delta_horizontal.push(delta_horizontal);
                            vec_text_and_properties[offset + i].delta_horizontal = delta_horizontal;
                            vec_text_and_properties[offset + i].lines_from_init = lines_from_init;
                        }
                        delta_horizontal_multiples = 0;
                        delta_horizontal = 0.0;
                    }
                }
            },
            _=>{}
        }

        println!("{:?}", vec_text_and_properties);
        //TODO: 
        //?Does this need to be in this loop?
        //Peek and look at all the text in the slide to determine where to start our cursor vertically
        //This is all down to do vertical center
        //Currently assuming all rows have the same font and font size
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
    //
    for iter in vec_text_and_properties.iter(){
        let font = HpdfFont::get_font_handle(&pdf, "Helvetica");
        let _cursor = page.render_text( &iter.text, &iter.size, &font, &iter.color, &[300.0, 300.0]);
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
                                                    slide_font_color: [1.0, 1.0, 1.0],
                                                    slide_text_pos: [0.5, 0.5],
                                                    slide_font: "Times", //Should prob be a handle of some kind.
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

