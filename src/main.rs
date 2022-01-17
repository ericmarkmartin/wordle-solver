use std::convert::TryFrom;
use wordle_solve::*;
fn main() {
    let word_list: word::WordList<5> = word_list::WORD_LIST
        .iter()
        .map(|s| word::Word::<5>::try_from(*s).unwrap())
        .collect();
    let strategy = strategy::StdinThenSolver::new(word_list.clone());
    let engine = engine::StdinEvaluator;
    run_game(engine, strategy);
}
