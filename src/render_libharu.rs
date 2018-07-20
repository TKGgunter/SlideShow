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
    struct TextProperties {
        size: f32,
        color: [f32;3],
        delta_horizontal: f32,
        lines_from_init: u32,
        text: String,
        //font
        //style
    };
    let mut vec_text_properties = Vec::<TextProperties>::new();

    {
        let mut delta_row = 1;
        let mut delta_horizontal = 0.0;
        let mut delta_horizontal_multiples = 0;
        let mut temp_text_string = String::new();
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
                                            _=>{
                                                println!("What ever you want we don't do!");
                                            }
                                        }

                                    }



                                },
                                _=>{}
                            }
                        unsafe{

                            let font = HpdfFont(HPDF_GetFont (pdf.0, CString::new("Helvetica").unwrap().as_ptr(), ptr::null_mut()));
                            HPDF_Page_SetFontAndSize (page.0, font.0, temp_font_size.clone());
                            delta_horizontal += HPDF_Page_TextWidth( page.0,  cstring!(temp_string.as_str()).as_ptr() );
                            delta_horizontal_multiples += 1;

                            let margin = 500.0;
                            if delta_horizontal > margin && temp_string.contains(" "){
                                delta_horizontal -= HPDF_Page_TextWidth( page.0,  cstring!(temp_string.as_str()).as_ptr() );
                                for word in temp_string.split_whitespace(){
                                    delta_horizontal += HPDF_Page_TextWidth( page.0,  cstring!(word).as_ptr() );
                                    vec_text_properties.push(
                                        TextProperties{ size: temp_font_size,
                                        color:temp_font_color,
                                        delta_horizontal: 0.0,
                                        lines_from_init: 0,
                                        text: word.to_string()});
                                    if delta_horizontal > margin{
                                        delta_horizontal = 0.0;
                                        let offset = vec_text_properties.len();
                                        vec_text_properties[ offset - 1].lines_from_init += 1;

                                    }
                                    else{
                                        delta_horizontal_multiples += 1;
                                    }
                                }
                            }
                            else{
                                vec_text_properties.push(
                                    TextProperties{ size: temp_font_size,
                                    color:temp_font_color,
                                    delta_horizontal: 0.0,
                                    lines_from_init: 0,
                                    text: temp_string});

                            }

                        }
                        if i+1 < slide_card.slide_data.len(){ 
                            if (slide_card.slide_data[i+1].text_row - slide_card.slide_data[i].text_row) > 1 {
                                let offset = vec_delta_horizontal.len();
                                let lines_from_init = if offset == 0 { vec_text_properties[0].lines_from_init+1} else {vec_text_properties[offset-1].lines_from_init + 1};
                                for i in 0..delta_horizontal_multiples {
                                    vec_delta_horizontal.push(delta_horizontal);
                                    //I think this caused a bug when offset is > 1
                                    vec_text_properties[offset + i].delta_horizontal = delta_horizontal;
                                    vec_text_properties[offset + i].lines_from_init = lines_from_init;
                                }
                                delta_horizontal_multiples = 0;
                                delta_horizontal = 0.0;
                                delta_row += 1;
                            }
                        }
                        if (slide_card.slide_data[i].text_row - prev_row) > 1 {
                            let offset = vec_delta_horizontal.len();
                            let lines_from_init = if offset == 0 { vec_text_properties[0].lines_from_init+1} else {vec_text_properties[offset-1].lines_from_init + 1};
                            for i in 0..delta_horizontal_multiples {
                                vec_delta_horizontal.push(delta_horizontal);
                                vec_text_properties[offset + i].delta_horizontal = delta_horizontal;
                                vec_text_properties[offset + i].lines_from_init = lines_from_init;
                            }
                            delta_horizontal_multiples = 0;
                            delta_horizontal = 0.0;
                        }
                    }
                },
                _=>{}
            }

            println!("{:?}", vec_text_properties);
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
    }
    //
    

    let mut temp_cursor = None;
    let mut temp_font_size = None;
    let mut temp_font_color = None;
    let mut temp_font_braced_line = None;
    //let mut temp_align = None;
    //let mut temp_font = None;
    let mut temp_default_cursor_x = 0.0;
    let mut next_text_row = 0;
    let mut text_row = 0;
    let mut prev_text_row = 0;
    let mut vec_delta_horizontal_cursor = 0;
    //for iter in slide_card.slide_data.iter(){
    let mut _iter = slide_card.slide_data.iter().peekable();
    loop{
        let __iter = _iter.next();
        if __iter == None{ break; }
        let iter = __iter.unwrap();
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
                            let mut _font_size = match_value_f32data(&font_size, &iter_config.data);
                            if _font_size <= 1.0{
                                _font_size *= slide_height;
                            }

                            temp_font_size = Some(_font_size);
                        },
                        ConfigKwds::font_color=>{
                            let mut _font_color = font_color.clone();
                            match_value_arrf32data(&mut _font_color, &iter_config.data);
                            for i in 0.._font_color.len(){
                                if _font_color[i] > 1.0{
                                    _font_color[i] = _font_color[i] / 255.0;
                                }
                            }
                            temp_font_color = Some(_font_color);
                        },
                        ConfigKwds::font_braced=>{
                            temp_font_braced_line = Some( match_value_f32data(&0.0, &iter_config.data));
                        }
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
                }
                else{
                    prev_text_row = text_row;
                    text_row = iter.text_row;
                    next_text_row = match _iter.peek(){ Some(_next_iter)=> _next_iter.text_row, None=>text_row*1};
                    unsafe{
                        //TODO: This is temparary we only want fonts from ttf files
                        let font = HpdfFont(HPDF_GetFont (pdf.0, CString::new("Helvetica").unwrap().as_ptr(), ptr::null_mut()));

                        //TODO:
                        //I don't like the way this looks be we have to handle temporary changes in
                        //font size.
                        let mut _font_size = font_size.clone();
                        if temp_font_size != None{ _font_size = temp_font_size.unwrap() }


                        let mut delta = 0.0;
                        if text_align == Alignment::Center{
                            delta = 0.5; 
                        }
                        else if text_align == Alignment::Right{
                            delta = 1.0; 
                        }


                        //TODO: 
                        //I can't believe this works with font example...
                        //Why does this work
                        if (text_row - prev_text_row) > 1 {
                            cursor[0] = cloned_document_settings.slide_text_pos[0] * slide_width;
                            cursor[0] = cursor[0] - delta*vec_delta_horizontal[vec_delta_horizontal_cursor];//- delta;
                            println!("{} {} {} ", temp_string, delta, cursor[0]);
                            
                        //TODO: Use correct next line thing
                            cursor[1] -= font_size; 
                        }
                            vec_delta_horizontal_cursor += 1;

                        if temp_cursor.is_some() {
                            if text_align == Alignment::Center{
                                delta = 0.5; 
                                unsafe{
                                    HPDF_Page_SetFontAndSize (page.0, font.0, _font_size*1.0);
                                    delta *= HPDF_Page_TextWidth( page.0,  cstring!(temp_string.as_str()).as_ptr() );
                                }
                            }
                            else if text_align == Alignment::Right{
                                delta = 1.0; 
                                unsafe{
                                    HPDF_Page_SetFontAndSize (page.0, font.0, _font_size*1.0);
                                    delta = HPDF_Page_TextWidth( page.0,  cstring!(temp_string.as_str()).as_ptr() );
                                }
                            }

                            temp_cursor = Some( [temp_default_cursor_x - delta, temp_cursor.unwrap()[1] - _font_size * temp_font_braced_line.unwrap()] ); //TODO: Use correct next line thing
                        }
                       


                        //TODO:
                        //Make all options swappable like this!
                        let mut _cursor = cursor.clone();
                        if temp_cursor != None{ _cursor = temp_cursor.unwrap() }

                        let mut _font_color = font_color.clone();
                        if temp_font_color != None{ _font_color = temp_font_color.unwrap() }

    //TODO:
    //let mut temp_align = None;
    //let mut temp_font = None;

                        _cursor = page.render_text( &temp_string, &_font_size, &font, &_font_color, &_cursor);
                        if temp_cursor.is_some(){
                            temp_cursor = Some(_cursor);
                        }
                        else{
                            cursor = _cursor;
                        }

                temp_cursor = None;
                temp_font_size = None;
                temp_font_color = None;


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

