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



fn construct_latex_file(tex_string: Option<String>){
    let prepend = String::from(
"\\documentclass[12pt]{article}
\\usepackage{lingmacros}
\\usepackage{tree-dvips}
\\begin{document}"); 

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
    let mut file = File::create("output.tex").unwrap();
    file.write_all(file_str.as_bytes()).unwrap();
}



pub fn run_latex(tex_string: Option<String>)->Vec<u8>{

    construct_latex_file(tex_string);
    if cfg!(target_os = "windows") {
        println!("You're shit out of luck.\nThis program is not configured to run latex on windows.");
        return vec![0u8];
    } 
    let output = Command::new("latex")
                        .arg("-interaction=nonstopmode")
                        .arg("output.tex")
                        .output()
                        .expect("failed to execute process");
    output.stdout
}



pub fn run_dvipng(file_name: Option<String>)->Vec<u8>{
    let mut name = String::new();
    match file_name {
        Some(s) => name = s,
        None => name = String::from("output")
    }

    if cfg!(target_os = "windows") {
        println!("You're shit out of luck.\n This program is not configured to run dvipng on windows.");
        return vec![0u8];
    } 
    let output = Command::new("dvipng")
                        .args(&["-q", "-T tight"])
                        .arg("output.dvi")
                        .output()
                        .expect("failed to execute process");

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
            .args(&[format!("{}.tex", name.clone()), format!("{}.dvi", name.clone()), format!("{}.aux", name.clone())])
            .output()
            .expect("failed to execute process");

}
