use std::collections::HashSet;

/// An enumerable representing whether the game has ended or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
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
    pub fn new() -> Game {
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

    /// Function that returns the current game-state of the board.
    pub fn game_state(&self) -> GameState {
        self.game_state
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

    /// Parses the current board to get the game-state. Returns the new game-state.
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

    /// Recursively parses the board to get the game-state. Returns the new game-state.
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

    /// Takes a char of either r, q, n, or b, setting the promotion to be Rook, Queen, Knight or Bishop.
    pub fn select_promotion(&mut self, piece: char) {
        match piece.to_lowercase().next().unwrap() {
            'r' => self.selected_promotion = Piece::Rook(self.current_turn),
            'b' => self.selected_promotion = Piece::Bishop(self.current_turn),
            'n' => self.selected_promotion = Piece::Knight(self.current_turn),
            'q' => self.selected_promotion = Piece::Queen(self.current_turn),
            _ => panic!()
        }
    }

    /// Returns the state of the game as a string in FEN-notation.
    pub fn get_fen(&self) -> String {
        use std::char;

        let mut fen: String = "".to_string();
        for x in 0..8 {
            let mut rank: String = "".to_string();
            let mut empties = 0;
            for y in 0..8 {
                match self.board[x][y] {
                    Piece::Empty => empties = empties + 1,
                    Piece::King(colour) => {
                        if empties > 0 {rank.push(char::from_digit(empties, 10).unwrap()); empties = 0; }
                        if colour == Colour::White {
                            rank.push('K');
                        } else {
                            rank.push('k');
                        }
                    },
                    Piece::Queen(colour) => {
                        if empties > 0 {rank.push(char::from_digit(empties, 10).unwrap()); empties = 0; }
                        if colour == Colour::White {
                            rank.push('Q');
                        } else {
                            rank.push('q');
                        }
                    },
                    Piece::Bishop(colour) => {
                        if empties > 0 {rank.push(char::from_digit(empties, 10).unwrap()); empties = 0; }
                        if colour == Colour::White {
                            rank.push('B');
                        } else {
                            rank.push('b');
                        }
                    },
                    Piece::Rook(colour) => {
                        if empties > 0 {rank.push(char::from_digit(empties, 10).unwrap()); empties = 0; }
                        if colour == Colour::White {
                            rank.push('R');
                        } else {
                            rank.push('r');
                        }
                    },
                    Piece::Knight(colour) => {
                        if empties > 0 {rank.push(char::from_digit(empties, 10).unwrap()); empties = 0; }
                        if colour == Colour::White {
                            rank.push('N');
                        } else {
                            rank.push('n');
                        }
                    },
                    Piece::Pawn(colour) => {
                        if empties > 0 {rank.push(char::from_digit(empties, 10).unwrap()); empties = 0; }
                        if colour == Colour::White {
                            rank.push('P');
                        } else {
                            rank.push('p');
                        }
                    },
                }
            }
            if empties > 0 {rank.push(char::from_digit(empties, 10).unwrap()); }
            if x != 7 { rank.push('/'); }
            fen.push_str(&rank);
        }

        if self.current_turn == Colour::White {
            fen.push_str(" w ");
        } else {
            fen.push_str(" b ");
        }

        if self.castlings.0 {fen.push_str("K")}
        if self.castlings.1 {fen.push_str("Q")}
        if self.castlings.2 {fen.push_str("k")}
        if self.castlings.3 {fen.push_str("q")}

        let x = self.en_passant_square.0;
        let y = self.en_passant_square.1;

        fen.push(' ');
        match y {
            0 => {fen.push('a'); 
            fen.push(char::from_digit(8 - x as u32, 10).unwrap());
            },
            1 => {fen.push('b');
            fen.push(char::from_digit(8 - x as u32, 10).unwrap());
            },
            2 => {fen.push('c');
            fen.push(char::from_digit(8 - x as u32, 10).unwrap());
            },
            3 => {fen.push('d');
            fen.push(char::from_digit(8 - x as u32, 10).unwrap());
            },
            4 => {fen.push('e');
            fen.push(char::from_digit(8 - x as u32, 10).unwrap());
            },
            5 => {fen.push('f');
            fen.push(char::from_digit(8 - x as u32, 10).unwrap());
            },
            6 => {fen.push('g');
            fen.push(char::from_digit(8 - x as u32, 10).unwrap());
            },
            7 => {fen.push('h');
            fen.push(char::from_digit(8 - x as u32, 10).unwrap());
            },
            _ => {fen.push('-')}
        }
        fen.push_str(&" ");
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push_str(&" ");
        fen.push_str(&self.turn.to_string());
        fen
    }

    /// Takes a string in the form "\<square\> \<square\>", moving from the first square to the second.
    /// Also updates relevant game-tracking variables, such as the halfmove-clock, castlings and the en-passant square.
    pub fn take_turn(&mut self, mov: String) -> Option<GameState> {
        let movs = mov.split(" ").collect::<Vec<&str>>();
        let from = convert_square(movs[0]);
        let to = convert_square(movs[1]);

        self.halfmove_clock = self.halfmove_clock + 1;

        if self.board[from.0][from.1] == Piece::Empty || self.board[from.0][from.1].get_colour().unwrap() != &self.current_turn { return None; }
        let valids = self.board[from.0][from.1].get_valid_moves(from, &self.board, self.en_passant_square, self.castlings);

        if valids.contains(&to) {
            let cur_piece = self.board[from.0][from.1];
            match cur_piece {
                Piece::King(Colour::Black) => {
                    if to == convert_square("g8") && self.castlings.2 {
                        self.board[0][7] = Piece::Empty;
                        self.board[0][5] = Piece::Rook(Colour::Black);
                    }
                    if to == convert_square("c8") && self.castlings.3 {
                        self.board[0][0] = Piece::Empty;
                        self.board[0][3] = Piece::Rook(Colour::Black);
                    }
                    self.castlings.2 = false;
                    self.castlings.3 = false;
                },
                Piece::King(Colour::White) => {
                    if to == convert_square("g1") && self.castlings.0 {
                        self.board[7][7] = Piece::Empty;
                        self.board[7][5] = Piece::Rook(Colour::White);
                    }
                    if to == convert_square("c1") && self.castlings.1 {
                        self.board[7][0] = Piece::Empty;
                        self.board[7][3] = Piece::Rook(Colour::White);
                    }
                    self.castlings.0 = false;
                    self.castlings.1 = false;
                },
                Piece::Pawn(colour) => {
                    if to == self.en_passant_square {
                        match self.en_passant_square.0 {
                            5 => { 
                                self.board[self.en_passant_square.0 - 1][self.en_passant_square.1] = Piece::Empty;
                            }
                            2 => {
                                self.board[self.en_passant_square.0 + 1][self.en_passant_square.1] = Piece::Empty;
                            }
                            _ => panic!()
                        }
                    }
                    self.halfmove_clock = 0;
                },
                _ => (),
            }
        }

        if self.board[from.0][from.1] == Piece::Pawn(Colour::Black) && to.0 == from.0 + 2 {
            self.en_passant_square = (from.0 + 1, from.1);
        } else if self.board[from.0][from.1] == Piece::Pawn(Colour::White) && to.0 == from.0 - 2 {
            self.en_passant_square = (from.0 - 1, from.1);
        } else {
            self.en_passant_square = (8, 8);
        }
        
        match from {
            (0, 0) => self.castlings.3 = false,
            (0, 7) => self.castlings.2 = false,
            (7, 0) => self.castlings.1 = false,
            (7, 7) => self.castlings.0 = false,
            _ => ()
        }
        match to {
            (0, 0) => self.castlings.3 = false,
            (0, 7) => self.castlings.2 = false,
            (7, 0) => self.castlings.1 = false,
            (7, 7) => self.castlings.0 = false,
            _ => ()
        }

        if self.board[to.0][to.1] != Piece::Empty {
            self.halfmove_clock = 0;
        }

        self.board[to.0][to.1] = self.board[from.0][from.1];
        self.board[from.0][from.1] = Piece::Empty;

        if self.board[to.0][to.1] == Piece::Pawn(Colour::White) && to.0 == 0 {
            self.board[to.0][to.1] = self.selected_promotion;
        }
        if self.board[to.0][to.1] == Piece::Pawn(Colour::Black) && to.0 == 7 {
            self.board[to.0][to.1] = self.selected_promotion;
        }

        if self.current_turn == Colour::Black {
            self.turn = self.turn + 1;
            self.current_turn = Colour::White;
        } else {
            self.current_turn = Colour::Black;
        }

        match self.selected_promotion {
            Piece::Bishop(_colour) => self.selected_promotion = Piece::Bishop(self.current_turn),
            Piece::Rook(_colour) => self.selected_promotion = Piece::Rook(self.current_turn),
            Piece::Knight(_colour) => self.selected_promotion = Piece::Knight(self.current_turn),
            Piece::Queen(_colour) => self.selected_promotion = Piece::Queen(self.current_turn),
            _ => panic!()
        }

        self.game_state = self.get_game_state(true);
        Some(self.game_state)
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

    /// The public function to return any valid moves for the single piece it is called from. 
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
                let threatened_squares = {
                    let mut threat = Vec::new();
                    for x in 0..8 {
                        for y in 0..8 {
                            if board[x][y] != Piece::Empty && board[x][y].get_colour().unwrap() != &Colour::White {
                                threat.append(&mut board[x][y].get_threatened_squares((x, y), board));
                            }
                        }
                    }
                    threat
                };
                if castlings.0 {
                    let sq1 = convert_square("f1");
                    let sq2 = convert_square("g1");
                    if board[sq1.0][sq1.1] == Piece::Empty 
                        && board[sq2.0][sq2.1] == Piece::Empty 
                        && !threatened_squares.contains(&sq1) 
                        && !threatened_squares.contains(&sq2) {
                        moves.push(convert_square("g1"));
                    }
                }
                if castlings.1 {
                    let sq1 = convert_square("d1");
                    let sq2 = convert_square("c1");
                    let sq3 = convert_square("b1");
                    if board[sq1.0][sq1.1] == Piece::Empty 
                        && board[sq2.0][sq2.1] == Piece::Empty 
                        && board[sq3.0][sq3.1] == Piece::Empty
                        && !threatened_squares.contains(&sq1)
                        && !threatened_squares.contains(&sq2)
                        && !threatened_squares.contains(&sq3) {
                        moves.push(convert_square("c1"));
                    }
                }
            },
            Colour::Black => {
                let threatened_squares = {
                    let mut threat = Vec::new();
                    for x in 0..8 {
                        for y in 0..8 {
                            if board[x][y] != Piece::Empty && board[x][y].get_colour().unwrap() != &Colour::Black {
                                threat.append(&mut board[x][y].get_threatened_squares((x, y), board));
                            }
                        }
                    }
                    threat
                };
                if castlings.2 {
                    let sq1 = convert_square("f8");
                    let sq2 = convert_square("g8");
                    if board[sq1.0][sq1.1] == Piece::Empty 
                        && board[sq2.0][sq2.1] == Piece::Empty 
                        && !threatened_squares.contains(&sq1) 
                        && !threatened_squares.contains(&sq2) {
                        moves.push(convert_square("g8"));
                    }
                }
                if castlings.3 {
                    let sq1 = convert_square("d8");
                    let sq2 = convert_square("c8");
                    let sq3 = convert_square("b8");
                    if board[sq1.0][sq1.1] == Piece::Empty 
                        && board[sq2.0][sq2.1] == Piece::Empty 
                        && board[sq3.0][sq3.1] == Piece::Empty
                        && !threatened_squares.contains(&sq1)
                        && !threatened_squares.contains(&sq2)
                        && !threatened_squares.contains(&sq3) {
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

/// Colour enumerable used to identify the colour that any given piece belongs to.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Colour {
    White,
    Black
}

/// Goes through all the moves given in moves, and removes any that would place the player in check.
/// 
/// # Arguments
/// 
/// `pos`: The position of the piece which is being moved.
/// `board`: The board of the game.
/// `moves`: The moves to be cleaned.
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

/// Takes a string such as a4 or c6 and converts it into a tuple of x and y friendly to the game board.
/// 
/// # Arguments
/// 
/// `square`: A string literal with a square in chess notation.
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