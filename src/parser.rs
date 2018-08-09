//Thoth Gunter
//
//TODO:
//Parser bug font { <= infinite loop when trying to peek
//something about divs


#![allow(dead_code)]
extern crate ansi_term;

struct ParserCursor{
    file_string: Vec<char>,
    col: usize,
    row: usize,
    pos: usize, 
    prev_col: usize,
    prev_row: usize,
    prev_pos: usize, 
}

impl ParserCursor{
    fn new(file_string: String)->ParserCursor{
        ParserCursor{
            file_string: file_string.chars().collect(),
            col: 0,
            row: 0,
            pos: 0,
            prev_col: 0,
            prev_row: 0,
            prev_pos: 0,
        }
    }


    fn peek(&self)->Option<char>{
        if self.pos + 1 < self.file_string.len(){
            Some(self.file_string[self.pos + 1])
        }
        else{
            println!("Ran out of space to peek");
            None
        }
    }

    fn current(&mut self)->Option<char>{
        if self.pos < self.file_string.len(){
            Some(self.file_string[self.pos])
        }
        else{
            println!("Ran out of space to current");
            None
        }
    }

    fn next(&mut self)->Option<char>{
        self.prev_col = self.col;
        self.prev_row = self.row;
        self.prev_pos = self.pos;
        if self.pos + 1 < self.file_string.len(){
            self.pos += 1;
            if self.file_string[self.pos] == '\n'{
                self.row += 1;
            }
            else{
                self.col += 1;
            }
            Some(self.file_string[self.pos])
        }
        else{
            None
        }
    }

    fn previous(&mut self){
        self.col = self.prev_col;
        self.row = self.prev_row;
        self.pos = self.prev_pos;
    }

    fn croak(&self, msg: &str){
        println!("{} pos:{}, row:{}, col:{}", msg, self.pos, self.row, self.col);
    }

}



#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LexType{
    Num,
    Str,
    Arr,
    SlideStr,
    Braced,
}




//TODO: Maybe the enum name should not be the same as the struct names
#[derive(Debug, PartialEq)]
pub enum Card{
    SlideCard(SlideCard),
    ConfigCard(ConfigCard),
    Default,
}

#[derive(Debug, PartialEq)]
pub struct SlideCard{
    pub config: Option<ConfigCard>,
    pub slide_data: Vec<SlideData>,
}


