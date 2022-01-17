pub mod engine;
pub mod strategy;
pub mod word;
pub mod word_list;

use engine::{Engine, GuessResult};
use strategy::Strategy;

fn run_round<E, S, const WORD_LENGTH: usize>(
    engine: &E,
    strategy: &mut S,
) -> GuessResult<WORD_LENGTH>
where
    E: Engine<WORD_LENGTH>,
    S: Strategy<WORD_LENGTH>,
{
    let guess = strategy.make_guess();
    let score = engine.score_guess(&guess);
    if let GuessResult::Continue(score) = score {
        strategy.receive_score(&score);
    }
    score
}

pub fn run_game<E, S, const WORD_LENGTH: usize>(engine: E, mut strategy: S) -> bool
where
    E: Engine<WORD_LENGTH>,
    S: Strategy<WORD_LENGTH>,
{
    loop {
        if let GuessResult::Done(did_win) = run_round(&engine, &mut strategy) {
            break did_win;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use word::{LetterScore::*, *};

    fn evaluate<const N: usize>(guess: &str, secret_word: &str) -> word::Score<N> {
        Word::try_from(secret_word)
            .unwrap()
            .evaluate_guess(&Word::try_from(guess).unwrap())
    }

    #[test]
    fn test_evaluate() {
        assert_eq!(evaluate("xxx", "aaa"), [Wrong, Wrong, Wrong]);
        assert_eq!(evaluate("xxx", "xxx"), [RightPlace, RightPlace, RightPlace]);
        assert_eq!(
            evaluate("abc", "cab"),
            [RightLetter, RightLetter, RightLetter]
        );
        assert_eq!(
            evaluate("abc", "cba"),
            [RightLetter, RightPlace, RightLetter]
        );
        assert_eq!(evaluate("aab", "aaa"), [RightPlace, RightPlace, Wrong]);
        assert_eq!(evaluate("yyx", "xxx"), [Wrong, Wrong, RightPlace]);
        assert_eq!(evaluate("xxx", "yyx"), [Wrong, Wrong, RightPlace]);
        assert_eq!(
            evaluate("xyy", "yyx"),
            [RightLetter, RightPlace, RightLetter]
        );
    }

    #[test]
    fn test_simple_strategy() {
        let word_list: WordList<5> = word_list::WORD_LIST
            .iter()
            .map(|s| Word::<5>::try_from(*s).unwrap())
            .collect();
        let strategy = strategy::SimpleStrategy::new(word_list.clone());
        let word = Word::try_from("favor").unwrap();
        let engine = engine::StandardEngine::new(word, word_list, 10);
        assert!(matches!(run_game(engine, strategy), true));
    }
}
