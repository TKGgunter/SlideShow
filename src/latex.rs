use std::fs::File;
use std::process::Command;
use std::io::Write;

//File names?


pub fn run_external(){
    if cfg!(target_os = "windows") {
        println!("You're shit out of luck.");
    } 
    let output = Command::new("sh")
                        .arg("-c")
                        .arg("echo hello")
                        .output()
                        .expect("failed to execute process");

    let hello = output.stdout;
    println!("{:?}", hello );
}

fn construct_latex_file(font_color: &[f64; 3] ,tex_string: Option<String>, name_string: &String){
    let mut prepend = String::from(
"\\documentclass[32pt]{article}
\\usepackage{lingmacros}
\\usepackage{tree-dvips}
\\usepackage{xcolor}
\\pagestyle{empty}
\\usepackage{geometry}
 \\geometry{
      a4paper,
      total={170mm,257mm},
      left=5mm,
      top=5mm,
}");

    prepend.push_str(&format!("\\definecolor{{custom}}{{RGB}}{{ {}, {}, {} }}", font_color[0] as u32,
                                                                                   font_color[1] as u32, 
                                                                                   font_color[2] as u32));

    prepend.push_str("\\begin{document}\n\\color{custom}\n");

    let postpend = String::from("\\end{document}");

    let mut body = String::new();
    match tex_string{
        None => {
            body = String::from("\\section*{Notes for My Paper} \n some temp shit.");
        }
        Some(_string) => {
            body.push_str("\\section*{Notes for My Paper}");
            body = _string;
        }
    }

    let file_str = format!("{}{}{}", prepend, body, postpend);
    let mut file = File::create(format!("{}.tex", name_string)).unwrap();
    file.write_all(file_str.as_bytes()).unwrap();
}



pub fn run_latex(tex_string: Option<String>, name_string: String)->Vec<u8>{

    construct_latex_file(&[1.0, 1.0, 1.0], tex_string, &name_string);
    if cfg!(target_os = "windows") {
        println!("You're shit out of luck.\nThis program is not configured to run latex on windows.");
        return vec![0u8];
    } 
    let output = Command::new("latex")
                        .arg("-interaction=nonstopmode")
                        .arg(&format!("{}.tex", name_string))
                        .output()
                        .expect("failed to execute process");
    output.stdout
}



pub fn run_dvipng(file_name: String)->Vec<u8>{

    if cfg!(target_os = "windows") {
        println!("You're shit out of luck.\n This program is not configured to run dvipng on windows.");
        return vec![0u8];
    } 
    let output = Command::new("dvipng")
                        .args(&["-T tight", "-D 640", 
                              "-bg", "Transparent", 
                              &format!("{}.dvi", file_name)])
                        .output()
                        .expect("failed to execute process");

    //println!("{:?}", String::from_utf8(output.stdout.clone()));
    output.stdout
}


pub fn clean_tex(file_name: Option<String>){
    let mut name = String::new();
    match file_name {
        Some(s) => name = s,
        None => name = String::from("output")
    }
    if cfg!(target_os = "windows") {
        println!("You're shit out of luck.\n This program is not configured to run dvipng on windows.");
        return
    } 
    Command::new("rm")
            .args(&[format!("{}.tex", name.clone()),
                    format!("{}.dvi", name.clone()),
                    format!("{}.aux", name.clone())
            ])
            .output()
            .expect("failed to execute process");

}
