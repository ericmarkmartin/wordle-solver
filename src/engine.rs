use crate::word::*;
use std::convert::TryInto;

use std::io;

pub enum GuessResult<const WORD_LENGTH: usize> {
    Done(bool),
    Continue(Score<WORD_LENGTH>),
}

pub trait Engine<const WORD_LENGTH: usize> {
    fn score_guess(&self, guess: &Word<WORD_LENGTH>) -> GuessResult<WORD_LENGTH>;
}

pub struct StandardEngine<const WORD_LENGTH: usize> {
    word: Word<WORD_LENGTH>,
    word_list: WordList<WORD_LENGTH>,
    guesses_remaining: std::cell::Cell<usize>,
}

impl<const WORD_LENGTH: usize> StandardEngine<WORD_LENGTH> {
    pub fn new(
        secret_word: Word<WORD_LENGTH>,
        word_list: WordList<WORD_LENGTH>,
        num_guesses: usize,
    ) -> Self {
        Self {
            word: secret_word,
            word_list,
            guesses_remaining: std::cell::Cell::new(num_guesses),
        }
    }
}

impl<const WORD_LENGTH: usize> Engine<WORD_LENGTH> for StandardEngine<WORD_LENGTH> {
    fn score_guess(&self, guess: &Word<WORD_LENGTH>) -> GuessResult<WORD_LENGTH> {
        let score = self
            .word_list
            .0
            .contains(&guess)
            .then(|| self.word.evaluate_guess(&guess))
            .expect(&format!("guess not in wordlist: {}", guess));

        let guesses_remaining = self.guesses_remaining.get();

        if score
            .iter()
            .all(|annotation| *annotation == LetterScore::RightPlace)
        {
            GuessResult::Done(true)
        } else if guesses_remaining == 0 {
            GuessResult::Done(false)
        } else {
            self.guesses_remaining.set(guesses_remaining - 1);
            GuessResult::Continue(score)
        }
    }
}

pub struct StdinEvaluator<const WORD_LENGTH: usize>;

impl<const WORD_LENGTH: usize> StdinEvaluator<WORD_LENGTH> {
    fn letter_score_of_char(c: char) -> Option<LetterScore> {
        use LetterScore::*;
        match c.to_ascii_lowercase() {
            'g' => Some(RightPlace),
            'y' => Some(RightLetter),
            'b' => Some(Wrong),
            _ => None,
        }
    }

    fn read_score(&self) -> Option<Score<WORD_LENGTH>> {
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line.");

        println!("buffer: {:?}", buffer);

        let score_vec = buffer
            .trim_end()
            .chars()
            .filter_map(Self::letter_score_of_char)
            .collect::<Vec<_>>();

        println!("score_vec: {:?}", score_vec);

        (score_vec.len() == WORD_LENGTH).then(|| score_vec.try_into().unwrap())
    }
}

impl<const WORD_LENGTH: usize> Engine<WORD_LENGTH> for StdinEvaluator<WORD_LENGTH> {
    fn score_guess(&self, guess: &Word<WORD_LENGTH>) -> GuessResult<WORD_LENGTH> {
        println!("Enter score for {}:", guess);
        loop {
            if let Some(score) = self.read_score() {
                break GuessResult::Continue(score);
            }
            println!("Invalid score, try again:");
        }
    }
}
