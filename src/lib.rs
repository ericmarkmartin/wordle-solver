use std::collections::HashSet;
use std::convert::TryInto;

#[derive(Debug)]
pub enum WordleError {
    TooManyGuesses,
    NotInWordList,
}

pub type WordleResult<T> = Result<T, WordleError>;

pub type Word<const N: usize> = [char; N];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LetterAnnotation {
    RightPlace,
    RightLetter,
    Wrong,
}

use LetterAnnotation::*;
pub type Guess<const N: usize> = [LetterAnnotation; N];
pub type WordList<const N: usize> = Vec<Word<N>>;

pub struct Engine<const WORD_LENGTH: usize> {
    word_list: WordList<WORD_LENGTH>,
    secret_word: Word<WORD_LENGTH>,
}

impl<const WORD_LENGTH: usize> Engine<WORD_LENGTH> {
    pub fn evaluate_no_wordlist(&self, word: Word<WORD_LENGTH>) -> Guess<WORD_LENGTH> {
        let mut secret_letter_used = [false; WORD_LENGTH];

        word.iter()
            .enumerate()
            .map(|(i, c)| {
                let annotation = if self.secret_word[i] == *c {
                    secret_letter_used[i] = true;
                    RightPlace
                } else {
                    Wrong
                };
                (c, annotation)
            })
            .collect::<Vec<_>>()
            .iter()
            .map(|(c, annotation)| {
                if let Wrong = annotation {
                    self.secret_word
                        .iter()
                        .enumerate()
                        .find_map(|(i, c_sec)| {
                            (!secret_letter_used[i] && *c == c_sec).then(|| {
                                secret_letter_used[i] = true;
                                RightLetter
                            })
                        })
                        .unwrap_or(*annotation)
                } else {
                    *annotation
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    pub fn evaluate(&self, word: Word<WORD_LENGTH>) -> WordleResult<Guess<WORD_LENGTH>> {
        self.word_list
            .contains(&word)
            .then(|| self.evaluate_no_wordlist(word))
            .ok_or(WordleError::NotInWordList)
    }
}

pub struct State<const WORD_LENGTH: usize> {
    guesses: Vec<(Word<WORD_LENGTH>, Guess<WORD_LENGTH>)>,
}

pub struct Game<const WORD_LENGTH: usize, const NUM_GUESSES: usize> {
    pub state: State<WORD_LENGTH>,
    engine: Engine<WORD_LENGTH>,
}

pub enum GuessResult<const WORD_LENGTH: usize> {
    Done(bool),
    Continue(Guess<WORD_LENGTH>),
}

impl<const WORD_LENGTH: usize, const NUM_GUESSES: usize> Game<WORD_LENGTH, NUM_GUESSES> {
    pub fn make_guess(
        &mut self,
        word: Word<WORD_LENGTH>,
    ) -> WordleResult<GuessResult<WORD_LENGTH>> {
        if self.state.guesses.len() < NUM_GUESSES {
            let guess = self.engine.evaluate(word)?;
            self.state.guesses.push((word, guess));
            Ok(
                if guess.iter().all(|annotation| *annotation == RightPlace) {
                    GuessResult::Done(true)
                } else if self.state.guesses.len() == NUM_GUESSES {
                    GuessResult::Done(false)
                } else {
                    GuessResult::Continue(guess)
                },
            )
        } else {
            Err(WordleError::TooManyGuesses)
        }
    }
}

pub trait Strategy<const WORD_LENGTH: usize, const NUM_GUESSES: usize> {
    fn init(word_list: WordList<WORD_LENGTH>) -> Self;

    fn make_guess(&mut self) -> Word<WORD_LENGTH>;

    fn receive_answer(&mut self, answer: Guess<WORD_LENGTH>);
}

pub fn run_strategy<S, const WORD_LENGTH: usize, const NUM_GUESSES: usize>(
    word_list: WordList<WORD_LENGTH>,
    secret_word: Word<WORD_LENGTH>,
) -> WordleResult<(bool, State<WORD_LENGTH>)>
where
    S: Strategy<WORD_LENGTH, NUM_GUESSES>,
{
    assert!(word_list.contains(&secret_word));
    let engine = Engine {
        word_list: word_list.clone(),
        secret_word,
    };

    let state = State { guesses: vec![] };

    let mut game = Game::<WORD_LENGTH, NUM_GUESSES> { state, engine };

    let mut strategy = S::init(word_list);

    loop {
        match game.make_guess(strategy.make_guess())? {
            GuessResult::Continue(answer) => strategy.receive_answer(answer),
            GuessResult::Done(result) => return Ok((result, game.state)),
        }
    }
}

struct SimpleStrategy<const WORD_LENGTH: usize> {
    word_list: WordList<WORD_LENGTH>,
    viable_words: WordList<WORD_LENGTH>,
    viable_bags: Vec<HashSet<char>>,
    last_guess: Option<Word<WORD_LENGTH>>,
    right_place: HashSet<char>,
    num_guesses: usize,
}

impl<const WORD_LENGTH: usize> SimpleStrategy<WORD_LENGTH> {
    fn score(&self, word: &Word<WORD_LENGTH>) -> usize {
        let bag = HashSet::<char>::from(*word);
        self.viable_words
            .iter()
            .map(|secret| {
                let engine = Engine {
                    word_list: vec![],
                    secret_word: *secret,
                };
                let answer = engine.evaluate_no_wordlist(*word);
                self.viable_words
                    .iter()
                    .filter(|viable_word| {
                        let engine = Engine {
                            word_list: vec![],
                            secret_word: **viable_word,
                        };
                        *viable_word != word && answer != engine.evaluate_no_wordlist(*word)
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

    fn recompute_viable_bags(&mut self) {
        self.viable_bags = self
            .viable_words
            .iter()
            .cloned()
            .map(HashSet::<char>::from)
            .collect::<Vec<_>>();
    }
}

impl<const WORD_LENGTH: usize, const NUM_GUESSES: usize> Strategy<WORD_LENGTH, NUM_GUESSES>
    for SimpleStrategy<WORD_LENGTH>
{
    fn init(word_list: WordList<WORD_LENGTH>) -> Self {
        let mut s = Self {
            word_list: word_list.clone(),
            viable_words: word_list,
            viable_bags: vec![],
            last_guess: None,
            right_place: HashSet::new(),
            num_guesses: 0,
        };
        s.recompute_viable_bags();
        s
    }

    fn make_guess(&mut self) -> Word<WORD_LENGTH> {
        let guess = if self.num_guesses == 0 {
            unsafe { std::mem::transmute_copy(&['c', 'r', 'a', 't', 'e']) }
        } else if self.num_guesses == 1 {
            unsafe { std::mem::transmute_copy(&['c', 'r', 'a', 't', 'e']) }
        } else {
            // let n = self.viable_words.len() / 2;
            // let dont_discount = self.viable_words.len() == 1 || self.num_guesses == 9;
            // let guess =
            *((if self.viable_words.len() == 1 || self.num_guesses == 9 {
                self.viable_words.clone()
            } else {
                self.word_list.clone()
            })
            // *self
            //     .word_list
            // .clone()
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
        println!("Guess: {:?}, {:?}", guess, self.score(&guess));

        self.num_guesses += 1;

        guess
    }

    fn receive_answer(&mut self, answer: Guess<WORD_LENGTH>) {
        println!("Answer: {:?}", answer);
        let last_guess = self.last_guess.expect("Should've made guess by now");
        self.viable_words.retain(|viable_word| {
            let engine = Engine {
                word_list: vec![last_guess],
                secret_word: *viable_word,
            };
            engine.evaluate(last_guess).unwrap() == answer
        });

        last_guess
            .iter()
            .zip(answer.iter())
            .for_each(|(c, annotation)| {
                if let RightPlace = annotation {
                    self.right_place.insert(*c);
                }
            });

        println!("Right places: {:?}", self.right_place);
        println!("Num viable words left: {:?}", self.viable_words.len());

        self.recompute_viable_bags();
    }
}

#[cfg(test)]
mod word_list;

#[cfg(test)]
mod tests {

    use super::*;

    fn evaluate<const N: usize>(word: Word<N>, secret_word: Word<N>) -> Guess<N> {
        let engine = Engine {
            word_list: vec![],
            secret_word,
        };

        engine.evaluate_no_wordlist(word)
    }

    #[test]
    fn test_evaluate() {
        assert_eq!(
            evaluate(['x', 'x', 'x'], ['a', 'a', 'a']),
            [Wrong, Wrong, Wrong]
        );
        assert_eq!(
            evaluate(['x', 'x', 'x'], ['x', 'x', 'x']),
            [RightPlace, RightPlace, RightPlace]
        );
        assert_eq!(
            evaluate(['a', 'b', 'c'], ['c', 'a', 'b']),
            [RightLetter, RightLetter, RightLetter]
        );
        assert_eq!(
            evaluate(['a', 'b', 'c'], ['c', 'b', 'a']),
            [RightLetter, RightPlace, RightLetter]
        );
        assert_eq!(
            evaluate(['a', 'a', 'b'], ['a', 'a', 'a']),
            [RightPlace, RightPlace, Wrong]
        );
        assert_eq!(
            evaluate(['y', 'y', 'x'], ['x', 'x', 'x']),
            [Wrong, Wrong, RightPlace]
        );
        assert_eq!(
            evaluate(['x', 'x', 'x'], ['y', 'y', 'x']),
            [Wrong, Wrong, RightPlace]
        );
        assert_eq!(
            evaluate(['x', 'y', 'y'], ['y', 'y', 'x']),
            [RightLetter, RightPlace, RightLetter]
        );
    }

    #[test]
    fn test_simple_strategy() {
        let word_list = word_list::WORD_LIST
            .iter()
            .map(|s| s.chars().collect::<Vec<_>>().try_into().unwrap())
            .collect();
        assert!(matches!(
            run_strategy::<SimpleStrategy<5>, 5, 10>(word_list, ['f', 'a', 'v', 'o', 'r']),
            Ok((true, _))
        ));
    }
}
