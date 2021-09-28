#[cfg(test)]
mod game_tests {
    macro_rules! test {
        {
            name: $name:ident,
            fen: $fen:literal,
            piece: $piece:ident,
            legal_moves: [$($token:tt)*],
        } => {
            #[test]
            fn $name() {
                use crate::Game;
                use crate::convert_square;
                
                let mut game = Game::new_empty();
                let square = convert_square(stringify!($piece));
                game.set_state_from_fen($fen);
                let mut expected_moves: Vec<(usize, usize)> = moves!($($token)*);
                let mut actual_moves = game.board[square.0][square.1].get_valid_moves(square, &game.board, game.en_passant_square, game.castlings);
                actual_moves.sort();
                expected_moves.sort();
                assert_eq!(expected_moves, actual_moves);
            }
        };
    }

    macro_rules! moves {
        () => {vec![]};
        ($mov:ident) => {vec![convert_square(stringify!($mov))]};
        ($mov:ident, $($movs:tt),*) => { 
            {
                let mut all_moves = vec![convert_square(stringify!($mov))];
                all_moves.append(&mut moves!($($movs),*));
                all_moves
            }
        }
    }

    #[test]
    fn fen_sets_start_correctly() {
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

    #[test]
    fn fen_sets_inprogress_correctly() {
        use crate::Piece;
        use crate::Game;
        use crate::Colour;

        let mut fen_game = Game::new();
        fen_game.set_state_from_fen("rnbqkbnr/pp1ppppp/2p5/8/4P3/8/PPPP1PPP/RNBQKBNR b kq e3 20 2");
        let mut test_game = Game::new_empty();
        
        let _board = vec![
        vec![Piece::Rook(Colour::Black), Piece::Knight(Colour::Black), Piece::Bishop(Colour::Black), Piece::Queen(Colour::Black), Piece::King(Colour::Black), Piece::Bishop(Colour::Black), Piece::Knight(Colour::Black), Piece::Rook(Colour::Black)],
        vec![Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Empty, Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black), Piece::Pawn(Colour::Black)],
        vec![Piece::Empty, Piece::Empty, Piece::Pawn(Colour::Black), Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty],
        vec![Piece::Empty; 8],
        vec![Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Pawn(Colour::White), Piece::Empty, Piece::Empty, Piece::Empty],
        vec![Piece::Empty; 8],
        vec![Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Empty, Piece::Pawn(Colour::White), Piece::Pawn(Colour::White), Piece::Pawn(Colour::White)],
        vec![Piece::Rook(Colour::White), Piece::Knight(Colour::White), Piece::Bishop(Colour::White), Piece::Queen(Colour::White), Piece::King(Colour::White), Piece::Bishop(Colour::White), Piece::Knight(Colour::White), Piece::Rook(Colour::White)],
        ];
        test_game.board = _board;
        test_game.turn = 2;
        test_game.current_turn = Colour::Black;
        test_game.en_passant_square = (5, 4);
        test_game.castlings = (false, false, true, true);
        test_game.halfmove_clock = 20;

        assert_eq!(fen_game, test_game);
    }

    #[test]
    fn checkmate_correctly_applies() {
        use crate::Game;
        use crate::GameState;

        let mut game = Game::new();
        game.set_state_from_fen("8/8/8/8/8/2b5/1q6/K7 w  - 0 0");
        let state = game.get_game_state(true);

        assert_eq!(state, GameState::Checkmate);
    }

    test!{
        name: bishop_takes_correctly,
        fen: "1B6/8/8/8/8/8/8/8 w  - 0 0",
        piece: b8,
        legal_moves: [a7, c7, d6, e5, f4, g3, h2],
    }

    test!{
        name: bishop_moves_correctly,
        fen: "1B6/8/8/8/8/8/8/8 w  - 0 0",
        piece: b8,
        legal_moves: [a7, c7, d6, e5, f4, g3, h2],
    }

    test!{
        name: rook_moves_correctly,
        fen: "8/8/2R5/2R1R3/8/8/8/8 w  - 0 0",
        piece: c5,
        legal_moves: [a5, b5, d5, c4, c3, c2, c1],
    }

    test!{
        name: rook_takes_correctly,
        fen: "8/8/2r5/2R1R3/8/8/8/8 w  - 0 0",
        piece: c5,
        legal_moves: [c6, a5, b5, d5, c4, c3, c2, c1],
    }

    test!{
        name: knight_moves_correctly,
        fen: "8/1N6/8/8/8/8/8/8 w  - 0 0",
        piece: b7,
        legal_moves: [a5, c5, d8, d6],
    }

    test!{
        name: knight_takes_correctly,
        fen: "3r4/1N6/3R4/8/8/8/8/8 w  - 0 0",
        piece: b7,
        legal_moves: [a5, c5, d8],
    }

    test!{
        name: pawn_moves_correctly,
        fen: "8/8/8/8/8/8/2P5/8 w  - 0 0",
        piece: c2,
        legal_moves: [c3, c4],
    }

    test!{
        name: pawn_takes_correctly,
        fen: "8/8/2Pp4/1pP5/8/8/8/8 w  b6 0 0",
        piece: c5,
        legal_moves: [b6, d6],
    }

    test!{
        name: king_moves_correctly,
        fen: "8/8/8/8/8/8/8/K7 w  - 0 0",
        piece: a1,
        legal_moves: [a2, b2, b1],
    }

    test!{
        name: king_takes_correctly,
        fen: "8/8/8/8/P7/Kp6/8/8 w  - 0 0",
        piece: a3,
        legal_moves: [b4, b3, b2],
    }

    test!{
        name: king_checks_correctly,
        fen: "8/8/8/8/r7/K7/8/8 w  - 0 0",
        piece: a3,
        legal_moves: [a4, b3, b2],
    }

    test!{
        name: king_pins_correctly,
        fen: "8/8/8/2b5/1P6/K7/8/8 w  - 0 0",
        piece: b4,
        legal_moves: [c5],
    }

    test!{
        name: king_castle_correctly,
        fen: "8/8/8/8/8/8/8/3QK2R w KQ - 0 0",
        piece: e1,
        legal_moves: [f1, g1, d2, e2, f2],
    }
}