#[derive(Debug, PartialEq, Clone)]
pub struct ConfigCard{
    pub config_data: Vec<ConfigData>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ConfigData{
    pub kwd: ConfigKwds,
    pub data: ValueType,
}


#[derive(Debug, PartialEq)]
pub struct SlideData{
    pub config: Option<ConfigCard>,
    pub kwd: ConfigKwds,
    pub data: ValueType,
    pub text_row: usize,
    //pub div_ids: [u8;2],

}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType{
    Num(f64), //TODO: Move to f32
    Str(String),
    Arr(Vec<ValueType>),
    Err,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConfigKwds{
    slide_width,
    slide_height,
    slide_background_color,

    align,
    valign,

    header1,
    header2,
    header3,

    text,
    text_position,
    text_align,
    text_margin,

    div,
    div_background_color,
    div_x,
    div_y,
    div_width,
    div_height,
    div_position,
    div_align,
    div_nth,

    font,
    font_color,
    font_size,
    font_current,
    font_style,
    font_braced,

    latex,
    latex_color,
    latex_size,
    latex_current,
    latex_position,
    latex_style,
    latex_nth,
    latex_margin,

    image,
    image_path,
    image_position,
    image_width,
    image_height,

    default,
}

impl SlideCard{
    pub fn print(&self){
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
    }
}

impl ConfigCard{
    pub fn print(&self){
        println!("\n\t\tConfigure: ");
        for (i, iter) in self.config_data.iter().enumerate(){
            println!("\t\t\tKeywords: {:?}\t data: {:?}", iter.kwd, iter.data);
        }
        println!("\n");
    }
}





static ACC_NUM:  [char; 10] =  ['0','1','2','3','4','5','6','7','8','9'];
static KEYWORD_IDENTIFIER:   [char; 1]  =  ['#'];
static WHITE_SPACE:[char; 3] = [' ','\t', '\n'];
static HEADER : [&str; 3] =  ["#h1", "#h2", "#h3"];

fn parse_error(message: &str, parser_cursor: &ParserCursor){
    println!("\t\t{}: {}\n\tUser file row:{}  col:{}  pos:{}\n", ansi_term::Colour::Red.paint("Error"), message, parser_cursor.row, parser_cursor.col, parser_cursor.pos);
}

fn is_keyword(parser_cursor: &mut ParserCursor, keyword: &str, reset_pos: bool)->bool{
    let mut _is_keyword = true;
    let pos = parser_cursor.pos;
    let row = parser_cursor.row;
    let col = parser_cursor.col;
    let previous_pos = parser_cursor.prev_pos;
    let previous_row = parser_cursor.prev_row;
    let previous_col = parser_cursor.prev_col;

    for conf_character in keyword.chars(){
        if conf_character != parser_cursor.current().unwrap(){
            _is_keyword = false;
            break;
        }
        parser_cursor.next();
    }

    if reset_pos == true{
        parser_cursor.pos = pos;
        parser_cursor.row = row;
        parser_cursor.col = col;
        parser_cursor.prev_pos = previous_pos;
        parser_cursor.prev_row = previous_row;
        parser_cursor.prev_col = previous_col;
    }
    _is_keyword
}



fn is_slide(parser_cursor: &mut ParserCursor)->bool{
    is_keyword(parser_cursor, "#slide", true)
}

fn is_config(parser_cursor: &mut ParserCursor)->bool{
    is_keyword(parser_cursor, "#config", true)
}




fn image_func(parser_cursor: &mut ParserCursor)->SlideData{
    let config = gen_config_func(parser_cursor, 
                                 &[("path", ConfigKwds::image_path, LexType::Str),
                                 ("position", ConfigKwds::image_position, LexType::Arr),
                                 ("width", ConfigKwds::image_width, LexType::Num),
                                 ("height", ConfigKwds::image_height, LexType::Num),],
                                 "#image");
    parser_cursor.next();
    let slide_data = ValueType::Err; 
    let data = SlideData{config: Some(config),
                         kwd: ConfigKwds::image,
                         data: slide_data,
                         text_row: parser_cursor.row};
    data
}

fn font_func(parser_cursor: &mut ParserCursor)->Vec<SlideData>{
    let mut config = gen_config_func(parser_cursor, 
                                 &[("family", ConfigKwds::font_current, LexType::Str),
                                 ("size", ConfigKwds::font_size, LexType::Num),
                                 ("position", ConfigKwds::text_position, LexType::Arr),
                                 ("color", ConfigKwds::font_color, LexType::Arr),
                                 ("style", ConfigKwds::font_style, LexType::Str),
                                 ("margin", ConfigKwds::text_margin, LexType::Num),
                                 ("align", ConfigKwds::text_align, LexType::Str),
                                 ],
                                 "#font");
    parser_cursor.next();
    let mut return_data = Vec::new();
    if parser_cursor.current() != Some('{'){
        config.config_data.push(ConfigData{ kwd: ConfigKwds::font_braced, data: ValueType::Num(0.0)});
        let mut data = SlideData{  config: Some(config),
                                   kwd: ConfigKwds::text,
                                   data: ValueType::Err,
                                   text_row: parser_cursor.row}; 
        let slide_data = gather_value(parser_cursor, LexType::SlideStr, None);
        data.data = slide_data; 
        return_data.push(data);
    }
    else{
        parser_cursor.next();
        let mut nth = 0;
        loop{
            if parser_cursor.current() == Some('}') {
                break;
            }
            else{
                let mut _config = config.clone();
                _config.config_data.push(ConfigData{ kwd: ConfigKwds::font_braced, data: ValueType::Num(nth as f64)});
                let mut data = SlideData{  config: Some(_config),
                                           kwd: ConfigKwds::text,
                                           data: ValueType::Err,
                                           text_row: parser_cursor.row}; 
                let slide_data = gather_value(parser_cursor, LexType::SlideStr, Some(LexType::Braced));

                data.data = slide_data; 
                return_data.push(data);

                if parser_cursor.next() == None{break;}
                nth += 1;
            }
        }
        return_data.reverse();
    }
    return_data
}

// TODO: Finish me
fn header_func(parser_cursor: &mut ParserCursor)->SlideData{
    let header_arr = [("#header1", ConfigKwds::header1) , ("#header2", ConfigKwds::header2), ("#header3", ConfigKwds::header3)];
    let mut configkwd = ConfigKwds::default;
    for header in header_arr.iter(){
        if is_keyword(parser_cursor, header.0, false) {
            configkwd = header.1; 
            break;
        }
    }
   


    let data = gather_value(parser_cursor, LexType::SlideStr, None);

    SlideData{
        config: None,
        kwd: configkwd,
        data: data, 
        text_row: parser_cursor.row,
    }

}

fn tex_func(parser_cursor: &mut ParserCursor)->SlideData{
    let mut config = ConfigCard{config_data: Vec::new()};
    if is_keyword(parser_cursor, "#tex(", true) {
        config = gen_config_func(parser_cursor, 
                                     &[("family", ConfigKwds::latex_current, LexType::Str),
                                     ("size",     ConfigKwds::latex_size, LexType::Num),
                                     ("position", ConfigKwds::latex_position, LexType::Arr),
                                     ("color",    ConfigKwds::latex_color, LexType::Arr),
                                     ("style",    ConfigKwds::latex_style, LexType::Str),
                                     ("margin",   ConfigKwds::latex_margin, LexType::Num),
                                     ],
                                     "#tex");
        parser_cursor.next();
    }
    else {
        is_keyword(parser_cursor, "#tex", false);
    }

    let mut slide_data = SlideData{  config: Some(config),
                               kwd: ConfigKwds::latex,
                               data: ValueType::Err,
                               text_row: parser_cursor.row}; 
    if parser_cursor.current() != Some('{'){
        let data = gather_value(parser_cursor, LexType::SlideStr, None);
        slide_data.data = data; 
    }
    else{
        parser_cursor.next();
        let mut nth = 0;
        let mut data_string = String::new();
        loop{
            if parser_cursor.current() == Some('}'){
                break;
            }
            else{
                match gather_value(parser_cursor, LexType::SlideStr, None){
                    ValueType::Str(s) =>{data_string.push_str(&s);},
                    _ => {println!("Unexpected ValueType during Tex parsing.");}
                }
                parser_cursor.next();
            }
        }
        slide_data.data = ValueType::Str(data_string); 
    }
    slide_data
}
//TODO: Work in progress
//The div could use a frame work like nth_font 
fn div_func(parser_cursor: &mut ParserCursor)->Vec<SlideData>{
    
    let mut config = None;
    let mut data = Vec::new(); 
    if is_keyword(parser_cursor, "#div(", true) {
        config = Some(gen_config_func(parser_cursor, 
                                     &[("size",     ConfigKwds::font_size, LexType::Num),
                                     ("pos_x", ConfigKwds::div_x, LexType::Num),
                                     ("pos_y", ConfigKwds::div_y, LexType::Num),
                                     ("height",   ConfigKwds::div_height, LexType::Num),
                                     ("width",   ConfigKwds::div_width, LexType::Num),
                                     ("position", ConfigKwds::div_position, LexType::Arr),
                                     ("align", ConfigKwds::div_align, LexType::Str),
                                     ("background_color",    ConfigKwds::div_background_color, LexType::Arr),
                                     ],
                                     "#div"));
        
        if parser_cursor.next() != Some('{'){
            return data;
        }
    }
    loop{ 
        parser_cursor.next(); 
        if is_keyword(parser_cursor, "#font(", true) {
            let mut font_data = font_func(parser_cursor);
            loop{
                if font_data.len() <= 0 {break;}
                data.push(font_data.pop().unwrap());
            }
        }
        else if is_keyword(parser_cursor, "#image(", true) {
            data.push(image_func(parser_cursor));
        }
        else if is_keyword(parser_cursor, "#tex(", true) || 
                is_keyword(parser_cursor, "#tex", true) {
            data.push(tex_func(parser_cursor));
        }
        else if is_keyword(parser_cursor, "#head", true) {
            data.push(header_func(parser_cursor));
        }
        else {
            let text_row = parser_cursor.row;
            data.push( SlideData{ config: config.clone(),
                                     kwd: ConfigKwds::text,
                                     data: gather_value(parser_cursor, LexType::SlideStr, Some(LexType::Braced)),
                                     text_row: text_row }); 
        }
    
        match parser_cursor.next(){
            Some('}') =>  break,
            None => break,
            Some(_)=>{}
        }
    } 
    for it in data.iter_mut(){
        let mut temp_config = config.clone().unwrap();
        if it.config.is_some(){
            let mut ref_config = it.config.as_mut().unwrap();
            ref_config.config_data.append(&mut temp_config.config_data);
        }
        else{
            it.config = config.clone();
        }
    }
    data.reverse();
    return data;
}


fn slide_config(parser_cursor: &mut ParserCursor)->ConfigCard{
    gen_config_func(parser_cursor, &[("background_color", ConfigKwds::slide_background_color, LexType::Arr),
                                     ("width", ConfigKwds::slide_width, LexType::Num),
                                     ("height", ConfigKwds::slide_height, LexType::Num),
                                     ("align", ConfigKwds::align, LexType::Str),
                                     ("valign", ConfigKwds::valign, LexType::Str),
                                     ("text_position", ConfigKwds::text_position, LexType::Arr),
                                     ("text_margin", ConfigKwds::text_margin, LexType::Num),
                                     ("font_color", ConfigKwds::font_color, LexType::Arr),
                                    ], "#slide")
}
fn slide_func(parser_cursor: &mut ParserCursor)->Card{
    let mut card = SlideCard{config:None, slide_data:Vec::new()};
    let mut init = false;
    
    loop{
        let current_char = parser_cursor.current().unwrap();
        if current_char == '#'{
            if is_keyword(parser_cursor, "#slide", true){
                if init == true{
                    parser_cursor.previous();
                    break;
                }
                if init == false{
                    init = true;

                    if is_keyword(parser_cursor, "#slide(", true) {
                        card.config = Some(slide_config(parser_cursor));
                        parser_cursor.next();
                    }
                    else{is_keyword(parser_cursor, "#slide", false);}

                }
            }
        }
        if init == true{
            
        //Maybe this should be moved out so that div can use the same code

            if is_keyword(parser_cursor, "#font(", true) {
                let mut font_data = font_func(parser_cursor);
                loop{
                    if font_data.len() <= 0 {break;}
                    card.slide_data.push(font_data.pop().unwrap());
                }
            }
            else if is_keyword(parser_cursor, "#image(", true) {
                card.slide_data.push(image_func(parser_cursor));
            }
            else if is_keyword(parser_cursor, "#tex(", true) || 
                    is_keyword(parser_cursor, "#tex", true) {
                card.slide_data.push(tex_func(parser_cursor));
            }
            else if is_keyword(parser_cursor, "#head", true) {
                card.slide_data.push(header_func(parser_cursor));
            }
            else if is_keyword(parser_cursor, "#div(", true){
                let mut div_data = div_func(parser_cursor);
                loop{
                    if div_data.len() <= 0 {break;}
                    card.slide_data.push(div_data.pop().unwrap());
                }
            }
            else {
                let text_row = parser_cursor.row;
                card.slide_data.push( SlideData{ config: None,
                                                 kwd: ConfigKwds::text,
                                                 data: gather_value(parser_cursor, LexType::SlideStr, None),
                                                 text_row: text_row }); 
            }

        /////////////////////////

        }
        if parser_cursor.next() == None { break; }
    }
    Card::SlideCard(card)
}

//TODO: This is here just to help me think about div_func
fn return_slide_data(parser_cursor: &mut ParserCursor )->Vec<SlideData>{

    let mut slide_data_vec = Vec::new();
    if is_keyword(parser_cursor, "#font(", true) {
        let mut font_data = font_func(parser_cursor);
        loop{
            if font_data.len() <= 0 {break;}
            slide_data_vec.push(font_data.pop().unwrap());
        }
    }
    else if is_keyword(parser_cursor, "#image(", true) {
        slide_data_vec.push(image_func(parser_cursor));
    }
    else if is_keyword(parser_cursor, "#tex(", true) || 
            is_keyword(parser_cursor, "#tex", true) {
        slide_data_vec.push(tex_func(parser_cursor));
    }
    else if is_keyword(parser_cursor, "#head", true) {
        slide_data_vec.push(header_func(parser_cursor));
    }
    else if is_keyword(parser_cursor, "#div(", true) {
        let mut div_data = div_func(parser_cursor);
        loop{
            if div_data.len() <= 0 {break;}
            slide_data_vec.push(div_data.pop().unwrap());
        }
    }
    else {
        slide_data_vec.push( SlideData{ config: None,
                                         kwd: ConfigKwds::text,
                                         data: gather_value(parser_cursor, LexType::SlideStr, None),
                                         text_row: parser_cursor.row}); 
    }
    slide_data_vec
}

fn gather_value(parser_cursor: &mut ParserCursor, expected_type: LexType, arr_type_or_braced: Option<LexType>)->ValueType{
    match expected_type{
        LexType::Num => {
            let mut value = String::from("");
            loop{
                for white_space in WHITE_SPACE.iter(){
                    if parser_cursor.current().unwrap() == *white_space{
                        parser_cursor.next();
                        continue;
                    }
                }
                if parser_cursor.current().unwrap() == ',' || parser_cursor.current().unwrap() == ')' || parser_cursor.current().unwrap() == '}' || parser_cursor.current().unwrap() == ']'{
                    parser_cursor.previous();
                   break;
                }
                else{
                    let mut good_number = false;
                    for num_char in ACC_NUM.iter(){
                        if parser_cursor.current().unwrap() == *num_char && good_number == false { good_number = true; }
                    }
                    if good_number == true { 
                        value.push(parser_cursor.current().unwrap()); 
                        if parser_cursor.peek().unwrap() == '.'{
                            parser_cursor.next();
                            value.push(parser_cursor.current().unwrap()); 
                        }
                    }
                    else {

                        let err_str = format!("Unexpected character: \"{}\", expected \"0-9\". Function gather_value line {} does not like.", parser_cursor.current().unwrap(), line!());
                        parse_error(&err_str, parser_cursor);
                        break;
                    }
                    parser_cursor.next();
                }
            }
            if value == "" { return ValueType::Err; }
            ValueType::Num(value.parse::<f64>().unwrap())
        },
        LexType::Arr => {
            let mut arr = Vec::<ValueType>::new();
            let mut value = String::from("");
            let mut init = false;
            loop{
                for white_space in WHITE_SPACE.iter(){
                    if parser_cursor.current().unwrap() == *white_space{
                        parser_cursor.next();
                        continue;
                    }
                }
                if parser_cursor.current().unwrap() == ')' || parser_cursor.current().unwrap() == '}' || parser_cursor.current().unwrap() == ']'{
                   break;
                }
                else{
                    if parser_cursor.current().unwrap() == ','{
                        parser_cursor.next();
                        continue;
                    }
                    if init == false{
                        if parser_cursor.current().unwrap() == '[' { init = true; } 
                    }
                    else{
                        if let Some(lex_type) = arr_type_or_braced{ 
                            let value = gather_value(parser_cursor, lex_type, None);
                            arr.push(value);
                        }
                        else{
                            let value = gather_value(parser_cursor, LexType::Num, None);
                            arr.push(value);
                        }
                    }
                    parser_cursor.next();
                }
            }
            ValueType::Arr(arr)
            //ValueType::Err 
        },
        LexType::Str => {
            let mut value = String::from("");
            let mut init = false;
            loop{
                if init == false {
                    if parser_cursor.current().unwrap() == '\"'{
                        init = true;
                    }
                parser_cursor.next();
                }
                else{
                    if parser_cursor.current().unwrap() == '\"'{ break; }
                    value.push(parser_cursor.current().unwrap());
                    parser_cursor.next();
                }
            }
            ValueType::Str(value)
        }
        LexType::SlideStr => { 
            let mut value = String::from("");
            let mut is_special_braced_str = false;
            if arr_type_or_braced.is_some(){
                is_special_braced_str = true;
            }
            if parser_cursor.file_string[parser_cursor.prev_pos] == '{'{
                is_special_braced_str = true;
            }
            loop{
                //TODO: 
                //What's going on here are we just breaking if we get a newline?
                if parser_cursor.current().unwrap() == '\n'{ 
                    if let Some(p) = parser_cursor.peek(){ 
                        if p == '\n'{ break; }
                        if p == '#' { break; }
                        if p == '}' { break; }
                    } else {break;}
                    //println!("New line continued {:?}", parser_cursor.peek());
                    if parser_cursor.next() == None { break; } //Redunent?
                    continue; 
                }

                // This is used to make sure we get out of Curlly braced #font(){STRINGS}
                if is_special_braced_str == true{
                    if parser_cursor.peek() == Some('}') { break; }
                }
                ///////// TODO:  In line alterations
                if parser_cursor.peek().unwrap() == '#' && parser_cursor.current().unwrap() == ' '{
                    if is_keyword(parser_cursor ," #font", true){
                        println!("I would break here!");
                        break;
                    }
                }
                /////////

                value.push(parser_cursor.current().unwrap());
                if parser_cursor.next() == None { break; };
            }
            ValueType::Str(value)
        },
        _ => ValueType::Err
    } 
}

fn config_func(parser_cursor: &mut ParserCursor)->Card{
    use self::LexType::*;
    let card = gen_config_func(parser_cursor,
                     &[("width", ConfigKwds::slide_width, Num),
                       ("height", ConfigKwds::slide_height, Num),
                       ("background_color", ConfigKwds::slide_background_color, Arr),
                       ("align", ConfigKwds::align, Str),
                       ("valign", ConfigKwds::valign, Str),
                       ("font", ConfigKwds::font, Str),
                       ("font_color", ConfigKwds::font_color, Arr),
                       ("text_position", ConfigKwds::text_position, Arr),
                       ("font_size", ConfigKwds::font_size, Num)],
                     "#config");
    Card::ConfigCard(card)
}

fn gen_config_func(parser_cursor: &mut ParserCursor, config_keywords: &[(&str, ConfigKwds, LexType)], keyword: &str)->ConfigCard{
    use self::LexType::*;
    let mut card = ConfigCard{config_data: Vec::new()};

    is_keyword(parser_cursor, keyword, false);
    if parser_cursor.current().unwrap() != '('{
        parse_error(&format!("expected ( for {:?}", parser_cursor.current()), parser_cursor);
        return card;
    }
    else{parser_cursor.next();}

    let mut keyword_primed = false;
    let mut value = Num;
    let mut kwd = ConfigKwds::default;
    loop{
        if parser_cursor.current().unwrap() == ')'{break;} 
        if WHITE_SPACE.contains(&parser_cursor.current().unwrap()){
            parser_cursor.next();
            continue;
        } 
        if keyword_primed == false{
            for keyword_value in config_keywords.iter(){
                let keyword = keyword_value.0;
                let previous_col = parser_cursor.col;
                let previous_row = parser_cursor.row;
                let previous_pos = parser_cursor.pos;
                if is_keyword(parser_cursor, keyword, false){
                    if parser_cursor.current().unwrap() == ' ' || parser_cursor.current().unwrap() == '='{
                        keyword_primed = true;
                        
                        value = keyword_value.2;
                        kwd = keyword_value.1;

                        card.config_data.push(ConfigData{kwd: keyword_value.1, data: ValueType::Err} );
                        parser_cursor.previous();
                        break;
                    }
                    else{
                        parser_cursor.col = previous_col;
                        parser_cursor.row = previous_row;
                        parser_cursor.pos = previous_pos;
                    }
                }
                else{
                    parser_cursor.col = previous_col;
                    parser_cursor.row = previous_row;
                    parser_cursor.pos = previous_pos;
                }
            }
        }
        else{
            let index = card.config_data.len() - 1; 
            if parser_cursor.current().unwrap() != '='{

                let error_str = format!("Unexpected character: \"{}\", expected \"=\".", parser_cursor.current().unwrap());
                parse_error(&error_str, parser_cursor);
                card.config_data[index] = ConfigData{ kwd: kwd, data: ValueType::Err };
                keyword_primed = false;
            } 
            else{
                parser_cursor.next(); 
                let mut arr_lextype: Option<LexType> = None;
                if value == LexType::Arr{ 
                    if kwd == ConfigKwds::font{ arr_lextype = Some(LexType::Str); }
                    else if kwd == ConfigKwds::font_color{ arr_lextype = Some(LexType::Num); }
                    else if kwd == ConfigKwds::slide_background_color{ arr_lextype = Some(LexType::Num); }
                } 
                let gathered_value = gather_value(parser_cursor, value, arr_lextype);

                card.config_data[index] = ConfigData{ kwd: kwd, data: gathered_value};
                keyword_primed =  false;
            }
        }
        parser_cursor.next();
    }
    card
}


fn read_contents(parser_cursor: &mut ParserCursor)->Card{
    //Is Looped until the end of file
    //if #slide do slide stuff
    //if #config do config stuff
    //else warning
    //words, look out for other ID funcs 
    let mut card = Card::Default;
    let current_char = parser_cursor.current().unwrap();
    if KEYWORD_IDENTIFIER.contains(&current_char){ 
        if is_slide(parser_cursor) { card = slide_func(parser_cursor); }
        if is_config(parser_cursor){ card = config_func(parser_cursor); } 
    }
    return card;
}

pub fn construct_document(input_string: Option<String>)->Vec<Card>{

    let mut slide_string = String::new();
    let example_string = String::from(
"
//Thoth Gunter
//Rust style comments

//Currently in temp

#config(
width = 254.0,
height = 190.5,
background_color = [100,0,100], //array style
font = [\"Times\", \"ASDF/aSDFA/FASDFA\"],
font_color = [200, 200, 200],
font_size = 32.0,
align = \"center\"
extern= \"/path\"
)


#slide
#header1 We can do headers #font(color=[200, 20, 10]) too!

We can write a slide like this. 
No need for a bracketted structure.

We can create a paragraph with consecutive \\n\\n.
We can create a new line with #newline

#slide
#div(width=0.5, pos_x=0.1){
ASDFTHOM
#font(color=[1,0,0]) Birds

Yup
}

#slide
#left_div{
SDFOMER #newline
ERADOFM
}
"
);

