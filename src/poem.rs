use itertools::{Itertools};

#[derive(Debug, Clone)]
pub struct Poem {
    stanzas: Vec<Stanza>
}

#[derive(Debug, Clone)]
pub struct Stanza {
    verses: Vec<String>
}

impl Poem {
    pub fn stanzas(&self) -> &Vec<Stanza> {
        &self.stanzas
    }
}

impl From<&str> for Poem {
    fn from(value: &str) -> Self {
        // Split the poem into the stanzas and just take it from there.
        let lines = value.lines();
        
        let mut stanzas = Vec::new();
        let chunks = lines.chunk_by(|line| *line != "");
        let chunks = chunks.into_iter().filter(|(key, _)| *key);
        for (_, line_chunk) in chunks {
            let stanza = Stanza {verses: line_chunk.map(String::from).collect()};
            stanzas.push(stanza);
        }

        Self {stanzas}
    }
}

impl Stanza {
    pub fn verses(&self) -> &Vec<String> {
        &self.verses
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build_poem() {
        let poem_text = "just a\ncouple of\nlines\n\nand another\nstanza";
        let poem = Poem::from(poem_text);
        assert_eq!(poem.stanzas().len(), 2)
    }

}