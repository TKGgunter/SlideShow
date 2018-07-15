use std::time::Instant;
use std::time::Duration;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

mod render_libharu;
use render_libharu::{render};

mod parser;
use parser::*; //{Card, ConfigCard, SlideCard, ConfigKwds, ValueType};



fn print_cards(cards: &Vec<Card>){
    for card in cards.iter(){
        match card{
            Card::SlideCard(slide)=> slide.print(),
            Card::ConfigCard(config)=>config.print(),
            _=>{}
        }
    }
    
}


fn main(){


    let file_path = Path::new("./temp.txt");

    if file_path.exists() {
        let mut last_modified = std::fs::metadata(file_path).unwrap().modified().unwrap();
        let mut init = false; 
        //I don't use this but it might be helpful later!
        //let mut elapsed_time = Instant::now();


        loop {
            //I don't use this but it might be helpful later!
            //let mut current_time =  elapsed_time.elapsed().as_secs() as f64 * 1000.0 + elapsed_time.elapsed().subsec_nanos() as f64 / 10.0f64.powf(6.0);

            if let Ok(Ok(modified)) = std::fs::metadata(&file_path).map(|m| m.modified())
            {
                if modified > last_modified || !init 
                {
                    println!("text file was modified... updating");
                    let mut file_contents = String::new();
                    {
                        let mut file = File::open(file_path).unwrap();
                        file.read_to_string(&mut file_contents).unwrap();
                    }


                    println!("{}", file_contents);

                    let cards = construct_document(Some(file_contents));
                    print_cards(&cards);

                    render_libharu::render(&cards);


                    println!("update complete");
                    init = true;
                    last_modified = modified;

                }
            }
            ::std::thread::sleep(Duration::new(0, 16 * 1_000_000 ));
        }
    
    }
    else{
        let cards = construct_document(None);
        print_cards(&cards);
    
    }



}

