//Thoth Gunter

//ToDo:
//div
//image path 
//=======Move to Slide Show ================
//latex strings
//headers, bullets, newline <= for new line we might not need to do any thing, but we must remove
//\n and \t s.


#![allow(dead_code)]
extern crate ansi_term;

struct ParserCursor{
    file_string: String,
    col: usize,
    row: usize,
    pos: usize, 
    prev_col: usize,
    prev_row: usize,
    prev_pos: usize, 
}

impl ParserCursor{
    fn new()->ParserCursor{
        ParserCursor{
            file_string: String::from(""),
            col: 0,
            row: 0,
            pos: 0,
            prev_col: 0,
            prev_row: 0,
            prev_pos: 0,
        }
    }


    //I SHOULD NOT BE REMAKEING THESE COLLECTIONS EVERY TIME WANT THE NEXT OR CURRENT CHARACTER
    fn peek(&self)->Option<char>{
        let arr_char: Vec<char> = self.file_string.chars().collect();
        if self.pos + 1 < arr_char.len(){
            Some(arr_char[self.pos + 1])
        }
        else{
            println!("Ran out of space to peek");
            None
        }
    }

    fn current(&mut self)->Option<char>{
        let arr_char: Vec<char> = self.file_string.chars().collect();
        if self.pos < arr_char.len(){
            Some(arr_char[self.pos])
        }
        else{
            println!("Ran out of space to current");
            None
        }
    }

