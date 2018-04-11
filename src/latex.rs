use std::fs::File;
use std::process::Command;
use std::io::Write;

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

fn construct_latex_file(font_color: &[f64; 3] ,tex_string: Option<String>){
    let mut prepend = String::from(
"\\documentclass[12pt]{article}
\\usepackage{lingmacros}
\\usepackage{tree-dvips}
\usepackage{xcolor}");

    prepend.push_str(format!("\definecolor\{custom\}\{RGB\}\{{}, {}, {}\}", font_color[0] as u32,
                                                                            font_color[1] as u32,

    prepend.push_str("\\begin{document}\n\color{custom}\n");

    let postpend = String::from("\\end{document}");

    let mut body = String::new();
    if tex_string == None{
        body = String::from("\\section*{Notes for My Paper} \n some temp shit.");
    }
    else{
        body = tex_string.unwrap();
    }

    let file_str = format!("{}{}{}", prepend, body, postpend);
    let mut file = File::create("output.tex").unwrap();
    file.write_all(file_str.as_bytes()).unwrap();
}



pub fn run_latex(tex_string: Option<String>)->Vec<u8>{

    if cfg!(target_os = "windows") {
        println!("You're shit out of luck.\nThis program is not configured to run latex on windows.");
    } 
    let output = Command::new("latex")
                        .arg("-interaction=nonstopmode")
                        .arg("output.tex")
                        .output()
                        .expect("failed to execute process");
    output.stdout
}



pub fn run_dvipng(file_name: Option<String>)->Vec<u8>{

    if cfg!(target_os = "windows") {
        println!("You're shit out of luck.\n This program is not configured to run dvipng on windows.");
    } 
    let output = Command::new("dvipng")
                        .args(&["-q", "-T tight"])
                        .arg("output.dvi")
                        .output()
                        .expect("failed to execute process");

    output.stdout
}
