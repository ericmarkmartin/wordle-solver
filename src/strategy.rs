use crate::word::*;
use std::{collections::HashSet, convert::TryInto};

pub trait Strategy<const WORD_LENGTH: usize> {
    fn make_guess(&mut self) -> Word<WORD_LENGTH>;

    fn receive_score(&mut self, score: &Score<WORD_LENGTH>);
}

pub struct SimpleStrategy<const WORD_LENGTH: usize> {
    word_list: WordList<WORD_LENGTH>,
    viable_words: WordList<WORD_LENGTH>,
    last_guess: Option<Word<WORD_LENGTH>>,
    right_place: HashSet<char>,
    num_guesses: usize,
}

impl<const WORD_LENGTH: usize> SimpleStrategy<WORD_LENGTH> {
    fn set_viable_words(&mut self, viable_words: WordList<WORD_LENGTH>) {
        self.viable_words = viable_words;
        // Don't seed with the first guess word b/c we've already done that in
        // the interactive session
        self.num_guesses = 1;
    }
}

impl<const WORD_LENGTH: usize> SimpleStrategy<WORD_LENGTH> {
    pub fn new(word_list: WordList<WORD_LENGTH>) -> Self {
        Self {
            word_list: word_list.clone(),
            viable_words: word_list,
            last_guess: None,
            right_place: HashSet::new(),
            num_guesses: 0,
        }
    }

    fn score(&self, word: &Word<WORD_LENGTH>) -> usize {
        self.viable_words
            .0
            .iter()
            .map(|secret| {
                let score = secret.evaluate_guess(word);
                self.viable_words
                    .0
                    .iter()
                    .filter(|viable_word| {
                        *viable_word != word && score != viable_word.evaluate_guess(word)
                    })
                    .count()
            })
            .min()
            .unwrap()

        // self.viable_bags
        //     .iter()
        //     .map(|viable_bag| {
        //         let inter = bag.intersection(viable_bag);

        //         if discount_right_place {
        //             inter.filter(|c| !self.right_place.contains(c)).count()
        //         } else {
        //             inter.count()
        //         }
        //     })
        //     .sum()
    }
}

pub struct StdinGuesser<const WORD_LENGTH: usize>;

impl<const WORD_LENGTH: usize> StdinGuesser<WORD_LENGTH> {
    fn read_guess(&self) -> Option<Word<WORD_LENGTH>> {
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");

        if !buffer.is_ascii() {
            return None;
        }

        buffer
            .to_ascii_lowercase()
            .chars()
            .filter(|c| matches!(c, 'a'..='z'))
            .collect::<String>()
            .as_str()
            .try_into()
            .ok()
    }
}

impl<const WORD_LENGTH: usize> Strategy<WORD_LENGTH> for StdinGuesser<WORD_LENGTH> {
    fn make_guess(&mut self) -> Word<WORD_LENGTH> {
        println!("Enter guess:");
        loop {
            if let Some(guess) = self.read_guess() {
                return guess;
            }
            println!("Not valid guess:");
        }
    }

    fn receive_score(&mut self, score: &Score<WORD_LENGTH>) {
        println!("Score was {:?}", score);
    }
}

enum StdinOrAlgo<const WORD_LENGTH: usize> {
    Stdin(StdinGuesser<WORD_LENGTH>),
    Algo(SimpleStrategy<WORD_LENGTH>),
}

impl<const WORD_LENGTH: usize> StdinOrAlgo<WORD_LENGTH> {
    fn is_stdin(&self) -> bool {
        matches!(self, Self::Stdin(_))
    }
}

pub struct StdinThenSolver<const WORD_LENGTH: usize> {
    word_list: WordList<WORD_LENGTH>,
    last_guess: Option<Word<WORD_LENGTH>>,
    viable_words: WordList<WORD_LENGTH>,
    strategy: std::cell::RefCell<StdinOrAlgo<WORD_LENGTH>>,
}

impl<const WORD_LENGTH: usize> StdinThenSolver<WORD_LENGTH> {
    fn is_stdin(&self) -> bool {
        self.strategy.borrow().is_stdin()
    }
}

impl<const WORD_LENGTH: usize> StdinThenSolver<WORD_LENGTH> {
    pub fn new(word_list: WordList<WORD_LENGTH>) -> Self {
        Self {
            strategy: std::cell::RefCell::new(StdinOrAlgo::Stdin(StdinGuesser)),
            viable_words: word_list.clone(),
            word_list,
            last_guess: None,
        }
    }