    if let Some(s) = input_string{ slide_string = s; } else{ slide_string = example_string}
    slide_string = remove_comments(slide_string);
    slide_string = replace_newline(slide_string);
    slide_string = replace_bullet(slide_string);
    slide_string = replace_divs(slide_string);

    let mut parser_cursor = ParserCursor::new(slide_string);

    println!("\n\nBEGINNING CARD GENERATION\n\n");

    let mut chars = Vec::<char>::new();
    let mut document_structure = Vec::<Card>::new();
    loop{
        
        //Push Cards in to document 
        {
            let card = read_contents(&mut parser_cursor);
            if card != Card::Default{
              document_structure.push(card); 
            }
            match parser_cursor.next(){
                Some(c) => {chars.push(c);},
                None => {break;}
            }
        }
        if parser_cursor.peek() == None{break;} 
    }
    document_structure
}




//TODO: This function maybe heavy on memory usage
fn remove_comments(contents: String)->String{
    let mut clean_contents = String::new();
    for line in contents.split('\n'){

        clean_contents += "\n";
        let mut length = line.len();
        if line == "" {
            continue;    
        }
        if line.contains("//"){
            if line.starts_with("//"){
                continue
            }
            else{
                match line.find("//"){
                    None=> {},
                    Some(l)=> length = l,
                };
            }
        }
        clean_contents += &line[0..length]; 
    }
    clean_contents
}


fn replace_newline(contents: String)->String{
    let mut clean_contents = String::new();
    clean_contents = contents.replace("#newline", "\n");
    clean_contents
}

fn replace_bullet(contents: String)->String{
    let mut clean_contents = String::new();
    clean_contents = contents.replace("\n#bul ", "\n\n\u{2022} ");
    clean_contents
}

//TODO
//Allow left_div and right_div parameters.
//look for () copy within () and paste that in the div command
fn replace_divs(contents: String)->String{
    let mut clean_contents = String::new();
    clean_contents = contents.replace("\n#left_div{",
                "\n#div(position=[0.05, 0.8], width=0.4, align=\"left\"){");
    clean_contents = clean_contents.replace("\n#right_div{",
                "\n#div(position=[0.505, 0.8], width=0.4, align=\"left\"){");
    clean_contents
}




