


mod parser;
use parser::*; //{Card, ConfigCard, SlideCard, ConfigKwds, ValueType};



fn main(){
    println!("OK");

    let cards = construct_document(None);
    for card in cards.iter(){
        match card{
            Card::SlideCard(slide)=> slide.print(),
            Card::ConfigCard(config)=>config.print(),
            _=>{}
        }
    }
}

