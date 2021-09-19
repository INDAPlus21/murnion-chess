/// A struct implementing the full state of the chess board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    board: Vec<Vec<Piece>>,
    current_turn: Colour,
    castlings: (bool, bool, bool, bool),
    en_passant_square: (usize, usize),
    halfmove_clock: usize,
    turn: usize,
}

impl Game {
    /// Creates a new game board, with standard starting positions.
    fn new() -> Game {
        let mut game = Game {
            board: vec!(vec!(Piece::Empty)),
            current_turn: Colour::White,
            castlings: (true, true, true, true),
            en_passant_square: (0, 0),
            halfmove_clock: 0,
            turn: 0,
        };
        game.set_state_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        game
    }

    fn new_empty() -> Game {
        Game {
            board: vec!(vec!(Piece::Empty)),
            current_turn: Colour::White,
            castlings: (true, true, true, true),
            en_passant_square: (8, 8),
            halfmove_clock: 0,
            turn: 1,
        }
    }

    /// Sets the game state using a FEN-notated string.
    /// Note that currently it does not check for nor handle any case wherein the string given is not in FEN-notation.
    /// 
    /// # Arguments
    /// 
    /// * `fen` - string in FEN-notation containing the desired state of the chess game.
    fn set_state_from_fen(&mut self, fen: &str) {
        let fen_split = fen.split(" ").map(|_s| _s.to_string()).collect::<Vec<String>>();
        // assert_eq!(fen_split.len(), 6, "Given invalid string when attempting to set state from FEN notaion.");
        self.board = {
            fen_split[0].split("/")
                        .map(|_rank| { 
                            let mut c_rank = Vec::new();
                            for _char in _rank.chars() { match _char {
                                'K' => c_rank.push(Piece::King(Colour::White)),
                                'k' => c_rank.push(Piece::King(Colour::Black)),
                                'Q' => c_rank.push(Piece::Queen(Colour::White)),
                                'q' => c_rank.push(Piece::Queen(Colour::Black)),
                                'R' => c_rank.push(Piece::Rook(Colour::White)),
                                'r' => c_rank.push(Piece::Rook(Colour::Black)),
                                'B' => c_rank.push(Piece::Bishop(Colour::White)),
                                'b' => c_rank.push(Piece::Bishop(Colour::Black)),
                                'N' => c_rank.push(Piece::Knight(Colour::White)),
                                'n' => c_rank.push(Piece::Knight(Colour::Black)),
                                'P' => c_rank.push(Piece::Pawn(Colour::White)),
                                'p' => c_rank.push(Piece::Pawn(Colour::Black)),
                                _ => for _ in 0.._char.to_digit(10).unwrap() as usize { c_rank.push(Piece::Empty); },
                            }};
                            return c_rank
                        }).collect::<Vec<Vec<Piece>>>()
        };
        self.current_turn = match fen_split[1].chars().collect::<Vec<char>>()[0] {
            'w' => Colour::White,
            'b' => Colour::Black,
            _ => panic!(),
        };
        self.castlings = (fen_split[2].contains('K'), fen_split[2].contains('Q'), fen_split[2].contains('k'), fen_split[2].contains('q'));
        self.en_passant_square = {
            let fen_chars = fen_split[3].chars().collect::<Vec<char>>();
            if fen_chars[0] == '-' {
                let square = (8, 8);
                square
            } else {
            let x: usize = match fen_chars[0] {
                'a' => 7,
                'b' => 6,
                'c' => 5,
                'd' => 4,
                'e' => 3,
                'f' => 2,
                'g' => 1,
                'h' => 0,
                _ => panic!(),
            };
            let y: usize = fen_chars[1].to_digit(10).unwrap() as usize;
            let square = (x, 8 - y);
            square
            }
        };
        self.halfmove_clock = fen_split[4].parse::<usize>().unwrap();
        self.turn = fen_split[5].parse::<usize>().unwrap();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Piece {
    King(Colour),
    Queen(Colour),
    Rook(Colour),
    Bishop(Colour),
    Knight(Colour),
    Pawn(Colour),
    Empty,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Colour {
    White,
    Black
}

#[cfg(test)]
mod game_tests {
    #[test]
    fn fen_sets_correctly() {
        use crate::Piece;
        use crate::Game;
        use crate::Colour;

        let fen_game = Game::new();
        let mut test_game = Game::new_empty();
        
        let _board = vec![
        vec![Piece::Rook(Colour::Black), Piece::Knight(Colour::Black), Piece::Bishop(Colour::Black), Piece::Queen(Colour::Black), Piece::King(Colour::Black), Piece::Bishop(Colour::Black), Piece::Knight(Colour::Black), Piece::Rook(Colour::Black)],
        vec![Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black)],
        vec![Piece::Empty; 8],
        vec![Piece::Empty; 8],
        vec![Piece::Empty; 8],
        vec![Piece::Empty; 8],
        vec![Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Pawn(Colour::White)],
        vec![Piece::Rook(Colour::White), Piece::Knight(Colour::White), Piece::Bishop(Colour::White), Piece::Queen(Colour::White), Piece::King(Colour::White), Piece::Bishop(Colour::White), Piece::Knight(Colour::White), Piece::Rook(Colour::White)],
        ];
        test_game.board = _board;

        assert_eq!(fen_game, test_game);
    }
}