    fn next(&mut self)->Option<char>{
        let arr_char: Vec<char> = self.file_string.chars().collect();
        self.prev_col = self.col;
        self.prev_row = self.row;
        self.prev_pos = self.pos;
        if self.pos + 1 < arr_char.len(){
            self.pos += 1;
            if arr_char[self.pos] == '\n'{
                self.row += 1;
            }
            else{
                self.col += 1;
            }
            Some(arr_char[self.pos])
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
    Punc,
    Num,
    Str,
    Kw,
    Var,
    Op,
    Id,
    Arr,
    SlideStr,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType{
    Num(f64),
    Str(String),
    Arr(Vec<ValueType>),
    Err,
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConfigKwds{
    width,
    height,
    background_color,
    align,
    text,
    tex,
    font,
    font_color,
    font_size,
    font_current,
    font_position,
    font_style,
    font_nth,
    font_margin,
    tex_color,
    tex_size,
    tex_current,
    tex_position,
    tex_style,
    tex_nth,
    tex_margin,
    image,
    image_path,
    image_position,
    image_width,
    image_height,
    default,
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
}
#[derive(Debug, PartialEq)]
pub struct SlideCard{
    pub config: Option<ConfigCard>,
    pub slide_data: Vec<SlideData>,
}


#[derive(Debug, PartialEq)]
pub enum Card{
    SlideCard(SlideCard),
    ConfigCard(ConfigCard),
    Default,
}



static ACC_PUNC: [char; 7]  =  ['{', '}', '(', ')', '.', ',', ';'];
static ACC_NUM:  [char; 10] =  ['0','1','2','3','4','5','6','7','8','9'];
static ACC_OP:   [&str; 5]  =  ["=","+", "-", "/", "*"];
static ACC_STR:  [char; 1]  =  ['"'];
static ACC_ID:   [char; 1]  =  ['#'];

static WHITE_SPACE:[char; 3] = [' ','\t', '\n'];

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
                                 ("height", ConfigKwds::image_height, LexType::Num),
],
                                 "#image");
    parser_cursor.next();
    let slide_data = ValueType::Err; 
    let data = SlideData{config: Some(config),
                         kwd: ConfigKwds::image,
                         data: slide_data};
    data
}

fn font_func(parser_cursor: &mut ParserCursor)->Vec<SlideData>{
    let mut config = gen_config_func(parser_cursor, 
                                 &[("family", ConfigKwds::font_current, LexType::Str),
                                 ("size", ConfigKwds::font_size, LexType::Num),
                                 ("position", ConfigKwds::font_position, LexType::Arr),
                                 ("color", ConfigKwds::font_color, LexType::Arr),
                                 ("style", ConfigKwds::font_style, LexType::Str),
                                 ("margin", ConfigKwds::font_margin, LexType::Num),
                                 ],
                                 "#font");
    parser_cursor.next();
    for i in 0..config.config_data.len(){
        if config.config_data[i].kwd == ConfigKwds::font_position{
            config.config_data.push(ConfigData{ kwd: ConfigKwds::font_nth, data: ValueType::Num(0.0)});
            break;
        }
    }

    let mut return_data = Vec::new();
    if parser_cursor.current() != Some('{'){
        config.config_data.push(ConfigData{ kwd: ConfigKwds::font_nth, data: ValueType::Num(0.0)});
        let mut data = SlideData{  config: Some(config),
                                   kwd: ConfigKwds::text,
                                   data: ValueType::Err}; 
        let slide_data = gather_value(parser_cursor, LexType::SlideStr, None);
        data.data = slide_data; 
        return_data.push(data);
    }
    else{
        parser_cursor.next();
        let mut nth = 0;
        loop{
            if parser_cursor.current() == Some('}'){break;}
            else{
                let mut _config = config.clone();
                _config.config_data.push(ConfigData{ kwd: ConfigKwds::font_nth, data: ValueType::Num(nth as f64)});
                let mut data = SlideData{  config: Some(_config),
                                           kwd: ConfigKwds::text,
                                           data: ValueType::Err}; 
                let slide_data = gather_value(parser_cursor, LexType::SlideStr, None);
                data.data = slide_data; 
                return_data.push(data);

                parser_cursor.next();
                nth += 1;
            }
        }
    }
    return_data
}

fn tex_func(parser_cursor: &mut ParserCursor)->SlideData{
    let mut config = ConfigCard{config_data: Vec::new()};
    if is_keyword(parser_cursor, "#tex(", true) {
        config = gen_config_func(parser_cursor, 
                                     &[("family", ConfigKwds::tex_current, LexType::Str),
                                     ("size",     ConfigKwds::tex_size, LexType::Num),
                                     ("position", ConfigKwds::tex_position, LexType::Arr),
                                     ("color",    ConfigKwds::tex_color, LexType::Arr),
                                     ("style",    ConfigKwds::tex_style, LexType::Str),
                                     ("margin",   ConfigKwds::tex_margin, LexType::Num),
                                     ],
                                     "#tex");
    }
    else {
        is_keyword(parser_cursor, "#tex", false);
    }

    let mut slide_data = SlideData{  config: Some(config),
                               kwd: ConfigKwds::tex,
                               data: ValueType::Err}; 
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
    println!("\n\n\ntexFunc slide data!!! {:?} \n\n\n\n", slide_data);
    slide_data
}
//Work in progress
fn div_func(parser_cursor: &mut ParserCursor)->SlideData{
    println!("Div function");
    
    let config = gen_config_func(parser_cursor, 
                                 &[("font", ConfigKwds::font, LexType::Str)],
                                 "#div");
    parser_cursor.next();
    let slide_data = gather_value(parser_cursor, LexType::SlideStr, None);
    let data = SlideData{config: Some(config),
                         kwd: ConfigKwds::text,
                         data: slide_data};
    data
}

fn slide_config(parser_cursor: &mut ParserCursor)->ConfigCard{
    gen_config_func(parser_cursor, &[("background_color", ConfigKwds::background_color, LexType::Arr)], "#slide")
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
            else {
                card.slide_data.push( SlideData{ config: None, kwd: ConfigKwds::text, data: gather_value(parser_cursor, LexType::SlideStr, None) }); 
            }
        }
        if parser_cursor.next() == None { break; }
    }
    Card::SlideCard(card)
}



fn gather_value(parser_cursor: &mut ParserCursor, expected_type: LexType, arr_type: Option<LexType>)->ValueType{
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
                        if let Some(lex_type) = arr_type{ 
                            let value = gather_value(parser_cursor, lex_type, None);
                            arr.push(value);
                        }
                        else{
                            println!("No array value given!");
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
            loop{
                if parser_cursor.current().unwrap() == '\n'{ 
                    if let Some(p) = parser_cursor.peek(){ 
                        if p == '\n'{ break; }
                        if p == '#' { break; }
                        if p == '}' { break; }
                    } else {break;}
                    if parser_cursor.next() == None { break; } //Redunant?
                    value.push(' ');
                    continue; 
                }
                value.push(parser_cursor.current().unwrap());
                if parser_cursor.next() == None {break; };
            }
            ValueType::Str(value)
        },
        _ => ValueType::Err
    } 
}

