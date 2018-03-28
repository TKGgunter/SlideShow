use std::process::Command;

pub fn run_external(){
    let output = if cfg!(target_os = "windows") {
        println!("You are shit out of luck.");
    } else {
            Command::new("sh")
                        .arg("-c")
                        .arg("echo hello")
                        .output()
                        .expect("failed to execute process")
    };

    let hello = output.stdout;
}