    pub fn start_solver(&mut self) {
        if let StdinOrAlgo::Stdin(_) = self.strategy.replace(StdinOrAlgo::Stdin(StdinGuesser)) {
            let mut algo = SimpleStrategy::new(self.word_list.clone());
            let mut viable_words = WordList(Vec::new());
            std::mem::swap(&mut viable_words, &mut self.viable_words);
            algo.set_viable_words(viable_words);
            self.strategy.replace(StdinOrAlgo::Algo(algo));
        } else {
            panic!("already started solver")
        }
    }

    fn should_switch_to_solver(&self) -> bool {
        if self.is_stdin() {
            println!("There are {} viable words remaining\nDo you want to let the solver take over? [y/n]", self.viable_words.0.len());
            loop {
                let mut buffer = String::new();
                std::io::stdin()
                    .read_line(&mut buffer)
                    .expect("Failed to read line");

                println!("buffer: {:?}", buffer);

                match buffer.trim_end().to_ascii_lowercase().as_str() {
                    "y" => {
                        println!("y!");
                        return true;
                    }
                    "n" => {
                        println!("n!");
                        return false;
                    }
                    _ => {
                        println!("trying again");
                    }
                }
            }
        } else {
            false
        }
    }
}

impl<const WORD_LENGTH: usize> Strategy<WORD_LENGTH> for SimpleStrategy<WORD_LENGTH> {
    fn make_guess(&mut self) -> Word<WORD_LENGTH> {
        let guess = if self.num_guesses == 0 {
            unsafe { std::mem::transmute_copy(&['a', 'r', 'o', 's', 'e']) }
        } else {
            // let n = self.viable_words.len() / 2;
            // let dont_discount = self.viable_words.len() == 1 || self.num_guesses == 9;
            // let guess =
            *((if self.viable_words.0.len() == 1 || self.num_guesses == 9 {
                self.viable_words.clone()
            } else {
                self.word_list.clone()
            })
            // *self
            //     .word_list
            // .clone()
            .0
            .iter()
            .max_by_key(|viable_word| self.score(*viable_word))
            .unwrap())
            // .max_by_key(|viable )
            // .select_nth_unstable_by_key(n, |viable_word| self.score(viable_word))
            // .1
            // .iter()
            // .map(|word| (word,))
            // .min_by_key(|viable_word| self.score(*viable_word))
            // .expect("viable words shouldn't be empty")
            // };
        };

        self.last_guess = Some(guess);
        println!("Score: {:?}, {:?}", guess, self.score(&guess));

        self.num_guesses += 1;

        guess
    }

    fn receive_score(&mut self, score: &Score<WORD_LENGTH>) {
        println!("Score: {:?}", score);
        let last_guess = self.last_guess.expect("Should've made a guess by now");
        self.viable_words.retain_viable_words(&last_guess, score);

        last_guess
            .0
            .iter()
            .zip(score.iter())
            .for_each(|(c, annotation)| {
                if let LetterScore::RightPlace = annotation {
                    self.right_place.insert(*c);
                }
            });

        println!("Right places: {:?}", self.right_place);
        println!("Num viable words left: {:?}", self.viable_words);
    }
}

impl<const WORD_LENGTH: usize> Strategy<WORD_LENGTH> for StdinThenSolver<WORD_LENGTH> {
    fn make_guess(&mut self) -> Word<WORD_LENGTH> {
        if self.should_switch_to_solver() {
            self.start_solver();
        }

        match &mut *self.strategy.borrow_mut() {
            StdinOrAlgo::Stdin(stdin) => {
                let guess = stdin.make_guess();
                self.last_guess = Some(guess);
                guess
            }
            StdinOrAlgo::Algo(ref mut strat) => {
                println!("Computing...");
                strat.make_guess()
            }
        }
    }

    fn receive_score(&mut self, score: &Score<WORD_LENGTH>) {
        match &mut *self.strategy.borrow_mut() {
            StdinOrAlgo::Stdin(stdin) => {
                let last_guess = self.last_guess.expect("should've made a guess by now");
                self.viable_words.retain_viable_words(&last_guess, &score);
                stdin.receive_score(score);
            }
            StdinOrAlgo::Algo(strat) => strat.receive_score(score),
        }
    }
}
