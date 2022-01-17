use std::collections::{hash_map::Entry::Occupied, HashMap, HashSet};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LetterScore {
    RightPlace,
    RightLetter,
    Wrong,
}

pub type Score<const N: usize> = [LetterScore; N];

#[derive(Clone, Debug)]
pub struct WordList<const N: usize>(pub Vec<Word<N>>);

impl<const WORD_LENGTH: usize> WordList<WORD_LENGTH> {
    pub fn retain_viable_words(&mut self, guess: &Word<WORD_LENGTH>, score: &Score<WORD_LENGTH>) {
        self.0.retain(|word| word.evaluate_guess(guess) == *score);
    }
}

impl<const WORD_LENGTH: usize> From<Vec<Word<WORD_LENGTH>>> for WordList<WORD_LENGTH> {
    fn from(vec: Vec<Word<WORD_LENGTH>>) -> Self {
        Self(vec)
    }
}

impl<const WORD_LENGTH: usize> std::iter::FromIterator<Word<WORD_LENGTH>>
    for WordList<WORD_LENGTH>
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Word<WORD_LENGTH>>,
    {
        Vec::<Word<WORD_LENGTH>>::from_iter(iter).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word<const N: usize>(pub [char; N]);

impl<const WORD_LENGTH: usize> Word<WORD_LENGTH> {
    pub fn evaluate_guess(&self, guess: &Word<WORD_LENGTH>) -> Score<WORD_LENGTH> {
        use LetterScore::*;
        let mut score = [Wrong; WORD_LENGTH];
        let mut unused_letters = HashMap::new();

        let remaining_letters = self
            .0
            .iter()
            .zip(guess.0.iter())
            .enumerate()
            .filter_map(|(i, (letter, guess_letter))| {
                if letter == guess_letter {
                    score[i] = RightPlace;
                    None
                } else {
                    let counter = unused_letters.entry(letter).or_insert(0);
                    *counter += 1;
                    Some((i, guess_letter))
                }
            })
            .collect::<Vec<_>>();

        remaining_letters.iter().for_each(|(i, guess_letter)| {
            if let Occupied(mut entry) = unused_letters.entry(guess_letter) {
                score[*i] = RightLetter;
                if *entry.get() == 1 {
                    entry.remove_entry();
                } else {
                    *entry.get_mut() -= 1;
                }
            }
        });
        score
    }
}

impl<const WORD_LENGTH: usize> From<&Word<WORD_LENGTH>> for HashSet<char> {
    fn from(word: &Word<WORD_LENGTH>) -> Self {
        Self::from(word.0)
    }
}

impl<const WORD_LENGTH: usize> TryFrom<&str> for Word<WORD_LENGTH> {
    type Error = <[char; WORD_LENGTH] as TryFrom<&'static [char]>>::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Word(s.chars().collect::<Vec<_>>().as_slice().try_into()?))
    }
}

impl<const WORD_LENGTH: usize> From<Word<WORD_LENGTH>> for String {
    fn from(word: Word<WORD_LENGTH>) -> Self {
        word.0.iter().collect()
    }
}

impl<const WORD_LENGTH: usize> std::fmt::Display for Word<WORD_LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", <Self as Into<String>>::into(self.clone()))
    }
}
