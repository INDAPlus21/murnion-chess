use std::collections::HashSet;
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameState {
    InProgress,
    Check,
    Checkmate,
}

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
    game_state: GameState,
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
            game_state: GameState::InProgress,
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
            game_state: GameState::InProgress,
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
            let square = (8 - y, x);
            square
            }
        };
        self.halfmove_clock = fen_split[4].parse::<usize>().unwrap();
        self.turn = fen_split[5].parse::<usize>().unwrap();
    }

    /// Parses the current board to get the game state.
    fn get_game_state_no_recursion(&self) -> GameState {
        let mut threatened_squares: HashSet<(usize, usize)> = HashSet::new();
        for x in 0..8 {
            for y in 0..8 {
                if self.board[x][y] != Piece::Empty && &self.current_turn != self.board[x][y].get_colour().unwrap() {
                    threatened_squares.extend(&self.board[x][y]
                                        .get_threatened_squares((x, y), &self.board)
                                        .into_iter()
                                        .collect::<HashSet<(usize, usize)>>());
                }
            }
        }
        for x in 0..8 {
            for y in 0..8 {
                if self.board[x][y] == Piece::King(self.current_turn) && threatened_squares.contains(&(x, y)) {
                    return GameState::Check;
                }
            }
        }
        return GameState::InProgress;
    }

    fn get_game_state(&self, eot: bool) -> GameState {
        let mut state = self.get_game_state_no_recursion();
        let mut moves = Vec::new();
        if state == GameState::Check && eot {
            for x in 0..8 {
                for y in 0..8 {
                    if self.board[x][y] != Piece::Empty && self.board[x][y].get_colour().unwrap() == &self.current_turn {
                        moves.append(&mut self.board[x][y].get_valid_moves((x, y), &self.board, self.en_passant_square, self.castlings));
                    }
                }
            }
            if moves.len() == 0 {
                state = GameState::Checkmate;
            }
        }
        state
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
            Piece::Pawn(_colour) => {
                let mut moves = Vec::new();
                if pos.1 != 0 {
                    moves.push((pos.0 + 1, pos.1 - 1));
                }
                if pos.1 != 7 {
                    moves.push((pos.0 + 1, pos.1 + 1));
                }
                moves
            },
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
            _ => Vec::new()
        }
    }

    /// The public function to return any valid moves for the single piece it is called from. Does not check for check.
    fn get_valid_moves(&self, pos: (usize, usize), board: &Vec<Vec<Piece>>, en_passant_square: (usize, usize), castlings: (bool, bool, bool, bool)) -> Vec<(usize, usize)> {
        match self {
            Piece::Empty => Vec::new(),
            Piece::Queen(_colour) => {
                let mut moves = Vec::new();
                moves.append(&mut self.get_rook_moves(pos, board));
                moves.append(&mut self.get_bishop_moves(pos, board));
                clean_moves(pos, board, moves)
            },
            Piece::Rook(_colour) => {
                let moves = self.get_rook_moves(pos, board);
                clean_moves(pos, board, moves)
            },
            Piece::Bishop(_colour) => {
                let moves = self.get_bishop_moves(pos, board);
                clean_moves(pos, board, moves)
            },
            Piece::Knight(_colour) => {
                let moves = self.get_knight_moves(pos, board);
                clean_moves(pos, board, moves)
            },
            Piece::Pawn(_colour) => {
                let moves = self.get_pawn_moves(pos, board, en_passant_square);
                clean_moves(pos, board, moves)
            },
            Piece::King(_colour) => {
                let moves = self.get_king_moves(pos, board, castlings);
                clean_moves(pos, board, moves)
            },
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
        macro_rules! bishop_move {
            ($number1:tt, $axis1:tt, $number2:tt, $axis2:tt, $br1:tt, $br2:tt) => {
                for number in 1..8 {
                    if pos.0 + $number1 $axis1 number == $br1 || pos.1 + $number2 $axis2 number == $br2 { 
                        break;
                    }
                    if board[pos.0 $axis1 number][pos.1 $axis2 number] == Piece::Empty {
                        moves.push((pos.0 $axis1 number, pos.1 $axis2 number));
                    } else {
                        if board[pos.0 $axis1 number][pos.1 $axis2 number].get_colour().unwrap() == self.get_colour().unwrap() {
                            break;
                        } else {
                            moves.push((pos.0 $axis1 number, pos.1 $axis2 number));
                            break;
                        }
                    }
                };
            };
        }
        bishop_move!(0, +, 0, +, 8, 8);
        bishop_move!(0, +, 1, -, 8, 0);
        bishop_move!(1, -, 0, +, 0, 8);
        bishop_move!(1, -, 1, -, 0, 0);
        moves
    }

    /// Internal helper function which shouldn't be used outside of Piece implementation.
    /// Retrieves valid moves as if the piece is a king.
    /// Moves are returned as a non-sorted list of usize tuples.
    /// 
    /// # Arguments
    /// 
    /// * `pos`: The position of the piece that moves are gotten from. In usize tuple format.
    /// * `board`: The board. A 2d vector of Pieces.
    fn get_king_moves(&self, pos: (usize, usize), board: &Vec<Vec<Piece>>, castlings: (bool, bool, bool, bool)) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for x in 0..3 {
            if pos.0 + x == 0 || pos.0 + x == 9 { continue; }
            for y in 0..3 {
                if pos.1 + y == 0 || pos.1 + y == 9 { continue; }
                if board[pos.0 + x - 1][pos.1 + y - 1] == Piece::Empty {
                    moves.push((pos.0 + x - 1, pos.1 + y - 1));
                } else {
                    if board[pos.0 + x - 1][pos.1 + y - 1].get_colour().unwrap() != self.get_colour().unwrap() {
                        moves.push((pos.0 + x - 1, pos.1 + y - 1));
                    }
                }
            }
        }
        match self.get_colour().unwrap() {
            Colour::White => {
                if castlings.0 {
                    let sq1 = convert_square("f1");
                    let sq2 = convert_square("g1");
                    if board[sq1.0][sq1.1] == Piece::Empty && board[sq2.0][sq2.1] == Piece::Empty {
                        moves.push(convert_square("g1"));
                    }
                }
                if castlings.1 {
                    let sq1 = convert_square("d1");
                    let sq2 = convert_square("c1");
                    let sq3 = convert_square("b1");
                    if board[sq1.0][sq1.1] == Piece::Empty && board[sq2.0][sq2.1] == Piece::Empty && board[sq3.0][sq3.1] == Piece::Empty {
                        moves.push(convert_square("c1"));
                    }
                }
            },
            Colour::Black => {
                if castlings.0 {
                    let sq1 = convert_square("f8");
                    let sq2 = convert_square("g8");
                    if board[sq1.0][sq1.1] == Piece::Empty && board[sq2.0][sq2.1] == Piece::Empty {
                        moves.push(convert_square("g8"));
                    }
                }
                if castlings.1 {
                    let sq1 = convert_square("d8");
                    let sq2 = convert_square("c8");
                    let sq3 = convert_square("b8");
                    if board[sq1.0][sq1.1] == Piece::Empty && board[sq2.0][sq2.1] == Piece::Empty && board[sq3.0][sq3.1] == Piece::Empty {
                        moves.push(convert_square("c8"));
                    }
                }
            }
        }
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
        macro_rules! knight_move {
            ($number1:tt, $axis1:tt, $comp1:tt, $number2:tt, $axis2:tt, $comp2:tt) => {
                if (pos.0 as f32) $comp1 3.5 $axis1 3.5 $axis1 -(($number1 - 1) as f32) && (pos.1 as f32) $comp2 3.5 $axis2 3.5 $axis2 -(($number2 - 1) as f32) {
                    if board[pos.0 $axis1 $number1][pos.1 $axis2 $number2] == Piece::Empty {
                        moves.push((pos.0 $axis1 $number1, pos.1 $axis2 $number2));
                    } else {
                        if board[pos.0 $axis1 $number1][pos.1 $axis2 $number2].get_colour().unwrap() != self.get_colour().unwrap() {
                            moves.push((pos.0 $axis1 $number1, pos.1 $axis2 $number2));
                        }
                    }
                }
            };
        }
        knight_move!(1, -, >, 2, -, >);
        knight_move!(1, -, >, 2, +, <);
        knight_move!(1, +, <, 2, -, >);
        knight_move!(1, +, <, 2, +, <);
        knight_move!(2, -, >, 1, -, >);
        knight_move!(2, -, >, 1, +, <);
        knight_move!(2, +, <, 1, -, >);
        knight_move!(2, +, <, 1, +, <);
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
        println!("{:?}", en_passant_square);
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

fn clean_moves(pos: (usize, usize), board: &Vec<Vec<Piece>>, moves: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut bad_moves = Vec::new();
    let mut clean_moves = Vec::new();
    for mov_idx in 0..moves.len() {
        let mut theoretical_game = Game::new();
        theoretical_game.board = board.clone();
        theoretical_game.board[moves[mov_idx].0][moves[mov_idx].1] = board[pos.0][pos.1].clone();
        theoretical_game.board[pos.0][pos.1] = Piece::Empty;
        if theoretical_game.get_game_state(false) == GameState::Check {
            bad_moves.push(mov_idx);
        }
    }
    for number in 0..moves.len() {
        if !bad_moves.contains(&number) {
            clean_moves.push(moves[number]);
        }
    }
    clean_moves
}

fn convert_square(square: &str) -> (usize, usize) {
    let column = {
        match square.chars().nth(0).unwrap() {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => panic!(),
        }
    };
    let rank: usize = 8 - square.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
    (rank, column)
}