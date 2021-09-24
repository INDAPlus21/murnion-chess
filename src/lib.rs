mod tests;

/// A struct implementing the full state of the chess board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    board: Vec<Vec<Piece>>,
    current_turn: Colour,
    castlings: (bool, bool, bool, bool),
    en_passant_square: (usize, usize),
    halfmove_clock: usize,
    turn: usize,
    selected_promotion: Piece,
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
            selected_promotion: Piece::Queen(Colour::White),
        };
        game.set_state_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        game
    }

    /// Creates a new game board, with no pieces on it.
    fn new_empty() -> Game {
        Game {
            board: vec!(vec!(Piece::Empty)),
            current_turn: Colour::White,
            castlings: (true, true, true, true),
            en_passant_square: (8, 8),
            halfmove_clock: 0,
            turn: 1,
            selected_promotion: Piece::Queen(Colour::White),
        }
    }

    /// Sets the game state using a FEN-notated string.
    /// Note that currently it does not check for nor handle any case wherein the string given is not in FEN-notation.
    /// 
    /// # Arguments
    /// 
    /// * `fen` - string in FEN-notation containing the desired state of the chess game.
    pub fn set_state_from_fen(&mut self, fen: &str) {
        let fen_split = fen.split(" ").map(|_s| _s.to_string()).collect::<Vec<String>>();
        assert_eq!(fen_split.len(), 6, "Given invalid string when attempting to set state from FEN notaion.");
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
                'a' => 0,
                'b' => 1,
                'c' => 2,
                'd' => 3,
                'e' => 4,
                'f' => 5,
                'g' => 6,
                'h' => 7,
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

/// Enumerable that holds the state of a single piece on the board, with awareness of how it moves and captures.
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

impl Piece {
    /// Functions the same as get_valid_moves, but only returns the surrounding squares for Kings.
    /// Used for making sure there's no endless recursion when checking for checks.
    fn get_threatened_squares(&self, pos: (usize, usize), board: &Vec<Vec<Piece>>) -> Vec<(usize, usize)> {
        match self {
            Piece::King(_colour) => {
                let mut moves = Vec::new();
                for x in 0..3 {
                    if pos.0 + x == 0 { continue; }
                    for y in 0..3 {
                        if pos.1 + y == 0 { continue; }
                        moves.push((pos.0 + x - 1, pos.1 + y - 1));
                    }
                }
                moves
            },
            _ => {
                self.get_valid_moves(pos, board, (8, 8))
            },
        }
    }

    /// The public function to return any valid moves for the single piece it is called from. Does not check for check.
    fn get_valid_moves(&self, pos: (usize, usize), board: &Vec<Vec<Piece>>, en_passant_square: (usize, usize)) -> Vec<(usize, usize)> {
        match self {
            Piece::Empty => Vec::new(),
            Piece::Queen(_colour) => {
                let mut moves = Vec::new();
                moves.append(&mut self.get_rook_moves(pos, board));
                moves.append(&mut self.get_bishop_moves(pos, board));
                moves
            },
            Piece::Rook(_colour) => {
                self.get_rook_moves(pos, board)
            },
            Piece::Bishop(_colour) => {
                self.get_bishop_moves(pos, board)
            },
            Piece::Knight(_colour) => {
                self.get_knight_moves(pos, board)
            },
            Piece::Pawn(_colour) => {
                self.get_pawn_moves(pos, board, en_passant_square)
            },
            Piece::King(_colour) => panic!(),
        }
    }

    /// Internal helper function which shouldn't be used outside of Piece implementation.
    /// Retrieves valid moves as if the piece is a rook.
    /// Moves are returned as a non-sorted list of usize tuples.
    /// 
    /// # Arguments
    /// 
    /// * `pos`: The position of the piece that moves are gotten from. In usize tuple format.
    /// * `board`: The board. A 2d vector of Pieces.
    fn get_rook_moves(&self, pos: (usize, usize), board: &Vec<Vec<Piece>>) -> Vec<(usize, usize)>{
        let mut moves = Vec::new();
        for number in 1..8 {
            if pos.1 + number >= 8 { break; }
            if board[pos.0][ pos.1 + number] == Piece::Empty {
                moves.push((pos.0, pos.1 + number));
            } else {
                if board[pos.0][pos.1 + number].get_colour().unwrap() == self.get_colour().unwrap() {
                    break;
                } else {
                    moves.push((pos.0, pos.1 + number));
                    break;
                }
            }
        }
        for number in 1..8 {
            if pos.0 + number >= 8 { break; }
            if board[pos.0 + number][pos.1] == Piece::Empty {
                moves.push((pos.0 + number, pos.1));
            } else {
                if board[pos.0 + number][pos.1].get_colour().unwrap() == self.get_colour().unwrap() {
                    break;
                } else {
                    moves.push((pos.0 + number, pos.1));
                    break;
                }
            }
        }
        for number in 1..8 {
            if pos.1 + 1 - number == 0 { break; }
            if board[pos.0][pos.1 - number] == Piece::Empty {
                moves.push((pos.0, pos.1 - number));
            } else {
                if board[pos.0][pos.1 - number].get_colour().unwrap() == self.get_colour().unwrap() {
                    break;
                } else {
                    moves.push((pos.0, pos.1 - number));
                    break;
                }
            }
        }
        for number in 1..8 {
            if pos.0 + 1 - number == 0 { break; }
            if board[pos.0 - number][pos.1] == Piece::Empty {
                moves.push((pos.0 - number, pos.1));
            } else {
                if board[pos.0 - number][pos.1].get_colour().unwrap() == self.get_colour().unwrap() {
                    break;
                } else {
                    moves.push((pos.0 - number, pos.1));
                    break;
                }
            }
        }
        moves
    }

    /// Internal helper function which shouldn't be used outside of Piece implementation.
    /// Retrieves valid moves as if the piece is a bishop.
    /// Moves are returned as a non-sorted list of usize tuples.
    /// 
    /// # Arguments
    /// 
    /// * `pos`: The position of the piece that moves are gotten from. In usize tuple format.
    /// * `board`: The board. A 2d vector of Pieces.
    fn get_bishop_moves(&self, pos: (usize, usize), board: &Vec<Vec<Piece>>) -> Vec<(usize, usize)>{
        let mut moves = Vec::new();
        for number in 0..8 {
            if pos.0 + number == 8 || pos.1 + number == 8 { break; }
            if board[pos.0 + number][pos.1 + number] == Piece::Empty {
                moves.push((pos.0 + number, pos.1 + number));
            } else {
                if board[pos.0 + number][pos.1 + number].get_colour().unwrap() == self.get_colour().unwrap() {
                    break;
                } else {
                    moves.push((pos.0 + number, pos.1 + number));
                    break;
                }
            }
        };
        for number in 0..8 {
            if pos.0 + number == 8 || pos.1 + 1 - number == 0 { break; }
            if board[pos.0 + number][pos.1 - number] == Piece::Empty {
                moves.push((pos.0 + number, pos.1 - number));
            } else {
                if board[pos.0 + number][pos.1 - number].get_colour().unwrap() == self.get_colour().unwrap() {
                    break;
                } else {
                    moves.push((pos.0 + number, pos.1 - number));
                    break;
                }
            }
        };
        for number in 0..8 {
            if pos.0 + 1 - number == 0 || pos.1 + number == 8 { break; }
            if board[pos.0 - number][pos.1 + number] == Piece::Empty {
                moves.push((pos.0 - number, pos.1 + number));
            } else {
                if board[pos.0 - number][pos.1 + number].get_colour().unwrap() == self.get_colour().unwrap() {
                    break;
                } else {
                    moves.push((pos.0 - number, pos.1 + number));
                    break;
                }
            }
        };
        for number in 0..8 {
            if pos.0 + 1 - number == 0 || pos.1 + 1 - number == 0 { break; }
            if board[pos.0 - number][pos.1 - number] == Piece::Empty {
                moves.push((pos.0 - number, pos.1 - number));
            } else {
                if board[pos.0 - number][pos.1 - number].get_colour().unwrap() == self.get_colour().unwrap() {
                    break;
                } else {
                    moves.push((pos.0 - number, pos.1 - number));
                    break;
                }
            }
        };
        moves
    }

    /// Internal helper function which shouldn't be used outside of Piece implementation.
    /// Retrieves valid moves as if the piece is a knight.
    /// Moves are returned as a non-sorted list of usize tuples.
    /// 
    /// # Arguments
    /// 
    /// * `pos`: The position of the piece that moves are gotten from. In usize tuple format.
    /// * `board`: The board. A 2d vector of Pieces.
    fn get_knight_moves(&self, pos: (usize, usize), board: &Vec<Vec<Piece>>) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        if pos.0 > 0 && pos.1 > 1 { 
            if board[pos.0 - 1][pos.1 - 2] == Piece::Empty {
                moves.push((pos.0 - 1, pos.1 - 2)); 
            } else {
                if board[pos.0 - 1][pos.1 - 2].get_colour().unwrap() != self.get_colour().unwrap() {
                    moves.push((pos.0 - 1, pos.1 - 2));
                }
            }
        }
        if pos.0 > 0 && pos.1 < 6 { 
            if board[pos.0 - 1][pos.1 + 2] == Piece::Empty {
                moves.push((pos.0 - 1, pos.1 + 2)); 
            } else {
                if board[pos.0 - 1][pos.1 + 2].get_colour().unwrap() != self.get_colour().unwrap() {
                    moves.push((pos.0 - 1, pos.1 + 2));
                }
            }
         }
        if pos.0 < 7 && pos.1 > 1 {
            if board[pos.0 + 1][pos.1 - 2] == Piece::Empty {
                moves.push((pos.0 + 1, pos.1 - 2)); 
            } else {
                if board[pos.0 + 1][pos.1 - 2].get_colour().unwrap() != self.get_colour().unwrap() {
                    moves.push((pos.0 + 1, pos.1 - 2));
                }
            }
        }
        if pos.0 < 7 && pos.1 < 6 {
            if board[pos.0 + 1][pos.1 + 2] == Piece::Empty {
                moves.push((pos.0 + 1, pos.1 + 2)); 
            } else {
                if board[pos.0 + 1][pos.1 + 2].get_colour().unwrap() != self.get_colour().unwrap() {
                    moves.push((pos.0 + 1, pos.1 + 2));
                }
            }
        }
        if pos.0 > 1 && pos.1 > 0 {
            if board[pos.0 - 2][pos.1 - 1] == Piece::Empty {
                moves.push((pos.0 - 2, pos.1 - 1)); 
            } else {
                if board[pos.0 - 2][pos.1 - 1].get_colour().unwrap() != self.get_colour().unwrap() {
                    moves.push((pos.0 - 2, pos.1 - 1));
                }
            }
        }
        if pos.0 > 1 && pos.1 < 7 {
            if board[pos.0 - 2][pos.1 + 1] == Piece::Empty {
                moves.push((pos.0 - 2, pos.1 + 1)); 
            } else {
                if board[pos.0 - 2][pos.1 + 1].get_colour().unwrap() != self.get_colour().unwrap() {
                    moves.push((pos.0 - 2, pos.1 + 1));
                }
            }
        }
        if pos.0 < 6 && pos.1 > 0 {
            if board[pos.0 + 2][pos.1 - 1] == Piece::Empty {
                moves.push((pos.0 + 2, pos.1 - 1)); 
            } else {
                if board[pos.0 + 2][pos.1 - 1].get_colour().unwrap() != self.get_colour().unwrap() {
                    moves.push((pos.0 + 2, pos.1 - 1));
                }
            }
        }
        if pos.0 < 6 && pos.1 < 7 {
            if board[pos.0 + 2][pos.1 + 1] == Piece::Empty {
                moves.push((pos.0 + 2, pos.1 + 1)); 
            } else {
                if board[pos.0 + 2][pos.1 + 1].get_colour().unwrap() != self.get_colour().unwrap() {
                    moves.push((pos.0 + 2, pos.1 + 1));
                }
            }
        }
        moves
    }

    /// Internal helper function which shouldn't be used outside of Piece implementation.
    /// Retrieves valid moves as if the piece is a pawn.
    /// Moves are returned as a non-sorted list of usize tuples.
    /// 
    /// # Arguments
    /// 
    /// * `pos`: The position of the piece that moves are gotten from. In usize tuple format.
    /// * `board`: The board. A 2d vector of Pieces.
    /// * `en_passant_square`: The current square that can be captured through en_passant_square. Any non-existent square is accepted en-passant being impossible.
    fn get_pawn_moves(&self, pos: (usize, usize), board: &Vec<Vec<Piece>>, en_passant_square: (usize, usize)) -> Vec<(usize, usize)> {
        match self.get_colour().unwrap() {
            Colour::Black => {
                let mut moves = Vec::new();
                if pos.0 < 7 {
                    if pos.1 < 7 && ((board[pos.0 + 1][pos.1 + 1] != Piece::Empty && board[pos.0 + 1][pos.1 + 1].get_colour().unwrap() != self.get_colour().unwrap())
                        || (en_passant_square == (pos.0 + 1, pos.1 + 1))) {
                        moves.push((pos.0 + 1, pos.1 + 1));
                    }
                    if pos.1 > 0 && ((board[pos.0 + 1][pos.1 - 1] != Piece::Empty && board[pos.0 + 1][pos.1 - 1].get_colour().unwrap() != self.get_colour().unwrap())
                        || (en_passant_square == (pos.0 + 1, pos.1 - 1))) {
                        moves.push((pos.0 + 1, pos.1 - 1));
                    }
                    if board[pos.0 + 1][pos.1] == Piece::Empty {
                        moves.push((pos.0 + 1, pos.1));
                    }
                }
                if pos.0 == 1 && board[pos.0 + 1][pos.1] == Piece::Empty && board[pos.0 + 2][pos.1] == Piece::Empty {
                    moves.push((pos.0 + 2, pos.1));
                }
                moves
            },
            Colour::White => {
                let mut moves = Vec::new();
                if pos.0 > 1 {
                    if pos.1 < 7 && ((board[pos.0 - 1][pos.1 + 1] != Piece::Empty && board[pos.0 - 1][pos.1 + 1].get_colour().unwrap() != self.get_colour().unwrap())
                        || (en_passant_square == (pos.0 - 1, pos.1 + 1))) {
                        moves.push((pos.0 - 1, pos.1 + 1));
                    }
                    if pos.1 > 0 && ((board[pos.0 - 1][pos.1 - 1] != Piece::Empty && board[pos.0 - 1][pos.1 - 1].get_colour().unwrap() != self.get_colour().unwrap())
                        || (en_passant_square == (pos.0 - 1, pos.1 - 1))) {
                        moves.push((pos.0 - 1, pos.1 - 1));
                    }
                    if board[pos.0 - 1][pos.1] == Piece::Empty {
                        moves.push((pos.0 - 1, pos.1));
                    }
                }
                if pos.0 == 6 && board[pos.0 - 1][pos.1] == Piece::Empty && board[pos.0 - 2][pos.1] == Piece::Empty {
                    moves.push((pos.0 - 2, pos.1));
                }
                moves
            }
        }
    }

    /// Helper function to retrieve the colour out of a piece.
    /// Returns the relevant colour for any piece, and returns None for an empty piece.
    fn get_colour(&self) -> Option<&Colour> {
        match self {
            Piece::King(c) | Piece::Queen(c) | Piece::Rook(c) | Piece::Knight(c) | Piece::Bishop(c) | Piece::Pawn(c) => Some(c),
            Piece::Empty => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Colour {
    White,
    Black
}