fn config_func(parser_cursor: &mut ParserCursor)->Card{
    use self::LexType::*;
    let card = gen_config_func(parser_cursor,
                     &[("width", ConfigKwds::width, Num),
                       ("height", ConfigKwds::height, Num),
                       ("background_color", ConfigKwds::background_color, Arr),
                       ("align", ConfigKwds::align, Str),
                       ("font", ConfigKwds::font, Str),
                       ("font_color", ConfigKwds::font_color, Arr)],
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

                        //println!("Is a Keyword: {:?} {:?}", keyword, value);
                        
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
                card.config_data[index] = ConfigData{ kwd: kwd, data: ValueType::Err};
                keyword_primed = false;
            } 
            else{
                parser_cursor.next(); 
                let mut arr_lextype: Option<LexType> = None;
                if value == LexType::Arr{ 
                    if kwd == ConfigKwds::font{ arr_lextype = Some(LexType::Str);}
                    else if kwd == ConfigKwds::font_color{ arr_lextype = Some(LexType::Num);}
                    else if kwd == ConfigKwds::background_color{ arr_lextype = Some(LexType::Num);}
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
    if ACC_ID.contains(&current_char){ 
        if is_slide(parser_cursor) { card = slide_func(parser_cursor); }
        if is_config(parser_cursor){ card = config_func(parser_cursor); } 
    }
    return card;
}

pub fn example(input_string: Option<String>)->Vec<Card>{

    let mut slide_string = String::new();
    let example_string = String::from(
"
//Thoth Gunter
//Rust style comments


#config(
width = 254.0,
height = 190.5,
background_color = [100,0,100], //array style
font = [\"Times\", \"ASDF/aSDFA/FASDFA\"],
font_color = [200, 200, 200],
align = \"center\"
extern= \"/path\"
)


#slide
a

We can write a slide like this. 
No need for a bracketted structure.

We can create a paragraph with consecutive \\n\\n.
We can create a new line with #newline


#slide(background_color= [150,0,150])
We can also change slide configurations for specific slides

#slide
#font(size=42) We can do different sized text
#font(position=[0.3, 0.5], color=[200,50,50]) Place text where you want
#font(family=\"Times\", size=32, style=\"bold\", position=[0.2, 0.2]) We can change fonts!

#slide
#font(position=[122, 40], margin=0.5){
Something, something, something else and another thing

ASDFAF
}

#slide
We can even add images
#image(path=\"/some/path\")
#image(path=\"/some/path\", position=[0.7, 0.1], width=0.8, height=0.8)

#slide(background_color=[50,0,50])
New slides are easy!
#bul So are images!
#font(position=[0.9, 0.9]){
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
}

#font(position=[0.2,0.5], size=32) And we have auto wrap!

#image(path=\"some.img\", position=[0.6,0.6])
#font(position=[0.6, 0.55]) Oops, well we have default images :)

#slide
We can do tex functions and tables too

#tex y = m x + b 
#tex{ 
\\begin{center}
\\begin{tabular}{ c c c }
 cell1 & cell2 & cell3 \\ 
 cell4 & cell5 & cell6 \\  
 cell7 & cell8 & cell9    
\\end{tabular}
\\end{center}
}

adfadf
"
);

    if let Some(s) = input_string{ slide_string = s; } else{ slide_string = example_string}
    let mut parser_cursor = ParserCursor::new();
    slide_string = remove_comments(slide_string);
    slide_string = replace_bullet(slide_string);

    parser_cursor.file_string = slide_string;

    println!("FILE:\n\n{}\n\nBEGINNING CARD GENERATION", parser_cursor.file_string);

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



fn replace_bullet(contents: String)->String{
    let mut clean_contents = String::new();
    clean_contents = contents.replace("\n#bul ", "\n\n\u{2022} ");
    clean_contents
}
