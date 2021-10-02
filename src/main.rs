mod lib;

use text_io::read;
use lib::Game;
use lib::GameState;

fn main() {
    let mut game = Game::new();

    loop {
        fen_into_display(game.get_fen());
        println!("");
        println!("{:?}", game.game_state());

        if game.game_state() == GameState::Checkmate {println!("Game over!");}

        let line: String = read!("{}\n");

        if line == "exit" || game.game_state() == GameState::Checkmate {break;}
        game.take_turn(line);
    }
}
   
#[macro_use]
mod macros {
    macro_rules! match_to_letter {
        ($ch:ident) => {
            match $ch {
                a, b, c, d, e, f, g, h => true,
                _ => correct_input => false 
            }
        };
    }
}


fn fen_into_display(fen: String) {
    let split_string = fen.split(" ").collect::<Vec<&str>>();
    let board = split_string[0];
    let mut turn = split_string[1];

    let ranks = board.split("/").collect::<Vec<&str>>();

    if turn == "b" { turn = "Black"; } else { turn = "White"; }
    println!("Current turn: {}", turn);
    println!("");
    println!("   a b c d e f g h");
    println!(" ");
    for x in 0..8 {
        print!("{}  ", 8 - x);
        for ch in ranks[x].chars().collect::<Vec<char>>() {
            match ch {
                'K' => print!("K "),
                'k' => print!("k "),
                'Q' => print!("Q "),
                'q' => print!("q "),
                'R' => print!("R "),
                'r' => print!("r "),
                'B' => print!("B "),
                'b' => print!("b "),
                'N' => print!("N "),
                'n' => print!("n "),
                'P' => print!("P "),
                'p' => print!("p "),
                _ => for _ in 0..ch.to_digit(10).unwrap() as usize { print!("  "); },
            }
        }
        println!();
    }
}